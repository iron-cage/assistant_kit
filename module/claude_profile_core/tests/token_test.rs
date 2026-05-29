//! Token status unit tests
//!
//! ## Purpose
//!
//! Verify the pure logic in `claude_profile_core::token`:
//! - `parse_expires_at` extracts the millisecond timestamp from credential JSON
//! - `status_with_threshold` classifies the token as Expired, `ExpiringSoon`, or Valid
//! - `classify_ms` classifies a stored millisecond timestamp without file I/O
//!
//! These are the only functions that can be tested without touching the real
//! filesystem — `status()` requires `~/.claude/.credentials.json` to exist.
//!
//! ## Coverage
//!
//! - `parse_expires_at` returns the correct value from well-formed JSON
//! - `parse_expires_at` returns `None` when the key is absent
//! - `parse_expires_at` returns `None` for empty input
//! - `parse_expires_at` tolerates whitespace between `:` and the number
//! - `TokenStatus::Expired` when now ≥ expiresAt
//! - `TokenStatus::ExpiringSoon` when remaining ≤ warning threshold
//! - `TokenStatus::Valid` when remaining > warning threshold
//! - `WARNING_THRESHOLD_SECS` is exactly 3600 (60 minutes)
//! - `classify_ms` returns `Expired` for past timestamp
//! - `classify_ms` returns `ExpiringSoon` within threshold
//! - `classify_ms` returns `Valid` far from expiry
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `parse_expires_at_extracts_value` | nominal well-formed JSON |
//! | `parse_expires_at_absent_key_returns_none` | missing key → None |
//! | `parse_expires_at_empty_input_returns_none` | empty string → None |
//! | `parse_expires_at_tolerates_whitespace` | "expiresAt": 123 (space) |
//! | `warning_threshold_is_one_hour` | constant = 3600 |
//! | `token_expired_when_past_expiry` | past timestamp → Expired |
//! | `token_expiring_soon_within_threshold` | near future → ExpiringSoon |
//! | `token_valid_far_from_expiry` | far future → Valid |
//! | `classify_ms_expired_for_past_timestamp` | past ms → Expired (no file I/O) |
//! | `classify_ms_expiring_soon_within_threshold` | near future ms → ExpiringSoon |
//! | `classify_ms_valid_far_from_expiry` | far future ms → Valid |

use claude_profile_core::token::{ classify_ms, parse_expires_at, status_with_threshold, TokenStatus, WARNING_THRESHOLD_SECS };
use std::time::{ SystemTime, UNIX_EPOCH };

// ─── parse_expires_at ────────────────────────────────────────────────────────

#[test]
fn parse_expires_at_extracts_value()
{
  let json = r#"{"accessToken":"tok","expiresAt":1700000000000,"subscriptionType":"pro"}"#;
  assert_eq!( parse_expires_at( json ), Some( 1_700_000_000_000_u64 ) );
}

#[test]
fn parse_expires_at_absent_key_returns_none()
{
  let json = r#"{"accessToken":"tok","subscriptionType":"pro"}"#;
  assert_eq!( parse_expires_at( json ), None );
}

#[test]
fn parse_expires_at_empty_input_returns_none()
{
  assert_eq!( parse_expires_at( "" ), None );
}

#[test]
fn parse_expires_at_tolerates_whitespace()
{
  // The parser trims whitespace between ":" and the digits
  let json = r#"{"expiresAt":   99999999999 }"#;
  assert_eq!( parse_expires_at( json ), Some( 99_999_999_999_u64 ) );
}

// ─── WARNING_THRESHOLD_SECS ───────────────────────────────────────────────────

#[test]
fn warning_threshold_is_one_hour()
{
  assert_eq!( WARNING_THRESHOLD_SECS, 3600, "threshold must be exactly 60 minutes" );
}

// ─── status_with_threshold (pure logic via crafted credentials file) ──────────

fn now_ms() -> u64
{
  u64::try_from(
    SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .unwrap_or_default()
      .as_millis()
  ).unwrap_or( u64::MAX )
}

fn write_credentials( dir : &std::path::Path, expires_at_ms : u64 ) -> std::path::PathBuf
{
  let claude_dir = dir.join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).expect( "create .claude dir" );
  let creds_file = claude_dir.join( ".credentials.json" );
  let json = format!(
    r#"{{"accessToken":"test_tok","expiresAt":{expires_at_ms},"subscriptionType":"pro"}}"#
  );
  std::fs::write( &creds_file, json ).expect( "write credentials" );
  dir.to_path_buf()
}

#[test]
fn token_expired_when_past_expiry()
{
  let tmp = tempfile::tempdir().expect( "temp dir" );
  // Expired 1 hour ago
  let expired_ms = now_ms().saturating_sub( 3_600_000 );
  let home = write_credentials( tmp.path(), expired_ms );
  std::env::set_var( "HOME", &home );

  let status = status_with_threshold( 60 ).expect( "read status" );
  assert_eq!( status, TokenStatus::Expired, "past expiry must be Expired" );
}

#[test]
fn token_expiring_soon_within_threshold()
{
  let tmp = tempfile::tempdir().expect( "temp dir" );
  // Expires in 30 seconds — well within a 60-second threshold
  let soon_ms = now_ms() + 30_000;
  let home = write_credentials( tmp.path(), soon_ms );
  std::env::set_var( "HOME", &home );

  let status = status_with_threshold( 60 ).expect( "read status" );
  assert!(
    matches!( status, TokenStatus::ExpiringSoon { .. } ),
    "token within threshold must be ExpiringSoon, got: {status:?}"
  );
}

#[test]
fn token_valid_far_from_expiry()
{
  let tmp = tempfile::tempdir().expect( "temp dir" );
  // Expires in 2 hours — well outside a 60-second threshold
  let far_ms = now_ms() + 7_200_000;
  let home = write_credentials( tmp.path(), far_ms );
  std::env::set_var( "HOME", &home );

  let status = status_with_threshold( 60 ).expect( "read status" );
  assert!(
    matches!( status, TokenStatus::Valid { .. } ),
    "token far from expiry must be Valid, got: {status:?}"
  );
}

// ─── classify_ms (pure, no I/O) ───────────────────────────────────────────────

#[test]
fn classify_ms_expired_for_past_timestamp()
{
  // Expired 1 hour ago — always in the past
  let expired_ms = now_ms().saturating_sub( 3_600_000 );
  assert_eq!(
    classify_ms( expired_ms, WARNING_THRESHOLD_SECS ),
    TokenStatus::Expired,
    "past millisecond must be Expired"
  );
}

#[test]
fn classify_ms_expiring_soon_within_threshold()
{
  // Expires in 30 seconds — within a 60-second threshold
  let soon_ms = now_ms() + 30_000;
  assert!(
    matches!( classify_ms( soon_ms, 60 ), TokenStatus::ExpiringSoon { .. } ),
    "timestamp within threshold must be ExpiringSoon"
  );
}

#[test]
fn classify_ms_valid_far_from_expiry()
{
  // Expires in 2 hours — outside a 60-second threshold
  let far_ms = now_ms() + 7_200_000;
  assert!(
    matches!( classify_ms( far_ms, 60 ), TokenStatus::Valid { .. } ),
    "timestamp far from expiry must be Valid"
  );
}
