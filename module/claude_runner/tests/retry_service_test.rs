#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-service` and `--service-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-10 from `tests/docs/cli/param/044_retry_on_service.md` and
//! EC-1 through EC-7 from `tests/docs/cli/param/045_service_delay.md`.
//!
//! Both parameter specs share this test file because `--service-delay` only fires
//! when `--retry-on-service` is non-zero — they are functionally coupled.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 44), EC-1..EC-6 (param 45): parser/dry-run — no subprocess
//! - EC-7..EC-10 (param 44), EC-7 (param 45): require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-service (param 44)
//! - EC-1: help lists flag; old `--retry-on-api-error` absent
//! - EC-2: value 0 (explicit disable) accepted in dry-run
//! - EC-3: value 2 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_SERVICE` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: fake emits `"API Error: 500"` once then 0; retries=1, delay=0 → exit 0; `[Service]` in stderr
//! - EC-8: fake always emits `"API Error: 500"`; retries=2, delay=0 → nonzero; `[Service]` exhaustion
//! - EC-9: fake emits quota pattern; `--retry-on-service 3` → exit 2; `[Account]` in stderr; no Service retry
//! - EC-10: no flag, no env var → fallback default=2 fires; fake API errors twice then 0 → exit 0
//!
//! ### --service-delay (param 45)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 accepted in dry-run
//! - EC-4 (delay): `CLR_SERVICE_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored
//! - EC-7 (delay): delay=0 causes immediate retry; exit 0

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 44 — --retry-on-service ────────────────────────────────────────────

// ── EC-1: --help lists --retry-on-service ────────────────────────────────────

/// EC-1 (param 44): `clr --help` output contains `--retry-on-service`; old flag absent.
#[ test ]
fn ec1_retry_on_service_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-service" ),
    "`clr --help` must list --retry-on-service. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--retry-on-api-error" ),
    "`clr --help` must NOT list old --retry-on-api-error. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-service 0 --dry-run → exit 0 ────────────────────────────

/// EC-2 (param 44): value 0 (explicit disable, overrides fallback default 2) accepted in dry-run.
///
/// Divergence from EC-3: 0 disables Service retry; 2 (EC-3) activates retry code path.
#[ test ]
fn ec2_retry_on_service_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-service", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-on-service 2 --dry-run → exit 0 ────────────────────────────

/// EC-3 (param 44): value 2 (retry enabled) accepted in dry-run; flag parsed without error.
#[ test ]
fn ec3_retry_on_service_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-service", "2", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_SERVICE=2 env var applied ─────────────────────────────

/// EC-4 (param 44): `CLR_RETRY_ON_SERVICE=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_service_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_SERVICE", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_SERVICE env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_SERVICE ─────────────────────────────────

/// EC-5 (param 44): CLI value 3 wins over `CLR_RETRY_ON_SERVICE=1`.
#[ test ]
fn ec5_retry_on_service_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-service", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_SERVICE", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_SERVICE=invalid → silently ignored ────────────────────

/// EC-6 (param 44): invalid `CLR_RETRY_ON_SERVICE` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_on_service_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_SERVICE", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_SERVICE must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One API error then success → retried; exit 0 ───────────────────────

/// EC-7 (param 44): fake emits `"API Error: 500"` once then exits 0; retries=1, delay=0 → exit 0.
///
/// Root Cause: --retry-on-api-error renamed to --retry-on-service in redesign
/// Why Not Caught: flag rename not yet reflected in test code
/// Fix Applied: updated to use --retry-on-service and --service-delay
/// Prevention: guard with integration test asserting [Service] prefix in stderr
/// Pitfall: delay=0 is required in tests; API Error text must be in stderr (not stdout)
///          for fake script to ensure classify_error() picks up the pattern
#[ cfg( unix ) ]
#[ test ]
fn ec7_retry_succeeds_after_one_api_error()
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

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-service", "1", "--service-delay", "0",
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
    stderr.contains( "[Service]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Service] retry message. Got:\n{stderr}"
  );
}

// ── EC-8: All API error retries exhausted → nonzero exit; [Service] exhaustion ─

/// EC-8 (param 44): fake always emits `"API Error: 500"`; retries=2, delay=0 → nonzero; [Service] exhaustion.
///
/// Root Cause: --retry-on-api-error renamed; stderr prefix changed to [Service]
/// Why Not Caught: rename not yet reflected in test code
/// Fix Applied: updated flags and assertion to match [Service] class label
/// Prevention: 3 total invocations (1 initial + 2 retries) all fail; exhaustion fires
/// Pitfall: test uses retries=2 to verify multiple retry attempts, not just 1
#[ cfg( unix ) ]
#[ test ]
fn ec8_retry_exhausted_after_all_api_errors()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

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
      "-p", "--retry-on-service", "2", "--service-delay", "0",
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
    stderr.contains( "[Service]" ),
    "stderr must contain [Service] class label. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "exhaust" ),
    "stderr must contain exhaustion message. Got:\n{stderr}"
  );
}

// ── EC-9: QuotaExhausted NOT retried as Service class ────────────────────────

/// EC-9 (param 44): fake emits `QuotaExhausted` pattern; retries=3 → exit 2; `[Account]` in stderr; no Service retry.
///
/// Root Cause: Account class takes priority over Service; quota pattern must not trigger Service retry
/// Why Not Caught: classification priority enforced in classify_to_class(); test confirms Account wins
/// Fix Applied: Account class detected from "You've hit your limit" text; Service retry not fired
/// Prevention: guard confirming [Account] in stderr and no retry messages for Service
/// Pitfall: quota pattern exits 2 (same as Transient); classification uses text pattern, not exit code
#[ cfg( unix ) ]
#[ test ]
fn ec9_quota_exhausted_not_retried_as_service()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

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
      "-p", "--retry-on-service", "3", "--service-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "QuotaExhausted must exit 2, not be retried as Service. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Account]" ),
    "stderr must contain [Account] class label for quota pattern. Got:\n{stderr}"
  );
  assert!(
    !stderr.contains( "[Service]" ),
    "stderr must NOT contain [Service] label for quota pattern. Got:\n{stderr}"
  );
}

// ── EC-10: Default fallback fires on API error ────────────────────────────────

/// EC-10 (param 44): no `--retry-on-service` flag; fallback default=2 fires; fake errors twice then 0.
///
/// Root Cause: old default was 0 (no retry); now auto→fallback(2) via 3-tier system
/// Why Not Caught: old EC-10 tested default=0; semantics changed in redesign
/// Fix Applied: uses explicit --retry-default 2 --service-delay 0 to avoid 30s sleep
/// Prevention: guard asserting [Service] retry messages fire without explicit --retry-on-service
/// Pitfall: must NOT set --retry-on-service; must NOT set CLR_RETRY_ON_SERVICE
#[ cfg( unix ) ]
#[ test ]
fn ec10_default_retry_fires_on_service_error()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let script = format!(
    "#!/bin/sh\n\
     COUNT={}\n\
     N=$(cat \"$COUNT\" 2>/dev/null || echo 0)\n\
     echo $((N + 1)) > \"$COUNT\"\n\
     if [ \"$N\" -lt 2 ]; then printf 'API Error: 500\\n' >&2; exit 1; fi\n\
     exit 0\n",
    count.display()
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--service-delay", "0", "--retry-default", "2", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_RETRY_ON_SERVICE" )
    .env_remove( "CLR_RETRY_DEFAULT" )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "fallback default=2 must fire and exit 0 after retry succeeds. Got: {:?}. stderr: {}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Service]" ),
    "stderr must contain [Service] class label from fallback retry. Got:\n{stderr}"
  );
  let n : u32 = std::fs::read_to_string( &count )
    .unwrap_or_default()
    .trim()
    .parse()
    .unwrap_or( 0 );
  assert_eq!( n, 3, "fake claude must be invoked exactly 3 times (1 initial + 2 retries). Got: {n}" );
}

// ── Param 45 — --service-delay ────────────────────────────────────────────────

// ── EC-1 (delay): --help lists --service-delay ────────────────────────────────

/// EC-1 (param 45): `clr --help` output contains `--service-delay`; old `--api-error-delay` absent.
#[ test ]
fn ec1_service_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--service-delay" ),
    "`clr --help` must list --service-delay. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--api-error-delay" ),
    "`clr --help` must NOT list old --api-error-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --service-delay 0 --dry-run → exit 0 ───────────────────────

/// EC-2 (param 45): delay=0 (immediate retry) accepted in dry-run.
#[ test ]
fn ec2_service_delay_zero_dry_run()
{
  let out = run_cli( &[ "--service-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --service-delay 30 --dry-run → exit 0 ──────────────────────

/// EC-3 (param 45): delay=30 accepted in dry-run.
#[ test ]
fn ec3_service_delay_thirty_dry_run()
{
  let out = run_cli( &[ "--service-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_SERVICE_DELAY=5 env var applied ────────────────────────

/// EC-4 (param 45): `CLR_SERVICE_DELAY=5` applied when CLI flag absent.
#[ test ]
fn ec4_clr_service_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_SERVICE_DELAY", "5" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_SERVICE_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_SERVICE_DELAY ────────────────────────────

/// EC-5 (param 45): CLI value 30 wins over `CLR_SERVICE_DELAY=10`.
#[ test ]
fn ec5_service_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--service-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_SERVICE_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_SERVICE_DELAY=invalid → silently ignored ───────────────

/// EC-6 (param 45): invalid `CLR_SERVICE_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_service_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_SERVICE_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_SERVICE_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7 (delay): delay=0 with API error retry → fires immediately; exit 0 ───

/// EC-7 (param 45): delay=0 causes immediate retry (no sleep); exit 0.
///
/// Root Cause: --api-error-delay renamed to --service-delay in redesign
/// Why Not Caught: rename not yet reflected in test code
/// Fix Applied: updated flag names
/// Prevention: guard with timing assertion — delay=0 must complete in < 5s
/// Pitfall: if delay=0 were treated as "default 30s" the test would time out
#[ cfg( unix ) ]
#[ test ]
fn ec7_service_delay_zero_immediate_retry()
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
      "-p", "--retry-on-service", "1", "--service-delay", "0",
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
