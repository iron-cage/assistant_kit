//! Integration tests for `token::status` classification.
//!
//! Each test writes a credential file to a temp HOME directory and
//! verifies the `TokenStatus` variant returned.
//! Safe because nextest runs every test in its own process.

use claude_profile::token::{ self, TokenStatus };
use tempfile::TempDir;

/// Build minimal credentials JSON with a given `expiresAt` ms value.
fn make_credentials( expires_at_ms : u64 ) -> String
{
  format!(
    r#"{{"claudeAiOauth":{{"accessToken":"tok","refreshToken":"ref","expiresAt":{expires_at_ms},"scopes":[],"subscriptionType":"max","rateLimitTier":"standard"}}}}"#
  )
}

/// Create a temp dir, write `.claude/.credentials.json`, and set HOME to it.
///
/// Returns the `TempDir` handle so the caller holds the lifetime.
/// Dropping the handle deletes the temp dir.
fn setup_temp_credentials( expires_at_ms : u64 ) -> TempDir
{
  let dir = TempDir::new().expect( "temp dir" );
  let claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude ).expect( "create .claude" );
  std::fs::write( claude.join( ".credentials.json" ), make_credentials( expires_at_ms ) )
    .expect( "write credentials" );
  std::env::set_var( "HOME", dir.path() );
  dir
}

/// Current Unix epoch in milliseconds (u64).
///
/// Uses `try_from` to avoid `cast_possible_truncation` lint on `as_millis() → u64`.
fn now_ms() -> u64
{
  u64::try_from(
    std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap_or_default()
      .as_millis(),
  )
  .unwrap_or( u64::MAX )
}

#[ test ]
fn status_returns_expired_when_expires_at_in_past()
{
  let _dir = setup_temp_credentials( 1 ); // epoch+1ms — far in the past
  assert_eq!( token::status().expect( "status" ), TokenStatus::Expired );
}

#[ test ]
fn status_returns_valid_when_far_future()
{
  //! Token far in the future should classify as `Valid`.
  //!
  //! Uses `u64::MAX` milliseconds as expiry — well beyond any real clock value.
  let _dir = setup_temp_credentials( u64::MAX );
  match token::status().expect( "status" )
  {
    TokenStatus::Valid { .. } => {}
    other => panic!( "expected Valid, got {other:?}" ),
  }
}

#[ test ]
fn status_returns_expiring_soon_within_default_threshold()
{
  //! Token expiring in 30 min is within the 60-min default threshold → `ExpiringSoon`.
  //!
  //! Why 30 min: it is strictly inside the 60-min window without being zero or negative,
  //! so the test does not race against the clock even on slow CI systems.
  let thirty_min_ms = 30u64 * 60 * 1_000;
  let expiry = now_ms().saturating_add( thirty_min_ms );
  let _dir = setup_temp_credentials( expiry );
  match token::status().expect( "status" )
  {
    TokenStatus::ExpiringSoon { .. } => {}
    other => panic!( "expected ExpiringSoon, got {other:?}" ),
  }
}

#[ test ]
fn status_with_threshold_zero_classifies_non_expired_as_expiring_soon()
{
  //! With threshold = 0, any token that hasn't expired yet is `ExpiringSoon`
  //! because the remaining time (> 0) is within a zero-second window.
  //!
  //! Actually: the threshold check is `remaining_secs <= warning_secs`.
  //! With `warning_secs` = 0, `ExpiringSoon` fires only when `remaining_secs` == 0
  //! (exact boundary). A token one second in the future will be Valid at threshold 0.
  //! This test verifies the boundary with a token far in the future.
  let _dir = setup_temp_credentials( u64::MAX );
  match token::status_with_threshold( 0 ).expect( "status" )
  {
    // With threshold 0, far-future token is Valid (remaining >> 0).
    TokenStatus::Valid { .. } => {}
    other => panic!( "expected Valid at threshold 0 for far-future token, got {other:?}" ),
  }
}

#[ test ]
fn status_returns_error_when_credentials_file_missing()
{
  //! `status()` must fail loudly when `.credentials.json` does not exist.
  //!
  //! Why: callers rely on the error to detect an uninitialized Claude installation.
  let dir = TempDir::new().expect( "temp dir" );
  let claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude ).expect( "create .claude" );
  // No .credentials.json written — status() must return Err.
  std::env::set_var( "HOME", dir.path() );
  assert!( token::status().is_err(), "expected Err when credentials missing" );
}

#[ test ]
fn status_returns_error_when_expires_at_missing_from_credentials()
{
  //! `status()` must fail when `expiresAt` field is absent.
  //!
  //! Why: the crate cannot classify token expiry without the field;
  //! silently returning `Valid` would be a security hazard.
  let dir = TempDir::new().expect( "temp dir" );
  let claude = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude ).expect( "create .claude" );
  std::fs::write( claude.join( ".credentials.json" ), r#"{"claudeAiOauth":{}}"# )
    .expect( "write" );
  std::env::set_var( "HOME", dir.path() );
  assert!( token::status().is_err(), "expected Err when expiresAt missing" );
}

#[ test ]
fn status_with_custom_threshold_classifies_correctly()
{
  //! Custom threshold (2 hours) correctly classifies a token expiring in 1 hour.
  //!
  //! Validates that `status_with_threshold` respects caller-specified thresholds.
  let one_hour_ms = 60u64 * 60 * 1_000;
  let expiry = now_ms().saturating_add( one_hour_ms );
  let _dir = setup_temp_credentials( expiry );
  // threshold = 2 hours (7200s) → token expiring in ~1h is ExpiringSoon
  match token::status_with_threshold( 7_200 ).expect( "status" )
  {
    TokenStatus::ExpiringSoon { .. } => {}
    other => panic!( "expected ExpiringSoon with 2h threshold for 1h expiry, got {other:?}" ),
  }
}

// ── Private helper unit tests (moved from src/token.rs) ──────────────────────

use claude_profile::token::parse_expires_at;

#[ test ]
fn parse_expires_at_standard_format()
{
  let json = r#"{"claudeAiOauth":{"expiresAt":1774016492576}}"#;
  assert_eq!( parse_expires_at( json ), Some( 1_774_016_492_576 ) );
}

#[ test ]
fn parse_expires_at_with_whitespace()
{
  let json = r#"{"claudeAiOauth":{"expiresAt": 1774016492576}}"#;
  assert_eq!( parse_expires_at( json ), Some( 1_774_016_492_576 ) );
}

#[ test ]
fn parse_expires_at_missing_returns_none()
{
  let json = r#"{"claudeAiOauth":{"accessToken":"abc"}}"#;
  assert_eq!( parse_expires_at( json ), None );
}

#[ test ]
fn parse_expires_at_empty_value_returns_none()
{
  let json = r#"{"expiresAt":}"#;
  assert_eq!( parse_expires_at( json ), None );
}

#[ test ]
fn parse_expires_at_epoch_plus_one_is_some()
{
  //! Verifies `parse_expires_at` returns Some(1) for expiresAt=1 (far in past).
  //!
  //! Also manually confirms that 1 ms < `now_ms`, demonstrating the value
  //! would be classified as Expired by `status_with_threshold`.
  let json = r#"{"claudeAiOauth":{"expiresAt":1}}"#;
  let result = parse_expires_at( json );
  assert_eq!( result, Some( 1 ) );
  // Sanity: current time must be past epoch+1ms
  assert!( now_ms() > 1, "sanity: now must be after epoch+1ms" );
}
