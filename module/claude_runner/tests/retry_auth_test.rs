//! Unix-only integration tests.
#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-auth` and `--auth-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-8 from `tests/docs/cli/param/042_retry_on_auth.md` and
//! EC-1 through EC-6 from `tests/docs/cli/param/043_auth_delay.md`.
//!
//! Both parameter specs share this test file because `--auth-delay` only fires
//! when `--retry-on-auth` is non-zero — they are functionally coupled.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 42), EC-1..EC-6 (param 43): parser/dry-run — no subprocess
//! - EC-7..EC-8 (param 42): require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-auth (param 42)
//! - EC-1: help lists flag
//! - EC-2: value 0 (explicit disable) accepted in dry-run
//! - EC-3: value 2 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_AUTH` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: fake emits auth pattern; retries=1, delay=0 → exit 1 immediately (fail-fast; no retry); `[Auth]` in stderr
//! - EC-8: fake always emits auth pattern; retries=2, delay=0 → exit 1 immediately (fail-fast; no retry or exhaustion)
//!
//! ### --auth-delay (param 43)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 accepted in dry-run
//! - EC-4 (delay): `CLR_AUTH_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 42 — --retry-on-auth ────────────────────────────────────────────────

// ── EC-1: --help lists --retry-on-auth ────────────────────────────────────────

/// EC-1 (param 42): `clr --help` output contains `--retry-on-auth`.
#[ test ]
fn ec1_retry_on_auth_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-auth" ),
    "`clr --help` must list --retry-on-auth. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-auth 0 --dry-run → exit 0 ───────────────────────────────

/// EC-2 (param 42): value 0 (explicit disable, overrides fallback default 2) accepted in dry-run.
///
/// Divergence from EC-3: 0 disables Auth retry; 2 (EC-3) activates retry code path.
#[ test ]
fn ec2_retry_on_auth_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-auth", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-on-auth 2 --dry-run → exit 0 ───────────────────────────────

/// EC-3 (param 42): value 2 (retry enabled) accepted in dry-run; flag parsed without error.
#[ test ]
fn ec3_retry_on_auth_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-auth", "2", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_AUTH=2 env var applied ─────────────────────────────────

/// EC-4 (param 42): `CLR_RETRY_ON_AUTH=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_auth_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_AUTH", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_AUTH env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_AUTH ─────────────────────────────────────

/// EC-5 (param 42): CLI value 3 wins over `CLR_RETRY_ON_AUTH=1`.
#[ test ]
fn ec5_retry_on_auth_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-auth", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_AUTH", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_AUTH=invalid → silently ignored ────────────────────────

/// EC-6 (param 42): invalid `CLR_RETRY_ON_AUTH` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_on_auth_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_AUTH", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_AUTH must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One auth error then success → retried; exit 0 ──────────────────────

/// EC-7 (param 42): auth error exits immediately (fail-fast) even when `--retry-on-auth 1`.
///
/// Fix(BUG-315): auth errors never retry regardless of `--retry-on-auth` setting.
/// The retry block is guarded by `!is_auth_error` in `execution.rs`; when an Auth-class
/// error is detected, the guard prevents entry and the process exits immediately.
///
/// Fake emits auth pattern + exits 1 (would exit 0 on 2nd call, but 2nd call never fires).
/// retries=1, delay=0 → exit 1 immediately; invocation count = 1; `[Auth]` in stderr.
///
/// Root Cause: retry loop lacked is_auth_error guard (BUG-315); auth failures consumed
///   retry budget sleeping between guaranteed re-failures.
/// Why Not Caught: no test asserted invocation count or wall-clock fail-fast behavior.
/// Fix Applied: `!is_auth_error` guard at retry block entry in execution.rs.
/// Prevention: assert exit=1 and invocation count=1 (not 2).
/// Pitfall: `--retry-on-auth N > 0` does NOT enable auth retries — the fail-fast guard
///   applies unconditionally; setting retry-on-auth has no effect on auth-class errors.
#[ cfg( unix ) ]
#[ test ]
fn ec7_auth_error_exits_immediately_without_retry()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  // Fake: exits 0 on 2nd call, but 2nd call never fires after BUG-315 fix.
  let script = format!(
    "#!/bin/sh\n\
     printf '1' >> \"{count_path}\"\n\
     if [ $(wc -c < \"{count_path}\") -gt 1 ]; then exit 0; fi\n\
     printf 'Your organization does not have access to Claude\\n'\n\
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
      "-p", "--retry-on-auth", "1", "--auth-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  // Auth errors fail-fast: exits 1 immediately, even with --retry-on-auth 1.
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "EC-7: auth error must exit 1 immediately (fail-fast). exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  // Exactly 1 invocation — retry guard prevents the 2nd call.
  let invocation_count = std::fs::read_to_string( &count ).unwrap_or_default().len();
  assert_eq!(
    invocation_count, 1,
    "EC-7: fake must be invoked exactly once (no retry). Got: {invocation_count}"
  );
  // [Auth] class label still appears in the terminal error line.
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Auth]" ),
    "EC-7: stderr must contain [Auth] class label. Got:\n{stderr}"
  );
}

// ── EC-8: All Auth retries exhausted → exit 1; [Auth] exhaustion ──────────────

/// EC-8 (param 42): auth error exits immediately (fail-fast) even when `--retry-on-auth 2`.
///
/// Fix(BUG-315): auth errors never retry regardless of `--retry-on-auth` setting.
/// Fake always emits auth pattern + exits 1; retries=2, delay=0 → exit 1 immediately;
/// invocation count = 1; no "exhausted" message (exhaustion requires ≥1 retry attempt).
///
/// Root Cause: retry loop lacked is_auth_error guard (BUG-315).
/// Why Not Caught: no test verified that auth errors with retry>0 still don't retry.
/// Fix Applied: `!is_auth_error` guard at retry block entry in execution.rs.
/// Prevention: assert exit=1 and invocation count=1 and no "exhaust" in stderr.
/// Pitfall: "retries exhausted" only appears when at least one retry fires; with fail-fast
///   no retry ever fires, so the error message uses the non-retry form: `Error: [Auth] ... (exit 1)`.
#[ cfg( unix ) ]
#[ test ]
fn ec8_auth_error_exits_immediately_regardless_of_retry_budget()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\n\
     printf '1' >> \"{count_path}\"\n\
     printf 'Your organization does not have access to Claude\\n'\n\
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
      "-p", "--retry-on-auth", "2", "--auth-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  // Auth errors fail-fast: exits 1 immediately, even with --retry-on-auth 2.
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "EC-8: auth error must exit 1 immediately (fail-fast). Got: {:?}", out.status.code()
  );
  // Exactly 1 invocation — no retries fired.
  let invocation_count = std::fs::read_to_string( &count ).unwrap_or_default().len();
  assert_eq!(
    invocation_count, 1,
    "EC-8: fake must be invoked exactly once (no retry). Got: {invocation_count}"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Auth]" ),
    "EC-8: stderr must contain [Auth] class label. Got:\n{stderr}"
  );
  // No retry attempt occurred, so "exhausted" must NOT appear (non-retry error form used).
  assert!(
    !stderr.to_lowercase().contains( "exhaust" ),
    "EC-8: no retry fired, so 'exhausted' must not appear in stderr. Got:\n{stderr}"
  );
}

// ── Param 43 — --auth-delay ──────────────────────────────────────────────────

// ── EC-1 (delay): --help lists --auth-delay ──────────────────────────────────

/// EC-1 (param 43): `clr --help` output contains `--auth-delay`.
#[ test ]
fn ec1_auth_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--auth-delay" ),
    "`clr --help` must list --auth-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --auth-delay 0 --dry-run → exit 0 ─────────────────────────

/// EC-2 (param 43): delay=0 (immediate retry) accepted in dry-run.
#[ test ]
fn ec2_auth_delay_zero_dry_run()
{
  let out = run_cli( &[ "--auth-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --auth-delay 30 --dry-run → exit 0 ────────────────────────

/// EC-3 (param 43): delay=30 accepted in dry-run.
#[ test ]
fn ec3_auth_delay_nonzero_dry_run()
{
  let out = run_cli( &[ "--auth-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_AUTH_DELAY=30 env var applied ──────────────────────────

/// EC-4 (param 43): `CLR_AUTH_DELAY=30` applied when CLI flag absent.
#[ test ]
fn ec4_clr_auth_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_AUTH_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_AUTH_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_AUTH_DELAY ───────────────────────────────

/// EC-5 (param 43): CLI value 30 wins over `CLR_AUTH_DELAY=10`.
#[ test ]
fn ec5_auth_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--auth-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_AUTH_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_AUTH_DELAY=invalid → silently ignored ──────────────────

/// EC-6 (param 43): invalid `CLR_AUTH_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_auth_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_AUTH_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_AUTH_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── MRE BUG-315: auth error exits retry loop immediately ──────────────────────

/// MRE BUG-315: `authentication_error` 401 in print-mode exits immediately (fail-fast).
///
/// # Root Cause
///
/// `run_print_mode` retry loop had no `is_auth_error` guard; auth failures burned the entire
/// retry budget sleeping between guaranteed re-failures (same stale credential, same 401 on
/// every attempt). With `--retry-on-auth 3 --auth-delay 5` this wasted 3 × 5s = 15s.
///
/// # Why Not Caught
///
/// EC-7/EC-8 verify Auth-class retry counting (success after 1, exhaustion after N) but do
/// not verify fail-fast on a persistent auth error. No test asserted wall-clock exit time.
///
/// # Fix Applied
///
/// Added `is_auth_error` flag; retry block entry guarded by `!is_auth_error` in
/// `execution.rs` — auth errors fall through directly to `process::exit(exit_code)`.
///
/// # Prevention
///
/// This test fails immediately if the guard is removed: the binary would sleep 3 × 5s = 15s
/// before exiting, exceeding the 10s wall-clock assertion.
///
/// # Pitfall
///
/// Never use `break` inside the retry loop to exit early — `break` bypasses
/// `process::exit`. Guard the block ENTRY instead (`!is_auth_error && attempts < limit`).
#[ cfg( unix ) ]
#[ test ]
fn mre_bug315_auth_error_exits_retry_loop_immediately()
{
  // test_kind: bug_reproducer(BUG-315)
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  // Fake always emits an auth-class pattern and exits 1.
  // Without the fix the binary sleeps 3 × 5s before exiting.
  // With the fix it exits after exactly 1 invocation, well under 10s.
  let script = format!(
    "#!/bin/sh\n\
     printf '1' >> \"{count_path}\"\n\
     printf 'Your organization does not have access to Claude\\n'\n\
     exit 1\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let start = std::time::Instant::now();
  let out   = Command::new( bin )
    .args( [
      "-p", "--retry-on-auth", "3", "--auth-delay", "5",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "MRE BUG-315: exit must be code 1. Got: {:?}  stderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  // Exactly 1 invocation — no retries fired.
  let invocation_count = std::fs::read_to_string( &count ).unwrap_or_default().len();
  assert_eq!(
    invocation_count, 1,
    "MRE BUG-315: fake must be invoked exactly once (no retries). Got: {invocation_count}"
  );
  // No sleep delay consumed — must exit well under the first retry delay (5s × 1 attempt).
  assert!(
    elapsed.as_secs() < 10,
    "MRE BUG-315: exit must be immediate (< 10s). Elapsed: {elapsed:?} — \
     auth error retries sleeping means the fail-fast guard is missing"
  );
}

// ── B4: authentication_error 401 format fail-fast (BUG-314 + BUG-315 combined) ─

/// B4: Full Claude CLI 401 `authentication_error` format also triggers fail-fast.
///
/// Integration test verifying BUG-314 and BUG-315 fixes work together end-to-end.
///
/// BUG-314 pre-fix: `"authentication_error"` 401 string contains `"API Error: "` as a
/// substring — the `ERROR_PATTERNS` catch-all fired first → `ApiError` → `ErrorClass::Service`
/// → retry loop consumed the full budget sleeping between guaranteed re-failures.
///
/// BUG-315 pre-fix: even if correctly classified as `AuthError`, no `!is_auth_error` guard
/// existed, so auth errors still retried.
///
/// Post-fix (both): `"authentication_error"` matches BEFORE `"API Error: "` in `ERROR_PATTERNS`
/// → `AuthError` → `ErrorClass::Auth` → `!is_auth_error` guard fires → fail-fast, 1 invocation.
///
/// # Root Cause
///
/// BUG-314: `ERROR_PATTERNS` priority let `"API Error: "` catch-all fire before `"authentication_error"`.
/// BUG-315: No `is_auth_error` guard in `run_print_mode` retry block entry.
///
/// # Why Not Caught
///
/// EC-7/EC-8/MRE-315 all use `"Your organization does not have access to Claude"` — a simple auth
/// string without `"API Error: "` conflict. No prior test used the actual 401 format string in a
/// live subprocess to verify end-to-end fail-fast behavior for `authentication_error` responses.
///
/// # Fix Applied
///
/// BUG-314 + BUG-315 (see FT-19 in `classify_error_test.rs` and `mre_bug315_...` in this file).
///
/// # Prevention
///
/// If either fix regresses: the `[Auth]` check fails (BUG-314 regression → `[Service]`) OR the
/// timing/count check fails (BUG-315 regression → 3 invocations, ~15s elapsed).
///
/// # Pitfall
///
/// Neither fix alone is sufficient. BUG-314 ensures correct classification; BUG-315 ensures
/// the correct class triggers fail-fast. Regression in either produces full retry-budget waste.
#[ cfg( unix ) ]
#[ test ]
fn b4_authentication_error_401_format_exits_immediately()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  // Emits the exact Claude CLI 401 format: contains BOTH "authentication_error" AND "API Error: ".
  // Before BUG-314 fix: "API Error: " catch-all matched first → ApiError → Service → retries.
  // After BUG-314 fix: "authentication_error" matches first → AuthError → Auth → fail-fast (BUG-315).
  let script = format!(
    "#!/bin/sh\n\
     printf '1' >> \"{count_path}\"\n\
     printf 'Failed to authenticate. API Error: 401 authentication_error\\n'\n\
     exit 1\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let start = std::time::Instant::now();
  let out   = Command::new( bin )
    .args( [
      "-p", "--retry-on-auth", "3", "--auth-delay", "5",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  // BUG-314 + BUG-315 combined: authentication_error 401 must fail-fast.
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "B4: authentication_error 401 must exit 1 immediately. Got: {:?}  stderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  // Exactly 1 invocation — fail-fast prevents all 3 retries from firing.
  let invocation_count = std::fs::read_to_string( &count ).unwrap_or_default().len();
  assert_eq!(
    invocation_count, 1,
    "B4: fake must be invoked exactly once (no retries). Got: {invocation_count} — \
     if >1: BUG-315 guard is missing or BUG-314 misclassified as Service"
  );
  // No sleep consumed — must exit well under 3 × 5s = 15s budget.
  assert!(
    elapsed.as_secs() < 10,
    "B4: exit must be immediate (< 10s). Elapsed: {elapsed:?} — \
     if ≥ 10s: either BUG-314 or BUG-315 fix is missing"
  );
  // [Auth] class label confirms BUG-314 classification is correct ([Service] = regression).
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Auth]" ),
    "B4: stderr must contain [Auth] (not [Service]); [Service] means BUG-314 regression — \
     'authentication_error' misclassified by 'API Error: ' catch-all. Got:\n{stderr}"
  );
}
