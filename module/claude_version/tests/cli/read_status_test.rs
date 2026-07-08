//! Integration tests for `.status` — E2.
//!
//! Also covers format/verbosity edge cases exercised via `.status` (TC-242–TC-245)
//! and preferred-version display (TC-419, TC-420).
//!
//! ## E2 — `.status`
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-099 | `.status` exits 0 always | P | 0 |
//! | TC-096 | `.status` with empty PATH → version "not found", still exits 0 | P | 0 |
//! | TC-097 | `.status v::0` → 3 bare lines | P | 0 |
//! | TC-098 | `.status v::1` → labeled Version/Processes/Account lines | P | 0 |
//! | TC-100 | `.status format::json` → valid JSON with required keys | P | 0 |
//! | TC-104 | `.status v::0` has fewer lines than `.status v::1` | P | 0 |
//! | TC-105 | `.status` HOME not set → account "unknown", no crash | P | 0 |
//! | TC-419 | `.status` with no preference → no "Preferred" line | P | 0 |
//! | TC-420 | `.status` with preference → shows "Preferred" line | P | 0 |
//!
//! ## Format/verbosity edge cases (via `.status`)
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-242 | `format::xml` → exit 1 (unknown format) | N | 1 |
//! | TC-243 | `format::JSON` (uppercase) → exit 1 | N | 1 |
//! | TC-244 | `format::` (empty) → exit 1 | N | 1 |
//! | TC-245 | Last `v::` wins when duplicated | P | 0 |
//!
//! ## Lock-state visibility (Task 314)
//! | TC | Description | P/N | Exit |
//! |----|-------------|-----|------|
//! | TC-515 | pinned install, all keys compliant → `Lock:` section, no mismatch | P | 0 |
//! | TC-516 | pinned install, `chmod` drifted to 755 → flagged mismatch | P | 0 |
//! | TC-517 | pinned install, `autoUpdates` flipped to true → flagged mismatch | P | 0 |
//! | TC-518 | unpinned install → `Lock:` section, all compliant | P | 0 |
//! | TC-519 | `v::0`/`v::1` output unchanged by the Lock: feature | P | 0 |
//! | TC-520 | `v::3` continues to exit 1 as out-of-range (IT-11 regression) | N | 1 |
//! | TC-521 | `format::json`, pinned, all compliant → `"lock"` object present | P | 0 |
//! | TC-522 | unpinned install, versions dir never created (fresh install) → `chmod: absent`, no false mismatch | P | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stdout, write_settings };

/// Create the versions directory under `home_dir` with the given `chmod` mode.
///
/// # Panics
///
/// Panics if the directory cannot be created or its permissions cannot be set.
#[ cfg( unix ) ]
fn write_versions_dir( home_dir : &std::path::Path, mode : u32 )
{
  use std::os::unix::fs::PermissionsExt;
  let dir = home_dir.join( ".local" ).join( "share" ).join( "claude" ).join( "versions" );
  std::fs::create_dir_all( &dir ).unwrap();
  std::fs::set_permissions( &dir, std::fs::Permissions::from_mode( mode ) ).unwrap();
}

/// Write a full pinned-install `settings.json` fixture with all 5 lock-mechanism
/// keys (`autoUpdates`, `autoUpdatesChannel`, `minimumVersion`, and the nested
/// `env.DISABLE_AUTOUPDATER`/`env.DISABLE_UPDATES` pair).
///
/// `auto_updates` is parameterized so tests can simulate drift in that one key
/// while keeping the rest compliant.
///
/// # Panics
///
/// Panics if the directory cannot be created or the file cannot be written.
fn write_pinned_settings( home_dir : &std::path::Path, resolved_version : &str, auto_updates : &str )
{
  let dir = home_dir.join( ".claude" );
  std::fs::create_dir_all( &dir ).unwrap();
  let json = format!(
    "{{\n  \"preferredVersionSpec\": \"stable\",\n  \"preferredVersionResolved\": \"{resolved_version}\",\n  \"autoUpdates\": \"{auto_updates}\",\n  \"autoUpdatesChannel\": \"stable\",\n  \"minimumVersion\": \"{resolved_version}\",\n  \"env\": {{\"DISABLE_AUTOUPDATER\": \"1\", \"DISABLE_UPDATES\": \"1\"}}\n}}"
  );
  std::fs::write( dir.join( "settings.json" ), json ).unwrap();
}

// ─── E2: status ──────────────────────────────────────────────────────────────

// TC-099
#[ test ]
fn tc099_status_exits_0()
{
  let out = run_clv( &[ ".status" ] );
  assert_exit( &out, 0 );
}

// TC-096: no symlink + empty PATH → version "not found", still exits 0
#[ test ]
fn tc096_status_no_claude_in_path_exits_0()
{
  let dir = TempDir::new().unwrap();
  let fake_home = dir.path().to_str().unwrap();
  let out = run_clv_with_env(
    &[ ".status" ],
    &[ ( "PATH", "" ), ( "HOME", fake_home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "not found" ) || text.contains( "unknown" ),
    "expected 'not found' or 'unknown' in output, got: {text}"
  );
}

// TC-097: v::0 → exactly 3 lines (version, processes, account) when no preference set
#[ test ]
fn tc097_status_v0_has_3_lines()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );
  let out = run_clv_with_env(
    &[ ".status", "v::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().collect();
  assert_eq!( lines.len(), 3, "v::0 must produce exactly 3 lines, got: {text:?}" );
}

// TC-098: v::1 → labels present
#[ test ]
fn tc098_status_v1_has_labels()
{
  let out = run_clv( &[ ".status", "v::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Version:" ),  "missing 'Version:' label, got: {text}" );
  assert!( text.contains( "Processes:" ), "missing 'Processes:' label, got: {text}" );
  assert!( text.contains( "Account:" ),  "missing 'Account:' label, got: {text}" );
}

// TC-100: format::json → valid JSON with required keys
#[ test ]
fn tc100_status_format_json()
{
  let out = run_clv( &[ ".status", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"version\"" ),  "missing 'version' key in JSON: {text}" );
  assert!( text.contains( "\"processes\"" ), "missing 'processes' key in JSON: {text}" );
  assert!( text.contains( "\"account\"" ),  "missing 'account' key in JSON: {text}" );
}

// TC-104: v::0 fewer lines than v::1
#[ test ]
fn tc104_status_v0_fewer_lines_than_v1()
{
  let out0 = run_clv( &[ ".status", "v::0" ] );
  let out1 = run_clv( &[ ".status", "v::1" ] );
  assert_exit( &out0, 0 );
  assert_exit( &out1, 0 );
  let n0 = stdout( &out0 ).lines().count();
  let n1 = stdout( &out1 ).lines().count();
  assert!( n0 <= n1, "v::0 ({n0} lines) must have \u{2264} lines than v::1 ({n1} lines)" );
}

// TC-105: HOME not set → account "unknown", no crash
#[ test ]
fn tc105_status_no_home_shows_unknown_account()
{
  let out = run_clv_with_env( &[ ".status" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "unknown" ),
    "expected 'unknown' account with no HOME, got: {text}"
  );
}

// ─── Preferred version display in status ─────────────────────────────────────

// TC-419: status with no preference → no "Preferred" line
#[ test ]
fn tc419_status_no_preference_no_preferred_line()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".status" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Preferred" ),
    "status without preference must not show Preferred line: {text}"
  );
}

// TC-420: status with preference → shows "Preferred" line
#[ test ]
fn tc420_status_with_preference_shows_preferred()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let settings_json = r#"{
  "preferredVersionSpec": "stable",
  "preferredVersionResolved": "2.1.78"
}"#;
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  let out = run_clv_with_env(
    &[ ".status" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Preferred:" ) && text.contains( "stable" ),
    "status with preference must show Preferred line: {text}"
  );
}

// ─── Format/verbosity edge cases (via .status) ───────────────────────────────

// TC-242: format::xml → exit 1 (unknown format)
#[ test ]
fn tc242_unknown_format_exits_1()
{
  let out = run_clv( &[ ".status", "format::xml" ] );
  assert_exit( &out, 1 );
}

// TC-243: format::JSON (uppercase) → exit 1
#[ test ]
fn tc243_uppercase_format_exits_1()
{
  let out = run_clv( &[ ".status", "format::JSON" ] );
  assert_exit( &out, 1 );
}

// TC-244: format:: (empty value) → exit 1
#[ test ]
fn tc244_empty_format_exits_1()
{
  let out = run_clv( &[ ".status", "format::" ] );
  assert_exit( &out, 1 );
}

// TC-245: last v:: wins when duplicated
#[ test ]
fn tc245_last_occurrence_wins_for_verbosity()
{
  let out = run_clv( &[ ".status", "v::2", "v::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // v::0 is last → bare output (no labels)
  assert!(
    !text.contains( "Version:" ),
    "v::0 (last-wins) must produce bare output, got: {text}"
  );
}

// ─── Lock-state visibility (Task 314) ────────────────────────────────────────

// TC-515 (T01): pinned install, all keys compliant → Lock: section, no mismatch
#[ cfg( unix ) ]
#[ test ]
fn tc515_status_lock_pinned_all_compliant()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_pinned_settings( dir.path(), "2.1.78", "false" );
  write_versions_dir( dir.path(), 0o555 );

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Lock:" ), "missing Lock: section: {text}" );
  for key in [ "autoUpdates", "autoUpdatesChannel", "minimumVersion", "env.DISABLE_AUTOUPDATER", "env.DISABLE_UPDATES", "chmod" ]
  {
    assert!( text.contains( key ), "Lock: section missing key {key}: {text}" );
  }
  assert!( !text.contains( "MISMATCH" ), "expected no mismatch for fully compliant pinned install: {text}" );
}

// TC-516 (T02): pinned install, chmod drifted to 755 → flagged mismatch
#[ cfg( unix ) ]
#[ test ]
fn tc516_status_lock_chmod_drift_flagged()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_pinned_settings( dir.path(), "2.1.78", "false" );
  write_versions_dir( dir.path(), 0o755 ); // drifted — pinned expects 555

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let chmod_line = text.lines().find( | l | l.contains( "chmod" ) )
    .unwrap_or_else( || panic!( "no chmod line in output: {text}" ) );
  assert!(
    chmod_line.contains( "755" ) && chmod_line.contains( "555" ) && chmod_line.contains( "MISMATCH" ),
    "chmod line must show actual 755, expected 555, and MISMATCH: {chmod_line}"
  );
}

// TC-517 (T03): pinned install, autoUpdates flipped to true → flagged mismatch
#[ cfg( unix ) ]
#[ test ]
fn tc517_status_lock_autoupdates_drift_flagged()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_pinned_settings( dir.path(), "2.1.78", "true" ); // drifted — pinned expects false
  write_versions_dir( dir.path(), 0o555 );

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let auto_updates_line = text.lines().find( | l | l.contains( "autoUpdates:" ) )
    .unwrap_or_else( || panic!( "no autoUpdates line in output: {text}" ) );
  assert!(
    auto_updates_line.contains( "MISMATCH" ),
    "autoUpdates line must show MISMATCH when flipped to true: {auto_updates_line}"
  );
}

// TC-518 (T04): unpinned (no preference set) install → Lock: section, all compliant
#[ cfg( unix ) ]
#[ test ]
fn tc518_status_lock_unpinned_all_compliant()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );
  write_versions_dir( dir.path(), 0o755 );

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Lock:" ), "missing Lock: section for unpinned install: {text}" );
  assert!( !text.contains( "MISMATCH" ), "expected no mismatch for compliant unpinned install: {text}" );
}

// TC-519 (T05): v::0 / v::1 output unchanged by the Lock: feature (no regression)
#[ test ]
fn tc519_status_v0_v1_unchanged_by_lock_feature()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out0 = run_clv_with_env( &[ ".status", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out0, 0 );
  let text0 = stdout( &out0 );
  assert!( !text0.contains( "Lock:" ), "v::0 must not show Lock: section: {text0}" );
  let lines0 : Vec< &str > = text0.lines().collect();
  assert_eq!( lines0.len(), 3, "v::0 must still produce exactly 3 lines: {text0:?}" );

  let out1 = run_clv_with_env( &[ ".status", "v::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out1, 0 );
  let text1 = stdout( &out1 );
  assert!( !text1.contains( "Lock:" ), "v::1 must not show Lock: section: {text1}" );
  assert!(
    text1.contains( "Version:" ) && text1.contains( "Processes:" ) && text1.contains( "Account:" ),
    "v::1 must still show labeled lines: {text1}"
  );
}

// TC-520 (T06): v::3 continues to exit 1 as out-of-range (IT-11 regression check)
#[ test ]
fn tc520_status_v3_out_of_range_exits_1()
{
  let out = run_clv( &[ ".status", "v::3" ] );
  assert_exit( &out, 1 );
}

// TC-521 (T07): format::json, pinned, all compliant → "lock" object present
#[ cfg( unix ) ]
#[ test ]
fn tc521_status_lock_json_object_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_pinned_settings( dir.path(), "2.1.78", "false" );
  write_versions_dir( dir.path(), 0o555 );

  let out = run_clv_with_env( &[ ".status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"lock\"" ), "missing lock object in JSON: {text}" );
  for key in [ "autoUpdates", "autoUpdatesChannel", "minimumVersion", "env.DISABLE_AUTOUPDATER", "env.DISABLE_UPDATES", "chmod" ]
  {
    assert!( text.contains( key ), "lock JSON object missing key {key}: {text}" );
  }
  assert!( text.contains( "\"compliant\":true" ), "expected compliant:true entries in lock JSON: {text}" );
}

// TC-522 (bugfix, MAAV-found): unpinned install with the versions directory
// never created (genuinely fresh install, nothing ever run through
// `.version.install`) → the `chmod` row must report "absent" rather than a
// false `MISMATCH`. Regression test for the case `write_versions_dir` is
// deliberately NOT called — T04/TC-518 always pre-creates the directory, so
// it never exercised this branch.
#[ cfg( unix ) ]
#[ test ]
fn tc522_status_lock_chmod_absent_dir_not_flagged()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );
  // Deliberately no write_versions_dir( .. ) call — directory does not exist.

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let chmod_line = text.lines().find( | l | l.contains( "chmod" ) )
    .unwrap_or_else( || panic!( "no chmod line in output: {text}" ) );
  assert!(
    chmod_line.contains( "absent" ) && !chmod_line.contains( "MISMATCH" ),
    "chmod line must report absent without a false mismatch when the versions dir doesn't exist: {chmod_line}"
  );
  assert!( !text.contains( "MISMATCH" ), "expected no mismatch anywhere for a fresh install: {text}" );
}

// ADVERSARIAL PROBE (temporary, MAAV Round 1) — project `.claude/settings.json`
// in `cwd` shadows the nested `env.DISABLE_AUTOUPDATER`/`env.DISABLE_UPDATES`
// rows via `config_resolve::resolve()`'s Project layer, which sits ABOVE the
// User (home) layer. Unpinned home state (no env keys) + a project config
// declaring env.DISABLE_AUTOUPDATER=1 should, if the Lock: feature is
// comparing against true home-level lock state, still see "actual: absent"
// for that row (unpinned expects None/absent) OR should legitimately flag it
// -- either way we need to observe what actually happens on the 2 keys that
// route through resolve() vs the 3 keys that route through direct get_setting().
#[ cfg( unix ) ]
#[ test ]
fn probe_status_lock_project_config_shadow()
{
  let home_dir = TempDir::new().unwrap();
  let home = home_dir.path().to_str().unwrap();
  // Unpinned home: no preference, no lock keys at all.
  write_settings( home_dir.path(), &[] );
  write_versions_dir( home_dir.path(), 0o755 ); // compliant unpinned chmod

  // Project dir (acts as cwd) with its own .claude/settings.json declaring
  // env.DISABLE_AUTOUPDATER=1 -- simulates a repo-level config a user's cwd
  // might have for unrelated reasons (team policy, etc.), while the user's
  // real HOME/pin-state is genuinely unpinned & compliant.
  let project_dir = TempDir::new().unwrap();
  let proj_claude = project_dir.path().join( ".claude" );
  std::fs::create_dir_all( &proj_claude ).unwrap();
  std::fs::write(
    proj_claude.join( "settings.json" ),
    "{\n  \"env\": {\"DISABLE_AUTOUPDATER\": \"1\"}\n}",
  ).unwrap();

  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  let out = std::process::Command::new( bin )
    .args( [ ".status", "v::2" ] )
    .env( "HOME", home )
    .current_dir( project_dir.path() )
    .output()
    .expect( "failed to execute claude_version binary" );

  let text = stdout( &out );
  eprintln!( "=== PROBE OUTPUT ===\n{text}\n=== END PROBE OUTPUT ===" );
  // Deliberately no assertion yet -- this is observational. The eprintln
  // output is what we need to inspect.
}

// ADVERSARIAL PROBE 2 (temporary, MAAV Round 1) -- a genuinely pinned install
// (chmod 555 on the versions dir, real pin state) whose settings.json becomes
// corrupted/truncated AFTER the pin was applied (disk write torn by crash,
// power loss, concurrent writer, etc.). `get_setting()` returns `Err` for
// invalid JSON; `status.rs` collapses that `Err` via `.ok().flatten()` to
// `None` for every settings-derived actual, AND `read_preferred_version()`
// (which also calls `get_setting` and propagates `Err` through `?`) returns
// `None`, forcing `is_pinned = false`. If that reasoning is right, `chmod`
// gets compared against the UNPINNED expectation (755) even though the real
// on-disk mode is still 555 (correctly locked, unchanged) -- producing a
// false MISMATCH on a directory that is not actually out of compliance.
#[ cfg( unix ) ]
#[ test ]
fn probe_status_lock_corrupted_settings_pinned_chmod()
{
  let home_dir = TempDir::new().unwrap();
  let home = home_dir.path().to_str().unwrap();

  // Real pin state on disk: chmod 555 (as lock_version() would have set for a
  // pinned install).
  write_versions_dir( home_dir.path(), 0o555 );

  // Settings file exists but is corrupted (truncated mid-write) --
  // NOT valid JSON, NOT even a parseable partial object.
  let claude_dir = home_dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write(
    claude_dir.join( "settings.json" ),
    "{\n  \"preferredVersionSpec\": \"stable\",\n  \"preferredVersionResolved\": \"2.1.7", // truncated
  ).unwrap();

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  let text = stdout( &out );
  eprintln!( "=== PROBE 2 OUTPUT (exit={:?}) ===\n{text}\n=== END PROBE 2 OUTPUT ===",
    out.status.code() );
}
