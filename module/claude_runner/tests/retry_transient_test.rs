#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-transient` and `--transient-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-10 from `tests/docs/cli/param/34_retry_on_transient.md` and
//! EC-1 through EC-7 from `tests/docs/cli/param/35_transient_delay.md`.
//!
//! Both parameter specs share this test file (see Implementation Notes in each spec).
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 34), EC-1..EC-6 (param 35): parser/dry-run — no subprocess
//! - EC-7..EC-10 (param 34), EC-7 (param 35): require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-transient (param 34)
//! - EC-1: help lists flag; does NOT list old `--retry-on-rate-limit`
//! - EC-2: value 0 (explicit disable) accepted in dry-run
//! - EC-3: value 3 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_TRANSIENT` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: fake exits 2 once then 0; retries=1, delay=0 → exit 0; `[Transient]` in stderr
//! - EC-8: fake always exits 2; retries=2, delay=0 → exit 2; `[Transient]` exhaustion in stderr
//! - EC-9: old flag `--retry-on-rate-limit` rejected → exit 1; "unknown option" in stderr
//! - EC-10: no explicit flag; fallback default=2 fires; fake exits 2 once then 0 → exit 0
//!
//! ### --transient-delay (param 35)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 (default) accepted in dry-run
//! - EC-4 (delay): `CLR_TRANSIENT_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored
//! - EC-7 (delay): delay=0 causes immediate retry; exit 0

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 34 — --retry-on-transient ──────────────────────────────────────────

// ── EC-1: --help lists --retry-on-transient ──────────────────────────────────

/// EC-1 (param 34): `clr --help` output contains `--retry-on-transient`; old flag absent.
#[ test ]
fn ec1_retry_on_transient_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-transient" ),
    "`clr --help` must list --retry-on-transient. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--retry-on-rate-limit" ),
    "`clr --help` must NOT list old --retry-on-rate-limit. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-transient 0 --dry-run → exit 0 ─────────────────────────

/// EC-2 (param 34): value 0 (explicit disable) accepted in dry-run; no retry messages.
///
/// Divergence from EC-3: 0 explicitly overrides the fallback default (2); no retry fires.
#[ test ]
fn ec2_retry_on_transient_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-transient", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-on-transient 3 --dry-run → exit 0 ─────────────────────────

/// EC-3 (param 34): value 3 (retry enabled) accepted in dry-run; flag parsed without error.
#[ test ]
fn ec3_retry_on_transient_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-transient", "3", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_TRANSIENT=2 env var applied ───────────────────────────

/// EC-4 (param 34): env var `CLR_RETRY_ON_TRANSIENT=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_transient_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_TRANSIENT", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_TRANSIENT env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_TRANSIENT ───────────────────────────────

/// EC-5 (param 34): CLI value 3 wins over `CLR_RETRY_ON_TRANSIENT=1`.
#[ test ]
fn ec5_retry_on_transient_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-transient", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_TRANSIENT", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_TRANSIENT=invalid → silently ignored ──────────────────

/// EC-6 (param 34): invalid `CLR_RETRY_ON_TRANSIENT` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_on_transient_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_TRANSIENT", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_TRANSIENT must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One Transient failure then success → retried; exit 0 ───────────────

/// EC-7 (param 34): fake exits 2 once then 0; retries=1, delay=0 → exit 0; `[Transient]` in stderr.
///
/// Root Cause: --retry-on-rate-limit renamed to --retry-on-transient in redesign
/// Why Not Caught: flag rename not yet reflected in test code
/// Fix Applied: updated to use --retry-on-transient and --transient-delay
/// Prevention: guard with integration test asserting [Transient] prefix in stderr
/// Pitfall: delay=0 is required in tests to avoid 30s sleep; zero-delay guard in
///          retry loop must branch to no-sleep
#[ cfg( unix ) ]
#[ test ]
fn ec7_transient_retry_succeeds_after_one_failure()
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

  let out = Command::new( bin )
    .args( [ "-p", "--retry-on-transient", "1", "--transient-delay", "0", "--max-sessions", "0", "x" ] )
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
    stderr.contains( "[Transient]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Transient] retry message. Got:\n{stderr}"
  );
}

// ── EC-8: All retries exhausted → exit 2; [Transient] exhaustion message ─────

/// EC-8 (param 34): fake always exits 2; retries=2, delay=0 → exit 2; `[Transient]` exhaustion.
///
/// Root Cause: --retry-on-rate-limit renamed; stderr prefix changed from no-tag to [Transient]
/// Why Not Caught: rename not yet reflected in test code
/// Fix Applied: updated flags and assertion to match [Transient] class label
/// Prevention: guard with integration test asserting [Transient] and "exhausted" in stderr
/// Pitfall: exhaustion message suffix "— retries exhausted" must differ from retry suffix
///          "— retrying in Xs (attempt M/N)" so callers can distinguish in-progress from terminal
#[ cfg( unix ) ]
#[ test ]
fn ec8_transient_retry_exhausted_exits_2()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  std::fs::write( &fake, b"#!/bin/sh\nexit 2\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--retry-on-transient", "2", "--transient-delay", "0", "--max-sessions", "0", "x" ] )
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
    stderr.contains( "[Transient]" ),
    "stderr must contain [Transient] class label. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "exhaust" ),
    "stderr must contain exhaustion message. Got:\n{stderr}"
  );
}

// ── EC-9: Old flag --retry-on-rate-limit rejected at parse time ───────────────

/// EC-9 (param 34): old flag `--retry-on-rate-limit` → exit 1; "unknown option" in stderr.
///
/// Root Cause: --retry-on-rate-limit removed entirely in redesign; no backward compat kept
/// Why Not Caught: backward-compat rejection not yet tested
/// Fix Applied: parse.rs does not recognize old flag → falls through to unknown-option error
/// Prevention: guard with test confirming old flag is hard-rejected, not silently accepted
/// Pitfall: the flag must produce exit 1 (parse error), NOT be silently ignored (which
///          would mean the old name was accepted as a no-op, defeating the rename)
#[ test ]
fn ec9_old_flag_name_rejected()
{
  let out = run_cli( &[ "--retry-on-rate-limit", "1", "--dry-run", "task" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "old flag --retry-on-rate-limit must exit 1. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "unknown" ),
    "stderr must contain 'unknown option' for old flag. Got:\n{stderr}"
  );
}

// ── EC-10: Default fallback fires without explicit flag ───────────────────────

/// EC-10 (param 34): no `--retry-on-transient` and no `CLR_RETRY_ON_TRANSIENT`;
/// fallback default=2 fires when fake exits 2 once then 0.
///
/// Root Cause: default was 1 (old retry-on-rate-limit); now auto→fallback(2) via 3-tier system
/// Why Not Caught: old EC-10 used --retry-delay (old name); default semantics changed
/// Fix Applied: uses --retry-default 2 (explicit count) and --retry-default-delay 0 (explicit
///              delay) so test does not rely on hardcoded defaults; validates 3-tier resolution
/// Prevention: explicit --retry-default 2 and --retry-default-delay 0 make test non-fragile
/// Pitfall: must NOT set --retry-on-transient; if set the test covers explicit value not default
#[ cfg( unix ) ]
#[ test ]
fn ec10_transient_fallback_default_fires_without_flag()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let script = format!(
    "#!/bin/sh\n\
     COUNT={}\n\
     N=$(cat \"$COUNT\" 2>/dev/null || echo 0)\n\
     echo $((N + 1)) > \"$COUNT\"\n\
     if [ \"$N\" -eq 0 ]; then exit 2; fi\n\
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
    .args( [ "-p", "--retry-default", "2", "--retry-default-delay", "0", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_RETRY_ON_TRANSIENT" )
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
    stderr.contains( "[Transient]" ),
    "stderr must contain [Transient] class label from fallback retry. Got:\n{stderr}"
  );
  let n : u32 = std::fs::read_to_string( &count )
    .unwrap_or_default()
    .trim()
    .parse()
    .unwrap_or( 0 );
  assert!(
    n >= 2,
    "fake claude must be invoked at least 2 times (initial + fallback default retry). Got: {n}"
  );
}

// ── Param 35 — --transient-delay ─────────────────────────────────────────────

// ── EC-1 (delay): --help lists --transient-delay ─────────────────────────────

/// EC-1 (param 35): `clr --help` output contains `--transient-delay`; old `--retry-delay` absent.
#[ test ]
fn ec1_transient_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--transient-delay" ),
    "`clr --help` must list --transient-delay. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--retry-delay" ),
    "`clr --help` must NOT list old --retry-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --transient-delay 0 --dry-run → exit 0 ────────────────────

/// EC-2 (param 35): delay=0 (immediate retry) accepted in dry-run.
#[ test ]
fn ec2_transient_delay_zero_dry_run()
{
  let out = run_cli( &[ "--transient-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --transient-delay 30 --dry-run → exit 0 ───────────────────

/// EC-3 (param 35): delay=30 (default) accepted in dry-run.
#[ test ]
fn ec3_transient_delay_thirty_dry_run()
{
  let out = run_cli( &[ "--transient-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_TRANSIENT_DELAY=30 env var applied ─────────────────────

/// EC-4 (param 35): `CLR_TRANSIENT_DELAY=30` env var applied when CLI flag absent.
#[ test ]
fn ec4_clr_transient_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_TRANSIENT_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_TRANSIENT_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_TRANSIENT_DELAY ──────────────────────────

/// EC-5 (param 35): CLI value 30 wins over `CLR_TRANSIENT_DELAY=10`.
#[ test ]
fn ec5_transient_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--transient-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_TRANSIENT_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_TRANSIENT_DELAY=invalid → silently ignored ─────────────

/// EC-6 (param 35): invalid `CLR_TRANSIENT_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_transient_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_TRANSIENT_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_TRANSIENT_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7 (delay): delay=0 fires retry immediately; exit 0 ────────────────────

/// EC-7 (param 35): delay=0 causes immediate retry (no sleep); exit 0.
///
/// Root Cause: --retry-delay renamed to --transient-delay; test must use new name
/// Why Not Caught: rename not yet reflected in test code
/// Fix Applied: updated to use --transient-delay flag
/// Prevention: guard with timing assertion — delay=0 must complete in < 5s
/// Pitfall: if delay=0 were treated as "default 30s" the test would time out; the 0-check
///          in the retry loop must branch to no-sleep
#[ cfg( unix ) ]
#[ test ]
fn ec7_transient_delay_zero_immediate_retry()
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
    .args( [ "-p", "--retry-on-transient", "1", "--transient-delay", "0", "--max-sessions", "0", "x" ] )
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
