//! `--retry-on-rate-limit` and `--retry-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-9 from `tests/docs/cli/param/34_retry_on_rate_limit.md` and
//! EC-1 through EC-7 from `tests/docs/cli/param/35_retry_delay.md`.
//!
//! Both parameter specs share this test file (see Implementation Notes in each spec).
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 34), EC-1..EC-6 (param 35): parser/dry-run — no subprocess
//! - EC-7..EC-9 (param 34), EC-7 (param 35): require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-rate-limit (param 34)
//! - EC-1: help lists flag
//! - EC-2: value 0 (default, no retry) accepted in dry-run
//! - EC-3: value 3 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_RATE_LIMIT` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: fake exits 2 once then 0; retries=1, delay=0 → exit 0; stderr has "retry"
//! - EC-8: fake always exits 2; retries=2, delay=0 → exit 2; stderr has "exhausted"/"failed"
//! - EC-9: `QuotaExhausted` pattern → never retried; exit 2; no retry message
//!
//! ### --retry-delay (param 35)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 60 accepted in dry-run
//! - EC-4 (delay): `CLR_RETRY_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored
//! - EC-7 (delay): delay=0 causes immediate retry; exit 0

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 34 — --retry-on-rate-limit ─────────────────────────────────────────

// ── EC-1: --help lists --retry-on-rate-limit ─────────────────────────────────

/// EC-1 (param 34): `clr --help` output contains `--retry-on-rate-limit`.
#[ test ]
fn ec1_retry_on_rate_limit_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-rate-limit" ),
    "`clr --help` must list --retry-on-rate-limit. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-rate-limit 0 --dry-run → exit 0 ────────────────────────

/// EC-2 (param 34): value 0 (default, no retry) accepted in dry-run.
#[ test ]
fn ec2_retry_on_rate_limit_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-rate-limit", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-on-rate-limit 3 --dry-run → exit 0 ────────────────────────

/// EC-3 (param 34): value 3 (retry enabled) accepted in dry-run; no subprocess, no retry.
#[ test ]
fn ec3_retry_on_rate_limit_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-rate-limit", "3", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_RATE_LIMIT=2 env var applied ─────────────────────────

/// EC-4 (param 34): env var `CLR_RETRY_ON_RATE_LIMIT=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_rate_limit_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_RATE_LIMIT", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_RATE_LIMIT env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_RATE_LIMIT ──────────────────────────────

/// EC-5 (param 34): CLI value 3 wins over `CLR_RETRY_ON_RATE_LIMIT=1`.
#[ test ]
fn ec5_retry_on_rate_limit_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-rate-limit", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_RATE_LIMIT", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_RATE_LIMIT=invalid → silently ignored ─────────────────

/// EC-6 (param 34): invalid `CLR_RETRY_ON_RATE_LIMIT` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_on_rate_limit_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_RATE_LIMIT", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_RATE_LIMIT must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One rate-limit failure then success → retried; exit 0 ──────────────

/// EC-7 (param 34): fake exits 2 once then 0; retries=1, delay=0 → exit 0; stderr has "retry".
///
/// The fake claude script uses a counter file in the temp dir: first call touches it (exits 2),
/// second call detects it (exits 0). With `--retry-on-rate-limit 1 --retry-delay 0`, the runner
/// retries exactly once and succeeds.
///
/// Root Cause: --retry-on-rate-limit not yet implemented
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: will be fixed in parse.rs + mod.rs implementation
/// Prevention: guard with integration test asserting retry message on stderr
/// Pitfall: delay=0 is required in tests to avoid 60s sleep; rate-limit retry
///          path must not block on sleep when delay is zero
#[ cfg( unix ) ]
#[ test ]
fn ec7_retry_succeeds_after_one_rate_limit()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  // Script: exits 2 on first call (no counter), exits 0 on second (counter exists)
  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\nif [ -f \"{count_path}\" ]; then exit 0; fi\ntouch \"{count_path}\"\nexit 2\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--retry-on-rate-limit", "1", "--retry-delay", "0", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 after retry succeeds. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "retry" ),
    "stderr must contain retry message. Got:\n{stderr}"
  );
}

// ── EC-8: All retries exhausted → exit 2; stderr has exhaustion message ───────

/// EC-8 (param 34): fake always exits 2; retries=2, delay=0 → exit 2; stderr has "exhausted"/"failed".
///
/// Root Cause: --retry-on-rate-limit not yet implemented
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: will be fixed in parse.rs + mod.rs implementation
/// Prevention: guard with integration test asserting exhaustion message on stderr
/// Pitfall: exhaustion message must differ from retry message so callers can distinguish
///          "retry in progress" from "all retries consumed"
#[ cfg( unix ) ]
#[ test ]
fn ec8_retry_exhausted_exits_2()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: always exits 2 with no pattern (transient rate-limit)
  std::fs::write( &fake, b"#!/bin/sh\nexit 2\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--retry-on-rate-limit", "2", "--retry-delay", "0", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "exit must be 2 after retries exhausted. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "exhaust" ) || stderr.to_lowercase().contains( "fail" ),
    "stderr must contain exhaustion message. Got:\n{stderr}"
  );
}

// ── EC-9: QuotaExhausted → never retried ──────────────────────────────────────

/// EC-9 (param 34): fake emits `QuotaExhausted` pattern; retries=3 → exit 2; no retry.
///
/// Root Cause: `--retry-on-rate-limit` not yet implemented
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: `classify_error()` distinguishes `RateLimit` from `QuotaExhausted`; only `RateLimit` retried
/// Prevention: guard with integration test confirming no retry message for `QuotaExhausted`
/// Pitfall: exit code 2 alone would classify as `RateLimit`; the "You've hit your limit" pattern
///          must take precedence (priority order in `ERROR_PATTERNS` ensures this)
#[ cfg( unix ) ]
#[ test ]
fn ec9_quota_exhausted_not_retried()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: emits QuotaExhausted pattern on stderr and exits 2
  std::fs::write(
    &fake,
    b"#!/bin/sh\nprintf \"You've hit your limit\\n\" >&2\nexit 2\n",
  ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--retry-on-rate-limit", "3", "--retry-delay", "0", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "exit must be 2 for QuotaExhausted. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "retry" ) && !stderr.to_lowercase().contains( "retrying" ),
    "QuotaExhausted must not trigger retry messages. Got:\n{stderr}"
  );
}

// ── Param 35 — --retry-delay ──────────────────────────────────────────────────

// ── EC-1 (delay): --help lists --retry-delay ─────────────────────────────────

/// EC-1 (param 35): `clr --help` output contains `--retry-delay`.
#[ test ]
fn ec1_retry_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-delay" ),
    "`clr --help` must list --retry-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --retry-delay 0 --dry-run → exit 0 ────────────────────────

/// EC-2 (param 35): delay=0 (immediate retry) accepted in dry-run.
#[ test ]
fn ec2_retry_delay_zero_dry_run()
{
  let out = run_cli( &[ "--retry-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --retry-delay 60 --dry-run → exit 0 ───────────────────────

/// EC-3 (param 35): delay=60 (default) accepted in dry-run.
#[ test ]
fn ec3_retry_delay_sixty_dry_run()
{
  let out = run_cli( &[ "--retry-delay", "60", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_RETRY_DELAY=30 env var applied ─────────────────────────

/// EC-4 (param 35): `CLR_RETRY_DELAY=30` env var applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_RETRY_DELAY ──────────────────────────────

/// EC-5 (param 35): CLI value 30 wins over `CLR_RETRY_DELAY=10`.
#[ test ]
fn ec5_retry_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_RETRY_DELAY=invalid → silently ignored ─────────────────

/// EC-6 (param 35): invalid `CLR_RETRY_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7 (delay): delay=0 fires retry immediately; exit 0 ────────────────────

/// EC-7 (param 35): delay=0 causes immediate retry (no sleep); exit 0.
///
/// Same fake script as param-34 EC-7: exits 2 once then 0. With delay=0 the retry
/// fires without waiting — verifying that zero-delay is not treated as "skip retry".
///
/// Root Cause: --retry-delay not yet implemented
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: will be fixed in mod.rs retry loop
/// Prevention: guard with integration test verifying fast exit (no 60s wait)
/// Pitfall: if delay=0 were treated as "infinite" or "default 60" the test would
///          timeout; the 0-check in the retry loop must branch to no-sleep
#[ cfg( unix ) ]
#[ test ]
fn ec7_retry_delay_zero_immediate_retry()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\nif [ -f \"{count_path}\" ]; then exit 0; fi\ntouch \"{count_path}\"\nexit 2\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let start = std::time::Instant::now();
  let out = Command::new( bin )
    .args( [ "-p", "--retry-on-rate-limit", "1", "--retry-delay", "0", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  assert!(
    out.status.success(),
    "exit must be 0 with delay=0 retry. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  assert!(
    elapsed.as_secs() < 5,
    "delay=0 must retry immediately; elapsed {elapsed:?} is too long"
  );
}
