//! Integration tests: AS (Account Save), AW (Account Use), AD (Account Delete).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
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
//! | as10 | `as10_save_infer_absent_email_exits_1` | no `name::`, no `_active` marker → exit 1 | N |
//! | as11 | `as11_save_missing_credentials_exits_2` | no credentials file → exit 2 | N |
//! | as12 | `as12_save_auto_creates_credential_store` | credential store auto-created | P |
//! | as13 | `as13_save_dry_then_exec_match` | dry then exec → same output | P |
//! | as14 | `as14_save_file_matches_source` | saved content matches source | P |
//! | as15 | `as15_save_infers_name_from_active_marker` | no `name::`, `_active` marker present → exit 0 | P |
//! | as16 | `as16_save_writes_active_marker` | save writes active marker = name | P |
//! | as17 | `as17_save_slash_in_email_local_part_exits_1` | `/` in email local part → exit 1 | N |
//! | as18 | `as18_save_backslash_in_email_local_part_exits_1` | `\` in email local part → exit 1 | N |
//! | aw01 | `aw01_switch_swaps_credentials` | switch replaces .credentials.json | P |
//! | aw02 | `aw02_switch_dry_run` | `dry::1` → no file changed | P |
//! | aw03 | `aw03_switch_nonexistent_exits_2` | unknown account → exit 2 | N |
//! | aw04 | `aw04_switch_empty_name_exits_1` | empty name → exit 1 | N |
//! | aw05 | `aw05_switch_slash_name_exits_1` | name with `/` → exit 1 | N |
//! | aw06 | `aw06_switch_missing_name_param_exits_1` | no `name::` param → exit 1 | N |
//! | aw07 | `aw07_switch_updates_active_marker` | switch writes active marker | P |
//! | aw08 | `aw08_switch_same_account_idempotent` | switch to same account succeeds | P |
//! | aw09 | `aw09_switch_copies_credentials` | switch copies correct cred content | P |
//! | aw10 | `aw10_switch_dry_run_nonexistent_exits_2` | dry-run nonexistent → exit 2 | N |
//! | aw11 | `aw11_switch_slash_in_email_local_part_exits_1` | `/` in email local part → exit 1 | N |
//! | aw12 | `aw12_switch_patches_email_when_metadata_absent` | emailAddress patched (BUG-254) | P |
//! | — | `switch_restores_claude_json` | `~/.claude.json` restored (BUG-277) | P |
//! | — | `mre_bug_217_switch_account_enforces_emailaddress` | switch enforces emailAddress | P |
//! | aw13 | `aw13_use_positional_bare_arg` | positional email → switches | P |
//! | aw14 | `aw14_use_prefix_resolves` | prefix resolves to full email, switches | P |
//! | aw15 | `aw15_use_prefix_ambiguous_exits_1` | ambiguous prefix → exit 1 | N |
//! | aw16 | `aw16_exact_local_part_wins_over_ambiguous_prefix` | exact local part wins | P |
//! | aw17 | `aw17_use_prefix_ambiguous_no_exact_local_part_exits_1` | no exact match → exit 1 | N |
//! | ad01 | `ad01_delete_inactive_removes_file` | delete inactive removes file | P |
//! | ad02 | `ad02_delete_dry_run_keeps_file` | `dry::1` → file kept | P |
//! | ad03 | `ad03_delete_active_exits_0` | delete active account → exit 0 | P |
//! | ad04 | `ad04_delete_nonexistent_exits_2` | unknown account → exit 2 | N |
//! | ad05 | `ad05_delete_empty_name_exits_1` | empty name → exit 1 | N |
//! | ad06 | `ad06_delete_slash_name_exits_1` | name with `/` → exit 1 | N |
//! | ad07 | `ad07_delete_missing_name_param_exits_1` | no `name::` param → exit 1 | N |
//! | ad08 | `ad08_delete_then_list_absent` | delete then list → account gone | P |
//! | ad09 | `ad09_double_delete_exits_2` | delete twice → second exit 2 | N |
//! | ad10 | `ad10_delete_dry_run_active_exits_0` | dry delete active → exit 0 | P |
//! | ad11 | `ad11_delete_dry_run_nonexistent_exits_2` | dry delete nonexistent → exit 2 | N |
//! | ad12 | `ad12_delete_removes_snapshot_files` | delete removes snapshot | P |
//! | ad13 | `ad13_delete_positional_bare_arg` | positional email → deletes | P |
//! | ad14 | `ad14_delete_prefix_resolves` | prefix resolves, deletes | P |
//! | ad15 | `ad15_delete_removes_roles_json` | delete removes roles data | P |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
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

  let out = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "saved current credentials as 'alice@acme.com'" ), "must confirm save, got:\n{text}" );
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
  assert!( text.contains( "[dry-run] would save current credentials as 'alice@acme.com'" ), "must say dry-run preview, got:\n{text}" );
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
  // IT-10: no _active marker → inference fails → exit 1.
  // write_credentials writes only ~/.claude/.credentials.json, no _active marker is set,
  // so the inference branch finds no active account and must exit 1.
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "cannot infer account name: no active account set" ),
    "stderr must explain inference failure, got:\n{err}",
  );
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!( !store.exists(), "credential store must not be created on inference failure" );
}

#[ test ]
fn as15_save_infers_name_from_active_marker()
{
  // IT-14: _active marker present → inference succeeds → exit 0, saves under marker name.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Write _active marker = "alice@acme.com" (simulates prior .account.use).
  let store = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write(
    store.join( claude_profile::account::active_marker_filename() ),
    "alice@acme.com",
  ).unwrap();

  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "saved current credentials as 'alice@acme.com'" ), "must confirm save with inferred name, got:\n{text}" );
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

// ── AW: Account Use ───────────────────────────────────────────────────────────

#[ test ]
fn aw01_switch_swaps_credentials()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "name::alice@home.com" ], &[ ( "HOME", home ) ] );
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
  let out = run_cs_with_env( &[ ".account.use", "name::alice@home.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run] would switch to 'alice@home.com'" ), "must print full dry-run message, got:\n{text}" );
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

  let out = run_cs_with_env( &[ ".account.use", "name::missing@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn aw04_switch_empty_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.use", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn aw05_switch_slash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.use", "name::a/b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn aw06_switch_missing_name_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.use" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn aw07_switch_updates_active_marker()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.use", "name::alice@home.com" ], &[ ( "HOME", home ) ] );

  let marker = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( claude_profile::account::active_marker_filename() )
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

  let out = run_cs_with_env( &[ ".account.use", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn aw09_switch_copies_credentials()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.use", "name::alice@home.com" ], &[ ( "HOME", home ) ] );

  let creds = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let account_file = std::fs::read_to_string(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "alice@home.com.credentials.json" )
  ).unwrap();
  assert_eq!( creds, account_file, "credentials must match account file after switch" );
}
