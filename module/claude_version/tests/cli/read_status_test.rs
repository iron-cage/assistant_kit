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
//! | TC-523 | settings.json corrupted, versions dir genuinely locked (555) → all 6 rows UNVERIFIABLE, no false mismatch | P | 0 |
//! | TC-524 | `format::json`, settings.json corrupted → `"compliant":null` for every key, never `false` | P | 0 |
//! | TC-525 | settings.json permission-denied (mode 000), versions dir genuinely locked (555) → all 6 rows UNVERIFIABLE, no false mismatch | P | 0 |
//! | TC-526 | pinned install, `autoUpdatesChannel` drifted → flagged mismatch | P | 0 |
//! | TC-527 | pinned install, `minimumVersion` drifted → flagged mismatch | P | 0 |
//! | TC-528 | pinned install, `env.DISABLE_AUTOUPDATER` drifted → flagged mismatch | P | 0 |
//! | TC-529 | pinned install, `env.DISABLE_UPDATES` drifted → flagged mismatch | P | 0 |
//! | TC-530 | install interrupted after lock applied but before preference stored → all 6 rows report TRUE mismatch, none UNVERIFIABLE | P | 0 |
//! | TC-531 | unpinned install, `autoUpdates` explicit `"true"` → compliant, no mismatch | P | 0 |
//! | TC-532 | unpinned install, `autoUpdates` drifted to `"false"` → flagged mismatch | P | 0 |

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
/// `env.DISABLE_AUTOUPDATER`/`env.DISABLE_UPDATES` pair) individually
/// parameterized, so a test can plant drift in exactly one key while keeping
/// the other 4 compliant — isolating which row's comparison logic is under
/// test, the same discipline `tc516`/`tc517` already apply to `chmod`/`autoUpdates`.
///
/// # Panics
///
/// Panics if the directory cannot be created or the file cannot be written.
#[ allow( clippy::too_many_arguments ) ]
fn write_pinned_settings_with_drift(
  home_dir             : &std::path::Path,
  resolved_version     : &str,
  auto_updates         : &str,
  auto_updates_channel : &str,
  minimum_version      : &str,
  disable_autoupdater  : &str,
  disable_updates      : &str,
)
{
  let dir = home_dir.join( ".claude" );
  std::fs::create_dir_all( &dir ).unwrap();
  let json = format!(
    "{{\n  \"preferredVersionSpec\": \"stable\",\n  \"preferredVersionResolved\": \"{resolved_version}\",\n  \"autoUpdates\": \"{auto_updates}\",\n  \"autoUpdatesChannel\": \"{auto_updates_channel}\",\n  \"minimumVersion\": \"{minimum_version}\",\n  \"env\": {{\"DISABLE_AUTOUPDATER\": \"{disable_autoupdater}\", \"DISABLE_UPDATES\": \"{disable_updates}\"}}\n}}"
  );
  std::fs::write( dir.join( "settings.json" ), json ).unwrap();
}

/// Write a fully-compliant pinned-install `settings.json` fixture, with only
/// `auto_updates` parameterized so tests can simulate drift in that one key.
///
/// # Panics
///
/// Panics if the directory cannot be created or the file cannot be written.
fn write_pinned_settings( home_dir : &std::path::Path, resolved_version : &str, auto_updates : &str )
{
  write_pinned_settings_with_drift(
    home_dir, resolved_version, auto_updates, "stable", resolved_version, "1", "1",
  );
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

/// Extract the `"key":{...}` entry substring for one lock key from the
/// single-line `"lock"` JSON object, so a per-key assertion can't be
/// silently satisfied by a sibling key's entry (each entry has no nested
/// braces, so the first `}` after the key's opening brace closes it).
///
/// # Panics
///
/// Panics if `key`'s entry is not found in `json`.
fn lock_json_entry<'a>( json : &'a str, key : &str ) -> &'a str
{
  let needle = format!( "\"{key}\":{{" );
  let start = json.find( &needle ).unwrap_or_else( || panic!( "key {key} not found in {json}" ) );
  let end = json[ start.. ].find( '}' )
    .unwrap_or_else( || panic!( "unterminated entry for {key} in {json}" ) ) + start;
  &json[ start..=end ]
}

// TC-523 (bugfix, MAAV-found): settings.json exists but could not be read
// (invalid JSON), on an install whose versions directory is genuinely locked
// (555). `is_pinned` silently degrades to `false` when settings.json can't
// be read (see `read_preferred_version`, which swallows the read error via
// `.ok()`), so without this fix a genuinely-pinned-and-locked install would
// be compared against the unpinned (755) expectation and misreport a false
// `MISMATCH` on every settings-derived row. Every row must instead report
// `UNVERIFIABLE`.
#[ cfg( unix ) ]
#[ test ]
fn tc523_status_lock_corrupted_settings_reports_unverifiable()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{ not valid json" ).unwrap();
  write_versions_dir( dir.path(), 0o555 ); // genuinely locked

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Lock:" ), "missing Lock: section: {text}" );
  assert!(
    text.contains( "could not be read" ),
    "expected explanatory note about unreadable settings.json: {text}"
  );
  assert!( !text.contains( "MISMATCH" ), "corrupted settings must never assert a MISMATCH: {text}" );
  for key in [ "autoUpdates", "autoUpdatesChannel", "minimumVersion", "env.DISABLE_AUTOUPDATER", "env.DISABLE_UPDATES", "chmod" ]
  {
    let line = text.lines().find( | l | l.contains( &format!( "{key}:" ) ) )
      .unwrap_or_else( || panic!( "no {key} line in output: {text}" ) );
    assert!( line.contains( "UNVERIFIABLE" ), "{key} line must report UNVERIFIABLE: {line}" );
  }
}

// TC-524: format::json variant of TC-523 — `"compliant"` must serialize as
// JSON `null` (not `false`) for every one of the 6 keys individually (not
// merely "at least one key shows null, none shows false" — a whole-blob
// check would still pass if 5 of 6 rows regressed to `true`/`false` while
// only one correctly showed `null`), so machine consumers don't mistake an
// unverifiable row for a confirmed mismatch.
#[ cfg( unix ) ]
#[ test ]
fn tc524_status_lock_json_corrupted_settings_compliant_null()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), "{ not valid json" ).unwrap();
  write_versions_dir( dir.path(), 0o555 );

  let out = run_clv_with_env( &[ ".status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"lock\"" ), "missing lock object in JSON: {text}" );
  for key in [ "autoUpdates", "autoUpdatesChannel", "minimumVersion", "env.DISABLE_AUTOUPDATER", "env.DISABLE_UPDATES", "chmod" ]
  {
    let entry = lock_json_entry( &text, key );
    assert!(
      entry.contains( "\"compliant\":null" ),
      "{key} entry must report compliant:null when settings.json is corrupted, got: {entry}"
    );
  }
}

// TC-525 (bugfix, MAAV-found): settings.json exists, is valid JSON, and is
// genuinely pinned+locked, but becomes unreadable due to a permissions error
// (mode 000) rather than a parse failure. `read_preferred_version` swallows
// ANY read error via `.ok()`, not just parse failures, so a fix that only
// special-cased `ErrorKind::InvalidData` would still misreport a false
// `MISMATCH` here. Must be treated identically to TC-523: every row
// `UNVERIFIABLE`. Permissions are restored before assertions so `TempDir`
// cleanup succeeds even when a panic occurs mid-test.
#[ cfg( unix ) ]
#[ test ]
fn tc525_status_lock_unreadable_settings_permission_denied_reports_unverifiable()
{
  use std::os::unix::fs::PermissionsExt;

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_pinned_settings( dir.path(), "2.1.78", "false" ); // genuinely pinned, valid JSON
  write_versions_dir( dir.path(), 0o555 ); // genuinely locked

  let settings_file = dir.path().join( ".claude" ).join( "settings.json" );
  std::fs::set_permissions( &settings_file, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );

  // Restore before any assertion so TempDir cleanup can delete the directory.
  std::fs::set_permissions( &settings_file, std::fs::Permissions::from_mode( 0o644 ) ).unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "MISMATCH" ), "unreadable settings must never assert a MISMATCH: {text}" );
  for key in [ "autoUpdates", "autoUpdatesChannel", "minimumVersion", "env.DISABLE_AUTOUPDATER", "env.DISABLE_UPDATES", "chmod" ]
  {
    let line = text.lines().find( | l | l.contains( &format!( "{key}:" ) ) )
      .unwrap_or_else( || panic!( "no {key} line in output: {text}" ) );
    assert!(
      line.contains( "UNVERIFIABLE" ),
      "{key} line must report UNVERIFIABLE when settings.json is permission-denied: {line}"
    );
  }
}

// TC-526 (MAAV-found gap): `write_pinned_settings` hardcoded `autoUpdatesChannel`
// to the always-correct "stable", so no test could ever distinguish a working
// comparison from a `disable_autoupdater_status`-style hardcoded-Compliant bug
// in this row. Plants drift in `autoUpdatesChannel` alone, mirroring `tc517`'s
// discipline for `autoUpdates`.
#[ cfg( unix ) ]
#[ test ]
fn tc526_status_lock_autoupdates_channel_drift_flagged()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_pinned_settings_with_drift( dir.path(), "2.1.78", "false", "beta", "2.1.78", "1", "1" ); // drifted — pinned expects stable
  write_versions_dir( dir.path(), 0o555 );

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let line = text.lines().find( | l | l.contains( "autoUpdatesChannel:" ) )
    .unwrap_or_else( || panic!( "no autoUpdatesChannel line in output: {text}" ) );
  assert!( line.contains( "MISMATCH" ), "autoUpdatesChannel line must show MISMATCH when drifted: {line}" );
}

// TC-527 (MAAV-found gap): same rationale as TC-526, for `minimumVersion`.
#[ cfg( unix ) ]
#[ test ]
fn tc527_status_lock_minimum_version_drift_flagged()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_pinned_settings_with_drift( dir.path(), "2.1.78", "false", "stable", "2.1.70", "1", "1" ); // drifted — pinned expects 2.1.78
  write_versions_dir( dir.path(), 0o555 );

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let line = text.lines().find( | l | l.contains( "minimumVersion:" ) )
    .unwrap_or_else( || panic!( "no minimumVersion line in output: {text}" ) );
  assert!( line.contains( "MISMATCH" ), "minimumVersion line must show MISMATCH when drifted: {line}" );
}

// TC-528 (MAAV-found gap): same rationale as TC-526, for `env.DISABLE_AUTOUPDATER`.
#[ cfg( unix ) ]
#[ test ]
fn tc528_status_lock_disable_autoupdater_drift_flagged()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_pinned_settings_with_drift( dir.path(), "2.1.78", "false", "stable", "2.1.78", "0", "1" ); // drifted — pinned expects 1
  write_versions_dir( dir.path(), 0o555 );

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let line = text.lines().find( | l | l.contains( "env.DISABLE_AUTOUPDATER:" ) )
    .unwrap_or_else( || panic!( "no env.DISABLE_AUTOUPDATER line in output: {text}" ) );
  assert!( line.contains( "MISMATCH" ), "env.DISABLE_AUTOUPDATER line must show MISMATCH when drifted: {line}" );
}

// TC-529 (MAAV-found gap): same rationale as TC-526, for `env.DISABLE_UPDATES`.
#[ cfg( unix ) ]
#[ test ]
fn tc529_status_lock_disable_updates_drift_flagged()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_pinned_settings_with_drift( dir.path(), "2.1.78", "false", "stable", "2.1.78", "1", "0" ); // drifted — pinned expects 1
  write_versions_dir( dir.path(), 0o555 );

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let line = text.lines().find( | l | l.contains( "env.DISABLE_UPDATES:" ) )
    .unwrap_or_else( || panic!( "no env.DISABLE_UPDATES line in output: {text}" ) );
  assert!( line.contains( "MISMATCH" ), "env.DISABLE_UPDATES line must show MISMATCH when drifted: {line}" );
}

// TC-530 (regression, MAAV-found B1): simulates a crash during `perform_install()`
// after `store_preferred_version()` already ran (the corrected call order — see
// Fix(MAAV-found) in `commands/version.rs`) but before `lock_version()` applied
// the mechanism. `preferredVersionSpec`/`preferredVersionResolved` are recorded
// while every lock-mechanism key is still absent and the versions dir is still
// at its unpinned default mode — the state a real interrupted install leaves on
// disk. Must report a MISMATCH on every row (a TRUE one: the recorded intent
// genuinely isn't enforced yet), never silently `Compliant` and never
// `UNVERIFIABLE` (settings.json is fully valid and readable here).
#[ cfg( unix ) ]
#[ test ]
fn tc530_status_lock_interrupted_install_reports_true_mismatch()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[
    ( "preferredVersionSpec", "stable" ),
    ( "preferredVersionResolved", "2.1.78" ),
  ] );
  write_versions_dir( dir.path(), 0o755 ); // mechanism not yet applied — still unpinned default

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "UNVERIFIABLE" ), "settings.json is fully readable; no row should be UNVERIFIABLE: {text}" );
  for key in [ "autoUpdates", "autoUpdatesChannel", "minimumVersion", "env.DISABLE_AUTOUPDATER", "env.DISABLE_UPDATES", "chmod" ]
  {
    let line = text.lines().find( | l | l.contains( &format!( "{key}:" ) ) )
      .unwrap_or_else( || panic!( "no {key} line in output: {text}" ) );
    assert!(
      line.contains( "MISMATCH" ),
      "{key} line must report a true MISMATCH for an interrupted install (mechanism not yet applied): {line}"
    );
  }
}

// TC-531 (regression, MAAV-found A8): the unpinned `autoUpdates` comparison
// (`status.rs:126`) is `actual.is_none() || actual == Some("true")` — a
// two-way OR. TC-518 only exercises the `is_none()` disjunct (empty
// settings). Without this test, deleting the `== Some("true")` disjunct
// entirely would not fail any existing test, since no test ever sets
// `autoUpdates` to the literal string `"true"` while unpinned.
#[ cfg( unix ) ]
#[ test ]
fn tc531_status_lock_unpinned_autoupdates_explicit_true_compliant()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "autoUpdates", "true" ) ] );
  write_versions_dir( dir.path(), 0o755 );

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let line = text.lines().find( | l | l.contains( "autoUpdates:" ) )
    .unwrap_or_else( || panic!( "no autoUpdates line in output: {text}" ) );
  assert!( !line.contains( "MISMATCH" ), "autoUpdates=true while unpinned must be compliant, not MISMATCH: {line}" );
}

// TC-532 (regression, MAAV-found A8): companion to TC-531 — a genuine
// unpinned drift (`autoUpdates` flipped to `"false"` with no preference
// stored) must still be flagged, confirming the OR's `is_none()` disjunct
// doesn't accidentally swallow every value.
#[ cfg( unix ) ]
#[ test ]
fn tc532_status_lock_unpinned_autoupdates_false_drift_flagged()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "autoUpdates", "false" ) ] );
  write_versions_dir( dir.path(), 0o755 );

  let out = run_clv_with_env( &[ ".status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let line = text.lines().find( | l | l.contains( "autoUpdates:" ) )
    .unwrap_or_else( || panic!( "no autoUpdates line in output: {text}" ) );
  assert!( line.contains( "MISMATCH" ), "autoUpdates=false while unpinned must be flagged MISMATCH: {line}" );
}
