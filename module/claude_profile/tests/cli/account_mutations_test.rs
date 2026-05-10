//! Integration tests: AS (Account Save), AW (Account Switch), AD (Account Delete).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### AS — Account Save
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | as01 | `as01_save_creates_file` | save creates .credentials.json | P |
//! | as02 | `as02_save_dry_run` | `dry::1` → no file created | P |
//! | as03 | `as03_save_overwrite` | second save overwrites first | P |
//! | as04 | `as04_save_hyphened_name` | hyphenated name accepted | P |
//! | as05 | `as05_save_underscored_name` | underscored name accepted | P |
//! | as06 | `as06_save_empty_name_exits_1` | empty name → exit 1 | N |
//! | as07 | `as07_save_slash_name_exits_1` | name with `/` → exit 1 | N |
//! | as08 | `as08_save_backslash_name_exits_1` | name with `\` → exit 1 | N |
//! | as09 | `as09_save_star_name_exits_1` | name with `*` → exit 1 | N |
//! | as10 | `as10_save_infer_absent_email_exits_1` | no `name::`, no emailAddress → exit 1 | N |
//! | as15 | `as15_save_infers_email_from_claude_json` | no `name::`, emailAddress present → exit 0 | P |
//! | as11 | `as11_save_missing_credentials_exits_2` | no credentials file → exit 2 | N |
//! | as12 | `as12_save_auto_creates_credential_store` | credential store auto-created | P |
//! | as13 | `as13_save_dry_then_exec_match` | dry then exec → same output | P |
//! | as14 | `as14_save_file_matches_source` | saved content matches source | P |
//! | as16 | `as16_save_writes_active_marker` | save writes `_active` = name | P |
//!
//! ### AW — Account Switch
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | aw01 | `aw01_switch_swaps_credentials` | switch replaces .credentials.json | P |
//! | aw02 | `aw02_switch_dry_run` | `dry::1` → no file changed | P |
//! | aw03 | `aw03_switch_nonexistent_exits_2` | unknown account → exit 2 | N |
//! | aw04 | `aw04_switch_empty_name_exits_1` | empty name → exit 1 | N |
//! | aw05 | `aw05_switch_slash_name_exits_1` | name with `/` → exit 1 | N |
//! | aw06 | `aw06_switch_missing_name_param_exits_1` | no `name::` param → exit 1 | N |
//! | aw07 | `aw07_switch_updates_active_marker` | switch writes _active marker | P |
//! | aw08 | `aw08_switch_same_account_idempotent` | switch to same account succeeds | P |
//! | aw09 | `aw09_switch_copies_credentials` | switch copies correct cred content | P |
//!
//! ### AD — Account Delete
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ad01 | `ad01_delete_inactive_removes_file` | delete inactive removes file | P |
//! | ad02 | `ad02_delete_dry_run_keeps_file` | dry::1 → file kept | P |
//! | ad03 | `ad03_delete_active_exits_2` | delete active account → exit 2 | N |
//! | ad04 | `ad04_delete_nonexistent_exits_2` | unknown account → exit 2 | N |
//! | ad05 | `ad05_delete_empty_name_exits_1` | empty name → exit 1 | N |
//! | ad06 | `ad06_delete_slash_name_exits_1` | name with `/` → exit 1 | N |
//! | ad07 | `ad07_delete_missing_name_param_exits_1` | no name:: param → exit 1 | N |
//! | ad08 | `ad08_delete_then_list_absent` | delete then list → account gone | P |
//! | ad09 | `ad09_double_delete_exits_2` | delete twice → second exit 2 | N |
//! | ad10 | `ad10_delete_dry_run_active_exits_2` | dry delete active → exit 2 | N |
//! | ad11 | `ad11_delete_dry_run_nonexistent_exits_2` | dry delete nonexistent → exit 2 | N |
//! | ad12 | `ad12_delete_removes_snapshot_files` | delete removes .claude.json + .settings.json snapshots | P |

use crate::helpers::{
  run_cs_with_env,
  stdout, assert_exit,
  write_credentials, write_account, write_claude_json, account_exists,
  write_account_claude_json, write_account_settings_json,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── AS: Account Save ──────────────────────────────────────────────────────────

#[ test ]
fn as01_save_creates_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "saved" ), "must confirm save, got:\n{text}" );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "account file must exist" );
}

#[ test ]
fn as02_save_dry_run()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "dry-run" ), "must say dry-run, got:\n{text}" );
  assert!( !account_exists( dir.path(), "alice@acme.com" ), "dry-run must not create file" );
}

#[ test ]
fn as03_save_overwrite()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // First save
  let _ = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  // Update credentials and save again
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  // Verify new content
  let saved = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "alice@acme.com.credentials.json" )
  ).unwrap();
  assert!( saved.contains( "max" ), "overwrite must use new credentials, got: {saved}" );
}

#[ test ]
fn as04_save_hyphened_name()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice-work@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "alice-work@acme.com" ) );
}

#[ test ]
fn as05_save_underscored_name()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice_work@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "alice_work@acme.com" ) );
}

#[ test ]
fn as06_save_empty_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as07_save_slash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a/b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as08_save_backslash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a\\b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as09_save_star_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a*b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as10_save_infer_absent_email_exits_1()
{
  // IT-10: no ~/.claude.json → emailAddress absent → inference fails → exit 1.
  // write_credentials writes only ~/.claude/.credentials.json, not ~/.claude.json,
  // so the inference branch finds no emailAddress and must exit 1.
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as15_save_infers_email_from_claude_json()
{
  // IT-14: ~/.claude.json present with emailAddress → inference succeeds → exit 0.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "alice@acme.com" );

  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "credential file must be created under inferred name" );
}

#[ test ]
fn as11_save_missing_credentials_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credentials file — only create .claude dir
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn as12_save_auto_creates_credential_store()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // credential store does NOT exist

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "account file must be auto-created" );
}

#[ test ]
fn as13_save_dry_then_exec_match()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let dry = run_cs_with_env( &[ ".account.save", "name::alice@acme.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &dry, 0 );
  assert!( !account_exists( dir.path(), "alice@acme.com" ), "dry-run must not create file" );

  let exec = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &exec, 0 );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "exec must create file" );
}

#[ test ]
fn as14_save_file_matches_source()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let _ = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );

  let source = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let saved = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "alice@acme.com.credentials.json" )
  ).unwrap();
  assert_eq!( source, saved, "saved file must be byte-identical to source" );
}

// ── AW: Account Switch ────────────────────────────────────────────────────────

#[ test ]
fn aw01_switch_swaps_credentials()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.switch", "name::alice@home.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "switched" ), "must confirm switch, got:\n{text}" );
}

#[ test ]
fn aw02_switch_dry_run()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let before = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let out = run_cs_with_env( &[ ".account.switch", "name::alice@home.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "dry-run" ), "must say dry-run, got:\n{text}" );
  let after = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  assert_eq!( before, after, "dry-run must not change credentials" );
}

#[ test ]
fn aw03_switch_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.switch", "name::missing@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn aw04_switch_empty_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.switch", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn aw05_switch_slash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.switch", "name::a/b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn aw06_switch_missing_name_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.switch" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn aw07_switch_updates_active_marker()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.switch", "name::alice@home.com" ], &[ ( "HOME", home ) ] );

  let marker = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "_active" )
  ).unwrap();
  assert_eq!( marker.trim(), "alice@home.com" );
}

#[ test ]
fn aw08_switch_same_account_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.switch", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn aw09_switch_copies_credentials()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.switch", "name::alice@home.com" ], &[ ( "HOME", home ) ] );

  let creds = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let account_file = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "alice@home.com.credentials.json" )
  ).unwrap();
  assert_eq!( creds, account_file, "credentials must match account file after switch" );
}

// ── AD: Account Delete ────────────────────────────────────────────────────────

#[ test ]
fn ad01_delete_inactive_removes_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "alice@oldco.com" ), "account file must be removed" );
}

#[ test ]
fn ad02_delete_dry_run_keeps_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "dry-run" ), "must say dry-run, got:\n{text}" );
  assert!( account_exists( dir.path(), "alice@oldco.com" ), "dry-run must not delete file" );
}

#[ test ]
fn ad03_delete_active_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "active account must not be deleted" );
}

#[ test ]
fn ad04_delete_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::ghost@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ad05_delete_empty_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ad06_delete_slash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::a/b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ad07_delete_missing_name_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ad08_delete_then_list_absent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "keep@example.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );

  let out = run_cs_with_env( &[ ".accounts", "active::0", "sub::0", "tier::0", "expires::0", "email::0" ], &[ ( "HOME", home ) ] );
  let text = stdout( &out );
  assert!( !text.contains( "alice@oldco.com" ), "deleted account must not appear in list, got:\n{text}" );
  assert!( text.contains( "keep@example.com" ), "kept account must still appear, got:\n{text}" );
}

#[ test ]
fn ad09_double_delete_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let first = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &first, 0 );

  let second = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &second, 2 );
}

// test_kind: bug_reproducer(issue-switch-dry-validation)
//
// Root Cause: `account_switch_routine` checked `is_dry()` before validating account
//   existence, so `.account.switch dry::1 name::missing` returned exit 0 ("would switch
//   to 'missing'") even when the named account does not exist.
// Why Not Caught: `aw02_switch_dry_run` only exercises the happy-path dry-run (valid
//   account). No test covered the dry-run-with-nonexistent-account case.
// Fix Applied: `check_switch_preconditions()` extracted from `switch_account()` and
//   called in the command routine before the dry-run guard.
// Prevention: Dry-run must always run input validation + precondition checks; only the
//   mutation step is skipped.
// Pitfall: Placing `is_dry()` before domain validation produces misleading "would do X"
//   output for operations that would actually fail — always validate first, then dry-run.
#[ test ]
fn aw10_switch_dry_run_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.switch", "name::missing@example.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

// test_kind: bug_reproducer(issue-delete-dry-validation)
//
// Root Cause: `account_delete_routine` checked `is_dry()` before running the active-account
//   guard, so `.account.delete dry::1 name::work` (where `work` is active) returned exit 0
//   ("would delete account 'work'") bypassing `PermissionDenied`.
// Why Not Caught: `ad02_delete_dry_run_keeps_file` uses an inactive account — it never
//   exercised dry-run against the currently active account.
// Fix Applied: `check_delete_preconditions()` extracted from `delete()` and called in
//   the command routine before the dry-run guard.
// Prevention: Same as aw10: validate preconditions before the dry-run shortcut.
// Pitfall: The active-account guard is a safety invariant that must hold even in dry-run;
//   reporting "would delete active account" without error is a misleading no-op.
#[ test ]
fn ad10_delete_dry_run_active_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@acme.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "dry-run must not delete active account" );
}

// test_kind: bug_reproducer(issue-delete-dry-validation)
//
// Root Cause: Same as ad10 — `is_dry()` guard ran before any account existence check,
//   so `.account.delete dry::1 name::ghost` (nonexistent) returned exit 0 instead of
//   exit 2 (`NotFound`).
// Why Not Caught: `ad02` exercises an existing account; no test covered dry-run on a
//   nonexistent account.
// Fix Applied: See ad10 — `check_delete_preconditions()` runs before dry-run guard.
// Prevention: Dry-run path must include all validation; only file-system mutation is omitted.
// Pitfall: Missing existence check in dry-run gives a false "operation would succeed"
//   signal, masking configuration errors until the real run.
#[ test ]
fn ad11_delete_dry_run_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::ghost@example.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ad12_delete_removes_snapshot_files()
{
  // IT-11: delete removes all 3 files — credentials, .claude.json, and .settings.json snapshots.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "old@archive.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_claude_json(   dir.path(), "old@archive.com", "", "", "", "" );
  write_account_settings_json( dir.path(), "old@archive.com", "sonnet" );

  let out = run_cs_with_env( &[ ".account.delete", "name::old@archive.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!( !store.join( "old@archive.com.credentials.json" ).exists(), "credentials must be removed after delete" );
  assert!( !store.join( "old@archive.com.claude.json" ).exists(),      "claude.json snapshot must be removed after delete" );
  assert!( !store.join( "old@archive.com.settings.json" ).exists(),    "settings.json snapshot must be removed after delete" );
}

// ── as16 ──────────────────────────────────────────────────────────────────────

/// as16: `.account.save name::work@acme.com` writes `{store}/_active` = `"work@acme.com"`.
///
/// CLI-level symmetry test with aw07: reads `_active` directly (not via
/// `.credentials.status`) to confirm the write happened at the filesystem level.
///
/// ## Fix Documentation — issue-active-marker
///
/// - **Root Cause:** `save()` never wrote `_active`; only `switch_account()` did.
/// - **Why Not Caught:** No AS test verified the `_active` file after `.account.save`.
/// - **Fix Applied:** Added `std::fs::write(credential_store.join("_active"), name)?;` to `save()`.
/// - **Prevention:** This test guards `_active` at the filesystem level, independently of `.credentials.status`.
/// - **Pitfall:** Must assert the raw file content — not just exit code — to catch a write that produces wrong content.
#[ test ]
fn as16_save_writes_active_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::work@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let store  = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( "_active" ) )
    .expect( "_active must exist after .account.save" );
  assert_eq!(
    active.trim(),
    "work@acme.com",
    "_active must equal the saved account name",
  );
}
