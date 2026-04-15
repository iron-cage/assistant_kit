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
//! - `purge_stale_versions` deletes stale binaries, keeps pinned target, ignores non-version files, and is safe on missing dir
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `extract_semver_strips_lowercase_v` | "v1.2.3" → "1.2.3" |
//! | `extract_semver_strips_uppercase_v` | "V1.2.3" → "1.2.3" |
//! | `extract_semver_passes_bare_semver` | "1.2.3" → "1.2.3" |
//! | `extract_semver_finds_version_in_verbose_string` | "claude 1.2.3" → "1.2.3" |
//! | `validate_spec_accepts_known_aliases` | "latest", "stable", "month" |
//! | `validate_spec_accepts_three_part_semver` | "1.2.3" |
//! | `validate_spec_rejects_empty` | "" → error |
//! | `validate_spec_rejects_unknown` | "nightly" → error |
//! | `validate_spec_rejects_two_part_semver` | "1.2" → error |
//! | `resolve_latest_alias_returns_latest` | "latest" → "latest" |
//! | `resolve_stable_alias_returns_semver` | "stable" → semver |
//! | `resolve_unknown_spec_passthrough` | "9.9.9" → "9.9.9" |
//! | `version_aliases_table_all_have_names`                | aliases non-empty names |
//! | `version_aliases_table_contains_latest_stable_month`  | aliases contains latest, stable, month |
//! | `purge_stale_versions_keeps_pinned_deletes_others`          | 3 files in tempdir; 2 stale deleted, 1 kept |
//! | `purge_stale_versions_ignores_non_version_files`           | `lock` and `metadata` files survive purge |
//! | `purge_stale_versions_noop_on_missing_dir`                 | no panic when directory does not exist |
//! | `purge_stale_versions_noop_on_empty_dir`                   | empty dir: read_dir ok, iterator empty, no-op |
//! | `purge_stale_versions_skips_subdirectories`                | subdir with version name survives (remove_file fails silently) |
//! | `purge_stale_versions_deletes_all_stale_when_keep_not_present` | keep file absent: all version files deleted, non-version survives |

use claude_version_core::version::{
  extract_semver, validate_version_spec, resolve_version_spec, VERSION_ALIASES,
  purge_stale_versions,
};

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
    let result = validate_version_spec( alias.name );
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
  assert!( validate_version_spec( "1.2.3"   ).is_ok() );
  assert!( validate_version_spec( "2.1.78"  ).is_ok() );
  assert!( validate_version_spec( "10.0.0"  ).is_ok() );
}

#[test]
fn validate_spec_rejects_empty()
{
  assert!( validate_version_spec( "" ).is_err(), "empty string must be rejected" );
}

#[test]
fn validate_spec_rejects_unknown()
{
  assert!( validate_version_spec( "nightly" ).is_err() );
  assert!( validate_version_spec( "beta"    ).is_err() );
}

#[test]
fn validate_spec_rejects_two_part_semver()
{
  assert!( validate_version_spec( "1.2"   ).is_err() );
  assert!( validate_version_spec( "2.1"   ).is_err() );
}

// ─── resolve_version_spec ─────────────────────────────────────────────────────

#[test]
fn resolve_latest_alias_returns_latest()
{
  // "latest" has empty value → resolves to the alias name itself
  assert_eq!( resolve_version_spec( "latest" ), "latest" );
}

#[test]
fn resolve_stable_alias_returns_semver()
{
  let resolved = resolve_version_spec( "stable" );
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
  assert_eq!( resolve_version_spec( "9.9.9" ), "9.9.9" );
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
fn version_aliases_table_contains_latest_stable_month()
{
  let names : Vec< &str > = VERSION_ALIASES.iter().map( | a | a.name ).collect();
  assert!( names.contains( &"latest" ), "must have 'latest' alias" );
  assert!( names.contains( &"stable" ), "must have 'stable' alias" );
  assert!( names.contains( &"month"  ), "must have 'month' alias"  );
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
  // read_dir succeeds but iterator yields nothing — different code path
  // from missing dir (where read_dir itself fails).
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

#[test]
fn purge_stale_versions_deletes_all_stale_when_keep_not_present()
{
  // If the keep target is not in the directory (e.g., install placed it
  // elsewhere), all version-named files are still deleted. Non-version
  // files survive.
  let dir = tempfile::tempdir().unwrap();
  let p   = dir.path();
  std::fs::write( p.join( "2.1.73" ), b"elf" ).unwrap();
  std::fs::write( p.join( "2.1.74" ), b"elf" ).unwrap();
  std::fs::write( p.join( "lock"   ), b"x"   ).unwrap();
  purge_stale_versions( p.to_str().unwrap(), "2.1.78" );
  assert!( !p.join( "2.1.73" ).exists(), "stale 2.1.73 deleted" );
  assert!( !p.join( "2.1.74" ).exists(), "stale 2.1.74 deleted" );
  assert!(  p.join( "lock"   ).exists(), "non-version file survives" );
}
