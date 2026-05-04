//! Integration tests: H (Help), AL (Account List), ASTAT (Account Status).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### H — Help
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | h01 | `h01_dot_shows_help` | `.` → shows .account.list | P |
//! | h02 | `h02_help_lists_all_registered_commands` | `.help` → all commands listed | P |
//! | h03 | `h03_help_hides_dot` | `.help` → bare `.` not listed | P |
//! | h04 | `h04_help_exits_0` | `.help` → exit 0 | P |
//! | h05 | `h05_no_args_shows_help` | no args → help | P |
//! | h06 | `h06_double_dash_help` | `--help` → exit 1 (POSIX flags not supported) | N |
//! | h07 | `h07_unknown_command_exits_1` | `.nonexistent` → exit 1 + stderr | N |
//!
//! ### AL — Account List
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | al01 | `al01_list_text_v0_bare_names` | `v::0` → bare names only | P |
//! | al02 | `al02_list_text_v1_active_marker` | `v::1` → active account marked | P |
//! | al03 | `al03_list_text_v2_metadata` | `v::2` → metadata shown | P |
//! | al04 | `al04_list_json` | `format::json` → valid JSON array | P |
//! | al05 | `al05_list_absent_dir_text` | no accounts dir → empty output | P |
//! | al06 | `al06_list_empty_dir_text` | empty accounts dir → empty output | P |
//! | al07 | `al07_list_absent_dir_json` | no accounts dir + json → empty array | P |
//! | al08 | `al08_list_single_active` | one active account → active marker shown | P |
//! | al09 | `al09_list_single_not_active` | one inactive account → no active marker | P |
//! | al10 | `al10_list_multi_none_active` | multiple accounts, none active | P |
//! | al11 | `al11_list_home_unset_exits_2` | HOME unset → exit 2 | N |
//! | al12 | `al12_list_home_empty_exits_2` | HOME="" → exit 2 | N |
//! | al13 | `al13_list_sorted_alphabetically` | multiple accounts → sorted by name | P |
//! | al14 | `al14_list_format_xml_exits_1` | `format::xml` → exit 1 | N |
//! | al15 | `al15_list_name_single_account_status_view` | `name::EMAIL` → single-account status output | P |
//! | al16 | `al16_list_name_not_found_exits_2` | `name::` not in store → exit 2 | N |
//! | al17 | `al17_list_name_invalid_exits_1` | `name::notanemail` → exit 1 | N |
//! | al18 | `al18_list_name_matches_account_status` | `list name::X` output == `status name::X` output | P |
//!
//! ### ASTAT — Account Status
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | astat01 | `astat01_no_active_file_exits_2` | no _active file → exit 2 | N |
//! | astat02 | `astat02_empty_active_file_exits_2` | empty _active file → exit 2 | N |
//! | astat03 | `astat03_valid_token_shows_valid` | valid token → "Valid" in output | P |
//! | astat04 | `astat04_expired_token_shows_expired` | expired token → "Expired" | P |
//! | astat05 | `astat05_near_expiry_token_shows_expiring_in` | near-expiry → "Expiring" | P |
//! | astat06 | `astat06_missing_credentials_shows_unknown` | no creds file → "unknown" | N |
//! | astat07 | `astat07_v0_bare_name_and_status` | v::0 → bare name + status only | P |
//! | astat08 | `astat08_v1_default_shows_labels` | v::1 → labeled output | P |
//! | astat09 | `astat09_v2_shows_expires_line` | v::2 → Expires: line shown | P |
//! | astat10 | `astat10_json_format_returns_object` | format::json → JSON object | P |
//! | astat11 | `astat11_v1_shows_sub_tier_email_org` | v::1 → Sub/Tier/Email/Org shown | P |
//! | astat12 | `astat12_v1_empty_sub_in_creds_shows_n_a` | empty sub in creds → N/A | P |

use crate::helpers::{
  run_cs, run_cs_with_env, run_cs_without_home,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_claude_json,
  FAR_FUTURE_MS, PAST_MS, near_future_ms,
};
use tempfile::TempDir;

// ── H: Help commands ──────────────────────────────────────────────────────────

#[ test ]
fn h01_dot_shows_help()
{
  let out = run_cs( &[ "." ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".account.list" ), "help must list .account.list, got:\n{text}" );
}

#[ test ]
fn h02_help_lists_all_registered_commands()
{
  let out = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for cmd in &[
    ".account.list",
    ".account.status",
    ".account.save",
    ".account.switch",
    ".account.delete",
    ".token.status",
    ".paths",
    ".usage",
    ".credentials.status",
  ]
  {
    assert!( text.contains( cmd ), "help must list {cmd}, got:\n{text}" );
  }
}

#[ test ]
fn h03_help_hides_dot()
{
  let out = run_cs( &[ ".help" ] );
  let text = stdout( &out );
  // `.` is registered with `hidden_from_list: true` — must not appear as a listed command.
  // `.help` IS visible (auto-registered by unilang) — that's expected.
  let lines : Vec< &str > = text.lines()
    .filter( | l | l.trim().starts_with( '.' ) )
    .collect();
  for line in &lines
  {
    let cmd = line.split_whitespace().next().unwrap_or( "" );
    assert!( cmd != ".", "listing should not include bare '.', got line: {line}" );
  }
}

#[ test ]
fn h04_help_exits_0()
{
  let out = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn h05_no_args_shows_help()
{
  let out = run_cs( &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".account.list" ), "no-args help must list commands, got:\n{text}" );
}

#[ test ]
fn h06_double_dash_help()
{
  // POSIX flags (--help, -h) are not supported — use `.help` command instead.
  let out = run_cs( &[ "--help" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "unexpected flag" ), "--help must produce unexpected flag error, got:\n{err}" );
}

#[ test ]
fn h07_unknown_command_exits_1()
{
  let out = run_cs( &[ ".nonexistent" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "unknown command must produce stderr" );
}

// ── AL: Account List ──────────────────────────────────────────────────────────

#[ test ]
fn al01_list_text_v0_bare_names()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@home.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.list", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().collect();
  assert_eq!( lines.len(), 2, "v::0 must produce 2 lines, got:\n{text}" );
  assert_eq!( lines[ 0 ], "alice@acme.com" );
  assert_eq!( lines[ 1 ], "alice@home.com" );
}

#[ test ]
fn al02_list_text_v1_active_marker()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@home.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com *" ), "active must have marker, got:\n{text}" );
  // alice@home.com should NOT have marker
  let inactive_line = text.lines().find( | l | l.starts_with( "alice@home.com" ) ).unwrap();
  assert!( !inactive_line.contains( '*' ), "inactive must not have marker, got: {inactive_line}" );
}

#[ test ]
fn al03_list_text_v2_metadata()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.list", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "pro" ), "v::2 must show subscription type, got:\n{text}" );
  assert!( text.contains( "standard" ), "v::2 must show rate limit tier, got:\n{text}" );
  assert!( text.contains( "active" ), "v::2 must show active indicator, got:\n{text}" );
}

#[ test ]
fn al04_list_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.list", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.starts_with( '[' ), "JSON must start with '[', got:\n{text}" );
  assert!( text.contains( "\"name\":\"alice@acme.com\"" ), "JSON must contain name, got:\n{text}" );
  assert!( text.contains( "\"is_active\":true" ), "JSON must contain is_active, got:\n{text}" );
}

#[ test ]
fn al05_list_absent_dir_text()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No .claude directory at all
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "no accounts configured" ), "absent accounts dir must say no accounts, got:\n{text}" );
}

#[ test ]
fn al06_list_empty_dir_text()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "no accounts configured" ), "empty accounts dir must say no accounts, got:\n{text}" );
}

#[ test ]
fn al07_list_absent_dir_json()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.list", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!( text.trim(), "[]", "empty JSON list expected, got:\n{text}" );
}

#[ test ]
fn al08_list_single_active()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "solo@example.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "solo@example.com" ), "must list the account, got:\n{text}" );
  assert!( text.contains( '*' ), "active must have marker, got:\n{text}" );
}

#[ test ]
fn al09_list_single_not_active()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "solo@example.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "solo@example.com" ), "must list the account, got:\n{text}" );
  assert!( !text.contains( '*' ), "inactive must not have marker, got:\n{text}" );
}

#[ test ]
fn al10_list_multi_none_active()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "b@acme.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.list" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( '*' ), "no active marker expected, got:\n{text}" );
}

#[ test ]
fn al11_list_home_unset_exits_2()
{
  let out = run_cs_without_home( &[ ".account.list" ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn al12_list_home_empty_exits_2()
{
  let out = run_cs_with_env( &[ ".account.list" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn al13_list_sorted_alphabetically()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "zebra@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alpha@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "mid@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.list", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().map( str::trim ).collect();
  assert_eq!( lines, vec![ "alpha@acme.com", "mid@acme.com", "zebra@acme.com" ] );
}

#[ test ]
fn al14_list_format_xml_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.list", "format::xml" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn al15_list_name_single_account_status_view()
{
  // al15: .account.list name::X → single-account status view (Account:/Token: labels).
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.list", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "must show account name, got:\n{text}" );
  assert!( text.contains( "Account:" ), "must show Account: label, got:\n{text}" );
  assert!( text.contains( "Token:" ), "must show Token: label, got:\n{text}" );
  assert!( text.contains( "valid" ), "must show valid token state, got:\n{text}" );
}

#[ test ]
fn al16_list_name_not_found_exits_2()
{
  // al16: .account.list name::ghost@example.com when that account doesn't exist → exit 2.
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.list", "name::ghost@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "not found" ) || err.contains( "ghost@example.com" ),
    "must report account not found, got:\n{err}" );
}

#[ test ]
fn al17_list_name_invalid_exits_1()
{
  // al17: .account.list name::notanemail (not an email address) → exit 1.
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.list", "name::notanemail" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn al18_list_name_matches_account_status()
{
  // al18: .account.list name::X and .account.status name::X produce identical output.
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let env = &[ ( "HOME", home ) ];
  let list_out   = run_cs_with_env( &[ ".account.list",   "name::alice@home.com" ], env );
  let status_out = run_cs_with_env( &[ ".account.status", "name::alice@home.com" ], env );
  assert_exit( &list_out,   0 );
  assert_exit( &status_out, 0 );
  assert_eq!(
    stdout( &list_out ),
    stdout( &status_out ),
    ".account.list name:: and .account.status name:: must produce identical output",
  );
}

// ── ASTAT: Account Status ─────────────────────────────────────────────────────

#[ test ]
fn astat01_no_active_file_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // no _active file created

  let out = run_cs_with_env( &[ ".account.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "no active account" ), "must say no active account, got:\n{err}" );
  assert!( stdout( &out ).is_empty(), "stdout must be empty, got:\n{}", stdout( &out ) );
}

#[ test ]
fn astat02_empty_active_file_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  std::fs::write( credential_store.join( "_active" ), "" ).unwrap();

  let out = run_cs_with_env( &[ ".account.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "no active account" ), "empty _active must error, got:\n{err}" );
}

#[ test ]
fn astat03_valid_token_shows_valid()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "must show account name, got:\n{text}" );
  assert!( text.contains( "valid" ), "must show valid token state, got:\n{text}" );
}

#[ test ]
fn astat04_expired_token_shows_expired()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", PAST_MS, true );
  write_credentials( dir.path(), "pro", "standard", PAST_MS );

  let out = run_cs_with_env( &[ ".account.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "must show account name, got:\n{text}" );
  assert!( text.contains( "expired" ), "must show expired token state, got:\n{text}" );
}

#[ test ]
fn astat05_near_expiry_token_shows_expiring_in()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let near = near_future_ms();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", near, true );
  write_credentials( dir.path(), "pro", "standard", near );

  let out = run_cs_with_env( &[ ".account.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "must show account name, got:\n{text}" );
  assert!( text.contains( "expiring in" ), "must show expiring-in state, got:\n{text}" );
}

#[ test ]
fn astat06_missing_credentials_shows_unknown()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write _active but NO .credentials.json
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  std::fs::write( credential_store.join( "_active" ), "alice@acme.com" ).unwrap();

  let out = run_cs_with_env( &[ ".account.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "must show account name, got:\n{text}" );
  assert!( text.contains( "unknown" ), "missing credentials must show unknown token, got:\n{text}" );
}

#[ test ]
fn astat07_v0_bare_name_and_status()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().collect();
  assert_eq!( lines[ 0 ], "alice@acme.com", "v::0 line 0 must be bare name, got:\n{text}" );
  assert_eq!( lines[ 1 ], "valid", "v::0 line 1 must be bare token state, got:\n{text}" );
  assert!( !text.contains( "Account:" ), "v::0 must not have labels, got:\n{text}" );
  assert!( !text.contains( "Token:" ), "v::0 must not have labels, got:\n{text}" );
}

#[ test ]
fn astat08_v1_default_shows_labels()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Account: alice@acme.com" ), "v::1 must show Account: label, got:\n{text}" );
  assert!( text.contains( "Token:" ), "v::1 must show Token: label, got:\n{text}" );
  assert!( !text.contains( "Expires:" ), "v::1 must not show Expires: line, got:\n{text}" );
}

#[ test ]
fn astat09_v2_shows_expires_line()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Account: alice@acme.com" ), "v::2 must show Account: label, got:\n{text}" );
  assert!( text.contains( "Token:" ), "v::2 must show Token: label, got:\n{text}" );
  assert!( text.contains( "Expires:" ), "v::2 must show Expires: line, got:\n{text}" );
}

#[ test ]
fn astat10_json_format_returns_object()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.status", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim().starts_with( '{' ), "JSON must start with '{{', got:\n{text}" );
  assert!( text.contains( "\"account\":\"alice@acme.com\"" ), "JSON must contain account field, got:\n{text}" );
  assert!( text.contains( "\"token\":\"valid\"" ), "JSON must contain token field, got:\n{text}" );
}

// ── astat11: v::1 active path shows Sub + Tier + Email + Org ──────────────────

#[ test ]
fn astat11_v1_shows_sub_tier_email_org()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "alice@example.com", "Acme Corp" );

  let out = run_cs_with_env( &[ ".account.status" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Sub:" ),     "v::1 must show Sub: line, got:\n{text}" );
  assert!( text.contains( "Tier:" ),    "v::1 must show Tier: line, got:\n{text}" );
  assert!( text.contains( "pro" ),      "v::1 Sub must show subscription type, got:\n{text}" );
  assert!( text.contains( "standard" ), "v::1 Tier must show rate limit tier, got:\n{text}" );
  assert!( text.contains( "alice@example.com" ), "v::1 must show email, got:\n{text}" );
  assert!( text.contains( "Acme Corp" ),         "v::1 must show org, got:\n{text}" );
}

// ── astat12: empty subscriptionType in credentials.json → Sub: N/A ────────────

// test_kind: bug_reproducer(issue-empty-field-blank)
//
// Root Cause: `parse_string_field` returns `Some("")` for empty-string JSON fields.
//   `unwrap_or_else(|| "N/A")` fires only on `None` — `Some("")` bypasses it, so an
//   empty `subscriptionType` field produced a blank "Sub:     " line. A missing field
//   (None) correctly showed "N/A" — the two cases were inconsistently handled.
// Why Not Caught: All existing `astat` tests used non-empty subscription strings.
//   No test exercised credentials with `subscriptionType: ""` (empty vs. absent).
// Fix Applied: Added `.filter(|s| !s.is_empty())` before `.unwrap_or_else(|| "N/A")`
//   in `status_active`'s `read_live_cred_meta` parse chain for sub and tier fields.
// Prevention: Every `parse_string_field(...).unwrap_or_else(|| "N/A")` chain MUST
//   include `.filter(|s| !s.is_empty())`; empty and absent are equivalent for display.
// Pitfall: The same pattern appears in `status_named` — any new credential field
//   added to display output must include the filter or the same blank-line bug recurs.

#[ test ]
fn astat12_v1_empty_sub_in_creds_shows_n_a()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  // Write credentials with subscriptionType = "" (empty string)
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();
  std::fs::write(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
    r#"{"oauthAccount":{"subscriptionType":"","rateLimitTier":"standard"},"expiresAt":9999999999000}"#,
  ).unwrap();

  let out = run_cs_with_env( &[ ".account.status" ], &[ ( "HOME", home ) ] );

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Sub:     N/A" ),
    "empty subscriptionType must show 'Sub:     N/A', got:\n{text}",
  );
}
