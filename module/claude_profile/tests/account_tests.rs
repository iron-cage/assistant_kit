//! Integration tests for account CRUD: save, list, switch, delete, active-guard.
//!
//! Each test creates a fully isolated temp HOME directory so tests never
//! touch the real `~/.claude/` installation.
//! Safe because nextest runs every test in its own process.
//!
//! ## Test Matrix
//!
//! | ID   | Test Function | Condition | P/N |
//! |------|---------------|-----------|-----|
//! | A-01 | `save_creates_credential_store_when_missing` | first save creates credential store | P |
//! | A-02 | `save_copies_credentials_to_named_file` | save produces named file with same content | P |
//! | A-03 | `save_overwrites_existing_entry` | second save overwrites first | P |
//! | A-04 | `save_rejects_empty_name` | empty name → `Err` | N |
//! | A-05 | `save_rejects_name_with_slash` | name contains `/` → `Err` | N |
//! | A-06 | `list_returns_empty_when_credential_store_missing` | no credential store → empty vec | P |
//! | A-07 | `list_returns_saved_accounts_with_metadata` | credential store has files → vec with metadata | P |
//! | A-08 | `list_marks_active_account_via_active_marker` | `_active` marker file → `is_active = true` | P |
//! | A-09 | `list_returns_accounts_sorted_by_name` | multiple accounts → sorted ascending | P |
//! | A-10 | `switch_account_overwrites_credentials_file` | switch copies named account to `.credentials.json` | P |
//! | A-11 | `switch_account_updates_active_marker` | switch writes `_active` marker | P |
//! | A-12 | `switch_account_returns_not_found_for_missing_account` | name not in credential store → `Err` NotFound | N |
//! | A-13 | `delete_removes_credential_file` | delete removes named file | P |
//! | A-14 | `delete_active_account_succeeds` | active account → succeeds, `_active` cleaned up | P |
//! | A-15 | `delete_returns_not_found_for_missing_account` | non-existent name → `Err` NotFound | N |
//! | A-22 | `credential_stem_valid` | `.credentials.json` file → `Some(stem)` | P |
//! | A-23 | `credential_stem_filters_active_marker` | `_active` file → `None` | P |
//! | A-24 | `credential_stem_filters_plain_json` | non-credentials `.json` → `None` | P |
//! | A-25 | `parse_string_field_standard` | standard JSON field → `Some(value)` | P |
//! | A-26 | `parse_string_field_with_space` | field with spaces → `Some(value)` | P |
//! | A-27 | `parse_string_field_missing` | field absent → `None` | N |
//! | A-28 | `parse_u64_field_standard` | numeric JSON field → `Some(u64)` | P |
//! | A-29 | `parse_u64_field_with_space` | numeric field with spaces → `Some(u64)` | P |
//! | A-30 | `validate_name_empty_is_error` | empty string → `Err` | N |
//! | A-31 | `validate_name_slash_is_error` | name with `/` → `Err` | N |
//! | A-32 | `validate_name_null_byte_is_error` | name with NUL byte → `Err` | N |
//! | A-33 | `validate_name_valid` | valid email address → `Ok` | P |
//! | A-34 | `validate_name_must_be_email` | non-email name → `Err` with email message | N |
//! | SC-2/001 | `sc2_001_expires_at_stays_t0_manipulate_expires_at_in_memory_only` | `manipulate_expires_at` modifies in-memory only — disk file unchanged (BUG-162) | P |
//! | SC-4/002 | `sc4_002_account_json_is_2space_pretty_with_trailing_newline` | `{name}.json` uses 2-space indent and trailing newline (invariant/007) | P |
//! | SC-5/002 | `sc5_002_history_entry_appended_not_truncated` | second `write_history_entry` appends; first entry preserved | P |
//! | SC-6/002 | `sc6_002_quota_cache_all_subfields_written_atomically` | `write_quota_cache` writes all subfields in one coherent object | P |
//! | SC-2/005 | `sc2_005_active_marker_filename_uses_env_vars` | `active_marker_filename()` derives name from `HOSTNAME` + `USER` env vars | P |
//! | SC-3/005 | `sc3_005_active_marker_sanitizes_nonalphanumeric_to_underscore` | `@` in `USER`/`HOSTNAME` becomes `_` in marker filename | P |

use claude_profile::account;
use claude_profile::ClaudePaths;
use tempfile::TempDir;

// Minimal credentials JSON that satisfies the account module's parser.
const CREDENTIALS : &str = r#"{"claudeAiOauth":{"accessToken":"token-abc","refreshToken":"refresh-xyz","expiresAt":9999999999999,"scopes":[],"subscriptionType":"max","rateLimitTier":"standard"}}"#;

const CREDENTIALS_B : &str = r#"{"claudeAiOauth":{"accessToken":"token-def","refreshToken":"refresh-def","expiresAt":1000000000000,"scopes":[],"subscriptionType":"pro","rateLimitTier":"light"}}"#;

/// Create a temp HOME with `.claude/.credentials.json` pre-populated.
///
/// Returns `(TempDir, credential_store_path)`. Drop `TempDir` to clean up.
/// The credential store is at `{home}/.persistent/claude/credential/`.
fn setup_home( credentials : &str ) -> ( TempDir, std::path::PathBuf )
{
  let dir = TempDir::new().expect( "temp dir" );
  let claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude ).expect( "create .claude dir" );
  std::fs::write( claude.join( ".credentials.json" ), credentials ).expect( "write credentials" );
  std::env::set_var( "HOME", dir.path() );
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  ( dir, credential_store )
}

// ── FR-6: Account Store Initialization ───────────────────────────────────────

#[ test ]
fn save_creates_credential_store_when_missing()
{
  //! FR-6: `{home}/.persistent/claude/credential/` is created on first `save()` call.
  //!
  //! Why: callers must not have to pre-create the credential store; the
  //! function itself is responsible for initializing it.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  assert!( !credential_store.exists(), "credential_store must not exist before first save" );

  account::save( "alice@acme.com", &credential_store, &paths, true, None, None, None, None ).expect( "save" );

  assert!( credential_store.exists(), "credential_store must be created by save()" );
}

// ── FR-7: Save Account ────────────────────────────────────────────────────────

#[ test ]
fn save_copies_credentials_to_named_file()
{
  //! FR-7: `save("alice@acme.com")` creates `alice@acme.com.credentials.json`
  //! in the credential store with the same content as `.credentials.json`.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  account::save( "alice@acme.com", &credential_store, &paths, true, None, None, None, None ).expect( "save" );

  let saved = credential_store.join( "alice@acme.com.credentials.json" );
  assert!( saved.exists(), "alice@acme.com.credentials.json must exist after save" );
  assert_eq!(
    std::fs::read_to_string( saved ).unwrap(),
    CREDENTIALS,
    "saved content must match source credentials",
  );
}

#[ test ]
fn save_overwrites_existing_entry()
{
  //! FR-7 overwrite: saving the same name twice uses the latest credentials.
  let ( dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  account::save( "alice@acme.com", &credential_store, &paths, true, None, None, None, None ).expect( "first save" );

  // Overwrite active credentials with different content.
  let claude = dir.path().join( ".claude" );
  std::fs::write( claude.join( ".credentials.json" ), CREDENTIALS_B ).expect( "overwrite" );
  account::save( "alice@acme.com", &credential_store, &paths, true, None, None, None, None ).expect( "second save" );

  let saved = credential_store.join( "alice@acme.com.credentials.json" );
  assert_eq!(
    std::fs::read_to_string( saved ).unwrap(),
    CREDENTIALS_B,
    "second save must overwrite first",
  );
}

#[ test ]
fn save_rejects_empty_name()
{
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  let err = account::save( "", &credential_store, &paths, true, None, None, None, None ).expect_err( "empty name must fail" );
  assert_eq!( err.kind(), std::io::ErrorKind::InvalidInput );
}

#[ test ]
fn save_rejects_name_with_slash()
{
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  let err = account::save( "work/home", &credential_store, &paths, true, None, None, None, None ).expect_err( "slash must fail" );
  assert_eq!( err.kind(), std::io::ErrorKind::InvalidInput );
}

// ── FR-8: List Accounts ───────────────────────────────────────────────────────

#[ test ]
fn list_returns_empty_when_credential_store_missing()
{
  //! FR-8: empty account store is not an error — returns empty Vec.
  //!
  //! Why: callers should not need to guard against a first-time install.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let accounts = account::list( &credential_store ).expect( "list" );
  assert!( accounts.is_empty(), "list must return empty vec when credential_store absent" );
}

#[ test ]
fn list_returns_saved_accounts_with_metadata()
{
  //! FR-8: `list()` returns correct metadata from credential files.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  account::save( "alice@acme.com", &credential_store, &paths, true, None, None, None, None ).expect( "save" );

  let accounts = account::list( &credential_store ).expect( "list" );
  assert_eq!( accounts.len(), 1 );
  assert_eq!( accounts[ 0 ].name, "alice@acme.com" );
  assert_eq!( accounts[ 0 ].subscription_type, "max" );
  assert_eq!( accounts[ 0 ].rate_limit_tier, "standard" );
  assert_eq!( accounts[ 0 ].expires_at_ms, 9_999_999_999_999_u64 );
}

#[ test ]
fn list_marks_active_account_via_active_marker()
{
  //! FR-8: `is_active` reflects the `_active` marker file content.
  //!
  //! Why: callers use `is_active` to avoid redundant switches and to display
  //! which account is currently in use.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  account::save( "alice@acme.com", &credential_store, &paths, true, None, None, None, None ).expect( "save alice@acme.com" );
  account::save( "alice@home.com", &credential_store, &paths, true, None, None, None, None ).expect( "save alice@home.com" );

  // Write _active marker manually to "alice@acme.com".
  let marker = credential_store.join( account::active_marker_filename() );
  std::fs::write( &marker, "alice@acme.com" ).expect( "write _active" );

  let accounts = account::list( &credential_store ).expect( "list" );
  let work = accounts.iter().find( | a | a.name == "alice@acme.com" ).expect( "alice@acme.com" );
  let personal = accounts.iter().find( | a | a.name == "alice@home.com" ).expect( "alice@home.com" );
  assert!( work.is_active, "alice@acme.com must be active" );
  assert!( !personal.is_active, "alice@home.com must not be active" );
}

#[ test ]
fn list_returns_accounts_sorted_by_name()
{
  //! FR-8: list is deterministic — sorted alphabetically by name.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  account::save( "zebra@acme.com", &credential_store, &paths, true, None, None, None, None ).expect( "save zebra" );
  account::save( "alpha@acme.com", &credential_store, &paths, true, None, None, None, None ).expect( "save alpha" );

  let accounts = account::list( &credential_store ).expect( "list" );
  assert_eq!( accounts.len(), 2 );
  assert_eq!( accounts[ 0 ].name, "alpha@acme.com" );
  assert_eq!( accounts[ 1 ].name, "zebra@acme.com" );
}

// ── FR-9: Switch Account ──────────────────────────────────────────────────────

#[ test ]
fn switch_account_overwrites_credentials_file()
{
  //! FR-9 + NFR-6: atomic write-then-rename puts named credentials in place.
  let ( dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  // Save a second credential set as "alice@home.com".
  std::fs::create_dir_all( &credential_store ).expect( "credential_store dir" );
  std::fs::write(
    credential_store.join( "alice@home.com.credentials.json" ),
    CREDENTIALS_B,
  )
  .expect( "write alice@home.com" );

  account::switch_account( "alice@home.com", &credential_store, &paths ).expect( "switch" );

  let active_content = std::fs::read_to_string( dir.path().join( ".claude/.credentials.json" ) )
    .expect( "read credentials" );
  assert_eq!( active_content, CREDENTIALS_B, "credentials must be replaced by switch" );
}

#[ test ]
fn switch_account_updates_active_marker()
{
  //! FR-9: `_active` marker file is written with the new account name after switch.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  std::fs::create_dir_all( &credential_store ).expect( "credential_store dir" );
  std::fs::write(
    credential_store.join( "alice@home.com.credentials.json" ),
    CREDENTIALS_B,
  )
  .expect( "write alice@home.com" );

  account::switch_account( "alice@home.com", &credential_store, &paths ).expect( "switch" );

  let marker = std::fs::read_to_string( credential_store.join( account::active_marker_filename() ) )
    .expect( "read _active" );
  assert_eq!( marker.trim(), "alice@home.com" );
}

#[ test ]
fn switch_account_returns_not_found_for_missing_account()
{
  //! FR-9: switching to an account that doesn't exist must fail with `NotFound`.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  let err = account::switch_account( "ghost@example.com", &credential_store, &paths )
    .expect_err( "must fail" );
  assert_eq!( err.kind(), std::io::ErrorKind::NotFound );
}

// ── FR-10: Delete Account ─────────────────────────────────────────────────────

#[ test ]
fn delete_removes_credential_file()
{
  //! FR-10: `delete()` removes the named account file from the store.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  account::save( "alice@oldco.com", &credential_store, &paths, true, None, None, None, None ).expect( "save" );
  let file = credential_store.join( "alice@oldco.com.credentials.json" );
  assert!( file.exists(), "credential file must exist immediately after save()" );
  // save() now writes _active = "alice@oldco.com"; switch to a different account
  // so alice@oldco.com is inactive and deletion is permitted.
  std::fs::write( credential_store.join( account::active_marker_filename() ), "work@acme.com" ).expect( "overwrite _active" );

  account::delete( "alice@oldco.com", &credential_store ).expect( "delete" );

  assert!( !file.exists(), "credential file must be gone after delete" );
}

// Root Cause: `check_delete_preconditions()` returned `PermissionDenied` when the account
//   matched the `_active` marker; `delete()` never cleaned up `_active` on any deletion.
// Why Not Caught: A-14 asserted `PermissionDenied` as correct behavior; no test covered
//   the stale-marker scenario (no other account to switch to before deleting).
// Fix Applied: Removed the `PermissionDenied` guard from `check_delete_preconditions()`;
//   added best-effort `_active` cleanup in `delete()` after credential file removal.
// Prevention: Active-marker state must not block file operations — clean up stale markers
//   after deletion rather than refusing the operation.
// Pitfall: Checking `_active` in preconditions creates a deadlock when no other accounts
//   exist (must switch before delete, but can't switch with no other accounts).
#[ doc = "bug_reproducer(BUG-275)" ]
#[ test ]
fn delete_active_account_succeeds()
{
  //! FR-10: deleting the active account succeeds; `_active` marker is cleaned up.
  //!
  //! Why: external credential changes may leave `_active` pointing at an account
  //! the user needs to delete; blocking on `_active` adds no safety since
  //! `~/.claude/.credentials.json` is already live regardless of the marker.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  account::save( "alice@acme.com", &credential_store, &paths, true, None, None, None, None ).expect( "save" );
  let marker = credential_store.join( account::active_marker_filename() );
  std::fs::write( &marker, "alice@acme.com" ).expect( "write _active" );

  account::delete( "alice@acme.com", &credential_store )
    .expect( "delete must succeed even when account is active" );

  let file = credential_store.join( "alice@acme.com.credentials.json" );
  assert!( !file.exists(), "credential file must be removed after delete" );
  assert!( !marker.exists(), "_active marker must be cleaned up after deleting active account" );
}

#[ test ]
fn delete_returns_not_found_for_missing_account()
{
  //! FR-10: deleting an account that was never saved fails with `NotFound`.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  // Create credential_store so the not-found path is exercised.
  std::fs::create_dir_all( &credential_store ).expect( "credential_store dir" );

  let err = account::delete( "ghost@example.com", &credential_store ).expect_err( "must fail" );
  assert_eq!( err.kind(), std::io::ErrorKind::NotFound );
}

// ── FR-13: Auto Rotate ────────────────────────────────────────────────────────

// ── Private helper unit tests (moved from src/account.rs) ────────────────────

use claude_profile::account::{ credential_stem, parse_string_field, parse_u64_field, validate_name };
use std::path::PathBuf;

#[ test ]
fn credential_stem_valid()
{
  let path = PathBuf::from( "/home/user/.persistent/claude/credential/alice@acme.com.credentials.json" );
  assert_eq!( credential_stem( &path ), Some( "alice@acme.com".to_string() ) );
}

#[ test ]
fn credential_stem_filters_active_marker()
{
  let path = PathBuf::from( "/home/user/.persistent/claude/credential/_active" );
  assert_eq!( credential_stem( &path ), None );
}

#[ test ]
fn credential_stem_filters_plain_json()
{
  let path = PathBuf::from( "/home/user/.persistent/claude/credential/something.json" );
  assert_eq!( credential_stem( &path ), None );
}

#[ test ]
fn parse_string_field_standard()
{
  let json = r#"{"subscriptionType":"max"}"#;
  assert_eq!( parse_string_field( json, "subscriptionType" ), Some( "max".to_string() ) );
}

#[ test ]
fn parse_string_field_with_space()
{
  let json = r#"{"subscriptionType": "pro"}"#;
  assert_eq!( parse_string_field( json, "subscriptionType" ), Some( "pro".to_string() ) );
}

#[ test ]
fn parse_string_field_missing()
{
  let json = r#"{"other":"value"}"#;
  assert_eq!( parse_string_field( json, "subscriptionType" ), None );
}

#[ test ]
fn parse_u64_field_standard()
{
  let json = r#"{"expiresAt":1774016492576}"#;
  assert_eq!( parse_u64_field( json, "expiresAt" ), Some( 1_774_016_492_576 ) );
}

#[ test ]
fn parse_u64_field_with_space()
{
  let json = r#"{"expiresAt": 999}"#;
  assert_eq!( parse_u64_field( json, "expiresAt" ), Some( 999 ) );
}

#[ test ]
fn validate_name_empty_is_error()
{
  assert!( validate_name( "" ).is_err() );
}

#[ test ]
fn validate_name_slash_is_error()
{
  assert!( validate_name( "work/home" ).is_err() );
}

#[ test ]
fn validate_name_null_byte_is_error()
{
  assert!( validate_name( "a\0b" ).is_err() );
}

#[ test ]
fn validate_name_valid()
{
  assert!( validate_name( "alice@acme.com" ).is_ok() );
  assert!( validate_name( "alice-work@acme.com" ).is_ok() );
  assert!( validate_name( "alice.name@acme.com" ).is_ok() );
}

#[ test ]
fn validate_name_must_be_email()
{
  let err = validate_name( "notanemail" ).expect_err( "non-email name must fail" );
  assert_eq!( err.kind(), std::io::ErrorKind::InvalidInput );
  let msg = format!( "{err}" );
  assert!( msg.contains( "email address" ), "error must mention email address, got: {msg}" );
}

// ── BUG-174: surgical oauthAccount merge ──────────────────────────────────────

/// bug_reproducer(BUG-174): `switch_account()` must preserve machine-global
/// keys in `~/.claude.json` (e.g., `commands.*`, `mcpServers`).
///
/// # Root Cause
///
/// `save()` used `std::fs::copy` to snapshot the entire `~/.claude.json`,
/// and `switch_account()` used `std::fs::copy` to restore it wholesale —
/// clobbering machine-global state with stale snapshot values.
///
/// # Why Not Caught
///
/// No existing test verified the contents of `~/.claude.json` after a
/// save→switch round-trip; tests only checked `.credentials.json`.
///
/// # Fix Applied
///
/// `save()` extracts only the `oauthAccount` subtree. `switch_account()`
/// patches only the `oauthAccount` key in the live `~/.claude.json`,
/// leaving all other keys untouched.
///
/// # Prevention
///
/// This test creates a `~/.claude.json` with both `oauthAccount` and
/// `commands` keys, performs save→switch→switch-back, and asserts
/// `commands.foo` is preserved.
///
/// # Pitfall
///
/// The saved `{name}.json` must contain `oauthAccount` —
/// machine-global keys in `~/.claude.json` indicate a wholesale copy regression.
#[ doc = "bug_reproducer(BUG-174)" ]
#[ test ]
fn test_bug174_mre_switch_preserves_machine_global_commands()
{
  let ( dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );

  // Write ~/.claude.json with both oauthAccount and commands keys.
  let claude_json_a = r#"{"oauthAccount":{"emailAddress":"a@x.com","displayName":"A"},"commands":{"foo":42},"mcpServers":{"local":true}}"#;
  std::fs::write( paths.claude_json_file(), claude_json_a ).expect( "write .claude.json" );

  // Save as account A — snapshot must contain only oauthAccount.
  account::save( "a@x.com", &credential_store, &paths, true, None, None, None, None ).expect( "save A" );
  let saved_a = std::fs::read_to_string( credential_store.join( "a@x.com.json" ) )
    .expect( "read saved A .json" );
  assert!(
    saved_a.contains( "oauthAccount" ),
    "saved snapshot must contain oauthAccount",
  );
  assert!(
    !saved_a.contains( "commands" ),
    "saved snapshot must NOT contain commands (wholesale copy regression); got: {saved_a}",
  );
  assert!(
    !saved_a.contains( "mcpServers" ),
    "saved snapshot must NOT contain mcpServers; got: {saved_a}",
  );

  // Write new credentials and claude.json for account B.
  let claude = dir.path().join( ".claude" );
  std::fs::write( claude.join( ".credentials.json" ), CREDENTIALS_B ).expect( "write creds B" );
  let claude_json_b = r#"{"oauthAccount":{"emailAddress":"b@y.com","displayName":"B"},"commands":{"foo":42},"mcpServers":{"local":true}}"#;
  std::fs::write( paths.claude_json_file(), claude_json_b ).expect( "write .claude.json B" );
  account::save( "b@y.com", &credential_store, &paths, true, None, None, None, None ).expect( "save B" );

  // Mutate commands.foo in live file to simulate machine-local state change.
  let claude_json_live = r#"{"oauthAccount":{"emailAddress":"b@y.com","displayName":"B"},"commands":{"foo":99},"mcpServers":{"local":true}}"#;
  std::fs::write( paths.claude_json_file(), claude_json_live ).expect( "mutate live .claude.json" );

  // Switch to A — oauthAccount should change, commands.foo must stay 99.
  account::switch_account( "a@x.com", &credential_store, &paths ).expect( "switch to A" );
  let after_switch = std::fs::read_to_string( paths.claude_json_file() )
    .expect( "read .claude.json after switch" );

  assert!(
    after_switch.contains( r#""emailAddress": "a@x.com"# ),
    "oauthAccount must be patched to A's data; got: {after_switch}",
  );
  assert!(
    after_switch.contains( r#""foo": 99"# ),
    "BUG-174: commands.foo must be preserved (99, not 42); got: {after_switch}",
  );
  assert!(
    after_switch.contains( "mcpServers" ),
    "mcpServers must be preserved; got: {after_switch}",
  );
}

// ── Schema 001: Credentials JSON ─────────────────────────────────────────────

#[ test ]
fn sc2_001_expires_at_stays_t0_manipulate_expires_at_in_memory_only()
{
  //! SC-2/001: `manipulate_expires_at()` produces `expiresAt=1` in the returned
  //! string only — the on-disk credential file is NEVER written (BUG-162).
  //!
  //! Why: `run_isolated` receives the manipulated string in-memory to force a
  //! token refresh; the subprocess writeback only updates `accessToken` and
  //! `refreshToken`; `expiresAt` on disk retains the value from the last `save()`.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  account::save( "alice@acme.com", &credential_store, &paths, true, None, None, None, None )
    .expect( "save" );

  let creds_path = credential_store.join( "alice@acme.com.credentials.json" );
  let on_disk_before = std::fs::read_to_string( &creds_path ).expect( "read credentials" );

  // In-memory manipulation only — must not touch the file.
  let manipulated = account::manipulate_expires_at( &on_disk_before );

  assert!(
    manipulated.contains( "\"expiresAt\":1" ),
    "SC-2/001: manipulated string must have expiresAt=1; got: {manipulated}",
  );

  // On-disk file must be identical to before the call.
  let on_disk_after = std::fs::read_to_string( &creds_path ).expect( "re-read credentials" );
  assert_eq!(
    on_disk_before, on_disk_after,
    "SC-2/001: manipulate_expires_at must not modify the on-disk credential file (BUG-162)",
  );
}

// ── Schema 002: Account JSON ──────────────────────────────────────────────────

#[ test ]
fn sc4_002_account_json_is_2space_pretty_with_trailing_newline()
{
  //! SC-4/002: `{name}.json` is 2-space indented JSON ending with `\n`.
  //!
  //! Why: invariant/007 mandates this encoding for all persisted JSON files.
  let dir = TempDir::new().expect( "temp dir" );
  let credential_store = dir.path().join( "credential" );
  std::fs::create_dir_all( &credential_store ).expect( "create credential store" );

  account::write_quota_cache(
    &credential_store,
    "alice@acme.com",
    Some( ( 50.0, Some( "2026-07-08T00:00:00Z" ) ) ),
    Some( ( 25.0, Some( "2026-07-15T00:00:00Z" ) ) ),
    None,
  );

  let content = std::fs::read_to_string( credential_store.join( "alice@acme.com.json" ) )
    .expect( "SC-4/002: alice@acme.com.json must exist after write_quota_cache" );

  assert!(
    content.ends_with( '\n' ),
    "SC-4/002: {{name}}.json must end with newline; got: {content:?}",
  );
  assert!(
    content.lines().any( |l| l.starts_with( "  " ) ),
    "SC-4/002: {{name}}.json must use 2-space indentation; got: {content}",
  );
}

#[ test ]
fn sc5_002_history_entry_appended_not_truncated()
{
  //! SC-5/002: Two `write_history_entry()` calls with distinct timestamps produce
  //! two entries in `cache.history` — the prior entry is preserved, not overwritten.
  let dir = TempDir::new().expect( "temp dir" );
  let credential_store = dir.path().join( "credential" );
  std::fs::create_dir_all( &credential_store ).expect( "create credential store" );

  account::write_history_entry(
    &credential_store,
    "alice@acme.com",
    1_000,
    Some( ( 30.0, "2026-07-08T00:00:00Z" ) ),
    None,
    None,
  );
  account::write_history_entry(
    &credential_store,
    "alice@acme.com",
    2_000,
    Some( ( 40.0, "2026-07-08T05:00:00Z" ) ),
    None,
    None,
  );

  let content = std::fs::read_to_string( credential_store.join( "alice@acme.com.json" ) )
    .expect( "alice@acme.com.json must exist" );
  let val : serde_json::Value = serde_json::from_str( &content ).expect( "valid JSON" );
  let history = val[ "cache" ][ "history" ].as_array().expect( "history must be array" );

  assert_eq!(
    history.len(), 2,
    "SC-5/002: history must have 2 entries after 2 distinct-timestamp writes; got: {}",
    history.len(),
  );
  assert_eq!(
    history[ 0 ][ "t" ].as_u64(), Some( 1_000 ),
    "SC-5/002: first entry must be t=1000 (original entry preserved)",
  );
  assert_eq!(
    history[ 1 ][ "t" ].as_u64(), Some( 2_000 ),
    "SC-5/002: second entry must be t=2000 (appended)",
  );
}

#[ test ]
fn sc6_002_quota_cache_all_subfields_written_atomically()
{
  //! SC-6/002: `write_quota_cache()` writes `five_hour`, `seven_day`, and
  //! `seven_day_sonnet` as a single coherent object — no partial write leaves
  //! mismatched fields from a prior cache state.
  let dir = TempDir::new().expect( "temp dir" );
  let credential_store = dir.path().join( "credential" );
  std::fs::create_dir_all( &credential_store ).expect( "create credential store" );

  account::write_quota_cache(
    &credential_store,
    "alice@acme.com",
    Some( ( 12.5, Some( "2026-07-08T05:00:00Z" ) ) ),
    Some( ( 33.3, Some( "2026-07-15T00:00:00Z" ) ) ),
    Some( ( 75.0, Some( "2026-07-15T00:00:00Z" ) ) ),
  );

  let entry = account::read_quota_cache( &credential_store, "alice@acme.com" )
    .expect( "SC-6/002: read_quota_cache must return Some after write_quota_cache" );

  assert!( entry.five_hour.is_some(), "SC-6/002: five_hour must be present" );
  assert!( entry.seven_day.is_some(), "SC-6/002: seven_day must be present" );
  assert!( entry.seven_day_sonnet.is_some(), "SC-6/002: seven_day_sonnet must be present" );
  assert!( !entry.fetched_at.is_empty(), "SC-6/002: fetched_at must be non-empty" );
}

// ── Schema 005: Active Marker ─────────────────────────────────────────────────

#[ test ]
fn sc2_005_active_marker_filename_uses_env_vars()
{
  //! SC-2/005: `active_marker_filename()` derives the filename from `$HOSTNAME`
  //! and `$USER` env vars when both are present.
  std::env::set_var( "HOSTNAME", "testhost" );
  std::env::set_var( "USER", "testuser" );

  let name = account::active_marker_filename();

  assert_eq!(
    name, "_active_testhost_testuser",
    "SC-2/005: marker filename must be _active_{{HOSTNAME}}_{{USER}}; got: {name}",
  );
}

#[ test ]
fn sc3_005_active_marker_sanitizes_nonalphanumeric_to_underscore()
{
  //! SC-3/005: Non-alphanumeric characters in `$USER` or `$HOSTNAME` that are
  //! not `-` or `.` are replaced with `_` in the marker filename.
  //!
  //! Why: `@` in an email-format username must not appear in the filename
  //! component; only alphanumeric, `-`, and `.` are kept by the sanitizer.
  std::env::set_var( "HOSTNAME", "myhost" );
  std::env::set_var( "USER", "user@corp" );

  let name = account::active_marker_filename();

  assert_eq!(
    name, "_active_myhost_user_corp",
    "SC-3/005: '@' in USER must be replaced with '_'; got: {name}",
  );
}
