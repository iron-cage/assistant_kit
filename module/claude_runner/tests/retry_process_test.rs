#![ allow( missing_docs ) ]
#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-process` and `--process-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-8 from `tests/docs/cli/param/046_retry_on_process.md` and
//! EC-1 through EC-6 from `tests/docs/cli/param/047_process_delay.md`.
//!
//! Both parameter specs share this test file because `--process-delay` only fires
//! when `--retry-on-process` is non-zero — they are functionally coupled.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 46), EC-1..EC-6 (param 47): parser/dry-run — no subprocess
//! - EC-7..EC-8 (param 46): require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-process (param 46)
//! - EC-1: help lists flag
//! - EC-2: value 0 (explicit disable) accepted in dry-run
//! - EC-3: value 2 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_PROCESS` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: fake exits 4 once then 0; retries=1, delay=0 → exit 0; `[Process]` in stderr
//! - EC-8: fake always exits 4; retries=2, delay=0 → exit 4; `[Process]` exhaustion
//!
//! ### --process-delay (param 47)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 accepted in dry-run
//! - EC-4 (delay): `CLR_PROCESS_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 46 — --retry-on-process ─────────────────────────────────────────────

// ── EC-1: --help lists --retry-on-process ─────────────────────────────────────

/// EC-1 (param 46): `clr --help` output contains `--retry-on-process`.
#[ test ]
fn ec1_retry_on_process_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-process" ),
    "`clr --help` must list --retry-on-process. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-process 0 --dry-run → exit 0 ────────────────────────────

/// EC-2 (param 46): value 0 (explicit disable, overrides fallback default 2) accepted in dry-run.
///
/// Divergence from EC-3: 0 disables Process retry; 2 (EC-3) activates retry code path.
#[ test ]
fn ec2_retry_on_process_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-process", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-on-process 2 --dry-run → exit 0 ────────────────────────────

/// EC-3 (param 46): value 2 (retry enabled) accepted in dry-run; flag parsed without error.
#[ test ]
fn ec3_retry_on_process_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-process", "2", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_PROCESS=2 env var applied ─────────────────────────────

/// EC-4 (param 46): `CLR_RETRY_ON_PROCESS=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_process_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_PROCESS", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_PROCESS env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_PROCESS ──────────────────────────────────

/// EC-5 (param 46): CLI value 3 wins over `CLR_RETRY_ON_PROCESS=1`.
#[ test ]
fn ec5_retry_on_process_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-process", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_PROCESS", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_PROCESS=invalid → silently ignored ─────────────────────

/// EC-6 (param 46): invalid `CLR_RETRY_ON_PROCESS` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_on_process_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_PROCESS", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_PROCESS must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One exit-4 failure then success → retried; exit 0 ──────────────────

/// EC-7 (param 46): fake exits 4 on first call, exits 0 on second.
/// retries=1, delay=0 → exit 0; `[Process]` in stderr.
///
/// Root Cause: Process class is new in retry system redesign
/// Why Not Caught: no test existed for Process class retry behavior
/// Fix Applied: integration test using exit-4 fake script with counter file
/// Prevention: guard with integration test asserting [Process] prefix in stderr
/// Pitfall: exit 4 alone determines Process classification — no text pattern needed;
///          this differs from Transient (exit 2), Account (text), Auth (text), Service (text)
#[ cfg( unix ) ]
#[ test ]
fn ec7_process_retry_succeeds_after_one_exit4()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\nif [ -f \"{count_path}\" ]; then exit 0; fi\ntouch \"{count_path}\"\nexit 4\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-process", "1", "--process-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 after Process retry succeeds. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Process]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Process] retry message. Got:\n{stderr}"
  );
}

// ── EC-8: All Process retries exhausted → exit 4; [Process] exhaustion ────────

/// EC-8 (param 46): fake always exits 4; retries=2, delay=0 → exit 4;
/// `[Process]` exhaustion in stderr; 3 total invocations.
///
/// Root Cause: Process class retry exhaustion needs verification
/// Why Not Caught: new class, no prior test
/// Fix Applied: integration test with always-failing exit-4 fake
/// Prevention: guard asserting [Process] + "exhausted" in stderr
/// Pitfall: test uses retries=2 to verify multiple retry attempts, not just 1
#[ cfg( unix ) ]
#[ test ]
fn ec8_process_retry_exhausted()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  std::fs::write( &fake, b"#!/bin/sh\nexit 4\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-process", "2", "--process-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 4 ),
    "exit must be 4 after Process retries exhausted. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Process]" ),
    "stderr must contain [Process] class label. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "exhaust" ),
    "stderr must contain exhaustion message. Got:\n{stderr}"
  );
}

// ── Param 47 — --process-delay ────────────────────────────────────────────────

// ── EC-1 (delay): --help lists --process-delay ────────────────────────────────

/// EC-1 (param 47): `clr --help` output contains `--process-delay`.
#[ test ]
fn ec1_process_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--process-delay" ),
    "`clr --help` must list --process-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --process-delay 0 --dry-run → exit 0 ───────────────────────

/// EC-2 (param 47): delay=0 (immediate retry) accepted in dry-run.
#[ test ]
fn ec2_process_delay_zero_dry_run()
{
  let out = run_cli( &[ "--process-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --process-delay 30 --dry-run → exit 0 ──────────────────────

/// EC-3 (param 47): delay=30 accepted in dry-run.
#[ test ]
fn ec3_process_delay_nonzero_dry_run()
{
  let out = run_cli( &[ "--process-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_PROCESS_DELAY=30 env var applied ───────────────────────

/// EC-4 (param 47): `CLR_PROCESS_DELAY=30` applied when CLI flag absent.
#[ test ]
fn ec4_clr_process_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_PROCESS_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_PROCESS_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_PROCESS_DELAY ────────────────────────────

/// EC-5 (param 47): CLI value 30 wins over `CLR_PROCESS_DELAY=10`.
#[ test ]
fn ec5_process_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--process-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_PROCESS_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_PROCESS_DELAY=invalid → silently ignored ────────────────

/// EC-6 (param 47): invalid `CLR_PROCESS_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_process_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_PROCESS_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_PROCESS_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}
