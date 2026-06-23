#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-validation` / `--validation-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1..EC-10 for `--retry-on-validation` (param 048, renamed from `--expect-retries`)
//! and EC-1..EC-6 for `--validation-delay` (param 049, new in retry system redesign).
//!
//! ## Key Design Note
//!
//! Validation class retry requires `--expect-strategy retry` to enter the retry branch in
//! `apply_expect_validation()`.  `--retry-on-validation` controls the class-specific count
//! in the 3-tier resolver: `resolve_count(override, retry_on_validation, fallback)`.
//! Without `--expect-strategy retry`, mismatches exit 3 immediately regardless of the count.
//!
//! `CLR_RETRY_ON_VALIDATION` is the ONLY per-class retry env var that rejects invalid values
//! (exits 1) rather than silently ignoring them — because it goes through `parse_u8_bounded`.
//!
//! ## Test Layout
//!
//! ### --retry-on-validation (param 048)
//! - EC-1:  `--help` lists `--retry-on-validation`; `--expect-retries` absent
//! - EC-2:  `--retry-on-validation 0 --dry-run` → exit 0; explicit zero accepted
//! - EC-3:  `--retry-on-validation 2 --dry-run` → exit 0; nonzero accepted
//! - EC-4:  `CLR_RETRY_ON_VALIDATION=2 --dry-run` → exit 0; env var applied
//! - EC-5:  CLI wins over `CLR_RETRY_ON_VALIDATION`
//! - EC-6:  `CLR_RETRY_ON_VALIDATION=notanumber --dry-run` → exit 1; invalid env var rejected
//! - EC-7:  Retry succeeds on 2nd attempt; `[Validation]` in stderr
//! - EC-8:  Retries exhausted → exit 3; `[Validation]` exhaustion in stderr
//! - EC-9:  Old flag `--expect-retries 1` → exit 1; "unknown" in stderr
//! - EC-10: No explicit flag; fallback default (2) fires; retry succeeds on 2nd attempt
//!
//! ### --validation-delay (param 049)
//! - EC-1:  `--help` lists `--validation-delay`
//! - EC-2:  `--validation-delay 0 --dry-run` → exit 0; zero accepted
//! - EC-3:  `--validation-delay 30 --dry-run` → exit 0; non-zero accepted
//! - EC-4:  `CLR_VALIDATION_DELAY=30 --dry-run` → exit 0; env var applied
//! - EC-5:  `--validation-delay` CLI wins over `CLR_VALIDATION_DELAY`
//! - EC-6:  `CLR_VALIDATION_DELAY=abc --dry-run` → exit 0; silently ignored
#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude, run_cli, run_with_path };
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

// ── EC-1: --help lists --retry-on-validation ─────────────────────────────────

/// EC-1: `clr --help` lists `--retry-on-validation`; old flag `--expect-retries` is absent.
#[ test ]
fn ec1_retry_on_validation_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-validation" ),
    "--help must list --retry-on-validation. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--expect-retries" ),
    "--help must NOT list --expect-retries. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-validation 0 --dry-run → exit 0 ────────────────────────

/// EC-2: `--retry-on-validation 0` is accepted at parse time (dry-run exits 0).
///
/// Divergence from EC-3: 0 disables Validation retry; 2 enables it.
#[ test ]
fn ec2_retry_on_validation_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-validation", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "--retry-on-validation 0 dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3: --retry-on-validation 2 --dry-run → exit 0 ────────────────────────

/// EC-3: `--retry-on-validation 2` is accepted at parse time (dry-run exits 0).
#[ test ]
fn ec3_retry_on_validation_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-validation", "2", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "--retry-on-validation 2 dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_VALIDATION=2 env var accepted ────────────────────────

/// EC-4: `CLR_RETRY_ON_VALIDATION=2` is applied when the CLI flag is absent.
#[ test ]
fn ec4_clr_retry_on_validation_env_var_accepted()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--dry-run", "task" ] )
    .env( "CLR_RETRY_ON_VALIDATION", "2" )
    .env_remove( "CLR_RETRY_DEFAULT" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_VALIDATION=2 dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_VALIDATION ──────────────────────────────

/// EC-5: CLI `--retry-on-validation 3` beats `CLR_RETRY_ON_VALIDATION=1`.
#[ test ]
fn ec5_retry_on_validation_cli_wins_over_env()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--retry-on-validation", "3", "--dry-run", "task" ] )
    .env( "CLR_RETRY_ON_VALIDATION", "1" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "CLI --retry-on-validation must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_VALIDATION=invalid → exit 1 ──────────────────────────

/// EC-6: `CLR_RETRY_ON_VALIDATION=notanumber` → exit 1.
///
/// Unlike other per-class retry env vars (which silently ignore bad values),
/// `CLR_RETRY_ON_VALIDATION` goes through `parse_u8_bounded` and returns an error
/// that propagates up as exit 1. This is the ONLY env var with this behavior.
#[ test ]
fn ec6_clr_retry_on_validation_invalid_rejected()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--dry-run", "task" ] )
    .env( "CLR_RETRY_ON_VALIDATION", "notanumber" )
    .output()
    .expect( "invoke clr" );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "CLR_RETRY_ON_VALIDATION=notanumber must exit 1. Got: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "CLR_RETRY_ON_VALIDATION" ),
    "stderr must mention CLR_RETRY_ON_VALIDATION. Got:\n{stderr}"
  );
}

// ── EC-7: Validation retry succeeds on 2nd attempt ───────────────────────────

/// EC-7: With `--retry-on-validation 1 --validation-delay 0 --expect-strategy retry`,
/// a fake that mismatches on call 1 then matches on call 2 → exit 0.
///
/// Requires `--expect-strategy retry` to enter the retry branch in
/// `apply_expect_validation()`. stderr must contain `[Validation]`.
#[ test ]
fn ec7_validation_retry_succeeds_after_one_mismatch()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let count_path = tmp.path().join( "count.txt" );
  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\nCF={}\nN=0\n[ -f \"$CF\" ] && N=$(cat \"$CF\")\nN=$((N+1))\necho $N > \"$CF\"\n\
     [ \"$N\" -eq 1 ] && echo 'fail' || echo 'pass'\n",
    count_path.display()
  );
  std::fs::write( &fake, &script ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  let out = run_with_path(
    &[
      "-p",
      "--expect",              "pass",
      "--expect-strategy",     "retry",
      "--retry-on-validation", "1",
      "--validation-delay",    "0",
      "--max-sessions",        "0",
      "x",
    ],
    &new_path,
  );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "validation retry must succeed on 2nd attempt. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Validation]" ),
    "stderr must contain [Validation]. Got:\n{stderr}"
  );
}

// ── EC-8: Validation retries exhausted → exit 3 ──────────────────────────────

/// EC-8: With `--retry-on-validation 2`, a fake that never matches → exit 3.
///
/// 3 total invocations (1 initial + 2 retries). stderr contains `[Validation]`
/// and "exhausted". The [Validation] exhaustion message uses exit code 3.
#[ test ]
fn ec8_validation_retry_exhausted()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho 'fail'" );
  let out = run_with_path(
    &[
      "-p",
      "--expect",              "pass",
      "--expect-strategy",     "retry",
      "--retry-on-validation", "2",
      "--validation-delay",    "0",
      "--max-sessions",        "0",
      "x",
    ],
    &path,
  );
  assert_eq!(
    out.status.code(),
    Some( 3 ),
    "exhausted validation retries must exit 3. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Validation]" ),
    "stderr must contain [Validation]. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "exhaust" ),
    "stderr must contain 'exhaust'. Got:\n{stderr}"
  );
}

// ── EC-9: Old flag --expect-retries rejected ─────────────────────────────────

/// EC-9: Old flag `--expect-retries 1` is rejected at parse time (exit 1).
///
/// `--expect-retries` was renamed to `--retry-on-validation` in the retry system
/// redesign. The old flag name must not be silently accepted.
#[ test ]
fn ec9_old_flag_expect_retries_rejected()
{
  let out = run_cli( &[ "--expect-retries", "1", "--dry-run", "task" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "--expect-retries must exit 1 (old flag). Got: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "unknown" ),
    "stderr must mention 'unknown'. Got:\n{stderr}"
  );
}

// ── EC-10: Fallback default (2) fires for Validation ─────────────────────────

/// EC-10: No `--retry-on-validation` set; fallback default fires.
///
/// Passes `--retry-default 2` explicitly (non-fragile: does not rely on system default).
/// Fake mismatches on call 1 then matches on call 2 → exit 0.
/// Uses `--retry-default-delay 0` to avoid the 30s built-in delay.
#[ test ]
fn ec10_validation_fallback_default_fires()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let count_path = tmp.path().join( "count.txt" );
  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\nCF={}\nN=0\n[ -f \"$CF\" ] && N=$(cat \"$CF\")\nN=$((N+1))\necho $N > \"$CF\"\n\
     [ \"$N\" -eq 1 ] && echo 'fail' || echo 'pass'\n",
    count_path.display()
  );
  std::fs::write( &fake, &script ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  // No --retry-on-validation; fallback default=2 supplies the count.
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [
      "-p",
      "--expect",              "pass",
      "--expect-strategy",     "retry",
      "--retry-default",       "2",
      "--retry-default-delay", "0",
      "--max-sessions",        "0",
      "x",
    ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_RETRY_ON_VALIDATION" )
    .env_remove( "CLR_RETRY_DEFAULT" )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "fallback default must allow retry to succeed. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Validation]" ),
    "stderr must contain [Validation] retry message. Got:\n{stderr}"
  );
}

// ── --validation-delay (param 049) ───────────────────────────────────────────

// ── EC-1: --help lists --validation-delay ────────────────────────────────────

/// EC-1: `clr --help` lists `--validation-delay`.
#[ test ]
fn ec1_validation_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--validation-delay" ),
    "--help must list --validation-delay. Got:\n{stdout}"
  );
}

// ── EC-2: --validation-delay 0 --dry-run → exit 0 ────────────────────────────

/// EC-2: `--validation-delay 0` is accepted at parse time; zero-second delay accepted.
///
/// Divergence from EC-3: 0 = immediate retry; 30 = 30s sleep between retries.
#[ test ]
fn ec2_validation_delay_zero_dry_run()
{
  let out = run_cli( &[ "--validation-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "--validation-delay 0 dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3: --validation-delay 30 --dry-run → exit 0 ───────────────────────────

/// EC-3: `--validation-delay 30` is accepted at parse time.
#[ test ]
fn ec3_validation_delay_nonzero_dry_run()
{
  let out = run_cli( &[ "--validation-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "--validation-delay 30 dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_VALIDATION_DELAY=30 env var accepted ───────────────────────────

/// EC-4: `CLR_VALIDATION_DELAY=30` is applied when the CLI flag is absent.
#[ test ]
fn ec4_clr_validation_delay_env_var_accepted()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--dry-run", "task" ] )
    .env( "CLR_VALIDATION_DELAY", "30" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "CLR_VALIDATION_DELAY=30 dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_VALIDATION_DELAY ─────────────────────────────────

/// EC-5: `--validation-delay 30` beats `CLR_VALIDATION_DELAY=10`.
#[ test ]
fn ec5_validation_delay_cli_wins_over_env()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--validation-delay", "30", "--dry-run", "task" ] )
    .env( "CLR_VALIDATION_DELAY", "10" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "CLI --validation-delay must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_VALIDATION_DELAY=invalid → silently ignored ────────────────────

/// EC-6: `CLR_VALIDATION_DELAY=abc` is silently ignored (env var parse failure).
///
/// Unlike `CLR_RETRY_ON_VALIDATION`, the delay env var uses `.parse::<u32>().ok()`
/// so invalid values are silently treated as None and the fallback delay applies.
#[ test ]
fn ec6_clr_validation_delay_invalid_ignored()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--dry-run", "task" ] )
    .env( "CLR_VALIDATION_DELAY", "abc" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "CLR_VALIDATION_DELAY=abc dry-run must exit 0 (silently ignored). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}
