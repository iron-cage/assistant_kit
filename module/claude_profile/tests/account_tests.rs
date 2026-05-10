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
//! | A-14 | `delete_returns_error_if_account_is_active` | active account → `PermissionDenied` | N |
//! | A-15 | `delete_returns_not_found_for_missing_account` | non-existent name → `Err` NotFound | N |
//! | A-16 | `auto_rotate_switches_to_inactive_account` | inactive account present → switches | P |
//! | A-17 | `auto_rotate_returns_switched_account_name` | → returns name string | P |
//! | A-18 | `auto_rotate_picks_account_with_highest_expires_at` | multiple inactive → picks latest expiry | P |
//! | A-19 | `auto_rotate_fails_when_no_inactive_accounts` | all active → `Err` | N |
//! | A-20 | `auto_rotate_fails_when_account_store_empty` | no accounts → `Err` | N |
//! | A-21 | `auto_rotate_with_no_active_marker_picks_highest_expires_at` | no `_active` file → picks latest | P |
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

  account::save( "alice@acme.com", &credential_store, &paths ).expect( "save" );

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
  account::save( "alice@acme.com", &credential_store, &paths ).expect( "save" );

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
  account::save( "alice@acme.com", &credential_store, &paths ).expect( "first save" );

  // Overwrite active credentials with different content.
  let claude = dir.path().join( ".claude" );
  std::fs::write( claude.join( ".credentials.json" ), CREDENTIALS_B ).expect( "overwrite" );
  account::save( "alice@acme.com", &credential_store, &paths ).expect( "second save" );

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
  let err = account::save( "", &credential_store, &paths ).expect_err( "empty name must fail" );
  assert_eq!( err.kind(), std::io::ErrorKind::InvalidInput );
}

#[ test ]
fn save_rejects_name_with_slash()
{
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  let err = account::save( "work/home", &credential_store, &paths ).expect_err( "slash must fail" );
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
  account::save( "alice@acme.com", &credential_store, &paths ).expect( "save" );

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
  account::save( "alice@acme.com", &credential_store, &paths ).expect( "save alice@acme.com" );
  account::save( "alice@home.com", &credential_store, &paths ).expect( "save alice@home.com" );

  // Write _active marker manually to "alice@acme.com".
  let marker = credential_store.join( "_active" );
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
  account::save( "zebra@acme.com", &credential_store, &paths ).expect( "save zebra" );
  account::save( "alpha@acme.com", &credential_store, &paths ).expect( "save alpha" );

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

  let marker = std::fs::read_to_string( credential_store.join( "_active" ) )
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
  account::save( "alice@oldco.com", &credential_store, &paths ).expect( "save" );
  let file = credential_store.join( "alice@oldco.com.credentials.json" );
  assert!( file.exists() );
  // save() now writes _active = "alice@oldco.com"; switch to a different account
  // so alice@oldco.com is inactive and deletion is permitted.
  std::fs::write( credential_store.join( "_active" ), "work@acme.com" ).expect( "overwrite _active" );

  account::delete( "alice@oldco.com", &credential_store ).expect( "delete" );

  assert!( !file.exists(), "credential file must be gone after delete" );
}

#[ test ]
fn delete_returns_error_if_account_is_active()
{
  //! FR-10 active-guard: cannot delete the currently active account.
  //!
  //! Why: deleting the active account would leave the credentials pointer
  //! dangling — the next switch would succeed but point to nothing.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  account::save( "alice@acme.com", &credential_store, &paths ).expect( "save" );
  let marker = credential_store.join( "_active" );
  std::fs::write( &marker, "alice@acme.com" ).expect( "write _active" );

  let err = account::delete( "alice@acme.com", &credential_store )
    .expect_err( "must fail for active account" );
  assert_eq!( err.kind(), std::io::ErrorKind::PermissionDenied );
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

// Inactive account with high expires_at_ms — preferred rotation target.
const CREDENTIALS_EXPIRE_HIGH : &str = r#"{"claudeAiOauth":{"accessToken":"token-hi","refreshToken":"refresh-hi","expiresAt":9000000000000,"scopes":[],"subscriptionType":"max","rateLimitTier":"standard"}}"#;

// Inactive account with low expires_at_ms — non-preferred rotation target.
const CREDENTIALS_EXPIRE_LOW : &str = r#"{"claudeAiOauth":{"accessToken":"token-lo","refreshToken":"refresh-lo","expiresAt":2000000000000,"scopes":[],"subscriptionType":"pro","rateLimitTier":"light"}}"#;

#[ test ]
fn auto_rotate_switches_to_inactive_account()
{
  //! FR-13: `auto_rotate()` switches to the only inactive account.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  std::fs::create_dir_all( &credential_store ).expect( "credential_store dir" );
  std::fs::write( credential_store.join( "alice@acme.com.credentials.json" ), CREDENTIALS ).expect( "save alice@acme.com" );
  std::fs::write( credential_store.join( "alice@home.com.credentials.json" ), CREDENTIALS_B ).expect( "save alice@home.com" );
  std::fs::write( credential_store.join( "_active" ), "alice@acme.com" ).expect( "_active" );

  account::auto_rotate( &credential_store, &paths ).expect( "auto_rotate" );

  let marker = std::fs::read_to_string( credential_store.join( "_active" ) ).expect( "read _active" );
  assert_eq!( marker.trim(), "alice@home.com" );
}

#[ test ]
fn auto_rotate_returns_switched_account_name()
{
  //! FR-13: return value is the name of the account switched to.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS_B );
  let paths = ClaudePaths::new().expect( "HOME set" );
  std::fs::create_dir_all( &credential_store ).expect( "credential_store dir" );
  std::fs::write( credential_store.join( "alice@acme.com.credentials.json" ), CREDENTIALS ).expect( "save alice@acme.com" );
  std::fs::write( credential_store.join( "alice@home.com.credentials.json" ), CREDENTIALS_B ).expect( "save alice@home.com" );
  std::fs::write( credential_store.join( "_active" ), "alice@home.com" ).expect( "_active" );

  let switched_to = account::auto_rotate( &credential_store, &paths ).expect( "auto_rotate" );
  assert_eq!( switched_to, "alice@acme.com" );
}

#[ test ]
fn auto_rotate_picks_account_with_highest_expires_at()
{
  //! FR-13: with multiple inactive accounts, picks the one with the highest
  //! `expires_at_ms` — the one whose OAuth token lasts longest.
  //!
  //! Why: callers use `auto_rotate` to get the best remaining account, not an
  //! arbitrary one. The selection must be deterministic and optimal.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  std::fs::create_dir_all( &credential_store ).expect( "credential_store dir" );
  std::fs::write( credential_store.join( "alpha@acme.com.credentials.json" ), CREDENTIALS_EXPIRE_LOW ).expect( "save alpha" );
  std::fs::write( credential_store.join( "beta@acme.com.credentials.json" ), CREDENTIALS_EXPIRE_HIGH ).expect( "save beta" );
  std::fs::write( credential_store.join( "current@acme.com.credentials.json" ), CREDENTIALS ).expect( "save current" );
  std::fs::write( credential_store.join( "_active" ), "current@acme.com" ).expect( "_active" );

  // beta@acme.com has expiresAt=9000000000000 > alpha@acme.com's 2000000000000.
  let switched_to = account::auto_rotate( &credential_store, &paths ).expect( "auto_rotate" );
  assert_eq!( switched_to, "beta@acme.com" );
}

#[ test ]
fn auto_rotate_fails_when_no_inactive_accounts()
{
  //! FR-13: when the only account is the active one, `auto_rotate` fails.
  //!
  //! Why: there is no candidate to rotate to — the error surfaces this
  //! rather than silently succeeding by switching to the same account.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  std::fs::create_dir_all( &credential_store ).expect( "credential_store dir" );
  std::fs::write( credential_store.join( "solo@example.com.credentials.json" ), CREDENTIALS ).expect( "save solo" );
  std::fs::write( credential_store.join( "_active" ), "solo@example.com" ).expect( "_active" );

  let err = account::auto_rotate( &credential_store, &paths )
    .expect_err( "must fail with no inactive accounts" );
  assert_eq!( err.kind(), std::io::ErrorKind::NotFound );
}

#[ test ]
fn auto_rotate_fails_when_account_store_empty()
{
  //! FR-13: when no accounts are configured, `auto_rotate` fails with `NotFound`.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  // No credential_store directory — list() returns empty vec.

  let err = account::auto_rotate( &credential_store, &paths )
    .expect_err( "must fail with empty account store" );
  assert_eq!( err.kind(), std::io::ErrorKind::NotFound );
}

#[ test ]
fn auto_rotate_with_no_active_marker_picks_highest_expires_at()
{
  //! FR-13: when no _active marker exists all accounts are inactive;
  //! `auto_rotate` picks the one with the highest `expires_at_ms`.
  let ( _dir, credential_store ) = setup_home( CREDENTIALS );
  let paths = ClaudePaths::new().expect( "HOME set" );
  std::fs::create_dir_all( &credential_store ).expect( "credential_store dir" );
  std::fs::write( credential_store.join( "alpha@acme.com.credentials.json" ), CREDENTIALS_EXPIRE_LOW ).expect( "save alpha" );
  std::fs::write( credential_store.join( "beta@acme.com.credentials.json" ), CREDENTIALS_EXPIRE_HIGH ).expect( "save beta" );
  // No _active marker — both accounts appear inactive.

  let switched_to = account::auto_rotate( &credential_store, &paths ).expect( "auto_rotate" );
  assert_eq!( switched_to, "beta@acme.com" );
}

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
