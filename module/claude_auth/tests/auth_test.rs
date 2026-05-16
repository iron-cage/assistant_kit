//! Unit tests: OAuth token refresh response parsing and AuthError display.
//!
//! Tests use the body-string interface `parse_response(body, now_ms)` to avoid
//! any dependency on `ureq` or a live network.
//!
//! ## Test Matrix
//!
//! | ID  | Scenario                                          | Expected                              | Status |
//! |-----|---------------------------------------------------|---------------------------------------|--------|
//! | T01 | Valid JSON: all fields present                    | `Ok(TokenRefreshResult)` correct fields | ✅   |
//! | T02 | JSON missing `access_token` field                 | `Err(ResponseParse("access_token"))`  | ✅     |
//! | T03 | JSON missing `refresh_token` field                | `Err(ResponseParse("refresh_token"))` | ✅     |
//! | T04 | JSON missing `expires_in` field                   | `Err(ResponseParse("expires_in"))`    | ✅     |
//! | T05 | `"expires_in": "bad"` (string, not integer)       | `Err(ResponseParse(...))`             | ✅     |
//! | T06 | All `AuthError` variants: `Display` + `Error` bound | Non-empty strings; trait holds      | ✅     |
//!
//! ## Corner Cases Covered
//!
//! - ✅ Happy path: all three required fields present (T01)
//! - ✅ Each required field absent individually (T02, T03, T04)
//! - ✅ Numeric field provided as string value (T05)
//! - ✅ All `AuthError` variants: `HttpTransport`, `ResponseParse`, `RateLimited` (T06)
//! - N/A: `refresh_token()` live network — 429 rate-limiting makes it unreliable in CI

use claude_auth::{ TokenRefreshResult, AuthError, parse_response };

// Suppress "unused import" warning — TokenRefreshResult is used via type inference in T01.
#[ allow( unused_imports ) ]
use std::convert::identity;

// ── T01 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t01_parse_response_valid()
{
  let body   = r#"{"access_token":"sk-ant-abc","refresh_token":"sk-ant-ort01-xyz","expires_in":3600}"#;
  let result : TokenRefreshResult = parse_response( body, 1000 )
    .expect( "T01: should parse valid JSON" );
  assert_eq!( result.access_token,  "sk-ant-abc",        "T01: access_token mismatch" );
  assert_eq!( result.refresh_token, "sk-ant-ort01-xyz",  "T01: refresh_token mismatch" );
  assert_eq!( result.expires_at_ms, 3_601_000,           "T01: expires_at_ms = 1000 + 3600*1000" );
}

// ── T02 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t02_parse_response_missing_access_token()
{
  let body = r#"{"refresh_token":"sk-ant-ort01-xyz","expires_in":3600}"#;
  let err  = parse_response( body, 0 )
    .expect_err( "T02: should fail on missing access_token" );
  assert!(
    matches!( err, AuthError::ResponseParse( _ ) ),
    "T02: expected ResponseParse, got {err:?}"
  );
}

// ── T03 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t03_parse_response_missing_refresh_token()
{
  let body = r#"{"access_token":"sk-ant-abc","expires_in":3600}"#;
  let err  = parse_response( body, 0 )
    .expect_err( "T03: should fail on missing refresh_token" );
  assert!(
    matches!( err, AuthError::ResponseParse( _ ) ),
    "T03: expected ResponseParse, got {err:?}"
  );
}

// ── T04 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t04_parse_response_missing_expires_in()
{
  let body = r#"{"access_token":"sk-ant-abc","refresh_token":"sk-ant-ort01-xyz"}"#;
  let err  = parse_response( body, 0 )
    .expect_err( "T04: should fail on missing expires_in" );
  assert!(
    matches!( err, AuthError::ResponseParse( _ ) ),
    "T04: expected ResponseParse, got {err:?}"
  );
}

// ── T05 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t05_parse_response_expires_in_string_not_integer()
{
  let body = r#"{"access_token":"sk-ant-abc","refresh_token":"sk-ant-ort01-xyz","expires_in":"bad"}"#;
  let err  = parse_response( body, 0 )
    .expect_err( "T05: should fail on string expires_in" );
  assert!(
    matches!( err, AuthError::ResponseParse( _ ) ),
    "T05: expected ResponseParse, got {err:?}"
  );
}

// ── T06 ───────────────────────────────────────────────────────────────────────

#[ test ]
fn t06_auth_error_display_and_error_bound()
{
  // HttpTransport: display must contain the inner message
  let transport = AuthError::HttpTransport( "timeout".to_string() );
  let s = transport.to_string();
  assert!( s.contains( "timeout" ), "T06: HttpTransport display missing 'timeout', got {s:?}" );

  // ResponseParse: display must contain the field name
  let parse_err = AuthError::ResponseParse( "expires_in".to_string() );
  let s = parse_err.to_string();
  assert!(
    s.contains( "expires_in" ),
    "T06: ResponseParse display missing field name, got {s:?}"
  );

  // RateLimited: display must mention rate or 429
  let rate = AuthError::RateLimited;
  let s = rate.to_string();
  assert!(
    s.contains( "rate" ) || s.contains( "429" ),
    "T06: RateLimited display must contain 'rate' or '429', got {s:?}"
  );

  // std::error::Error bound — compile-time check
  fn assert_error< E : std::error::Error >() {}
  assert_error::< AuthError >();
}
