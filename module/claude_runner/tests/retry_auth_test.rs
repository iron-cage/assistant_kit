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
//! - EC-7: fake emits auth pattern (exits 1 on first call, exits 0 on second); retries=1, delay=0 → retry fires; invocation count=2; exit 0; `[Auth]` in stderr
//! - EC-8: fake always emits auth pattern; retries=2, delay=0 → 2 retries fire; invocation count=3; "retries exhausted" in stderr
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

// ── EC-7: One auth error then success → retry fires; exit 0 ──────────────────

/// EC-7 (param 42): auth error retries when `--retry-on-auth 1`; recovery succeeds on 2nd call.
///
/// Fix(BUG-325): removed `!is_auth_error` guard — Auth uses same 3-tier retry resolution
/// as all other error classes. `--retry-on-auth 1` allows one retry; the fake exits 0 on
/// the 2nd call, so clr recovers and exits 0.
///
/// Fake emits auth pattern + exits 1 on 1st call; exits 0 on 2nd call.
/// retries=1, delay=0 → retry fires; invocation count=2; `[Auth]…retrying` in stderr; exit 0.
///
/// Root Cause: `!is_auth_error` guard (BUG-315) blocked retry-block entry unconditionally,
///   making `--retry-on-auth` a dead parameter regardless of configured budget.
/// Why Not Caught: EC-7/EC-8/mre_bug315 all asserted fail-fast as correct; no test verified
///   the retry-fires path for Auth class.
/// Fix Applied: removed `!is_auth_error` guard and BUG-315 comment block (execution.rs:670–677).
/// Prevention: assert invocation count=2 (retry fired) and exit 0 (recovery succeeded).
/// Pitfall: guard tested class identity, not retry budget — class-identity gates bypass the
///   limit check entirely, making per-class retry params unconditionally dead.
#[ cfg( unix ) ]
#[ test ]
fn ec7_auth_error_retries_on_explicit_budget()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  // Fake: exits 1 with auth pattern on 1st call; exits 0 on 2nd call (recovery).
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

  // Retry fires and 2nd call succeeds — clr exits 0.
  assert!(
    out.status.success(),
    "EC-7: retry must fire and recovery must succeed (exit 0). exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  // Exactly 2 invocations — initial failure + 1 retry.
  let invocation_count = std::fs::read_to_string( &count ).unwrap_or_default().len();
  assert_eq!(
    invocation_count, 2,
    "EC-7: fake must be invoked exactly twice (1 retry fired). Got: {invocation_count}"
  );
  // [Auth] retry line appears in stderr.
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Auth]" ),
    "EC-7: stderr must contain [Auth] class label. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "retrying" ),
    "EC-7: stderr must contain retry progress line. Got:\n{stderr}"
  );
}

// ── EC-8: All Auth retries exhausted → exit 1; exhaustion message ─────────────

/// EC-8 (param 42): auth error exhausts full budget when `--retry-on-auth 2`; 3 total invocations.
///
/// Fix(BUG-325): removed `!is_auth_error` guard — Auth uses same 3-tier retry resolution
/// as all other error classes. `--retry-on-auth 2` allows 2 retries; fake always fails;
/// budget exhausted after 3 total invocations; clr exits 1 with "retries exhausted".
///
/// Fake always emits auth pattern + exits 1.
/// retries=2, delay=0 → 2 retries fire; invocation count=3; "retries exhausted" in stderr; exit 1.
///
/// Root Cause: `!is_auth_error` guard (BUG-315) blocked retry-block entry unconditionally.
/// Why Not Caught: EC-8 previously asserted invocation_count=1 and no "exhaust" — both were
///   wrong behavior locked in as correct by tests written to validate the BUG-315 guard.
/// Fix Applied: removed `!is_auth_error` guard (execution.rs:670–677); Auth now enters
///   retry block identically to Transient, Service, Account, Process, Unknown.
/// Prevention: assert invocation count=3 (2 retries) and "retries exhausted" in stderr.
/// Pitfall: "retries exhausted" requires ≥1 retry to have fired (attempts[idx] > 0 gate
///   in execution.rs); with the old guard, no retry ever fired so the non-retry form appeared.
#[ cfg( unix ) ]
#[ test ]
fn ec8_auth_error_exhausts_retry_budget()
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

  // Budget exhausted — exits 1.
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "EC-8: all retries exhausted, must exit 1. Got: {:?}", out.status.code()
  );
  // 2 retries + 1 initial = 3 total invocations.
  let invocation_count = std::fs::read_to_string( &count ).unwrap_or_default().len();
  assert_eq!(
    invocation_count, 3,
    "EC-8: fake must be invoked 3 times (1 initial + 2 retries). Got: {invocation_count}"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Auth]" ),
    "EC-8: stderr must contain [Auth] class label. Got:\n{stderr}"
  );
  // Budget fully consumed — exhaustion message must appear.
  assert!(
    stderr.to_lowercase().contains( "exhausted" ),
    "EC-8: 'retries exhausted' must appear after budget consumed. Got:\n{stderr}"
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

// ── MRE BUG-325: auth retry fires on configured budget ────────────────────────

/// MRE BUG-325: `--retry-on-auth` fires configured retry count for persistent auth failures.
///
/// # Root Cause
///
/// BUG-315 introduced `!is_auth_error` guard at retry-block entry in `execution.rs`,
/// unconditionally blocking the retry block for ALL Auth-class errors. `--retry-on-auth`
/// was parsed and stored in `CliArgs`, passed through `resolve_count()` into `limit`,
/// but `limit` was never consulted for Auth class — the guard fired first.
///
/// # Why Not Caught
///
/// mre_bug315 asserted invocation_count=1 and elapsed<10s, cementing the broken guard
/// as the correct invariant. No test existed asserting the retry-fires path for Auth.
///
/// # Fix Applied
///
/// Removed `!is_auth_error` guard and BUG-315 comment block (execution.rs lines 670–677).
/// Auth now enters retry block on the same `attempts[class_idx] < limit` gate as all
/// other error classes.
///
/// # Prevention
///
/// This test fails if the `!is_auth_error` guard is re-introduced: invocation_count
/// would be 1 instead of 4, and "retries exhausted" would not appear in stderr.
///
/// # Pitfall
///
/// Never gate the retry block on class identity — class-identity tests bypass the
/// limit check entirely, making per-class retry parameters unconditionally dead.
#[ cfg( unix ) ]
#[ test ]
fn mre_bug325_auth_retry_fires_on_configured_budget()
{
  // test_kind: bug_reproducer(BUG-325)
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  // Fake always emits an auth-class pattern and exits 1.
  // With the fix: 3 retries fire → 4 total invocations, "retries exhausted" in stderr.
  // Without the fix (guard present): invocation_count = 1, no "retries exhausted".
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
      "-p", "--retry-on-auth", "3", "--auth-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "MRE BUG-325: budget exhausted, must exit 1. Got: {:?}  stderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  // 3 retries + 1 initial = 4 total invocations.
  let invocation_count = std::fs::read_to_string( &count ).unwrap_or_default().len();
  assert_eq!(
    invocation_count, 4,
    "MRE BUG-325: fake must be invoked 4 times (1 initial + 3 retries). Got: {invocation_count} \
     — if 1: !is_auth_error guard is present (BUG-325 regression)"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "exhausted" ),
    "MRE BUG-325: 'retries exhausted' must appear after budget consumed. Got:\n{stderr}"
  );
}

// ── B4: authentication_error 401 format retries (BUG-314 + BUG-325 combined) ──

/// B4: Full Claude CLI 401 `authentication_error` format retries and exhausts budget.
///
/// Integration test verifying BUG-314 and BUG-325 fixes work together end-to-end.
///
/// BUG-314 pre-fix: `"authentication_error"` 401 string contains `"API Error: "` as a
/// substring — the `ERROR_PATTERNS` catch-all fired first → `ApiError` → `ErrorClass::Service`
/// → retry loop consumed the full budget sleeping between guaranteed re-failures.
///
/// BUG-325 pre-fix: even after BUG-314 correctly classified as `AuthError`, the
/// `!is_auth_error` guard added by BUG-315 blocked retry-block entry unconditionally —
/// `--retry-on-auth` was a dead parameter, no retries ever fired.
///
/// Post-fix (BUG-314 + BUG-325): `"authentication_error"` matches BEFORE `"API Error: "` in
/// `ERROR_PATTERNS` → `AuthError` → `ErrorClass::Auth` → same 3-tier retry resolution as all
/// other classes → `--retry-on-auth 3` fires 3 retries → 4 invocations total → "retries
/// exhausted" emitted.
///
/// # Root Cause
///
/// BUG-314: `ERROR_PATTERNS` priority let `"API Error: "` catch-all fire before `"authentication_error"`.
/// BUG-325: `!is_auth_error` guard (BUG-315 regression) blocked retry-block entry for all Auth errors.
///
/// # Why Not Caught
///
/// EC-7/EC-8/MRE-315 all use `"Your organization does not have access to Claude"` — a simple auth
/// string without `"API Error: "` conflict. No prior test used the actual 401 format string in a
/// live subprocess to verify end-to-end retry behavior for `authentication_error` responses.
///
/// # Fix Applied
///
/// BUG-314 (see FT-19 in `classify_error_test.rs`) + BUG-325 (`!is_auth_error` guard removed
/// from `run_print_mode` in `execution.rs`).
///
/// # Prevention
///
/// If either fix regresses: `[Auth]` check fails (BUG-314 regression → `[Service]`) OR
/// invocation_count != 4 (BUG-325 regression → 1 invocation, no retries fired).
///
/// # Pitfall
///
/// Neither fix alone is sufficient. BUG-314 ensures correct classification; BUG-325 ensures
/// retry fires for Auth class. Regression in either yields wrong invocation count or wrong class.
#[ cfg( unix ) ]
#[ test ]
fn b4_authentication_error_401_format_retries_and_exhausts()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  // Emits the exact Claude CLI 401 format: contains BOTH "authentication_error" AND "API Error: ".
  // Before BUG-314 fix: "API Error: " catch-all matched first → ApiError → Service → wrong class.
  // After BUG-314 fix: "authentication_error" matches first → AuthError → Auth → retries fire (BUG-325).
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

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-auth", "3", "--auth-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  // BUG-314 + BUG-325 combined: authentication_error 401 must retry and exhaust.
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "B4: authentication_error 401 must exit 1 after exhausting retries. Got: {:?}  stderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  // 4 invocations: 1 initial + 3 retries (--retry-on-auth 3).
  let invocation_count = std::fs::read_to_string( &count ).unwrap_or_default().len();
  assert_eq!(
    invocation_count, 4,
    "B4: fake must be invoked 4 times (1 + 3 retries). Got: {invocation_count} — \
     if 1: BUG-325 guard still present or BUG-314 misclassified as Service"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  // "retries exhausted" confirms retry budget was consumed (not fail-fast, not pass-through).
  assert!(
    stderr.contains( "exhausted" ),
    "B4: stderr must contain 'exhausted' (retries consumed). Got:\n{stderr}"
  );
  // [Auth] class label confirms BUG-314 classification is correct ([Service] = regression).
  assert!(
    stderr.contains( "[Auth]" ),
    "B4: stderr must contain [Auth] (not [Service]); [Service] means BUG-314 regression — \
     'authentication_error' misclassified by 'API Error: ' catch-all. Got:\n{stderr}"
  );
}
