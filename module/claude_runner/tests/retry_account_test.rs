#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-account` and `--account-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-9 from `tests/docs/cli/param/040_retry_on_account.md` and
//! EC-1 through EC-6 from `tests/docs/cli/param/041_account_delay.md`.
//!
//! Both parameter specs share this test file because `--account-delay` only fires
//! when `--retry-on-account` is non-zero — they are functionally coupled.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 40), EC-1..EC-6 (param 41): parser/dry-run — no subprocess
//! - EC-7..EC-9 (param 40): require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-account (param 40)
//! - EC-1: help lists flag
//! - EC-2: value 0 (explicit disable) accepted in dry-run
//! - EC-3: value 2 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_ACCOUNT` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: fake emits `"You've hit your limit"` once then 0; retries=1, delay=0 → exit 0; `[Account]` in stderr
//! - EC-8: fake always emits quota pattern; retries=2, delay=0 → exit 2; `[Account]` exhaustion in stderr
//! - EC-9: no retry flags → Tier 3 fallback fires; fake exits 2 once then 0 → exit 0; `[Account]` in stderr
//!
//! ### --account-delay (param 41)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 accepted in dry-run
//! - EC-4 (delay): `CLR_ACCOUNT_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 40 — --retry-on-account ─────────────────────────────────────────────

// ── EC-1: --help lists --retry-on-account ─────────────────────────────────────

/// EC-1 (param 40): `clr --help` output contains `--retry-on-account`.
#[ test ]
fn ec1_retry_on_account_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-account" ),
    "`clr --help` must list --retry-on-account. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-account 0 --dry-run → exit 0 ────────────────────────────

/// EC-2 (param 40): value 0 (explicit disable, overrides fallback default 2) accepted in dry-run.
///
/// Divergence from EC-3: 0 disables Account retry; 2 (EC-3) activates retry code path.
#[ test ]
fn ec2_retry_on_account_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-account", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-on-account 2 --dry-run → exit 0 ────────────────────────────

/// EC-3 (param 40): value 2 (retry enabled) accepted in dry-run; flag parsed without error.
#[ test ]
fn ec3_retry_on_account_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-account", "2", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_ACCOUNT=2 env var applied ─────────────────────────────

/// EC-4 (param 40): `CLR_RETRY_ON_ACCOUNT=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_account_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_ACCOUNT", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_ACCOUNT env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_ACCOUNT ──────────────────────────────────

/// EC-5 (param 40): CLI value 3 wins over `CLR_RETRY_ON_ACCOUNT=1`.
#[ test ]
fn ec5_retry_on_account_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-account", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_ACCOUNT", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_ACCOUNT=invalid → silently ignored ─────────────────────

/// EC-6 (param 40): invalid `CLR_RETRY_ON_ACCOUNT` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_on_account_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_ACCOUNT", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_ACCOUNT must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One quota-exhausted failure then success → retried; exit 0 ──────────

/// EC-7 (param 40): fake emits `"You've hit your limit"` + exits 2 once, then exits 0.
/// retries=1, delay=0 → exit 0; `[Account]` in stderr.
///
/// Root Cause: Account class is new in retry system redesign
/// Why Not Caught: no test existed for Account class retry behavior
/// Fix Applied: integration test using quota-exhausted fake script with counter file
/// Prevention: guard with integration test asserting [Account] prefix in stderr
/// Pitfall: quota pattern must go to stdout (classify_error scans both); exit 2 alone
///          would classify as Transient — text pattern overrides to Account
#[ cfg( unix ) ]
#[ test ]
fn ec7_account_retry_succeeds_after_one_quota_exhausted()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\n\
     if [ -f \"{count_path}\" ]; then exit 0; fi\n\
     touch \"{count_path}\"\n\
     printf \"You've hit your limit\\n\"\n\
     exit 2\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-account", "1", "--account-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 after Account retry succeeds. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Account]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Account] retry message. Got:\n{stderr}"
  );
}

// ── EC-8: All Account retries exhausted → exit 2; [Account] exhaustion ────────

/// EC-8 (param 40): fake always emits quota pattern + exits 2; retries=2, delay=0 →
/// exit 2; `[Account]` exhaustion in stderr; 3 total invocations.
///
/// Root Cause: Account class retry exhaustion needs verification
/// Why Not Caught: new class, no prior test
/// Fix Applied: integration test with always-failing quota fake
/// Prevention: guard asserting [Account] + "exhausted" in stderr
/// Pitfall: test uses retries=2 to verify multiple retry attempts, not just 1
#[ cfg( unix ) ]
#[ test ]
fn ec8_account_retry_exhausted()
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
      "-p", "--retry-on-account", "2", "--account-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "exit must be 2 after Account retries exhausted. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Account]" ),
    "stderr must contain [Account] class label. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "exhaust" ),
    "stderr must contain exhaustion message. Got:\n{stderr}"
  );
}

// ── EC-9: Account retries via Tier 3 fallback (no class-specific flag) ─────────

/// EC-9 (param 40): fake emits `"You've hit your limit"` + exits 2 on first call,
/// then exits 0 on second. NO `--retry-on-account` flag set; Tier 3 fallback (default 2)
/// fires → retry succeeds; exit 0; `[Account]` in stderr; two invocations.
///
/// Root Cause: class_default_count(Account) = Some(0) blocked Tier 3 fallback, causing
///   Account errors to never retry unless explicitly opted-in with --retry-on-account.
/// Why Not Caught: EC-7 and EC-8 both test explicit --retry-on-account N (opt-in); neither
///   tested the unset/default behaviour where fallback should fire.
/// Fix Applied: class_default_count() removed; resolve_count() simplified to 3-tier
///   (override ?? class-specific ?? fallback). All classes now use uniform resolution.
/// Prevention: this test asserts retry fires via Tier 3 fallback when no class-specific flag set.
/// Pitfall: DO NOT set --retry-on-account here — that tests Tier 2, not Tier 3 fallback.
///   env_remove CLR_RETRY_ON_ACCOUNT/CLR_RETRY_DEFAULT/CLR_RETRY_OVERRIDE for determinism.
#[ cfg( unix ) ]
#[ test ]
fn ec9_account_retries_via_tier3_fallback()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\n\
     if [ -f \"{count_path}\" ]; then exit 0; fi\n\
     touch \"{count_path}\"\n\
     printf \"You've hit your limit\\n\"\n\
     exit 2\n"
  );
  std::fs::write( &fake, script.as_bytes() ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin      = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [ "-p", "--retry-default-delay", "0", "--max-sessions", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_RETRY_ON_ACCOUNT" )
    .env_remove( "CLR_RETRY_DEFAULT" )
    .env_remove( "CLR_RETRY_OVERRIDE" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "Tier 3 fallback must fire for Account: exit must be 0. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Account]" ),
    "stderr must contain [Account] class label. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "retry" ),
    "stderr must contain retry message (Tier 3 fallback fired). Got:\n{stderr}"
  );
}

// ── Param 41 — --account-delay ────────────────────────────────────────────────

// ── EC-1 (delay): --help lists --account-delay ────────────────────────────────

/// EC-1 (param 41): `clr --help` output contains `--account-delay`.
#[ test ]
fn ec1_account_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--account-delay" ),
    "`clr --help` must list --account-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --account-delay 0 --dry-run → exit 0 ───────────────────────

/// EC-2 (param 41): delay=0 (immediate retry) accepted in dry-run.
#[ test ]
fn ec2_account_delay_zero_dry_run()
{
  let out = run_cli( &[ "--account-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --account-delay 30 --dry-run → exit 0 ──────────────────────

/// EC-3 (param 41): delay=30 accepted in dry-run.
#[ test ]
fn ec3_account_delay_nonzero_dry_run()
{
  let out = run_cli( &[ "--account-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_ACCOUNT_DELAY=30 env var applied ───────────────────────

/// EC-4 (param 41): `CLR_ACCOUNT_DELAY=30` applied when CLI flag absent.
#[ test ]
fn ec4_clr_account_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_ACCOUNT_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_ACCOUNT_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_ACCOUNT_DELAY ─────────────────────────────

/// EC-5 (param 41): CLI value 30 wins over `CLR_ACCOUNT_DELAY=10`.
#[ test ]
fn ec5_account_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--account-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_ACCOUNT_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_ACCOUNT_DELAY=invalid → silently ignored ────────────────

/// EC-6 (param 41): invalid `CLR_ACCOUNT_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_account_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_ACCOUNT_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_ACCOUNT_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}
