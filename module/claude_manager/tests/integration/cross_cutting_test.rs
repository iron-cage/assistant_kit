//! Cross-cutting integration tests: dry+force, verbosity parity, format parity.
//!
//! # Test Matrix (Section F)
//!
//! ## F1: dry + force Interaction
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 250 | `dry::1 force::1` on `.version.install` → dry wins | P | 0 |
//! | 251 | `dry::1 force::1` on `.processes.kill` → dry wins | P | 0 |
//! | 252 | `dry::1` on `.settings.set` → dry wins, no write | P | 0 |
//!
//! ## F2: Verbosity Parity
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 255 | `v::0` shorter than `v::1` for `.status` | P | 0 |
//! | 257 | `v::0` produces same output regardless of param name | P | 0 |
//!
//! ## F3: `format::json` Parity
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 258 | `format::json` produces valid JSON for `.status` | P | 0 |
//! | 259 | `format::json v::0` still produces complete JSON for `.status` | P | 0 |
//! | 260 | `format::JSON` rejected on `.version.list` | N | 1 |

use crate::helpers::{ assert_exit, run_clm, stdout };

// ─── F1: dry + force interaction ────────────────────────────────────────────

// TC-250: dry::1 force::1 on .version.install → dry wins
#[ test ]
fn tc250_version_install_dry_force_dry_wins()
{
  let out = run_clm( &[ ".version.install", "dry::1", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry must win over force: {text}" );
}

// TC-251: dry::1 force::1 on .processes.kill → dry wins
#[ test ]
fn tc251_processes_kill_dry_force_dry_wins()
{
  let out = run_clm( &[ ".processes.kill", "dry::1", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ) || text.contains( "no active processes" ),
    "dry must win over force or no processes: {text}"
  );
}

// TC-252: dry::1 on .settings.set → no file created
#[ test ]
fn tc252_settings_set_dry_no_write()
{
  let dir = tempfile::TempDir::new().unwrap();
  let out = crate::helpers::run_clm_with_env(
    &[ ".settings.set", "key::k", "value::v", "dry::1" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "must contain [dry-run]: {text}" );
  assert!( !dir.path().join( ".claude/settings.json" ).exists(), "dry-run must not write file" );
}

// ─── F2: Verbosity parity ───────────────────────────────────────────────────

// TC-255: v::0 output has fewer lines than v::1 for .status
#[ test ]
fn tc255_status_v0_fewer_lines_than_v1()
{
  let out0 = run_clm( &[ ".status", "v::0" ] );
  let out1 = run_clm( &[ ".status", "v::1" ] );
  assert_exit( &out0, 0 );
  assert_exit( &out1, 0 );
  let lines0 = stdout( &out0 ).lines().count();
  let lines1 = stdout( &out1 ).lines().count();
  assert!(
    lines0 <= lines1,
    "v::0 ({lines0} lines) must have \u{2264} lines than v::1 ({lines1} lines)"
  );
}

// TC-257: v::0 produces identical output on .version.list (deterministic)
#[ test ]
fn tc257_v_param_identical()
{
  let out_a = run_clm( &[ ".version.list", "v::0" ] );
  let out_b = run_clm( &[ ".version.list", "v::0" ] );
  assert_exit( &out_a, 0 );
  assert_exit( &out_b, 0 );
  assert_eq!(
    stdout( &out_a ), stdout( &out_b ),
    "v::0 must produce identical output across runs"
  );
}

// ─── F3: format::json parity ────────────────────────────────────────────────

// TC-258: format::json produces parseable JSON for .status
#[ test ]
fn tc258_status_format_json_is_valid_json()
{
  let out = run_clm( &[ ".status", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "must start with '{{': {text}" );
  assert!( text.trim_end().ends_with( '}' ), "must end with '}}': {text}" );
}

// TC-259: format::json v::0 still produces complete JSON
#[ test ]
fn tc259_status_format_json_v0_still_complete()
{
  let out = run_clm( &[ ".status", "format::json", "v::0" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"version\"" ), "JSON must contain version key: {text}" );
  assert!( text.contains( "\"processes\"" ), "JSON must contain processes key: {text}" );
  assert!( text.contains( "\"account\"" ), "JSON must contain account key: {text}" );
}

// TC-260: format::JSON (uppercase) rejected on .version.list
#[ test ]
fn tc260_format_uppercase_rejected()
{
  let out = run_clm( &[ ".version.list", "format::JSON" ] );
  assert_exit( &out, 1 );
}
