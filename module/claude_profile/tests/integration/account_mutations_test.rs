//! Integration tests: AS (Account Save), AW (Account Switch), AD (Account Delete).

use crate::helpers::{
  run_cs_with_env,
  stdout, assert_exit,
  write_credentials, write_account, account_exists,
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

  let out = run_cs_with_env( &[ ".account.save", "name::work" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "saved" ), "must confirm save, got:\n{text}" );
  assert!( account_exists( dir.path(), "work" ), "account file must exist" );
}

#[ test ]
fn as02_save_dry_run()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::work", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "dry-run" ), "must say dry-run, got:\n{text}" );
  assert!( !account_exists( dir.path(), "work" ), "dry-run must not create file" );
}

#[ test ]
fn as03_save_overwrite()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // First save
  let _ = run_cs_with_env( &[ ".account.save", "name::work" ], &[ ( "HOME", home ) ] );
  // Update credentials and save again
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  let out = run_cs_with_env( &[ ".account.save", "name::work" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  // Verify new content
  let saved = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( "accounts" ).join( "work.credentials.json" )
  ).unwrap();
  assert!( saved.contains( "max" ), "overwrite must use new credentials, got: {saved}" );
}

#[ test ]
fn as04_save_hyphened_name()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a-b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "a-b" ) );
}

#[ test ]
fn as05_save_underscored_name()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a_b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "a_b" ) );
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
fn as10_save_missing_name_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn as11_save_missing_credentials_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credentials file — only create .claude dir
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.save", "name::work" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn as12_save_auto_creates_accounts_dir()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // accounts dir does NOT exist

  let out = run_cs_with_env( &[ ".account.save", "name::work" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "work" ), "account file must be auto-created" );
}

#[ test ]
fn as13_save_dry_then_exec_match()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let dry = run_cs_with_env( &[ ".account.save", "name::work", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &dry, 0 );
  assert!( !account_exists( dir.path(), "work" ), "dry-run must not create file" );

  let exec = run_cs_with_env( &[ ".account.save", "name::work" ], &[ ( "HOME", home ) ] );
  assert_exit( &exec, 0 );
  assert!( account_exists( dir.path(), "work" ), "exec must create file" );
}

#[ test ]
fn as14_save_file_matches_source()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let _ = run_cs_with_env( &[ ".account.save", "name::work" ], &[ ( "HOME", home ) ] );

  let source = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let saved = std::fs::read_to_string( dir.path().join( ".claude" ).join( "accounts" ).join( "work.credentials.json" ) ).unwrap();
  assert_eq!( source, saved, "saved file must be byte-identical to source" );
}

// ── AW: Account Switch ────────────────────────────────────────────────────────

#[ test ]
fn aw01_switch_swaps_credentials()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "personal", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.switch", "name::personal" ], &[ ( "HOME", home ) ] );
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
  write_account( dir.path(), "personal", "max", "tier4", FAR_FUTURE_MS, false );

  let before = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let out = run_cs_with_env( &[ ".account.switch", "name::personal", "dry::1" ], &[ ( "HOME", home ) ] );
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
  std::fs::create_dir_all( dir.path().join( ".claude" ).join( "accounts" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.switch", "name::missing" ], &[ ( "HOME", home ) ] );
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
  write_account( dir.path(), "personal", "max", "tier4", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.switch", "name::personal" ], &[ ( "HOME", home ) ] );

  let marker = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( "accounts" ).join( "_active" )
  ).unwrap();
  assert_eq!( marker.trim(), "personal" );
}

#[ test ]
fn aw08_switch_same_account_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.switch", "name::work" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn aw09_switch_copies_credentials()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "personal", "max", "tier4", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.switch", "name::personal" ], &[ ( "HOME", home ) ] );

  let creds = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let account_file = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( "accounts" ).join( "personal.credentials.json" )
  ).unwrap();
  assert_eq!( creds, account_file, "credentials must match account file after switch" );
}

// ── AD: Account Delete ────────────────────────────────────────────────────────

#[ test ]
fn ad01_delete_inactive_removes_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "old", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "name::old" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "old" ), "account file must be removed" );
}

#[ test ]
fn ad02_delete_dry_run_keeps_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "old", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "name::old", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "dry-run" ), "must say dry-run, got:\n{text}" );
  assert!( account_exists( dir.path(), "old" ), "dry-run must not delete file" );
}

#[ test ]
fn ad03_delete_active_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.delete", "name::work" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  assert!( account_exists( dir.path(), "work" ), "active account must not be deleted" );
}

#[ test ]
fn ad04_delete_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ).join( "accounts" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::ghost" ], &[ ( "HOME", home ) ] );
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
  write_account( dir.path(), "keep", "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "old", "pro", "standard", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.delete", "name::old" ], &[ ( "HOME", home ) ] );

  let out = run_cs_with_env( &[ ".account.list", "v::0" ], &[ ( "HOME", home ) ] );
  let text = stdout( &out );
  assert!( !text.contains( "old" ), "deleted account must not appear in list, got:\n{text}" );
  assert!( text.contains( "keep" ), "kept account must still appear, got:\n{text}" );
}

#[ test ]
fn ad09_double_delete_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "old", "pro", "standard", FAR_FUTURE_MS, false );

  let first = run_cs_with_env( &[ ".account.delete", "name::old" ], &[ ( "HOME", home ) ] );
  assert_exit( &first, 0 );

  let second = run_cs_with_env( &[ ".account.delete", "name::old" ], &[ ( "HOME", home ) ] );
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
  std::fs::create_dir_all( dir.path().join( ".claude" ).join( "accounts" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.switch", "name::missing", "dry::1" ], &[ ( "HOME", home ) ] );
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
fn ad10_delete_dry_run_active_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.delete", "name::work", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  assert!( account_exists( dir.path(), "work" ), "dry-run must not delete active account" );
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
  std::fs::create_dir_all( dir.path().join( ".claude" ).join( "accounts" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::ghost", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

