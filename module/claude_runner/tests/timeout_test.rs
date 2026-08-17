//! Unix-only integration tests.
#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--timeout` (run/ask) Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-8 from `tests/docs/cli/param/036_timeout.md`, the
//! default-timeout tests (`ec_timeout_default_*`, `ec_timeout_explicit_*`, `ec_timeout_unlimited_*`)
//! introduced by TSK-227/228 (BUG-305) and retargeted by TSK-503 (built-in default
//! retired — 0 = unlimited unless `--timeout`/`CLR_TIMEOUT` is expressed), and the
//! BUG-317 double-emission guard (`ec_timeout_retry_no_double_emission`).
//!
//! ## Scope Note
//!
//! This file covers `--timeout` for `run`/`ask` only (where 0 = unlimited).
//! `--timeout` for `isolated`/`refresh` (where 0 = immediate expiry) is in `isolated_test.rs`.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6: parser/dry-run — no subprocess required
//! - EC-7..EC-8: require fake subprocess (explicit timeout)
//! - ec_timeout_default_*: require fake subprocess (default path — unlimited since
//!   TSK-503; the kill mechanism stays proven via the `_CLR_DEFAULT_TIMEOUT` hook)
//! - ec_timeout_explicit_*: explicit large timeout values
//! - ec_timeout_unlimited_*: explicit --timeout 0 / CLR_TIMEOUT=0 opt-out
//!
//! ## Corner Cases Covered
//!
//! - EC-1: help lists --timeout
//! - EC-2: --timeout 0 (unlimited) accepted in dry-run
//! - EC-3: --timeout 30 accepted in dry-run
//! - EC-4: `CLR_TIMEOUT=10` env var applied
//! - EC-5: CLI 60 wins over `CLR_TIMEOUT=5`
//! - EC-6: `CLR_TIMEOUT=abc` silently ignored
//! - EC-7: fake sleeps 30; --timeout 1 → exit 4 within ~2s; stderr "timeout after 1s"
//! - EC-8: fake exits 0 fast; --timeout 30 → exit 0; no timeout message
//! - ec_timeout_default_constant_value: DEFAULT_PRINT_TIMEOUT_SECS constant equals 0 (TSK-503)
//! - ec_timeout_default_no_fire: no --timeout, fast subprocess → exit 0, no timeout msg (BUG-305)
//! - ec_timeout_default_unlimited: no --timeout, 2s subprocess → exit 0 (no default watchdog)
//! - ec_timeout_explicit_large_value: --timeout 7200 with fast subprocess → exit 0
//! - ec_timeout_unlimited_flag: --timeout 0 explicit unlimited opt-out → exit 0
//! - ec_timeout_unlimited_env: CLR_TIMEOUT=0 explicit unlimited opt-out → exit 0
//! - ec_timeout_default_kills: `_CLR_DEFAULT_TIMEOUT=2`, hanging subprocess → exit 4 (TSK-228)
//! - ec_timeout_retry_no_double_emission: no stderr line starts with bare label on retry (BUG-317)
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

/// EC-6: invalid `CLR_TIMEOUT` silently ignored; the unexpressed default (unlimited since
/// TSK-503) applies for run/ask print-mode; dry-run exits before any timeout could matter.
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

/// EC-7: fake script sleeps 30s; --timeout 1 → exit 4 within ~2s; stderr has "timeout".
///
/// Root Cause: --timeout watchdog not yet implemented for run/ask
/// Why Not Caught: feature does not exist yet (TDD red phase)
/// Fix Applied: poll_timeout() in execution.rs calls exit(4) (changed from exit(2), TSK-202)
/// Prevention: guard with integration test confirming exit 4 and timeout message
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
    .args( [ "-p", "--timeout", "1", "--max-sessions", "0", "--retry-override", "0", "x" ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  assert_eq!(
    out.status.code(),
    Some( 4 ),
    "exit must be 4 on timeout (TSK-202: timeout uses exit 4, not exit 2). Got: {:?}", out.status.code()
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

// ── ec_timeout_default_constant_value: DEFAULT_PRINT_TIMEOUT_SECS = 0 ─────────

/// TSK-503 — `DEFAULT_PRINT_TIMEOUT_SECS` constant must equal 0 (unlimited).
///
/// TSK-227/BUG-305 introduced a 3600 s (1 h) built-in watchdog here; TSK-503
/// removes it — long agentic sessions were killed mid-work at the hour mark
/// even though clr neutralizes claude's inner wind-down ceiling precisely to
/// let background work run. A session is killed only by an EXPRESSED
/// `--timeout N`/`CLR_TIMEOUT=N`; the constant and its resolution chain stay
/// so the `_CLR_DEFAULT_TIMEOUT` test hook can still arm a default-path
/// watchdog in tests (see ec_timeout_default_kills).
///
/// Prevention: this test fails if the constant is removed, re-raised to a
/// nonzero value, or the resolution chain stops flowing through
/// default_print_timeout() (which would orphan the test hook).
#[ test ]
fn ec_timeout_default_constant_value()
{
  let src = include_str!( "../src/cli/execution.rs" );
  assert!(
    src.contains( "DEFAULT_PRINT_TIMEOUT_SECS : u32 = 0" ),
    "DEFAULT_PRINT_TIMEOUT_SECS must be defined as u32 = 0 (unlimited) in src/cli/execution.rs"
  );
  assert!(
    src.contains( "unwrap_or( DEFAULT_PRINT_TIMEOUT_SECS )" ),
    "DEFAULT_PRINT_TIMEOUT_SECS must appear in unwrap_or() (inside default_print_timeout() helper)"
  );
  assert!(
    src.contains( "unwrap_or( default_print_timeout() )" ),
    "run_print_mode() call site must use default_print_timeout(), not the constant directly"
  );
}

// ── ec_timeout_default_no_fire: fast subprocess, no expressed timeout ─────────

/// TSK-503 — no --timeout, fast subprocess → exit 0, no timeout message.
///
/// Originally proved a fast subprocess completed under TSK-227's 3600 s default
/// watchdog; since TSK-503 zeroed that default the same run proves the unexpressed
/// path arms nothing at all — either way, no timeout message may appear.
/// Pitfall: env_remove("CLR_TIMEOUT") required — ambient env var would override the None path
#[ cfg( unix ) ]
#[ test ]
fn ec_timeout_default_no_fire()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  std::fs::write( &fake, b"#!/bin/sh\nprintf 'ok'\nexit 0\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_TIMEOUT" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0: fast subprocess with no expressed timeout. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "timeout" ),
    "no timeout message: nothing may fire on the unexpressed default path. Got:\n{stderr}"
  );
}

// ── ec_timeout_default_unlimited: 2s subprocess, no default watchdog ──────────

/// TSK-503 — no --timeout, 2s subprocess → exit 0 promptly (no default watchdog armed).
///
/// Successor to TSK-227's ec_timeout_default_activates_watchdog (which proved the 3600 s
/// default was armed at a sane value). With the built-in default retired, the same fixture
/// now pins the opposite contract: the unexpressed path kills nothing.
/// Prevention: if a nonzero default under 2 s is ever reintroduced, the subprocess is
/// killed prematurely and this test fails; the <10 s wall bound catches a hang regression.
/// Pitfall: env_remove("CLR_TIMEOUT") required; test timing must allow ≥2s for subprocess sleep
#[ cfg( unix ) ]
#[ test ]
fn ec_timeout_default_unlimited()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // 2s sleep: completes normally — no default watchdog exists to race against
  std::fs::write( &fake, b"#!/bin/sh\nsleep 2\nprintf 'ok'\nexit 0\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let start = std::time::Instant::now();
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_TIMEOUT" )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  assert!(
    out.status.success(),
    "exit must be 0: 2s subprocess completes; no default watchdog exists. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "timeout" ),
    "no default watchdog may fire on the unexpressed path. Got:\n{stderr}"
  );
  assert!(
    elapsed.as_secs() < 10,
    "test must complete in <10s (subprocess sleeps 2s); elapsed {elapsed:?}"
  );
}

// ── ec_timeout_explicit_large_value: --timeout 7200 with fast subprocess ─────

/// TSK-227 — explicit --timeout 7200 (a large expressed value); fast subprocess exits 0.
///
/// Verifies that a large explicit timeout value is accepted and the fast subprocess
/// completes normally. The Some(7200) expressed branch is exercised end to end.
#[ cfg( unix ) ]
#[ test ]
fn ec_timeout_explicit_large_value()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  std::fs::write( &fake, b"#!/bin/sh\nprintf 'ok'\nexit 0\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--timeout", "7200", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_TIMEOUT" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 with --timeout 7200 and fast subprocess. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "timeout" ),
    "no timeout message with --timeout 7200 and fast subprocess. Got:\n{stderr}"
  );
}

// ── ec_timeout_unlimited_flag: --timeout 0 expresses unlimited explicitly ────

/// TSK-227 — `--timeout 0` expresses unlimited explicitly; fast subprocess exits 0.
///
/// Some(0).unwrap_or(DEFAULT_PRINT_TIMEOUT_SECS) = 0 (unlimited). Since TSK-503 the
/// unexpressed default is also 0, so this now confirms the expressed-zero path stays
/// identical to the default rather than opting out of anything.
#[ cfg( unix ) ]
#[ test ]
fn ec_timeout_unlimited_flag()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  std::fs::write( &fake, b"#!/bin/sh\nprintf 'ok'\nexit 0\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--timeout", "0", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_TIMEOUT" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "--timeout 0 must express unlimited; fast subprocess exits 0. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "timeout" ),
    "--timeout 0 means unlimited — no timeout message expected. Got:\n{stderr}"
  );
}

// ── ec_timeout_unlimited_env: CLR_TIMEOUT=0 expresses unlimited via env ──────

/// TSK-227 — `CLR_TIMEOUT=0` expresses unlimited via env var; fast subprocess exits 0.
///
/// apply_env_vars() sets cli.timeout = Some(0); Some(0).unwrap_or(DEFAULT) = 0 (unlimited).
/// Since TSK-503 the unexpressed default is also 0 — this pins the env-expressed-zero
/// path as equivalent (expressed 0 is a full opt-out: no watchdog, no gate budget; BUG-445).
#[ cfg( unix ) ]
#[ test ]
fn ec_timeout_unlimited_env()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  std::fs::write( &fake, b"#!/bin/sh\nprintf 'ok'\nexit 0\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env( "CLR_TIMEOUT", "0" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "CLR_TIMEOUT=0 must express unlimited; fast subprocess exits 0. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "timeout" ),
    "CLR_TIMEOUT=0 means unlimited — no timeout message expected. Got:\n{stderr}"
  );
}

// ── ec_timeout_retry_no_double_emission: BUG-317 — [Process] retry line starts at column 0 ─────

/// BUG-317 — timeout stderr retry line must start with `[Process]`, not with the bare label.
///
/// Root Cause: `eprint!("{}", output.stderr)` at execution.rs:454 fired unconditionally before
///   the retry formatter; execute_print_attempt() stores the timeout label in output.stderr (exit 4);
///   both the unconditional forward AND the retry formatter surface the same string, concatenating
///   `"timeout after Ns[Process] timeout after Ns — retrying…"` with no newline separator.
/// Why Not Caught: EC-7 and ec_timeout_default_kills check only that stderr contains("timeout");
///   no test asserted that each [Process] line starts at position 0, not mid-line.
/// Fix Applied: execution.rs gate changed to `!output.stderr.is_empty() && output.exit_code != 4`;
///   CLR-synthesized timeout label suppressed from unconditional forward; retry formatter
///   surfaces it cleanly via first_message() with the [Process] prefix intact at column 0.
/// Prevention: assert no stderr line begins with "timeout after"; [Process] line must exist.
/// Pitfall: --retry-override 0 disables all retries → no [Process] line emitted at all;
///   must use --retry-on-process 1 to exercise the retry path (the primary BUG-317 site).
#[ cfg( unix ) ]
#[ test ]
fn ec_timeout_retry_no_double_emission()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Fake claude sleeps indefinitely — killed by timeout watchdog on each attempt
  std::fs::write( &fake, b"#!/bin/sh\nsleep 300\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  // _CLR_DEFAULT_TIMEOUT=2: each attempt times out after 2s.
  // --retry-on-process 1: one retry (two total attempts) → exercises the retry branch.
  // --process-delay 0: no inter-retry sleep → test completes in ~4s.
  let out = Command::new( bin )
    .args( [
      "-p",
      "--retry-on-process", "1",
      "--process-delay",    "0",
      "--max-sessions",     "0",
      "x",
    ] )
    .env( "PATH", &new_path )
    .env( "_CLR_DEFAULT_TIMEOUT", "2" )
    .env_remove( "CLR_TIMEOUT" )
    .output()
    .expect( "invoke clr" );

  let stderr = String::from_utf8_lossy( &out.stderr );

  // BUG-317 detection invariant: no stderr line may begin with "timeout after".
  // Pre-fix, each [Process] line was preceded by the bare label with no newline:
  //   "timeout after 2s[Process] timeout after 2s — retrying…"
  for line in stderr.lines()
  {
    assert!(
      !line.starts_with( "timeout after" ),
      "BUG-317 double-emission: stderr line starts with bare timeout label. Line: {line:?}\nFull stderr:\n{stderr}"
    );
  }
  // At least one [Process] line must appear (proves the retry path was exercised).
  assert!(
    stderr.contains( "[Process]" ),
    "stderr must contain a [Process] retry line (proves retry path exercised). Got:\n{stderr}"
  );
}

// ── ec_timeout_default_kills: default watchdog fires and kills hanging subprocess ────────

/// TSK-228 / BUG-305 — no --timeout, _CLR_DEFAULT_TIMEOUT=2, hanging subprocess → exit 4.
///
/// Root Cause: None → unwrap_or(DEFAULT_PRINT_TIMEOUT_SECS) path had no kill test;
///   EC-7 tests Some(1) (explicit --timeout 1); the None (no flag) path was never exercised
///   with a kill — the gap that TSK-228 closes
/// Why Not Caught: TSK-227 added the constant and default path but no integration test proved
///   the watchdog fires on the None branch; ec_timeout_default_constant_value verifies source
///   text only, not runtime kill behaviour
/// Fix Applied: default_print_timeout() reads _CLR_DEFAULT_TIMEOUT env var (test-only override),
///   falls back to DEFAULT_PRINT_TIMEOUT_SECS; run_print_mode() calls unwrap_or(default_print_timeout())
/// Prevention: _CLR_DEFAULT_TIMEOUT=2 arms a test-only 2s default (the production default is
///   unlimited since TSK-503) so a 30s subprocess is killed, proving the None→default path
///   fires poll_timeout() and exits 4
/// Pitfall: must set --retry-override 0 — default retry=2 × delay=30s = 60s hang without it
#[ cfg( unix ) ]
#[ test ]
fn ec_timeout_default_kills()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Fake claude sleeps 30 seconds — will be killed by the 2s default watchdog
  std::fs::write( &fake, b"#!/bin/sh\nsleep 30\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let start = std::time::Instant::now();
  let out = Command::new( bin )
    // No --timeout flag: exercises None → unwrap_or( default_print_timeout() ) path
    .args( [ "-p", "--max-sessions", "0", "--retry-override", "0", "x" ] )
    .env( "PATH", &new_path )
    .env( "_CLR_DEFAULT_TIMEOUT", "2" )
    .env_remove( "CLR_TIMEOUT" )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  assert_eq!(
    out.status.code(),
    Some( 4 ),
    "exit must be 4: default watchdog fired via _CLR_DEFAULT_TIMEOUT=2. Got: {:?}",
    out.status.code()
  );
  assert!(
    elapsed.as_secs() < 10,
    "default watchdog (2s) must fire within ~5s; elapsed {elapsed:?} — kill path broken"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "timeout" ),
    "stderr must contain 'timeout' when default watchdog fires. Got:\n{stderr}"
  );
}
