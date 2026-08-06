//! Version management unit tests
//!
//! ## Purpose
//!
//! Verify pure domain logic in `claude_version_core::version`: semver
//! extraction, alias resolution, and version spec validation.
//!
//! ## Coverage
//!
//! - `extract_semver` strips leading `v`/`V` prefixes
//! - `extract_semver` passes through bare semver unchanged
//! - `extract_semver` finds semver inside verbose strings like `claude 1.2.3`
//! - `validate_version_spec` accepts all known aliases and 3-part semver
//! - `validate_version_spec` rejects empty strings and unknown inputs
//! - `resolve_version_spec` resolves each alias to a pinned value or `"latest"`
//! - `VERSION_ALIASES` table has consistent structure and required entries
//! - `purge_stale_versions` deletes stale binaries, keeps pinned target, ignores non-version files, is safe on missing dir, and refuses to run when the keep target is absent (BUG-016)
//! - `verify_install_outcome` gates purge/lock on the requested version actually being present after install (BUG-016)
//! - `unlock_settings_for_install` removes the 4 settings-level lock keys that block the official installer, and must stay in sync with `lock_version()` (BUG-017)
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `extract_semver_strips_lowercase_v` | "v1.2.3" → "1.2.3" |
//! | `extract_semver_strips_uppercase_v` | "V1.2.3" → "1.2.3" |
//! | `extract_semver_passes_bare_semver` | "1.2.3" → "1.2.3" |
//! | `extract_semver_finds_version_in_verbose_string` | "claude 1.2.3" → "1.2.3" |
//! | `validate_spec_accepts_known_aliases` | "latest", "stable" |
//! | `validate_spec_accepts_three_part_semver` | "1.2.3" |
//! | `validate_spec_rejects_empty` | "" → error |
//! | `validate_spec_rejects_unknown` | "nightly" → error |
//! | `validate_spec_rejects_two_part_semver` | "1.2" → error |
//! | `resolve_latest_alias_returns_latest` | "latest" → "latest" |
//! | `resolve_stable_alias_returns_semver` | "stable" → semver |
//! | `resolve_unknown_spec_passthrough` | "9.9.9" → "9.9.9" |
//! | `version_aliases_table_all_have_names`                | aliases non-empty names |
//! | `version_aliases_table_contains_latest_stable`  | aliases contains latest, stable |
//! | `purge_stale_versions_keeps_pinned_deletes_others`          | 3 files in tempdir; 2 stale deleted, 1 kept |
//! | `purge_stale_versions_ignores_non_version_files`           | `lock` and `metadata` files survive purge |
//! | `purge_stale_versions_noop_on_missing_dir`                 | no panic when directory does not exist |
//! | `purge_stale_versions_noop_on_empty_dir`                   | empty dir: read_dir ok, iterator empty, no-op |
//! | `purge_stale_versions_skips_subdirectories`                | subdir with version name survives (remove_file fails silently) |
//! | `purge_stale_versions_noop_when_keep_absent` | keep file absent: purge refuses to delete anything (BUG-016 reproducer) |
//! | `verify_install_outcome_pinned_match_passes` | pinned: detected == requested → pass |
//! | `verify_install_outcome_pinned_mismatch_fails` | pinned: detected != requested → fail |
//! | `verify_install_outcome_none_fails_even_for_latest` | no binary detectable → fail for pinned and latest (BUG-016 reproducer) |
//! | `verify_install_outcome_latest_accepts_any_version` | latest: any detected version passes |
//! | `lock_version_pin_writes_all_five_keys` | T01: pin writes autoUpdates/autoUpdatesChannel/minimumVersion/env.DISABLE_AUTOUPDATER/env.DISABLE_UPDATES |
//! | `lock_version_unpin_removes_all_five_keys` | T02: unpin resets/removes all 5 keys |
//! | `lock_version_repin_updates_minimum_version` | T03: re-pin to a different version updates minimumVersion, not stale |
//! | `lock_version_pin_all_five_keys_resolve_with_user_source` | T04: all 5 keys resolve with source: user after a pinned install |
//! | `purge_stale_versions_traces_function_and_parameters` | parameter-trace structural guard (task 313): first statement is `eprintln!` naming the function and both parameters |
//! | `unlock_settings_for_install_removes_all_four_lock_keys` | pinned lock applied; unlock clears all 4 install-blocking keys (BUG-017 reproducer) |
//! | `unlock_settings_for_install_noop_when_home_absent` | graceful no-op when HOME is unset |
//! | `unlock_settings_for_install_traces_function_name` | parameter-trace structural guard: first stmt is `eprintln!` naming the function |

use claude_version_core::version::{
  extract_semver, validate_version_spec, resolve_version_spec, VERSION_ALIASES,
  purge_stale_versions, lock_version, verify_install_outcome, unlock_settings_for_install,
};
use claude_core::settings_io::get_setting;
use claude_version_core::config_resolve::{ resolve, Layer };
use claude_version_core::config_catalog::catalog;

// ─── extract_semver ───────────────────────────────────────────────────────────

#[test]
fn extract_semver_strips_lowercase_v()
{
  assert_eq!( extract_semver( "v1.2.3" ), "1.2.3" );
}

#[test]
fn extract_semver_strips_uppercase_v()
{
  assert_eq!( extract_semver( "V2.1.78" ), "2.1.78" );
}

#[test]
fn extract_semver_passes_bare_semver()
{
  assert_eq!( extract_semver( "1.2.3" ),  "1.2.3"  );
  assert_eq!( extract_semver( "2.1.78" ), "2.1.78" );
}

#[test]
fn extract_semver_finds_version_in_verbose_string()
{
  // Claude's `--version` output is like "claude 2.1.78 (build …)"
  assert_eq!( extract_semver( "claude 2.1.78 (build 123)" ), "2.1.78" );
}

// ─── validate_version_spec ────────────────────────────────────────────────────

#[test]
fn validate_spec_accepts_known_aliases()
{
  for alias in VERSION_ALIASES
  {
    let result = validate_version_spec( alias.name, &[] );
    assert!(
      result.is_ok(),
      "expected Ok for alias '{}', got: {:?}",
      alias.name,
      result
    );
  }
}

#[test]
fn validate_spec_accepts_three_part_semver()
{
  assert!( validate_version_spec( "1.2.3",  &[] ).is_ok() );
  assert!( validate_version_spec( "2.1.78", &[] ).is_ok() );
  assert!( validate_version_spec( "10.0.0", &[] ).is_ok() );
}

#[test]
fn validate_spec_rejects_empty()
{
  assert!( validate_version_spec( "", &[] ).is_err(), "empty string must be rejected" );
}

#[test]
fn validate_spec_rejects_unknown()
{
  assert!( validate_version_spec( "nightly", &[] ).is_err() );
  assert!( validate_version_spec( "beta",    &[] ).is_err() );
}

#[test]
fn validate_spec_rejects_two_part_semver()
{
  assert!( validate_version_spec( "1.2", &[] ).is_err() );
  assert!( validate_version_spec( "2.1", &[] ).is_err() );
}

// ─── resolve_version_spec ─────────────────────────────────────────────────────

#[test]
fn resolve_latest_alias_returns_latest()
{
  // "latest" has empty value → resolves to the alias name itself
  assert_eq!( resolve_version_spec( "latest", &[] ), "latest" );
}

#[test]
fn resolve_stable_alias_returns_semver()
{
  let resolved = resolve_version_spec( "stable", &[] );
  // Must be a non-empty semver, not the literal "stable"
  assert_ne!( resolved, "stable", "stable must resolve to a pinned semver" );
  assert!(
    resolved.contains( '.' ),
    "stable must resolve to a semver like '2.1.78', got: {resolved}"
  );
}

#[test]
fn resolve_unknown_spec_passthrough()
{
  // Unknown specs pass through unchanged (callers validate separately)
  assert_eq!( resolve_version_spec( "9.9.9", &[] ), "9.9.9" );
}

// ─── VERSION_ALIASES table ────────────────────────────────────────────────────

#[test]
fn version_aliases_table_all_have_names()
{
  for alias in VERSION_ALIASES
  {
    assert!( !alias.name.is_empty(),        "alias name must not be empty" );
    assert!( !alias.description.is_empty(), "alias description must not be empty" );
  }
}

#[test]
fn version_aliases_table_contains_latest_stable()
{
  // The built-in table carries only the two compile-time aliases; ad hoc
  // aliases (formerly "month") are runtime custom markers via `.version.mark`.
  let names : Vec< &str > = VERSION_ALIASES.iter().map( | a | a.name ).collect();
  assert!( names.contains( &"latest" ), "must have 'latest' alias" );
  assert!( names.contains( &"stable" ), "must have 'stable' alias" );
}

// ─── purge_stale_versions ─────────────────────────────────────────────────────

#[test]
fn purge_stale_versions_keeps_pinned_deletes_others()
{
  let dir = tempfile::tempdir().unwrap();
  let p   = dir.path();
  std::fs::write( p.join( "2.1.78" ), b"elf" ).unwrap();
  std::fs::write( p.join( "2.1.73" ), b"elf" ).unwrap();
  std::fs::write( p.join( "2.1.74" ), b"elf" ).unwrap();
  purge_stale_versions( p.to_str().unwrap(), "2.1.78" );
  assert!(  p.join( "2.1.78" ).exists(), "pinned version must be kept" );
  assert!( !p.join( "2.1.73" ).exists(), "stale 2.1.73 must be deleted" );
  assert!( !p.join( "2.1.74" ).exists(), "stale 2.1.74 must be deleted" );
}

#[test]
fn purge_stale_versions_ignores_non_version_files()
{
  let dir = tempfile::tempdir().unwrap();
  let p   = dir.path();
  std::fs::write( p.join( "2.1.78"   ), b"elf" ).unwrap();
  std::fs::write( p.join( "lock"     ), b"x"   ).unwrap();
  std::fs::write( p.join( "metadata" ), b"x"   ).unwrap();
  purge_stale_versions( p.to_str().unwrap(), "2.1.78" );
  assert!(  p.join( "2.1.78"   ).exists(), "pinned version kept" );
  assert!(  p.join( "lock"     ).exists(), "non-version file 'lock' must not be deleted" );
  assert!(  p.join( "metadata" ).exists(), "non-version file 'metadata' must not be deleted" );
}

#[test]
fn purge_stale_versions_noop_on_missing_dir()
{
  // Must complete without panic when directory does not exist.
  purge_stale_versions( "/tmp/nonexistent_claude_versions_xyz_abc_987", "2.1.78" );
}

#[test]
fn purge_stale_versions_noop_on_empty_dir()
{
  // The keep target can never exist in an empty directory, so the BUG-016
  // keep-guard returns before read_dir is even consulted — still a silent no-op.
  let dir = tempfile::tempdir().unwrap();
  purge_stale_versions( dir.path().to_str().unwrap(), "2.1.78" );
  // No panic, no error — function is a silent no-op.
}

#[test]
fn purge_stale_versions_skips_subdirectories()
{
  // A subdirectory with a version-like name must survive: remove_file
  // fails on directories and the error is silently ignored.
  let dir = tempfile::tempdir().unwrap();
  let p   = dir.path();
  std::fs::write( p.join( "2.1.78" ), b"elf" ).unwrap();
  std::fs::create_dir( p.join( "2.1.73" ) ).unwrap();
  purge_stale_versions( p.to_str().unwrap(), "2.1.78" );
  assert!(  p.join( "2.1.78" ).exists(), "pinned version kept" );
  assert!(  p.join( "2.1.73" ).is_dir(), "subdirectory must survive purge" );
}

// BUG-016 — purge must be a no-op when the keep target is absent.
//
// Root Cause: `purge_stale_versions` deleted every version-named file even
// when `keep` was not present in the directory — combined with an installer
// that refused to install yet exited 0, this destroyed the only working
// binary on the host.
// Why Not Caught: this test's previous incarnation
// (`purge_stale_versions_deletes_all_stale_when_keep_not_present`) asserted
// the deletion as CORRECT behavior; no test asked whether a purge without its
// keep target should run at all, so every green run reconfirmed the data-loss
// path.
// Fix Applied: early return in `purge_stale_versions` when `versions_dir/keep`
// does not exist, before the deletion loop.
// Prevention: a cleanup parameterized as "keep X" must first prove X exists;
// otherwise "keep X" silently degrades into "delete everything".
// Pitfall: a test can cement a bug — renaming and inverting the pinning test
// is the fix here, not adding a parallel test beside it.
// test_kind: bug_reproducer(BUG-016)
#[test]
fn purge_stale_versions_noop_when_keep_absent()
{
  let dir = tempfile::tempdir().unwrap();
  let p   = dir.path();
  std::fs::write( p.join( "2.1.73" ), b"elf" ).unwrap();
  std::fs::write( p.join( "2.1.74" ), b"elf" ).unwrap();
  std::fs::write( p.join( "lock"   ), b"x"   ).unwrap();
  purge_stale_versions( p.to_str().unwrap(), "2.1.78" );
  assert!( p.join( "2.1.73" ).exists(), "2.1.73 must survive: keep target absent, purge must not run" );
  assert!( p.join( "2.1.74" ).exists(), "2.1.74 must survive: keep target absent, purge must not run" );
  assert!( p.join( "lock"   ).exists(), "non-version file survives" );
}

// ─── verify_install_outcome ───────────────────────────────────────────────────

#[test]
fn verify_install_outcome_pinned_match_passes()
{
  assert!( verify_install_outcome( "2.1.220", false, Some( "2.1.220" ) ) );
}

#[test]
fn verify_install_outcome_pinned_mismatch_fails()
{
  assert!( !verify_install_outcome( "2.1.220", false, Some( "2.1.197" ) ) );
}

// BUG-016 — the refusal shape: installer exits 0, nothing installed.
//
// Root Cause: perform_install() gated purge/lock solely on the installer's
// exit code; the official bootstrap exits 0 even when it refuses to install
// (e.g. update-disabling settings left by a previous pinned install), so the
// purge ran with its keep target never written and deleted every cached
// binary.
// Why Not Caught: no seam existed between "installer exited" and "purge/lock
// ran" — success had no definition beyond the exit code, so no test could
// even express "exited 0 but did not install".
// Fix Applied: verify_install_outcome() is that seam — pure and testable —
// and perform_install() now requires it to pass before purge/lock run.
// Prevention: any subprocess known to exit 0 on refusal gets independent
// outcome verification before destructive follow-up steps.
// Pitfall: an installer's "✅ done" banner and exit code describe the script
// finishing, not the install happening.
// test_kind: bug_reproducer(BUG-016)
#[test]
fn verify_install_outcome_none_fails_even_for_latest()
{
  assert!( !verify_install_outcome( "2.1.220", false, None ) );
  assert!( !verify_install_outcome( "latest", true, None ) );
}

#[test]
fn verify_install_outcome_latest_accepts_any_version()
{
  // The installer chooses the concrete semver for `latest`, so any detected
  // version passes; safe because the latest path never purges.
  assert!( verify_install_outcome( "latest", true, Some( "9.9.9" ) ) );
}

// ─── lock_version ─────────────────────────────────────────────────────────────

#[test]
fn lock_version_pin_writes_all_five_keys()
{
  let home       = tempfile::tempdir().unwrap();
  let no_project = tempfile::tempdir().unwrap();
  std::env::set_var( "HOME", home.path() );

  lock_version( false, "2.1.78" );

  let settings_file = home.path().join( ".claude" ).join( "settings.json" );
  assert_eq!(
    get_setting( &settings_file, "autoUpdates" ).unwrap().as_deref(),
    Some( "false" ), "autoUpdates must be false when pinned"
  );
  assert_eq!(
    get_setting( &settings_file, "autoUpdatesChannel" ).unwrap().as_deref(),
    Some( "stable" ), "autoUpdatesChannel must be stable when pinned"
  );
  assert_eq!(
    get_setting( &settings_file, "minimumVersion" ).unwrap().as_deref(),
    Some( "2.1.78" ), "minimumVersion must be the resolved pinned semver"
  );

  let auto_updater = resolve( "env.DISABLE_AUTOUPDATER", home.path(), no_project.path(), catalog() );
  assert_eq!( auto_updater.value.as_deref(), Some( "1" ), "env.DISABLE_AUTOUPDATER must be set to 1" );

  let disable_updates = resolve( "env.DISABLE_UPDATES", home.path(), no_project.path(), catalog() );
  assert_eq!( disable_updates.value.as_deref(), Some( "1" ), "env.DISABLE_UPDATES must be set to 1" );
}

#[test]
fn lock_version_unpin_removes_all_five_keys()
{
  let home       = tempfile::tempdir().unwrap();
  let no_project = tempfile::tempdir().unwrap();
  std::env::set_var( "HOME", home.path() );

  // Pin first so all 5 keys are set, then unpin.
  lock_version( false, "2.1.78" );
  lock_version( true, "" );

  let settings_file = home.path().join( ".claude" ).join( "settings.json" );
  assert_eq!(
    get_setting( &settings_file, "autoUpdates" ).unwrap().as_deref(),
    Some( "true" ), "autoUpdates must reset to true when unpinned"
  );
  assert_eq!(
    get_setting( &settings_file, "autoUpdatesChannel" ).unwrap(),
    None, "autoUpdatesChannel must be removed when unpinned"
  );
  assert_eq!(
    get_setting( &settings_file, "minimumVersion" ).unwrap(),
    None, "minimumVersion must be removed when unpinned"
  );

  let auto_updater = resolve( "env.DISABLE_AUTOUPDATER", home.path(), no_project.path(), catalog() );
  assert!( auto_updater.value.is_none(), "env.DISABLE_AUTOUPDATER must be removed when unpinned" );

  let disable_updates = resolve( "env.DISABLE_UPDATES", home.path(), no_project.path(), catalog() );
  assert!( disable_updates.value.is_none(), "env.DISABLE_UPDATES must be removed when unpinned" );
}

#[test]
fn lock_version_repin_updates_minimum_version()
{
  let home = tempfile::tempdir().unwrap();
  std::env::set_var( "HOME", home.path() );

  lock_version( false, "2.1.78" );
  lock_version( false, "2.1.90" );

  let settings_file = home.path().join( ".claude" ).join( "settings.json" );
  assert_eq!(
    get_setting( &settings_file, "minimumVersion" ).unwrap().as_deref(),
    Some( "2.1.90" ), "minimumVersion must update on re-pin, not remain stale at prior value"
  );
}

#[test]
fn lock_version_pin_all_five_keys_resolve_with_user_source()
{
  let home       = tempfile::tempdir().unwrap();
  let no_project = tempfile::tempdir().unwrap();
  std::env::set_var( "HOME", home.path() );

  lock_version( false, "2.1.78" );

  for key in [ "autoUpdates", "autoUpdatesChannel", "minimumVersion", "env.DISABLE_AUTOUPDATER", "env.DISABLE_UPDATES" ]
  {
    let rv = resolve( key, home.path(), no_project.path(), catalog() );
    assert_eq!( rv.source, Layer::User, "{key} must resolve with source: user after a pinned install" );
  }
}

// ─── purge_stale_versions: parameter-trace structural guard (Task 313) ────────

/// Extract the body of `fn {name}(...) { ... }` from `src` via brace-depth
/// counting. A naive "scan to next `pub fn`" heuristic is fragile when the
/// next function is hundreds of lines away with no other `eprintln!` between
/// — brace counting finds the exact matching close-brace instead.
fn extract_fn_body<'a>( src : &'a str, name : &str ) -> &'a str
{
  let sig        = format!( "fn {name}(" );
  let fn_start   = src.find( &sig ).unwrap_or_else( || panic!( "{name} not found in source" ) );
  let brace_start = src[ fn_start.. ].find( '{' )
    .unwrap_or_else( || panic!( "{name} body opening brace not found" ) ) + fn_start;

  let mut depth = 0usize;
  let mut end   = brace_start;
  for ( i, ch ) in src[ brace_start.. ].char_indices()
  {
    match ch
    {
      '{' => depth += 1,
      '}' =>
      {
        depth -= 1;
        if depth == 0 { end = brace_start + i; break; }
      }
      _ => {}
    }
  }
  &src[ brace_start + 1..end ]
}

#[test]
fn purge_stale_versions_traces_function_and_parameters()
{
  let src        = include_str!( "../src/version.rs" );
  let body       = extract_fn_body( src, "purge_stale_versions" );
  let first_stmt = body.trim_start().split( ';' ).next().unwrap().trim();

  assert!(
    first_stmt.starts_with( "eprintln!" ),
    "purge_stale_versions must emit eprintln! as its first statement, got: {first_stmt:?}"
  );
  assert!(
    first_stmt.contains( "purge_stale_versions" ) && first_stmt.contains( "versions_dir" ) && first_stmt.contains( "keep" ),
    "trace line must name the function and both parameters (versions_dir, keep): {first_stmt:?}"
  );
  assert_eq!(
    body.matches( "eprintln!" ).count(), 1,
    "purge_stale_versions must have exactly one eprintln! call, found {}", body.matches( "eprintln!" ).count()
  );
}

// ─── unlock_settings_for_install ─────────────────────────────────────────────

// Root Cause: unlock_settings_for_install() was private with no integration
//   test seam — if its key set drifted from lock_version()'s key set (e.g. a
//   new lock layer added to one but not the other), BUG-016 would silently
//   re-emerge with no failing test.
// Why Not Caught: private functions in a crate with all tests in tests/ are
//   untestable; the function was intentionally private to keep it off the
//   traced-public-functions list.
// Fix Applied: function promoted to pub (with eprintln! trace), enabling
//   direct integration tests that pin the unlock key set.
// Prevention: any new key added to lock_version()'s pinned path must be mirrored
//   in unlock_settings_for_install(); these tests are the regression tripwire.
// Pitfall: the official installer honors update-disabling keys in settings.json
//   as admin policy — every key that lock_version() writes for pinned installs
//   must be removed by unlock_settings_for_install() before invoking the installer.

// test_kind: bug_reproducer(BUG-017)
#[test]
fn unlock_settings_for_install_removes_all_four_lock_keys()
{
  let home       = tempfile::tempdir().unwrap();
  let no_project = tempfile::tempdir().unwrap();
  std::env::set_var( "HOME", home.path() );

  // Apply a pinned lock so all 4 install-blocking keys are present.
  lock_version( false, "2.1.220" );

  // Lift those keys — exactly what perform_install() does pre-install.
  unlock_settings_for_install();

  let settings_file = home.path().join( ".claude" ).join( "settings.json" );
  assert_eq!(
    get_setting( &settings_file, "autoUpdates" ).unwrap().as_deref(),
    Some( "true" ),
    "unlock must set autoUpdates=true so the installer treats updates as enabled"
  );
  assert_eq!(
    get_setting( &settings_file, "minimumVersion" ).unwrap(),
    None,
    "unlock must remove minimumVersion so the installer is not rejected by a version floor"
  );

  let auto_updater = resolve( "env.DISABLE_AUTOUPDATER", home.path(), no_project.path(), catalog() );
  assert!(
    auto_updater.value.is_none(),
    "unlock must remove env.DISABLE_AUTOUPDATER — one of the keys the bootstrap treats as an admin policy block"
  );

  let disable_updates = resolve( "env.DISABLE_UPDATES", home.path(), no_project.path(), catalog() );
  assert!(
    disable_updates.value.is_none(),
    "unlock must remove env.DISABLE_UPDATES — one of the keys the bootstrap treats as an admin policy block"
  );
}

#[test]
fn unlock_settings_for_install_noop_when_home_absent()
{
  std::env::remove_var( "HOME" );
  // Must not panic when ClaudePaths cannot resolve $HOME.
  unlock_settings_for_install();
}

#[test]
fn unlock_settings_for_install_traces_function_name()
{
  let src        = include_str!( "../src/version.rs" );
  let body       = extract_fn_body( src, "unlock_settings_for_install" );
  let first_stmt = body.trim_start().split( ';' ).next().unwrap().trim();
  assert!(
    first_stmt.starts_with( "eprintln!" ) && first_stmt.contains( "unlock_settings_for_install" ),
    "unlock_settings_for_install must emit eprintln! naming the function as its first statement, got: {first_stmt:?}"
  );
}
