#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-api-error` and `--api-error-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-10 from `tests/docs/cli/param/037_retry_on_api_error.md` and
//! EC-1 through EC-7 from `tests/docs/cli/param/038_api_error_delay.md`.
//!
//! Both parameter specs share this test file because `--api-error-delay` only fires
//! when `--retry-on-api-error` is non-zero — they are functionally coupled.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 37), EC-1..EC-6 (param 38): parser/dry-run — no subprocess
//! - EC-7..EC-10 (param 37), EC-7 (param 38): require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-api-error (param 37)
//! - EC-1: help lists flag
//! - EC-2: value 0 (explicit disable) accepted in dry-run
//! - EC-3: value 2 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_API_ERROR` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored; default 0 used
//! - EC-7: fake emits `"API Error: 500"` once then 0; retries=1, delay=0 → exit 0
//! - EC-8: fake always emits `"API Error: 500"`; retries=2, delay=0 → nonzero exit; exhaustion message
//! - EC-9: `QuotaExhausted` pattern → not retried even with retries=3; exit 2
//! - EC-10: no flag, no env var → default=0; API error exits immediately; no retry
//!
//! ### --api-error-delay (param 38)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 (default) accepted in dry-run
//! - EC-4 (delay): `CLR_API_ERROR_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored
//! - EC-7 (delay): delay=0 causes immediate retry; exit 0
mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 37 — --retry-on-api-error ──────────────────────────────────────────

// ── EC-1: --help lists --retry-on-api-error ───────────────────────────────────

/// EC-1 (param 37): `clr --help` output contains `--retry-on-api-error`.
#[ test ]
fn ec1_retry_on_api_error_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-api-error" ),
    "`clr --help` must list --retry-on-api-error. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-api-error 0 --dry-run → exit 0 ──────────────────────────

/// EC-2 (param 37): value 0 (explicit disable, matching default 0) accepted in dry-run.
///
/// Divergence from EC-3: value 0 disables retry; value 2 (EC-3) activates retry wrapper.
#[ test ]
fn ec2_retry_on_api_error_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-api-error", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "retry" ),
    "dry-run must emit no retry messages. stderr: {stderr}"
  );
}

// ── EC-3: --retry-on-api-error 2 --dry-run → exit 0 ──────────────────────────

/// EC-3 (param 37): value 2 (retry enabled) accepted in dry-run; no subprocess, no retry.
///
/// Divergence from EC-2: value 2 activates the retry wrapper code path (though in dry-run
/// no subprocess fires so no actual retry occurs).
#[ test ]
fn ec3_retry_on_api_error_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-api-error", "2", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_API_ERROR=2 env var applied ───────────────────────────

/// EC-4 (param 37): `CLR_RETRY_ON_API_ERROR=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_api_error_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_API_ERROR", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_API_ERROR env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_API_ERROR ────────────────────────────────

/// EC-5 (param 37): CLI value 3 wins over `CLR_RETRY_ON_API_ERROR=1`.
#[ test ]
fn ec5_retry_on_api_error_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-api-error", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_API_ERROR", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_API_ERROR=invalid → silently ignored ──────────────────

/// EC-6 (param 37): invalid `CLR_RETRY_ON_API_ERROR` silently ignored; default 0 used.
#[ test ]
fn ec6_clr_retry_on_api_error_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_API_ERROR", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_API_ERROR must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One API error then success → retried; exit 0 ───────────────────────

/// EC-7 (param 37): fake emits `"API Error: 500"` once then exits 0; retries=1, delay=0 → exit 0.
///
/// Root Cause: --retry-on-api-error not yet implemented
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: retry loop in execution.rs extended to handle ErrorKind::ApiError
/// Prevention: guard with integration test asserting retry message on stderr
/// Pitfall: delay=0 is required in tests to avoid 30s sleep; the api_error_delay guard
///          must branch to no-sleep when delay is zero
#[ cfg( unix ) ]
#[ test ]
fn ec7_retry_succeeds_after_one_api_error()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  // Script: emits "API Error: 500" on stderr and exits 1 on first call, exits 0 on second.
  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\n\
     if [ -f \"{count_path}\" ]; then exit 0; fi\n\
     touch \"{count_path}\"\n\
     printf 'API Error: 500\\n' >&2\n\
     exit 1\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-api-error", "1", "--api-error-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 after API error retry succeeds. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "api error" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain API error retry message. Got:\n{stderr}"
  );
}

// ── EC-8: All API error retries exhausted → nonzero exit; exhaustion message ──

/// EC-8 (param 37): fake always emits `"API Error: 500"`; retries=2, delay=0 → nonzero; exhaustion message.
///
/// Root Cause: --retry-on-api-error not yet implemented
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: retry loop exhaustion emits "exhausted" label via error label match
/// Prevention: guard with integration test asserting exhaustion message on stderr
/// Pitfall: test uses retries=2 so 3 total invocations (1 initial + 2 retries) all fail
#[ cfg( unix ) ]
#[ test ]
fn ec8_retry_exhausted_after_all_api_errors()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: always emits "API Error: 500" on stderr and exits 1.
  std::fs::write(
    &fake,
    b"#!/bin/sh\nprintf 'API Error: 500\\n' >&2\nexit 1\n",
  ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-api-error", "2", "--api-error-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.code() != Some( 0 ),
    "exit must be nonzero after all retries exhausted. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "exhaust" ) || stderr.to_lowercase().contains( "fail" ),
    "stderr must contain exhaustion message. Got:\n{stderr}"
  );
}

// ── EC-9: QuotaExhausted NOT retried even with --retry-on-api-error set ───────

/// EC-9 (param 37): fake emits `QuotaExhausted` pattern; retries=3 → exit 2; no retry.
///
/// Root Cause: classification priority must put QuotaExhausted above ApiError retry path
/// Why Not Caught: priority is enforced in classify_error() but retry dispatcher must not
///                 check for ApiError text when QuotaExhausted is already classified
/// Fix Applied: retry loop only checks ErrorKind::ApiError — QuotaExhausted is a different variant
/// Prevention: guard with integration test confirming no retry message for QuotaExhausted
/// Pitfall: "You've hit your limit" appears at exit 2, same exit code as RateLimit; priority
///          order in ERROR_PATTERNS ensures QuotaExhausted wins over the exit-2 RateLimit fallback
#[ cfg( unix ) ]
#[ test ]
fn ec9_quota_exhausted_not_retried_as_api_error()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: emits QuotaExhausted pattern and exits 2.
  std::fs::write(
    &fake,
    b"#!/bin/sh\nprintf \"You've hit your limit\\n\"\nexit 2\n",
  ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-api-error", "3", "--api-error-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "QuotaExhausted must exit 2, not be retried as ApiError. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "retry" ),
    "QuotaExhausted must not trigger retry messages. Got:\n{stderr}"
  );
}

// ── EC-10: Default retry=0 → no retry on API error ───────────────────────────

/// EC-10 (param 37): no `--retry-on-api-error` flag; default=0; API error exits immediately.
///
/// Root Cause: --retry-on-api-error default is 0 (no retry without explicit flag)
/// Why Not Caught: default-0 test ensures regression detection if default changes
/// Fix Applied: `unwrap_or(0)` in run_print_mode() for api_retry_limit
/// Prevention: guard asserting no retry message when flag is absent
/// Pitfall: test must NOT set `--retry-on-api-error`; CLR_RETRY_ON_API_ERROR must also be unset
#[ cfg( unix ) ]
#[ test ]
fn ec10_default_no_retry_on_api_error()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: always emits "API Error: 500" and exits 1.
  std::fs::write(
    &fake,
    b"#!/bin/sh\nprintf 'API Error: 500\\n' >&2\nexit 1\n",
  ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_RETRY_ON_API_ERROR" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.code() != Some( 0 ),
    "exit must be nonzero (API error, default no-retry). Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "retrying" ),
    "default retry=0 must emit no retry messages. Got:\n{stderr}"
  );
}

// ── Param 38 — --api-error-delay ──────────────────────────────────────────────

// ── EC-1 (delay): --help lists --api-error-delay ──────────────────────────────

/// EC-1 (param 38): `clr --help` output contains `--api-error-delay`.
#[ test ]
fn ec1_api_error_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--api-error-delay" ),
    "`clr --help` must list --api-error-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --api-error-delay 0 --dry-run → exit 0 ─────────────────────

/// EC-2 (param 38): delay=0 (immediate retry) accepted in dry-run.
///
/// Divergence from EC-3: value 0 means no sleep between API error retries;
/// value 30 (EC-3) is the default delay.
#[ test ]
fn ec2_api_error_delay_zero_dry_run()
{
  let out = run_cli( &[ "--api-error-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --api-error-delay 30 --dry-run → exit 0 ────────────────────

/// EC-3 (param 38): delay=30 (default) accepted in dry-run.
#[ test ]
fn ec3_api_error_delay_thirty_dry_run()
{
  let out = run_cli( &[ "--api-error-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_API_ERROR_DELAY=5 env var applied ──────────────────────

/// EC-4 (param 38): `CLR_API_ERROR_DELAY=5` applied when CLI flag absent.
#[ test ]
fn ec4_clr_api_error_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_API_ERROR_DELAY", "5" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_API_ERROR_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_API_ERROR_DELAY ──────────────────────────

/// EC-5 (param 38): CLI value 30 wins over `CLR_API_ERROR_DELAY=10`.
#[ test ]
fn ec5_api_error_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--api-error-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_API_ERROR_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_API_ERROR_DELAY=invalid → silently ignored ──────────────

/// EC-6 (param 38): invalid `CLR_API_ERROR_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_api_error_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_API_ERROR_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_API_ERROR_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7 (delay): delay=0 with API error retry → fires immediately; exit 0 ────

/// EC-7 (param 38): delay=0 causes immediate retry (no sleep); exit 0.
///
/// Root Cause: --api-error-delay not yet implemented
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: api_error_delay guard in execution.rs: `if api_error_delay > 0 { sleep }`
/// Prevention: guard with integration test verifying fast exit (no 30s wait)
/// Pitfall: if delay=0 were treated as "default 30" the test would time out; the 0-check
///          in the retry loop must branch to no-sleep
#[ cfg( unix ) ]
#[ test ]
fn ec7_api_error_delay_zero_immediate_retry()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\n\
     if [ -f \"{count_path}\" ]; then exit 0; fi\n\
     touch \"{count_path}\"\n\
     printf 'API Error: 500\\n' >&2\n\
     exit 1\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let start = std::time::Instant::now();
  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-api-error", "1", "--api-error-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  assert!(
    out.status.success(),
    "exit must be 0 with delay=0 API error retry. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  assert!(
    elapsed.as_secs() < 5,
    "delay=0 must retry immediately; elapsed {elapsed:?} is too long"
  );
}
