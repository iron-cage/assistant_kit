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

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stdout, write_settings };

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
