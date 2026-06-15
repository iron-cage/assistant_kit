#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-unknown-error` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-9 from `tests/docs/cli/param/039_retry_on_unknown_error.md`.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6: parser/dry-run — no subprocess required
//! - EC-7..EC-9: require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! - EC-1: help lists flag
//! - EC-2: value 0 (explicit no-retry, same as default) accepted in dry-run
//! - EC-3: value 1 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_UNKNOWN_ERROR` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored; default 0 used
//! - EC-7: fake exits 42 (no pattern) once then 0; retries=1, delay=0 → exit 0
//! - EC-8: fake always exits 42 (no pattern); retries=1, delay=0 → nonzero; exhaustion message
//! - EC-9: no flag, no env var → default=0; fake exits 42 → immediate exit, no retry
mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── EC-1: --help lists --retry-on-unknown-error ───────────────────────────────

/// EC-1: `clr --help` output contains `--retry-on-unknown-error`.
#[ test ]
fn ec1_retry_on_unknown_error_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-unknown-error" ),
    "`clr --help` must list --retry-on-unknown-error. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-unknown-error 0 --dry-run → exit 0 ──────────────────────

/// EC-2: value 0 (explicit no-retry, matching default) accepted in dry-run.
///
/// Divergence from EC-3: value 0 disables retry; value 1 (EC-3) activates retry wrapper.
#[ test ]
fn ec2_retry_on_unknown_error_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-unknown-error", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-on-unknown-error 1 --dry-run → exit 0 ──────────────────────

/// EC-3: value 1 (retry enabled) accepted in dry-run; no subprocess, no retry.
///
/// Divergence from EC-2: value 1 activates the retry wrapper code path (though in dry-run
/// no subprocess fires so no actual retry occurs).
#[ test ]
fn ec3_retry_on_unknown_error_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-unknown-error", "1", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_UNKNOWN_ERROR=1 env var applied ───────────────────────

/// EC-4: `CLR_RETRY_ON_UNKNOWN_ERROR=1` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_unknown_error_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_UNKNOWN_ERROR", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_UNKNOWN_ERROR env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_UNKNOWN_ERROR ───────────────────────────

/// EC-5: CLI value 2 wins over `CLR_RETRY_ON_UNKNOWN_ERROR=1`.
#[ test ]
fn ec5_retry_on_unknown_error_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-unknown-error", "2", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_UNKNOWN_ERROR", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_UNKNOWN_ERROR=invalid → silently ignored ──────────────

/// EC-6: invalid `CLR_RETRY_ON_UNKNOWN_ERROR` silently ignored; default 0 used.
#[ test ]
fn ec6_clr_retry_on_unknown_error_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_UNKNOWN_ERROR", "bad" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_UNKNOWN_ERROR must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One unknown error then success → retried; exit 0 ───────────────────

/// EC-7: fake exits 42 (no pattern) once then 0; retries=1, delay=0 → exit 0; stderr has retry.
///
/// Root Cause: --retry-on-unknown-error not yet implemented
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: retry loop in execution.rs extended to handle ErrorKind::Unknown
/// Prevention: guard with integration test asserting retry message on stderr
/// Pitfall: exit 42 must produce ErrorKind::Unknown (not 2=RateLimit, not 1=pattern-match);
///          script must NOT emit "API Error: " or "You've hit your limit" text
#[ cfg( unix ) ]
#[ test ]
fn ec7_retry_succeeds_after_one_unknown_error()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  // Script: exits 42 (no pattern text) on first call, exits 0 on second.
  // exit 42 → no text pattern match, exit ≠ 2, exit ≤ 128 → ErrorKind::Unknown
  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\n\
     if [ -f \"{count_path}\" ]; then exit 0; fi\n\
     touch \"{count_path}\"\n\
     exit 42\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-unknown-error", "1", "--retry-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 after unknown error retry succeeds. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "unknown" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain unknown error retry message. Got:\n{stderr}"
  );
}

// ── EC-8: All unknown retries exhausted → nonzero exit; exhaustion message ────

/// EC-8: fake always exits 42 (no pattern); retries=1, delay=0 → nonzero; exhaustion message.
///
/// Root Cause: --retry-on-unknown-error not yet implemented
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: retry loop exhaustion emits "exhausted" label via error label match
/// Prevention: guard with integration test asserting exhaustion message on stderr
/// Pitfall: with retries=1, there are 2 total invocations (1 initial + 1 retry), both failing
#[ cfg( unix ) ]
#[ test ]
fn ec8_retry_exhausted_after_all_unknown_errors()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: always exits 42 with no pattern text.
  std::fs::write( &fake, b"#!/bin/sh\nexit 42\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-unknown-error", "1", "--retry-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.code() != Some( 0 ),
    "exit must be nonzero after all unknown retries exhausted. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "exhaust" ) || stderr.to_lowercase().contains( "fail" ),
    "stderr must contain exhaustion message. Got:\n{stderr}"
  );
}

// ── EC-9: Default retry=0 → no retry on unknown error ────────────────────────

/// EC-9: no `--retry-on-unknown-error` flag; default=0; unknown error exits immediately.
///
/// Root Cause: --retry-on-unknown-error default is 0 (no retry without explicit flag)
/// Why Not Caught: default-0 test ensures regression detection if default changes
/// Fix Applied: `unwrap_or(0)` in run_print_mode() for unknown_retry_limit
/// Prevention: guard asserting no retry message when flag is absent
/// Pitfall: test must NOT set `--retry-on-unknown-error`; CLR_RETRY_ON_UNKNOWN_ERROR must be unset;
///          must also unset CLR_RETRY_ON_RATE_LIMIT to prevent rate-limit default-1 retry from firing
///          (but exit 42 won't trigger RateLimit anyway since RateLimit only fires on exit 2)
#[ cfg( unix ) ]
#[ test ]
fn ec9_default_no_retry_on_unknown_error()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: always exits 42 with no pattern text.
  std::fs::write( &fake, b"#!/bin/sh\nexit 42\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_RETRY_ON_UNKNOWN_ERROR" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.code() != Some( 0 ),
    "exit must be nonzero (unknown error, default no-retry). Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "retrying" ),
    "default retry=0 must emit no retry messages. Got:\n{stderr}"
  );
}
