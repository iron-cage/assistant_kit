//! Integration tests for account CRUD: save, list, switch, delete, active-guard.
//!
//! Each test creates a fully isolated temp HOME directory so tests never
//! touch the real `~/.claude/` installation.
//! Safe because nextest runs every test in its own process.

use claude_profile::account;
use tempfile::TempDir;

// Minimal credentials JSON that satisfies the account module's parser.
const CREDENTIALS : &str = r#"{"claudeAiOauth":{"accessToken":"token-abc","refreshToken":"refresh-xyz","expiresAt":9999999999999,"scopes":[],"subscriptionType":"max","rateLimitTier":"standard"}}"#;

const CREDENTIALS_B : &str = r#"{"claudeAiOauth":{"accessToken":"token-def","refreshToken":"refresh-def","expiresAt":1000000000000,"scopes":[],"subscriptionType":"pro","rateLimitTier":"light"}}"#;

/// Create a temp HOME with `.claude/.credentials.json` pre-populated.
///
/// Returns the `TempDir` handle — drop it to clean up.
fn setup_home( credentials : &str ) -> TempDir
{
  let dir = TempDir::new().expect( "temp dir" );
  let claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude ).expect( "create .claude dir" );
  std::fs::write( claude.join( ".credentials.json" ), credentials ).expect( "write credentials" );
  std::env::set_var( "HOME", dir.path() );
  dir
}

// ── FR-6: Account Store Initialization ───────────────────────────────────────

#[ test ]
fn save_creates_accounts_dir_when_missing()
{
  //! FR-6: `~/.claude/accounts/` is created on first `save()` call.
  //!
  //! Why: callers must not have to pre-create the accounts directory; the
  //! function itself is responsible for initializing the account store.
  let _dir = setup_home( CREDENTIALS );
  let accounts_dir = std::path::PathBuf::from( std::env::var( "HOME" ).unwrap() )
    .join( ".claude" )
    .join( "accounts" );
  assert!( !accounts_dir.exists(), "accounts/ must not exist before first save" );

  account::save( "work" ).expect( "save" );

  assert!( accounts_dir.exists(), "accounts/ must be created by save()" );
}

// ── FR-7: Save Account ────────────────────────────────────────────────────────

#[ test ]
fn save_copies_credentials_to_named_file()
{
  //! FR-7: `save("work")` creates `accounts/work.credentials.json`
  //! with the same content as `.credentials.json`.
  let dir = setup_home( CREDENTIALS );
  account::save( "work" ).expect( "save" );

  let saved = dir.path()
    .join( ".claude/accounts/work.credentials.json" );
  assert!( saved.exists(), "work.credentials.json must exist after save" );
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
  let dir = setup_home( CREDENTIALS );
  account::save( "work" ).expect( "first save" );

  // Overwrite active credentials with different content.
  let claude = dir.path().join( ".claude" );
  std::fs::write( claude.join( ".credentials.json" ), CREDENTIALS_B ).expect( "overwrite" );
  account::save( "work" ).expect( "second save" );

  let saved = dir.path().join( ".claude/accounts/work.credentials.json" );
  assert_eq!(
    std::fs::read_to_string( saved ).unwrap(),
    CREDENTIALS_B,
    "second save must overwrite first",
  );
}

#[ test ]
fn save_rejects_empty_name()
{
  let _dir = setup_home( CREDENTIALS );
  let err = account::save( "" ).expect_err( "empty name must fail" );
  assert_eq!( err.kind(), std::io::ErrorKind::InvalidInput );
}

#[ test ]
fn save_rejects_name_with_slash()
{
  let _dir = setup_home( CREDENTIALS );
  let err = account::save( "work/home" ).expect_err( "slash must fail" );
  assert_eq!( err.kind(), std::io::ErrorKind::InvalidInput );
}

// ── FR-8: List Accounts ───────────────────────────────────────────────────────

#[ test ]
fn list_returns_empty_when_accounts_dir_missing()
{
  //! FR-8: empty account store is not an error — returns empty Vec.
  //!
  //! Why: callers should not need to guard against a first-time install.
  let _dir = setup_home( CREDENTIALS );
  let accounts = account::list().expect( "list" );
  assert!( accounts.is_empty(), "list must return empty vec when accounts/ absent" );
}

#[ test ]
fn list_returns_saved_accounts_with_metadata()
{
  //! FR-8: `list()` returns correct metadata from credential files.
  let _dir = setup_home( CREDENTIALS );
  account::save( "work" ).expect( "save" );

  let accounts = account::list().expect( "list" );
  assert_eq!( accounts.len(), 1 );
  assert_eq!( accounts[ 0 ].name, "work" );
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
  let dir = setup_home( CREDENTIALS );
  account::save( "work" ).expect( "save work" );
  account::save( "personal" ).expect( "save personal" );

  // Write _active marker manually to "work".
  let marker = dir.path().join( ".claude/accounts/_active" );
  std::fs::write( &marker, "work" ).expect( "write _active" );

  let accounts = account::list().expect( "list" );
  let work = accounts.iter().find( | a | a.name == "work" ).expect( "work" );
  let personal = accounts.iter().find( | a | a.name == "personal" ).expect( "personal" );
  assert!( work.is_active, "work must be active" );
  assert!( !personal.is_active, "personal must not be active" );
}

#[ test ]
fn list_returns_accounts_sorted_by_name()
{
  //! FR-8: list is deterministic — sorted alphabetically by name.
  let _dir = setup_home( CREDENTIALS );
  account::save( "zebra" ).expect( "save zebra" );
  account::save( "alpha" ).expect( "save alpha" );

  let accounts = account::list().expect( "list" );
  assert_eq!( accounts.len(), 2 );
  assert_eq!( accounts[ 0 ].name, "alpha" );
  assert_eq!( accounts[ 1 ].name, "zebra" );
}

// ── FR-9: Switch Account ──────────────────────────────────────────────────────

#[ test ]
fn switch_account_overwrites_credentials_file()
{
  //! FR-9 + NFR-6: atomic write-then-rename puts named credentials in place.
  let dir = setup_home( CREDENTIALS );
  // Save a second credential set as "personal".
  let claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( claude.join( "accounts" ) ).expect( "accounts dir" );
  std::fs::write(
    claude.join( "accounts/personal.credentials.json" ),
    CREDENTIALS_B,
  )
  .expect( "write personal" );

  account::switch_account( "personal" ).expect( "switch" );

  let active_content = std::fs::read_to_string( claude.join( ".credentials.json" ) )
    .expect( "read credentials" );
  assert_eq!( active_content, CREDENTIALS_B, "credentials must be replaced by switch" );
}

#[ test ]
fn switch_account_updates_active_marker()
{
  //! FR-9: `_active` marker file is written with the new account name after switch.
  let dir = setup_home( CREDENTIALS );
  let claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( claude.join( "accounts" ) ).expect( "accounts dir" );
  std::fs::write(
    claude.join( "accounts/personal.credentials.json" ),
    CREDENTIALS_B,
  )
  .expect( "write personal" );

  account::switch_account( "personal" ).expect( "switch" );

  let marker = std::fs::read_to_string( claude.join( "accounts/_active" ) )
    .expect( "read _active" );
  assert_eq!( marker.trim(), "personal" );
}

#[ test ]
fn switch_account_returns_not_found_for_missing_account()
{
  //! FR-9: switching to an account that doesn't exist must fail with `NotFound`.
  let _dir = setup_home( CREDENTIALS );
  let err = account::switch_account( "ghost" ).expect_err( "must fail" );
  assert_eq!( err.kind(), std::io::ErrorKind::NotFound );
}

// ── FR-10: Delete Account ─────────────────────────────────────────────────────

#[ test ]
fn delete_removes_credential_file()
{
  //! FR-10: `delete()` removes the named account file from the store.
  let dir = setup_home( CREDENTIALS );
  account::save( "old" ).expect( "save" );
  let file = dir.path().join( ".claude/accounts/old.credentials.json" );
  assert!( file.exists() );

  account::delete( "old" ).expect( "delete" );

  assert!( !file.exists(), "credential file must be gone after delete" );
}

#[ test ]
fn delete_returns_error_if_account_is_active()
{
  //! FR-10 active-guard: cannot delete the currently active account.
  //!
  //! Why: deleting the active account would leave the credentials pointer
  //! dangling — the next switch would succeed but point to nothing.
  let dir = setup_home( CREDENTIALS );
  account::save( "work" ).expect( "save" );
  let marker = dir.path().join( ".claude/accounts/_active" );
  std::fs::write( &marker, "work" ).expect( "write _active" );

  let err = account::delete( "work" ).expect_err( "must fail for active account" );
  assert_eq!( err.kind(), std::io::ErrorKind::PermissionDenied );
}

#[ test ]
fn delete_returns_not_found_for_missing_account()
{
  //! FR-10: deleting an account that was never saved fails with `NotFound`.
  let _dir = setup_home( CREDENTIALS );
  // Create accounts dir so the not-found path is exercised.
  let accounts = std::path::PathBuf::from( std::env::var( "HOME" ).unwrap() )
    .join( ".claude/accounts" );
  std::fs::create_dir_all( &accounts ).expect( "accounts dir" );

  let err = account::delete( "ghost" ).expect_err( "must fail" );
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
  let dir = setup_home( CREDENTIALS );
  let accounts = dir.path().join( ".claude/accounts" );
  std::fs::create_dir_all( &accounts ).expect( "accounts dir" );
  std::fs::write( accounts.join( "work.credentials.json" ), CREDENTIALS ).expect( "save work" );
  std::fs::write( accounts.join( "personal.credentials.json" ), CREDENTIALS_B ).expect( "save personal" );
  std::fs::write( accounts.join( "_active" ), "work" ).expect( "_active" );

  account::auto_rotate().expect( "auto_rotate" );

  let marker = std::fs::read_to_string( accounts.join( "_active" ) ).expect( "read _active" );
  assert_eq!( marker.trim(), "personal" );
}

#[ test ]
fn auto_rotate_returns_switched_account_name()
{
  //! FR-13: return value is the name of the account switched to.
  let dir = setup_home( CREDENTIALS_B );
  let accounts = dir.path().join( ".claude/accounts" );
  std::fs::create_dir_all( &accounts ).expect( "accounts dir" );
  std::fs::write( accounts.join( "work.credentials.json" ), CREDENTIALS ).expect( "save work" );
  std::fs::write( accounts.join( "personal.credentials.json" ), CREDENTIALS_B ).expect( "save personal" );
  std::fs::write( accounts.join( "_active" ), "personal" ).expect( "_active" );

  let switched_to = account::auto_rotate().expect( "auto_rotate" );
  assert_eq!( switched_to, "work" );
}

#[ test ]
fn auto_rotate_picks_account_with_highest_expires_at()
{
  //! FR-13: with multiple inactive accounts, picks the one with the highest
  //! `expires_at_ms` — the one whose OAuth token lasts longest.
  //!
  //! Why: callers use `auto_rotate` to get the best remaining account, not an
  //! arbitrary one. The selection must be deterministic and optimal.
  let dir = setup_home( CREDENTIALS );
  let accounts = dir.path().join( ".claude/accounts" );
  std::fs::create_dir_all( &accounts ).expect( "accounts dir" );
  std::fs::write( accounts.join( "alpha.credentials.json" ), CREDENTIALS_EXPIRE_LOW ).expect( "save alpha" );
  std::fs::write( accounts.join( "beta.credentials.json" ), CREDENTIALS_EXPIRE_HIGH ).expect( "save beta" );
  std::fs::write( accounts.join( "current.credentials.json" ), CREDENTIALS ).expect( "save current" );
  std::fs::write( accounts.join( "_active" ), "current" ).expect( "_active" );

  // beta has expiresAt=9000000000000 > alpha's 2000000000000.
  let switched_to = account::auto_rotate().expect( "auto_rotate" );
  assert_eq!( switched_to, "beta" );
}

#[ test ]
fn auto_rotate_fails_when_no_inactive_accounts()
{
  //! FR-13: when the only account is the active one, `auto_rotate` fails.
  //!
  //! Why: there is no candidate to rotate to — the error surfaces this
  //! rather than silently succeeding by switching to the same account.
  let dir = setup_home( CREDENTIALS );
  let accounts = dir.path().join( ".claude/accounts" );
  std::fs::create_dir_all( &accounts ).expect( "accounts dir" );
  std::fs::write( accounts.join( "solo.credentials.json" ), CREDENTIALS ).expect( "save solo" );
  std::fs::write( accounts.join( "_active" ), "solo" ).expect( "_active" );

  let err = account::auto_rotate().expect_err( "must fail with no inactive accounts" );
  assert_eq!( err.kind(), std::io::ErrorKind::NotFound );
}

#[ test ]
fn auto_rotate_fails_when_account_store_empty()
{
  //! FR-13: when no accounts are configured, `auto_rotate` fails with `NotFound`.
  let _dir = setup_home( CREDENTIALS );
  // No accounts/ directory — list() returns empty vec.

  let err = account::auto_rotate().expect_err( "must fail with empty account store" );
  assert_eq!( err.kind(), std::io::ErrorKind::NotFound );
}

#[ test ]
fn auto_rotate_with_no_active_marker_picks_highest_expires_at()
{
  //! FR-13: when no _active marker exists all accounts are inactive;
  //! `auto_rotate` picks the one with the highest `expires_at_ms`.
  let dir = setup_home( CREDENTIALS );
  let accounts = dir.path().join( ".claude/accounts" );
  std::fs::create_dir_all( &accounts ).expect( "accounts dir" );
  std::fs::write( accounts.join( "alpha.credentials.json" ), CREDENTIALS_EXPIRE_LOW ).expect( "save alpha" );
  std::fs::write( accounts.join( "beta.credentials.json" ), CREDENTIALS_EXPIRE_HIGH ).expect( "save beta" );
  // No _active marker — both accounts appear inactive.

  let switched_to = account::auto_rotate().expect( "auto_rotate" );
  assert_eq!( switched_to, "beta" );
}

// ── Private helper unit tests (moved from src/account.rs) ────────────────────

use claude_profile::account::{ credential_stem, parse_string_field, parse_u64_field, validate_name };
use std::path::PathBuf;

#[ test ]
fn credential_stem_valid()
{
  let path = PathBuf::from( "/home/user/.claude/accounts/work.credentials.json" );
  assert_eq!( credential_stem( &path ), Some( "work".to_string() ) );
}

#[ test ]
fn credential_stem_filters_active_marker()
{
  let path = PathBuf::from( "/home/user/.claude/accounts/_active" );
  assert_eq!( credential_stem( &path ), None );
}

#[ test ]
fn credential_stem_filters_plain_json()
{
  let path = PathBuf::from( "/home/user/.claude/accounts/something.json" );
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
  assert!( validate_name( "work" ).is_ok() );
  assert!( validate_name( "my-account-2" ).is_ok() );
  assert!( validate_name( "user.name" ).is_ok() );
}
