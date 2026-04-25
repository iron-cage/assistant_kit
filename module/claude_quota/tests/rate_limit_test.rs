//! Unit tests: rate-limit header parsing and QuotaError display.
//!
//! Tests use the closure interface `parse_headers(|name| map.get(name).map(|s| s.to_string()))`
//! to avoid any dependency on `ureq::Response` or a live network.
//!
//! ## Test Matrix
//!
//! | ID  | Scenario                                        | Expected                              | Status |
//! |-----|-------------------------------------------------|---------------------------------------|--------|
//! | T01 | All 5 headers valid                             | Ok(RateLimitData) with correct fields | ✅     |
//! | T02 | `5h-utilization` header absent                  | Err(MissingHeader) naming the header  | ✅     |
//! | T03 | `7d-reset` header absent                        | Err(MissingHeader) naming the header  | ✅     |
//! | T04 | `5h-utilization` = `"not_a_float"`              | Err(MalformedHeader) with context     | ✅     |
//! | T05 | `5h-reset` = `"abc"`                            | Err(MalformedHeader) with context     | ✅     |
//! | T06 | `QuotaError::MissingHeader("x")` Display        | string contains "x"                   | ✅     |
//! | T07 | `QuotaError::HttpTransport("refused")` Display  | string contains "refused"             | ✅     |
//! | T08 | Field access on `RateLimitData`                 | utilization_5h = 0.42, status = "allowed" | ✅ |
//! | T09 | `QuotaError` implements `std::error::Error`     | all 3 variants pass trait bound       | ✅     |
//! | T10 | `5h-reset` header absent                        | Err(MissingHeader) naming the header  | ✅     |
//! | T11 | `7d-utilization` header absent                  | Err(MissingHeader) naming the header  | ✅     |
//! | T12 | `status` header absent                          | Err(MissingHeader) naming the header  | ✅     |
//! | T13 | `7d-utilization` = `"not_a_float"`              | Err(MalformedHeader) with context     | ✅     |
//! | T14 | `7d-reset` = `"not_a_u64"`                      | Err(MalformedHeader) with context     | ✅     |
//! | T15 | `QuotaError::MalformedHeader("ctx")` Display    | string contains "ctx"                 | ✅     |
//! | T16 | `ANTHROPIC_BETA` constant canary                | equals "oauth-2025-04-20"             | ✅     |
//!
//! ## Corner Cases Covered
//!
//! - ✅ All 5 headers present (happy path)
//! - ✅ Each of the 5 required headers absent individually (T02, T03, T10, T11, T12)
//! - ✅ Float headers malformed — 5h-utilization (T04), 7d-utilization (T13)
//! - ✅ u64 headers malformed — 5h-reset (T05), 7d-reset (T14)
//! - ✅ All 3 QuotaError variants: Display (T06, T07, T15) and std::error::Error bound (T09)
//! - ✅ RateLimitData field accessibility (T08)
//! - ✅ ANTHROPIC_BETA constant value canary (T16) — not in public docs; drift check
//! - N/A: status empty string — parses as Ok(status: "") by design; no range constraint
//! - N/A: utilization out of range (e.g. 1.5) — no range validation in spec; design choice

use std::collections::HashMap;
use claude_quota::{ parse_headers, RateLimitData, QuotaError, ANTHROPIC_BETA };

fn get( m : &HashMap< &str, &str >, name : &str ) -> Option< String >
{
  m.get( name ).map( |s| s.to_string() )
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn full_map() -> HashMap< &'static str, &'static str >
{
  let mut m = HashMap::new();
  m.insert( "anthropic-ratelimit-unified-5h-utilization", "0.42" );
  m.insert( "anthropic-ratelimit-unified-5h-reset",       "1700000000" );
  m.insert( "anthropic-ratelimit-unified-7d-utilization", "0.10" );
  m.insert( "anthropic-ratelimit-unified-7d-reset",       "1700086400" );
  m.insert( "anthropic-ratelimit-unified-status",         "allowed" );
  m
}

// ── T01 ───────────────────────────────────────────────────────────────────────

/// T01: All 5 headers present and valid — parse succeeds with correct field values.
#[ test ]
fn t01_all_headers_valid_returns_ok_with_correct_fields()
{
  let m = full_map();
  let result = parse_headers( |name| get( &m, name ) );
  let data = result.expect( "T01: expected Ok, got Err" );
  assert!(
    ( data.utilization_5h - 0.42 ).abs() < f64::EPSILON,
    "T01: utilization_5h should be 0.42, got {}", data.utilization_5h,
  );
  assert_eq!( data.reset_5h,       1700000000u64, "T01: reset_5h" );
  assert!(
    ( data.utilization_7d - 0.10 ).abs() < f64::EPSILON,
    "T01: utilization_7d should be 0.10, got {}", data.utilization_7d,
  );
  assert_eq!( data.reset_7d,       1700086400u64, "T01: reset_7d" );
  assert_eq!( data.status,         "allowed",     "T01: status" );
}

// ── T02 ───────────────────────────────────────────────────────────────────────

/// T02: `5h-utilization` header absent — returns MissingHeader naming the header.
#[ test ]
fn t02_missing_5h_utilization_returns_missing_header_error()
{
  let mut m = full_map();
  m.remove( "anthropic-ratelimit-unified-5h-utilization" );
  let result = parse_headers( |name| get( &m, name ) );
  match result
  {
    Err( QuotaError::MissingHeader( ref s ) ) =>
      assert!( s.contains( "5h-utilization" ), "T02: error should name '5h-utilization', got: {s}" ),
    other => panic!( "T02: expected MissingHeader, got: {other:?}" ),
  }
}

// ── T03 ───────────────────────────────────────────────────────────────────────

/// T03: `7d-reset` header absent — returns MissingHeader naming the header.
#[ test ]
fn t03_missing_7d_reset_returns_missing_header_error()
{
  let mut m = full_map();
  m.remove( "anthropic-ratelimit-unified-7d-reset" );
  let result = parse_headers( |name| get( &m, name ) );
  match result
  {
    Err( QuotaError::MissingHeader( ref s ) ) =>
      assert!( s.contains( "7d-reset" ), "T03: error should name '7d-reset', got: {s}" ),
    other => panic!( "T03: expected MissingHeader, got: {other:?}" ),
  }
}

// ── T04 ───────────────────────────────────────────────────────────────────────

/// T04: `5h-utilization` = `"not_a_float"` — returns MalformedHeader with context.
#[ test ]
fn t04_malformed_5h_utilization_returns_malformed_header_error()
{
  let mut m = full_map();
  m.insert( "anthropic-ratelimit-unified-5h-utilization", "not_a_float" );
  let result = parse_headers( |name| get( &m, name ) );
  match result
  {
    Err( QuotaError::MalformedHeader( ref s ) ) =>
      assert!( s.contains( "5h" ) || s.contains( "utilization" ), "T04: context should mention header, got: {s}" ),
    other => panic!( "T04: expected MalformedHeader, got: {other:?}" ),
  }
}

// ── T05 ───────────────────────────────────────────────────────────────────────

/// T05: `5h-reset` = `"abc"` — returns MalformedHeader with context.
#[ test ]
fn t05_malformed_5h_reset_returns_malformed_header_error()
{
  let mut m = full_map();
  m.insert( "anthropic-ratelimit-unified-5h-reset", "abc" );
  let result = parse_headers( |name| get( &m, name ) );
  match result
  {
    Err( QuotaError::MalformedHeader( ref s ) ) =>
      assert!( s.contains( "5h" ) || s.contains( "reset" ), "T05: context should mention header, got: {s}" ),
    other => panic!( "T05: expected MalformedHeader, got: {other:?}" ),
  }
}

// ── T06 ───────────────────────────────────────────────────────────────────────

/// T06: `QuotaError::MissingHeader("x")` Display contains the header name.
#[ test ]
fn t06_missing_header_display_contains_header_name()
{
  let err = QuotaError::MissingHeader( "x".to_string() );
  let s = err.to_string();
  assert!( s.contains( 'x' ), "T06: Display should contain 'x', got: {s}" );
}

// ── T07 ───────────────────────────────────────────────────────────────────────

/// T07: `QuotaError::HttpTransport("refused")` Display contains the message.
#[ test ]
fn t07_http_transport_display_contains_message()
{
  let err = QuotaError::HttpTransport( "refused".to_string() );
  let s = err.to_string();
  assert!( s.contains( "refused" ), "T07: Display should contain 'refused', got: {s}" );
}

// ── T08 ───────────────────────────────────────────────────────────────────────

/// T08: Fields on `RateLimitData` are accessible and hold expected values.
#[ test ]
fn t08_rate_limit_data_fields_accessible()
{
  let data = RateLimitData
  {
    utilization_5h : 0.42,
    reset_5h       : 100,
    utilization_7d : 0.10,
    reset_7d       : 200,
    status         : "allowed".to_string(),
  };
  assert!( ( data.utilization_5h - 0.42 ).abs() < f64::EPSILON );
  assert_eq!( data.status, "allowed" );
}

// ── T09 ───────────────────────────────────────────────────────────────────────

/// T09: `QuotaError` implements `std::error::Error`.
///
/// Verifies that all three variants are boxable as `Box<dyn Error>`, enabling
/// callers to use the `?` operator in functions returning `Box<dyn Error>`.
#[ test ]
fn t09_quota_error_implements_std_error()
{
  fn assert_error< E : std::error::Error >( _e : E ) {}
  assert_error( QuotaError::MissingHeader(  "h".to_string() ) );
  assert_error( QuotaError::MalformedHeader( "h: bad".to_string() ) );
  assert_error( QuotaError::HttpTransport(  "refused".to_string() ) );
}

// ── T10 ───────────────────────────────────────────────────────────────────────

/// T10: `5h-reset` header absent — returns MissingHeader naming that header.
///
/// Completes the missing-header matrix: T02 covers 5h-utilization, T03 covers
/// 7d-reset; this verifies the 5h-reset field is also individually required.
#[ test ]
fn t10_missing_5h_reset_returns_missing_header_error()
{
  let mut m = full_map();
  m.remove( "anthropic-ratelimit-unified-5h-reset" );
  let result = parse_headers( |name| get( &m, name ) );
  match result
  {
    Err( QuotaError::MissingHeader( ref s ) ) =>
      assert!( s.contains( "5h-reset" ), "T10: error should name '5h-reset', got: {s}" ),
    other => panic!( "T10: expected MissingHeader, got: {other:?}" ),
  }
}

// ── T11 ───────────────────────────────────────────────────────────────────────

/// T11: `7d-utilization` header absent — returns MissingHeader naming that header.
///
/// Completes the missing-header matrix for the second 7-day field.
#[ test ]
fn t11_missing_7d_utilization_returns_missing_header_error()
{
  let mut m = full_map();
  m.remove( "anthropic-ratelimit-unified-7d-utilization" );
  let result = parse_headers( |name| get( &m, name ) );
  match result
  {
    Err( QuotaError::MissingHeader( ref s ) ) =>
      assert!( s.contains( "7d-utilization" ), "T11: error should name '7d-utilization', got: {s}" ),
    other => panic!( "T11: expected MissingHeader, got: {other:?}" ),
  }
}

// ── T12 ───────────────────────────────────────────────────────────────────────

/// T12: `status` header absent — returns MissingHeader naming that header.
///
/// Completes the missing-header matrix for the fifth and final required header.
#[ test ]
fn t12_missing_status_returns_missing_header_error()
{
  let mut m = full_map();
  m.remove( "anthropic-ratelimit-unified-status" );
  let result = parse_headers( |name| get( &m, name ) );
  match result
  {
    Err( QuotaError::MissingHeader( ref s ) ) =>
      assert!( s.contains( "status" ), "T12: error should name 'status', got: {s}" ),
    other => panic!( "T12: expected MissingHeader, got: {other:?}" ),
  }
}

// ── T13 ───────────────────────────────────────────────────────────────────────

/// T13: `7d-utilization` = `"not_a_float"` — returns MalformedHeader with context.
///
/// Completes the malformed-header matrix for the second float field.
/// T04 covers 5h-utilization; this covers 7d-utilization.
#[ test ]
fn t13_malformed_7d_utilization_returns_malformed_header_error()
{
  let mut m = full_map();
  m.insert( "anthropic-ratelimit-unified-7d-utilization", "not_a_float" );
  let result = parse_headers( |name| get( &m, name ) );
  match result
  {
    Err( QuotaError::MalformedHeader( ref s ) ) =>
      assert!( s.contains( "7d" ) || s.contains( "utilization" ), "T13: context should mention header, got: {s}" ),
    other => panic!( "T13: expected MalformedHeader, got: {other:?}" ),
  }
}

// ── T14 ───────────────────────────────────────────────────────────────────────

/// T14: `7d-reset` = `"not_a_u64"` — returns MalformedHeader with context.
///
/// Completes the malformed-header matrix for the second u64 field.
/// T05 covers 5h-reset; this covers 7d-reset.
#[ test ]
fn t14_malformed_7d_reset_returns_malformed_header_error()
{
  let mut m = full_map();
  m.insert( "anthropic-ratelimit-unified-7d-reset", "not_a_u64" );
  let result = parse_headers( |name| get( &m, name ) );
  match result
  {
    Err( QuotaError::MalformedHeader( ref s ) ) =>
      assert!( s.contains( "7d" ) || s.contains( "reset" ), "T14: context should mention header, got: {s}" ),
    other => panic!( "T14: expected MalformedHeader, got: {other:?}" ),
  }
}

// ── T15 ───────────────────────────────────────────────────────────────────────

/// T15: `QuotaError::MalformedHeader("ctx")` Display contains the context string.
///
/// Completes the Display coverage matrix: T06 covers MissingHeader, T07 covers
/// HttpTransport; this verifies the third variant formats its context string.
#[ test ]
fn t15_malformed_header_display_contains_context()
{
  let err = QuotaError::MalformedHeader( "ctx".to_string() );
  let s = err.to_string();
  assert!( s.contains( "ctx" ), "T15: Display should contain 'ctx', got: {s}" );
}

// ── T16 ───────────────────────────────────────────────────────────────────────

/// T16: `ANTHROPIC_BETA` constant equals the expected OAuth beta string.
///
/// This is a canary test: the beta string is not in public Anthropic API docs
/// and was discovered via `strings $(which claude) | grep oauth`. If the Claude
/// binary updates and the beta string changes, live tests will fail with
/// "OAuth authentication is currently not supported" — this test makes that
/// failure loud and immediate rather than silently wrong.
///
/// To update: run `strings $(which claude) | grep oauth` to find the new value,
/// then update `ANTHROPIC_BETA` in `src/lib.rs` and this test together.
#[ test ]
fn t16_anthropic_beta_constant_has_expected_value()
{
  assert_eq!(
    ANTHROPIC_BETA,
    "oauth-2025-04-20",
    "T16: ANTHROPIC_BETA has drifted — re-run `strings $(which claude) | grep oauth` to find new value",
  );
}
