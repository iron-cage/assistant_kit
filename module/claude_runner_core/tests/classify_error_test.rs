//! Unit tests for `ExecutionOutput::classify_error()` and `ErrorKind`.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Expected |
//! |------|----------|---------|
//! | T01 | exit=2, empty stderr+stdout | `Some(RateLimit)` |
//! | T02 | exit=0, empty stderr+stdout | `None` |
//! | T03 | exit=1, stderr="You've hit your limit" | `Some(RateLimit)` |
//! | T04 | exit=1, stdout="Your organization does not have access to Claude" | `Some(AuthError)` |
//! | T05 | exit=1, stderr="API Error: 529 ..." | `Some(ApiError)` |
//! | T06 | exit=130, empty | `Some(Signal)` |
//! | T07 | exit=143, empty | `Some(Signal)` |
//! | T08 | exit=1, empty stderr+stdout | `Some(Unknown)` |
//! | T11 | exit=1, stderr="API Error: ..." | `Some(ApiError)` not `Unknown` |
//! | T12 | exit=1, stderr="Your organization does not have access to Claude" | `Some(AuthError)` |
//!
//! # Root Cause (BUG-037)
//!
//! `run_print_mode` emitted a generic "possible rate limit or quota exhaustion" message
//! for ALL silent non-zero exits, hiding the actual failure mode from callers and logs.
//!
//! # Why Not Caught
//!
//! No pre-existing test asserted specific `ErrorKind` variants; `classify_error()` did not exist.
//!
//! # Fix Applied
//!
//! `ErrorKind` enum added to `types.rs` with `classify_error()` on `ExecutionOutput`:
//! priority-ordered pattern scan (stderr+stdout) then exit-code fallbacks.
//!
//! # Prevention
//!
//! Cover all 5 `ErrorKind` variants + `None` (success) + both stderr and stdout scan paths.
//!
//! # Pitfall
//!
//! Pattern priority matters: auth pattern must match before `ApiError` for 401 responses
//! that contain both "Your organization does not have access" and "API Error: " text.

use claude_runner_core::{ ErrorKind, ExecutionOutput };

fn make_output( stdout : &str, stderr : &str, exit_code : i32 ) -> ExecutionOutput
{
  ExecutionOutput
  {
    stdout    : stdout.to_string(),
    stderr    : stderr.to_string(),
    exit_code,
  }
}

// ── T01 ───────────────────────────────────────────────────────────────────────

/// T01: exit code 2 with empty output → `RateLimit` (canonical rate-limit sentinel).
#[ test ]
fn classify_error_exit2_empty_is_rate_limit()
{
  let out = make_output( "", "", 2 );
  assert_eq!(
    out.classify_error(),
    Some( ErrorKind::RateLimit ),
    "T01: exit_code=2 with empty output must yield RateLimit"
  );
}

// ── T02 ───────────────────────────────────────────────────────────────────────

/// T02: exit code 0 → None (success, no classification).
#[ test ]
fn classify_error_exit0_is_none()
{
  let out = make_output( "", "", 0 );
  assert_eq!(
    out.classify_error(),
    None,
    "T02: exit_code=0 must yield None regardless of stderr/stdout"
  );
}

// ── T03 ───────────────────────────────────────────────────────────────────────

/// T03: rate-limit text in stderr with exit code 1 → `RateLimit` (pattern match wins over
/// exit-code fallback).
#[ test ]
fn classify_error_rate_limit_pattern_in_stderr()
{
  let out = make_output( "", "You've hit your limit", 1 );
  assert_eq!(
    out.classify_error(),
    Some( ErrorKind::RateLimit ),
    "T03: rate-limit pattern in stderr must yield RateLimit"
  );
}

// ── T04 ───────────────────────────────────────────────────────────────────────

/// T04: auth pattern in stdout only → `AuthError` (stdout scan path verified).
#[ test ]
fn classify_error_auth_pattern_in_stdout()
{
  let out = make_output(
    "Your organization does not have access to Claude",
    "",
    1,
  );
  assert_eq!(
    out.classify_error(),
    Some( ErrorKind::AuthError ),
    "T04: auth pattern in stdout must yield AuthError"
  );
}

// ── T05 ───────────────────────────────────────────────────────────────────────

/// T05: API error text in stderr → `ApiError`.
#[ test ]
fn classify_error_api_error_pattern_in_stderr()
{
  let out = make_output( "", "API Error: 529 overloaded", 1 );
  assert_eq!(
    out.classify_error(),
    Some( ErrorKind::ApiError ),
    "T05: API Error pattern in stderr must yield ApiError"
  );
}

// ── T06 ───────────────────────────────────────────────────────────────────────

/// T06: exit code 130 (SIGINT) with empty output → Signal.
#[ test ]
fn classify_error_exit130_is_signal()
{
  let out = make_output( "", "", 130 );
  assert_eq!(
    out.classify_error(),
    Some( ErrorKind::Signal ),
    "T06: exit_code=130 must yield Signal"
  );
}

// ── T07 ───────────────────────────────────────────────────────────────────────

/// T07: exit code 143 (SIGTERM) with empty output → Signal.
#[ test ]
fn classify_error_exit143_is_signal()
{
  let out = make_output( "", "", 143 );
  assert_eq!(
    out.classify_error(),
    Some( ErrorKind::Signal ),
    "T07: exit_code=143 must yield Signal"
  );
}

// ── T08 ───────────────────────────────────────────────────────────────────────

/// T08: exit code 1 with no pattern match and no signal code → Unknown.
#[ test ]
fn classify_error_exit1_empty_is_unknown()
{
  let out = make_output( "", "", 1 );
  assert_eq!(
    out.classify_error(),
    Some( ErrorKind::Unknown ),
    "T08: exit_code=1 with no pattern and no signal code must yield Unknown"
  );
}

// ── T11 ───────────────────────────────────────────────────────────────────────

/// T11: "API Error: " text in stderr with exit code 1 → `ApiError`, NOT `Unknown`.
/// Guards against a regression where only the exit-code path fires.
#[ test ]
fn classify_error_api_error_not_unknown()
{
  let out = make_output( "", "API Error: 500 internal server error", 1 );
  let kind = out.classify_error();
  assert_eq!(
    kind,
    Some( ErrorKind::ApiError ),
    "T11: API Error pattern must yield ApiError, not Unknown; got {kind:?}"
  );
}

// ── T12 ───────────────────────────────────────────────────────────────────────

/// T12: auth pattern in stderr (not stdout) → `AuthError` (stderr scan path verified).
#[ test ]
fn classify_error_auth_pattern_in_stderr()
{
  let out = make_output(
    "",
    "Your organization does not have access to Claude",
    1,
  );
  assert_eq!(
    out.classify_error(),
    Some( ErrorKind::AuthError ),
    "T12: auth pattern in stderr must yield AuthError"
  );
}

// ── Priority ──────────────────────────────────────────────────────────────────

/// Auth pattern takes priority over `ApiError` when both are present in stderr.
/// Guards BUG-037 priority ordering: 401 responses often contain both strings.
#[ test ]
fn classify_error_auth_before_api_error_priority()
{
  let out = make_output(
    "",
    "Your organization does not have access to Claude\nAPI Error: 401 unauthorized",
    1,
  );
  assert_eq!(
    out.classify_error(),
    Some( ErrorKind::AuthError ),
    "Priority: auth pattern must take precedence over API Error pattern"
  );
}

// ── Structural ────────────────────────────────────────────────────────────────

/// `ErrorKind` derives `Debug`, `Clone`, `PartialEq`, `Eq` — all five variants round-trip.
#[ test ]
fn error_kind_derives_are_correct()
{
  let variants = [
    ErrorKind::RateLimit,
    ErrorKind::ApiError,
    ErrorKind::AuthError,
    ErrorKind::Signal,
    ErrorKind::Unknown,
  ];
  for v in &variants
  {
    let cloned = v.clone();
    assert_eq!( v, &cloned, "ErrorKind::{v:?} must equal its clone" );
  }
  let debug = format!( "{:?}", ErrorKind::RateLimit );
  assert!( debug.contains( "RateLimit" ), "Debug must show variant name" );
}
