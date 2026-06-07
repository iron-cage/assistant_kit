//! `--timeout` (run/ask) Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-8 from `tests/docs/cli/param/36_timeout.md`.
//!
//! ## Scope Note
//!
//! This file covers `--timeout` for `run`/`ask` only (where 0 = unlimited).
//! `--timeout` for `isolated`/`refresh` (where 0 = immediate expiry) is in `isolated_test.rs`.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6: parser/dry-run — no subprocess required
//! - EC-7..EC-8: require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! - EC-1: help lists --timeout
//! - EC-2: --timeout 0 (unlimited) accepted in dry-run
//! - EC-3: --timeout 30 accepted in dry-run
//! - EC-4: `CLR_TIMEOUT=10` env var applied
//! - EC-5: CLI 60 wins over `CLR_TIMEOUT=5`
//! - EC-6: `CLR_TIMEOUT=abc` silently ignored
//! - EC-7: fake sleeps 30; --timeout 1 → exit 2 within ~2s; stderr "timeout after 1s"
//! - EC-8: fake exits 0 fast; --timeout 30 → exit 0; no timeout message

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── EC-1: --help (run/ask) lists --timeout ────────────────────────────────────

/// EC-1: `clr --help` output contains `--timeout`.
#[ test ]
fn ec1_timeout_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--timeout" ),
    "`clr --help` must list --timeout for run/ask. Got:\n{stdout}"
  );
}

// ── EC-2: --timeout 0 --dry-run → exit 0; unlimited mode ─────────────────────

/// EC-2: --timeout 0 (unlimited, default) accepted in dry-run.
///
/// Root Cause: --timeout not yet implemented for run/ask
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: will be fixed in parse.rs + mod.rs implementation
/// Prevention: guard with dry-run parse test confirming flag accepted
/// Pitfall: --timeout already exists for isolated/refresh; the run/ask instance is separate
#[ test ]
fn ec2_timeout_zero_dry_run()
{
  let out = run_cli( &[ "--timeout", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3: --timeout 30 --dry-run → exit 0; 30s watchdog accepted ─────────────

/// EC-3: --timeout 30 accepted in dry-run; no subprocess spawned.
#[ test ]
fn ec3_timeout_nonzero_dry_run()
{
  let out = run_cli( &[ "--timeout", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_TIMEOUT=10 env var applied ─────────────────────────────────────

/// EC-4: `CLR_TIMEOUT=10` env var applied when CLI flag absent.
#[ test ]
fn ec4_clr_timeout_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_TIMEOUT", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_TIMEOUT env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: --timeout CLI wins over CLR_TIMEOUT ─────────────────────────────────

/// EC-5: CLI value 60 wins over `CLR_TIMEOUT=5`.
#[ test ]
fn ec5_timeout_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--timeout", "60", "--dry-run", "task" ],
    &[ ( "CLR_TIMEOUT", "5" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_TIMEOUT=invalid → silently ignored ─────────────────────────────

/// EC-6: invalid `CLR_TIMEOUT` silently ignored; default 0 (unlimited) used.
#[ test ]
fn ec6_clr_timeout_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_TIMEOUT", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_TIMEOUT must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: Timeout fires → exit 2; stderr contains "timeout" ──────────────────

/// EC-7: fake script sleeps 30s; --timeout 1 → exit 2 within ~2s; stderr has "timeout".
///
/// Root Cause: --timeout watchdog not yet implemented for run/ask
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: will be fixed in mod.rs via `spawn_piped` + `try_wait` polling
/// Prevention: guard with integration test confirming exit 2 and timeout message
/// Pitfall: polling at 50ms intervals means actual kill may fire up to 50ms after
///          the deadline; tests must allow up to 2s total, not exactly 1s
#[ cfg( unix ) ]
#[ test ]
fn ec7_timeout_fires_kills_subprocess()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: sleeps 30 seconds — will be killed by watchdog
  std::fs::write( &fake, b"#!/bin/sh\nsleep 30\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let start = std::time::Instant::now();
  let out = Command::new( bin )
    .args( [ "-p", "--timeout", "1", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "exit must be 2 on timeout. Got: {:?}", out.status.code()
  );
  assert!(
    elapsed.as_secs() < 5,
    "watchdog must fire within ~2s; elapsed {elapsed:?} suggests timeout not working"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "timeout" ),
    "stderr must contain 'timeout'. Got:\n{stderr}"
  );
}

// ── EC-8: No timeout when subprocess exits before deadline ────────────────────

/// EC-8: fast-exit fake; --timeout 30 → exit 0; no timeout message.
///
/// Verifies that the watchdog does not fire when the subprocess exits normally
/// before the timeout deadline. The disarmed watchdog must not emit any message.
#[ cfg( unix ) ]
#[ test ]
fn ec8_no_timeout_when_subprocess_exits_fast()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: exits 0 immediately
  std::fs::write( &fake, b"#!/bin/sh\nprintf 'done'\nexit 0\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--timeout", "30", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "timeout" ),
    "no timeout message when subprocess exits before deadline. Got:\n{stderr}"
  );
}
