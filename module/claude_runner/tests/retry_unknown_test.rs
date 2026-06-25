#![ allow( missing_docs ) ]
#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-unknown` and `--unknown-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-10 from `tests/docs/cli/param/052_retry_on_unknown.md` and
//! EC-1 through EC-7 from `tests/docs/cli/param/053_unknown_delay.md`.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 52), EC-1..EC-6 (param 53): parser/dry-run — no subprocess
//! - EC-7..EC-10 (param 52), EC-7 (param 53): require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-unknown (param 52)
//! - EC-1: help lists flag; old `--retry-on-unknown-error` absent
//! - EC-2: value 0 (explicit no-retry) accepted in dry-run
//! - EC-3: value 2 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_UNKNOWN` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: fake emits unrecognized text + exits 5 once then 0; retries=1, delay=0 → exit 0; `[Unknown]` in stderr
//! - EC-8: fake always emits unrecognized text + exits 5; retries=2 → exit 5; `[Unknown]` exhaustion
//! - EC-9: old flag `--retry-on-unknown-error` rejected → exit 1; "unknown option" in stderr
//! - EC-10: no explicit flag; fallback default=2 fires; fake exits 5 once then 0 → exit 0
//!
//! ### --unknown-delay (param 53)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 accepted in dry-run
//! - EC-4 (delay): `CLR_UNKNOWN_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored
//! - EC-7 (delay): delay=0 causes immediate retry; exit 0

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 52 — --retry-on-unknown ────────────────────────────────────────────

// ── EC-1: --help lists --retry-on-unknown ────────────────────────────────────

/// EC-1 (param 52): `clr --help` output contains `--retry-on-unknown`; old flag absent.
#[ test ]
fn ec1_retry_on_unknown_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-unknown" ),
    "`clr --help` must list --retry-on-unknown. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--retry-on-unknown-error" ),
    "`clr --help` must NOT list old --retry-on-unknown-error. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-unknown 0 --dry-run → exit 0 ────────────────────────────

/// EC-2 (param 52): value 0 (explicit zero) accepted in dry-run; disables Unknown retry.
///
/// Divergence from EC-3: 0 beats fallback default (2); 2 activates retry code path.
#[ test ]
fn ec2_retry_on_unknown_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-unknown", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-on-unknown 2 --dry-run → exit 0 ────────────────────────────

/// EC-3 (param 52): value 2 (retry enabled) accepted in dry-run; flag parsed without error.
#[ test ]
fn ec3_retry_on_unknown_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-unknown", "2", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_UNKNOWN=2 env var applied ─────────────────────────────

/// EC-4 (param 52): `CLR_RETRY_ON_UNKNOWN=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_unknown_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_UNKNOWN", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_UNKNOWN env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_UNKNOWN ─────────────────────────────────

/// EC-5 (param 52): CLI value 3 wins over `CLR_RETRY_ON_UNKNOWN=1`.
#[ test ]
fn ec5_retry_on_unknown_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-unknown", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_UNKNOWN", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_UNKNOWN=invalid → silently ignored ────────────────────

/// EC-6 (param 52): invalid `CLR_RETRY_ON_UNKNOWN` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_on_unknown_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_UNKNOWN", "bad" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_UNKNOWN must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One unknown error then success → retried; exit 0 ───────────────────

/// EC-7 (param 52): fake emits "something went wrong" + exits 5 once then 0;
/// retries=1, delay=0 → exit 0; `[Unknown]` in stderr.
///
/// Root Cause: --retry-on-unknown-error renamed to --retry-on-unknown in redesign
/// Why Not Caught: flag rename not yet reflected in test code
/// Fix Applied: updated flags; added --unknown-delay 0 alongside --retry-on-unknown 1
/// Prevention: exit 5 → no recognized text pattern, exit ≤ 128, exit ≠ 2 → Unknown class
/// Pitfall: script must NOT emit "API Error: " or "You've hit your limit" — those would
///          classify as Service or Account, not Unknown
#[ cfg( unix ) ]
#[ test ]
fn ec7_unknown_retry_succeeds_after_one_failure()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\n\
     if [ -f \"{count_path}\" ]; then exit 0; fi\n\
     touch \"{count_path}\"\n\
     printf 'something went wrong\\n' >&2\n\
     exit 5\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-unknown", "1", "--unknown-delay", "0",
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
    stderr.contains( "[Unknown]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Unknown] retry message. Got:\n{stderr}"
  );
}

// ── EC-8: All unknown retries exhausted → exit 5; [Unknown] exhaustion ────────

/// EC-8 (param 52): fake always emits unrecognized text + exits 5; retries=2 → exit 5; [Unknown] exhaustion.
///
/// Root Cause: --retry-on-unknown-error renamed; stderr prefix changed to [Unknown]
/// Why Not Caught: rename not yet reflected in test code
/// Fix Applied: updated flags and assertion to match [Unknown] class label; uses exit 5
/// Prevention: exit 5 relayed as final exit code after exhaustion
/// Pitfall: with retries=2, there are 3 total invocations (1 initial + 2 retries)
#[ cfg( unix ) ]
#[ test ]
fn ec8_unknown_retry_exhausted()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  std::fs::write(
    &fake,
    b"#!/bin/sh\nprintf 'something went wrong\\n' >&2\nexit 5\n",
  ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-unknown", "2", "--unknown-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 5 ),
    "exit must relay subprocess exit code 5 after exhaustion. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Unknown]" ),
    "stderr must contain [Unknown] class label. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "exhaust" ),
    "stderr must contain exhaustion message. Got:\n{stderr}"
  );
}

// ── EC-9: Old flag --retry-on-unknown-error rejected at parse time ────────────

/// EC-9 (param 52): old flag `--retry-on-unknown-error` → exit 1; "unknown option" in stderr.
///
/// Root Cause: --retry-on-unknown-error removed entirely in redesign
/// Why Not Caught: backward-compat rejection not yet tested
/// Fix Applied: parse.rs does not recognize old flag → falls through to unknown-option error
/// Prevention: guard confirming old flag is hard-rejected, not silently accepted as no-op
/// Pitfall: the flag must produce exit 1 (parse error), NOT be silently ignored
#[ test ]
fn ec9_old_flag_retry_on_unknown_error_rejected()
{
  let out = run_cli( &[ "--retry-on-unknown-error", "1", "--dry-run", "task" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "old flag --retry-on-unknown-error must exit 1. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "unknown" ),
    "stderr must contain 'unknown option' for old flag. Got:\n{stderr}"
  );
}

// ── EC-10: Default fallback fires without explicit flag ───────────────────────

/// EC-10 (param 52): no `--retry-on-unknown` and no `CLR_RETRY_ON_UNKNOWN`;
/// fallback default=2 fires when fake exits 5 once then 0.
///
/// Root Cause: old default was 0 (no retry); now auto→fallback(2) via 3-tier system
/// Why Not Caught: old EC-9 tested default=0; semantics changed in redesign
/// Fix Applied: uses --retry-default 2 (explicit count) and --retry-default-delay 0 (explicit
///              delay) to make test non-fragile; validates fallback fires without class flag
/// Prevention: guard asserting [Unknown] retry fires without explicit --retry-on-unknown
/// Pitfall: must NOT set --retry-on-unknown; must NOT set CLR_RETRY_ON_UNKNOWN
#[ cfg( unix ) ]
#[ test ]
fn ec10_unknown_fallback_default_fires()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let script = format!(
    "#!/bin/sh\n\
     COUNT={}\n\
     N=$(cat \"$COUNT\" 2>/dev/null || echo 0)\n\
     echo $((N + 1)) > \"$COUNT\"\n\
     if [ \"$N\" -eq 0 ]; then printf 'something went wrong\\n' >&2; exit 5; fi\n\
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
    .env_remove( "CLR_RETRY_ON_UNKNOWN" )
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
    stderr.contains( "[Unknown]" ),
    "stderr must contain [Unknown] class label from fallback retry. Got:\n{stderr}"
  );
}

// ── Param 53 — --unknown-delay ────────────────────────────────────────────────

// ── EC-1 (delay): --help lists --unknown-delay ───────────────────────────────

/// EC-1 (param 53): `clr --help` output contains `--unknown-delay`.
#[ test ]
fn ec1_unknown_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--unknown-delay" ),
    "`clr --help` must list --unknown-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --unknown-delay 0 --dry-run → exit 0 ──────────────────────

/// EC-2 (param 53): delay=0 (immediate retry) accepted in dry-run.
#[ test ]
fn ec2_unknown_delay_zero_dry_run()
{
  let out = run_cli( &[ "--unknown-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --unknown-delay 30 --dry-run → exit 0 ─────────────────────

/// EC-3 (param 53): delay=30 accepted in dry-run.
#[ test ]
fn ec3_unknown_delay_thirty_dry_run()
{
  let out = run_cli( &[ "--unknown-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_UNKNOWN_DELAY=30 env var applied ──────────────────────

/// EC-4 (param 53): `CLR_UNKNOWN_DELAY=30` applied when CLI flag absent.
#[ test ]
fn ec4_clr_unknown_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_UNKNOWN_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_UNKNOWN_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_UNKNOWN_DELAY ────────────────────────────

/// EC-5 (param 53): CLI value 30 wins over `CLR_UNKNOWN_DELAY=10`.
#[ test ]
fn ec5_unknown_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--unknown-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_UNKNOWN_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_UNKNOWN_DELAY=invalid → silently ignored ───────────────

/// EC-6 (param 53): invalid `CLR_UNKNOWN_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_unknown_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_UNKNOWN_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_UNKNOWN_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7 (delay): delay=0 with unknown error retry → fires immediately; exit 0 ─

/// EC-7 (param 53): delay=0 causes immediate retry (no sleep); exit 0.
///
/// Root Cause: new delay param --unknown-delay; no previous test existed
/// Why Not Caught: unknown-delay is a new parameter in the redesign
/// Fix Applied: guard confirming delay=0 retries without sleep
/// Prevention: timing assertion — delay=0 must complete in < 5s
/// Pitfall: if delay=0 were treated as "default 30s" the test would time out
#[ cfg( unix ) ]
#[ test ]
fn ec7_unknown_delay_zero_immediate_retry()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\n\
     if [ -f \"{count_path}\" ]; then exit 0; fi\n\
     touch \"{count_path}\"\n\
     printf 'something went wrong\\n' >&2\n\
     exit 5\n"
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
      "-p", "--retry-on-unknown", "1", "--unknown-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );
  let elapsed = start.elapsed();

  assert!(
    out.status.success(),
    "exit must be 0 with delay=0 unknown error retry. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  assert!(
    elapsed.as_secs() < 5,
    "delay=0 must retry immediately; elapsed {elapsed:?} is too long"
  );
}
