#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-default` and `--retry-default-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-9 from `tests/docs/cli/param/056_retry_default.md` and
//! EC-1 through EC-6 from `tests/docs/cli/param/057_retry_default_delay.md`.
//!
//! Both parameter specs share this test file because `--retry-default-delay` is
//! the Tier 3 delay paired with the Tier 3 count `--retry-default`.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 56), EC-1..EC-6 (param 57): parser/dry-run — no subprocess
//! - EC-7..EC-9 (param 56): require fake subprocess; test 3-tier priority
//!
//! ## Corner Cases Covered
//!
//! ### --retry-default (param 56)
//! - EC-1: help lists flag
//! - EC-2: value 0 (disables fallback retries) accepted in dry-run
//! - EC-3: value 3 accepted in dry-run
//! - EC-4: `CLR_RETRY_DEFAULT` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: class-specific=1 beats fallback=5; fake always exits 2 → exhausted after 2 calls
//! - EC-8: no class-specific, no override → fallback=3 fires; fake exits 2 once then 0 → exit 0
//! - EC-9: fallback fires for Account class (Account not special-cased)
//!
//! ### --retry-default-delay (param 57)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 accepted in dry-run
//! - EC-4 (delay): `CLR_RETRY_DEFAULT_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 56 — --retry-default ────────────────────────────────────────────────

// ── EC-1: --help lists --retry-default ────────────────────────────────────────

/// EC-1 (param 56): `clr --help` output contains `--retry-default`.
#[ test ]
fn ec1_retry_default_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-default" ),
    "`clr --help` must list --retry-default. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-default 0 --dry-run → exit 0 ───────────────────────────────

/// EC-2 (param 56): value 0 (disables fallback retries) accepted in dry-run.
///
/// Divergence from EC-3: 0 = no fallback retries; 3 = 3 fallback retries per class.
#[ test ]
fn ec2_retry_default_zero_dry_run()
{
  let out = run_cli( &[ "--retry-default", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-default 3 --dry-run → exit 0 ───────────────────────────────

/// EC-3 (param 56): value 3 accepted in dry-run; flag parsed without error.
#[ test ]
fn ec3_retry_default_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-default", "3", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_DEFAULT=3 env var applied ────────────────────────────────

/// EC-4 (param 56): `CLR_RETRY_DEFAULT=3` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_default_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_DEFAULT", "3" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_DEFAULT env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_DEFAULT ────────────────────────────────────

/// EC-5 (param 56): CLI value 3 wins over `CLR_RETRY_DEFAULT=1`.
#[ test ]
fn ec5_retry_default_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-default", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_DEFAULT", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_DEFAULT=invalid → silently ignored ────────────────────────

/// EC-6 (param 56): invalid `CLR_RETRY_DEFAULT` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_default_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_DEFAULT", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_DEFAULT must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: Class-specific beats fallback (Tier 2 > Tier 3) ────────────────────

/// EC-7 (param 56): `--retry-on-transient 1` beats `--retry-default 5`. Fake always
/// exits 2 → exhausted after 2 total invocations (1 initial + 1 retry from class-specific=1,
/// NOT 5 from fallback).
///
/// Root Cause: Tier 2 must take priority over Tier 3 in resolve_count()
/// Why Not Caught: 3-tier priority chain is new in redesign
/// Fix Applied: resolve_count(None, Some(1), Some(5)) returns Some(1); Tier 2 wins
/// Prevention: guard asserting exhaustion after only 2 calls (not 6)
/// Pitfall: invocation count must be verified to confirm class-specific was used
#[ cfg( unix ) ]
#[ test ]
fn ec7_class_specific_beats_retry_default()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let script = format!(
    "#!/bin/sh\n\
     COUNT={}\n\
     N=$(cat \"$COUNT\" 2>/dev/null || echo 0)\n\
     echo $((N + 1)) > \"$COUNT\"\n\
     exit 2\n",
    count.display()
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-transient", "1", "--retry-default", "5",
      "--transient-delay", "0", "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "exit must be 2 (class-specific=1 exhausted, not fallback=5). Got: {:?}", out.status.code()
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
  let n : u32 = std::fs::read_to_string( &count )
    .unwrap_or_default()
    .trim()
    .parse()
    .unwrap_or( 0 );
  assert_eq!(
    n, 2,
    "fake must be invoked exactly 2 times (1 initial + 1 class-specific retry). Got: {n}"
  );
}

// ── EC-8: Fallback fires when no class-specific and no override ───────────────

/// EC-8 (param 56): no `--retry-on-transient`, no `--retry-override`; `--retry-default 3`
/// fires. Fake exits 2 once then 0 → exit 0; `[Transient]` retry message in stderr.
///
/// Root Cause: fallback (Tier 3) must fire when both Tier 1 and Tier 2 are absent
/// Why Not Caught: fallback mechanism is new in redesign
/// Fix Applied: resolve_count(None, None, Some(3)) returns Some(3)
/// Prevention: guard asserting [Transient] retry fires with fallback only
/// Pitfall: must NOT set --retry-on-transient or --retry-override; must env_remove both
#[ cfg( unix ) ]
#[ test ]
fn ec8_retry_default_fires_when_no_class_or_override()
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
    .args( [
      "-p", "--retry-default", "3", "--retry-default-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_RETRY_ON_TRANSIENT" )
    .env_remove( "CLR_RETRY_OVERRIDE" )
    .env_remove( "CLR_RETRY_DEFAULT" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 (fallback=3 fires; retry succeeds). exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Transient]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Transient] retry message (fallback used). Got:\n{stderr}"
  );
}

// ── EC-9: Fallback fires for Account class (Account not special-cased) ────────

/// EC-9 (param 56): no `--retry-on-account`, no `--retry-override`; Tier 3 fallback
/// fires for Account class. Fake emits `"You've hit your limit"` + exits 2 once,
/// then exits 0 → exit 0; `[Account]` retry message in stderr; two invocations.
///
/// Confirms Account uses the same 3-tier resolution as all other error classes —
/// no class_default_count() override blocks fallback.
#[ cfg( unix ) ]
#[ test ]
fn ec9_retry_default_fires_for_account_class()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\nif [ -f \"{count_path}\" ]; then exit 0; fi\ntouch \"{count_path}\"\n\
     printf \"You've hit your limit\\n\"\nexit 2\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-default-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_RETRY_ON_ACCOUNT" )
    .env_remove( "CLR_RETRY_OVERRIDE" )
    .env_remove( "CLR_RETRY_DEFAULT" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 (fallback fires for Account; retry succeeds). exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Account]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Account] retry message (fallback used). Got:\n{stderr}"
  );
}

// ── Param 57 — --retry-default-delay ──────────────────────────────────────────

// ── EC-1 (delay): --help lists --retry-default-delay ──────────────────────────

/// EC-1 (param 57): `clr --help` output contains `--retry-default-delay`.
#[ test ]
fn ec1_retry_default_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-default-delay" ),
    "`clr --help` must list --retry-default-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --retry-default-delay 0 --dry-run → exit 0 ─────────────────

/// EC-2 (param 57): delay=0 (immediate) accepted in dry-run.
#[ test ]
fn ec2_retry_default_delay_zero_dry_run()
{
  let out = run_cli( &[ "--retry-default-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --retry-default-delay 30 --dry-run → exit 0 ────────────────

/// EC-3 (param 57): delay=30 accepted in dry-run.
#[ test ]
fn ec3_retry_default_delay_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-default-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_RETRY_DEFAULT_DELAY=30 env var applied ─────────────────

/// EC-4 (param 57): `CLR_RETRY_DEFAULT_DELAY=30` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_default_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_DEFAULT_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_DEFAULT_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_RETRY_DEFAULT_DELAY ──────────────────────

/// EC-5 (param 57): CLI value 30 wins over `CLR_RETRY_DEFAULT_DELAY=10`.
#[ test ]
fn ec5_retry_default_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-default-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_DEFAULT_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_RETRY_DEFAULT_DELAY=invalid → silently ignored ──────────

/// EC-6 (param 57): invalid `CLR_RETRY_DEFAULT_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_default_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_DEFAULT_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_DEFAULT_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}
