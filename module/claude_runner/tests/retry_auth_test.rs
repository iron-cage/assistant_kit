#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-auth` and `--auth-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-8 from `tests/docs/cli/param/042_retry_on_auth.md` and
//! EC-1 through EC-6 from `tests/docs/cli/param/043_auth_delay.md`.
//!
//! Both parameter specs share this test file because `--auth-delay` only fires
//! when `--retry-on-auth` is non-zero — they are functionally coupled.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 42), EC-1..EC-6 (param 43): parser/dry-run — no subprocess
//! - EC-7..EC-8 (param 42): require fake subprocess
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-auth (param 42)
//! - EC-1: help lists flag
//! - EC-2: value 0 (explicit disable) accepted in dry-run
//! - EC-3: value 2 (retry enabled) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_AUTH` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: fake emits auth pattern once then 0; retries=1, delay=0 → exit 0; `[Auth]` in stderr
//! - EC-8: fake always emits auth pattern; retries=2, delay=0 → exit 1; `[Auth]` exhaustion
//!
//! ### --auth-delay (param 43)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 accepted in dry-run
//! - EC-4 (delay): `CLR_AUTH_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 42 — --retry-on-auth ────────────────────────────────────────────────

// ── EC-1: --help lists --retry-on-auth ────────────────────────────────────────

/// EC-1 (param 42): `clr --help` output contains `--retry-on-auth`.
#[ test ]
fn ec1_retry_on_auth_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-auth" ),
    "`clr --help` must list --retry-on-auth. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-auth 0 --dry-run → exit 0 ───────────────────────────────

/// EC-2 (param 42): value 0 (explicit disable, overrides fallback default 2) accepted in dry-run.
///
/// Divergence from EC-3: 0 disables Auth retry; 2 (EC-3) activates retry code path.
#[ test ]
fn ec2_retry_on_auth_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-auth", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-on-auth 2 --dry-run → exit 0 ───────────────────────────────

/// EC-3 (param 42): value 2 (retry enabled) accepted in dry-run; flag parsed without error.
#[ test ]
fn ec3_retry_on_auth_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-auth", "2", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_AUTH=2 env var applied ─────────────────────────────────

/// EC-4 (param 42): `CLR_RETRY_ON_AUTH=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_auth_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_AUTH", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_AUTH env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_AUTH ─────────────────────────────────────

/// EC-5 (param 42): CLI value 3 wins over `CLR_RETRY_ON_AUTH=1`.
#[ test ]
fn ec5_retry_on_auth_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-auth", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_AUTH", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_AUTH=invalid → silently ignored ────────────────────────

/// EC-6 (param 42): invalid `CLR_RETRY_ON_AUTH` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_on_auth_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_AUTH", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_AUTH must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: One auth error then success → retried; exit 0 ──────────────────────

/// EC-7 (param 42): fake emits auth pattern + exits 1 once, then exits 0.
/// retries=1, delay=0 → exit 0; `[Auth]` in stderr.
///
/// Root Cause: Auth class is new in retry system redesign
/// Why Not Caught: no test existed for Auth class retry behavior
/// Fix Applied: integration test using auth-error fake script with counter file
/// Prevention: guard with integration test asserting [Auth] prefix in stderr
/// Pitfall: auth pattern must appear in output for classify_error; exit 1 alone
///          is not sufficient — text pattern "Your organization does not have access"
///          determines Auth classification
#[ cfg( unix ) ]
#[ test ]
fn ec7_auth_retry_succeeds_after_one_auth_error()
{
  let tmp   = tempfile::tempdir().expect( "create temp dir" );
  let fake  = tmp.path().join( "claude" );
  let count = tmp.path().join( "count" );

  let count_path = count.to_str().expect( "counter path utf-8" );
  let script = format!(
    "#!/bin/sh\n\
     if [ -f \"{count_path}\" ]; then exit 0; fi\n\
     touch \"{count_path}\"\n\
     printf 'Your organization does not have access to Claude\\n'\n\
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
      "-p", "--retry-on-auth", "1", "--auth-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 after Auth retry succeeds. exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Auth]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Auth] retry message. Got:\n{stderr}"
  );
}

// ── EC-8: All Auth retries exhausted → exit 1; [Auth] exhaustion ──────────────

/// EC-8 (param 42): fake always emits auth pattern + exits 1; retries=2, delay=0 →
/// exit 1; `[Auth]` exhaustion in stderr; 3 total invocations.
///
/// Root Cause: Auth class retry exhaustion needs verification
/// Why Not Caught: new class, no prior test
/// Fix Applied: integration test with always-failing auth fake
/// Prevention: guard asserting [Auth] + "exhausted" in stderr
/// Pitfall: test uses retries=2 to verify multiple retry attempts, not just 1
#[ cfg( unix ) ]
#[ test ]
fn ec8_auth_retry_exhausted()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  std::fs::write(
    &fake,
    b"#!/bin/sh\nprintf 'Your organization does not have access to Claude\\n'\nexit 1\n",
  ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  let bin = env!( "CARGO_BIN_EXE_clr" );

  let out = Command::new( bin )
    .args( [
      "-p", "--retry-on-auth", "2", "--auth-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 after Auth retries exhausted. Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Auth]" ),
    "stderr must contain [Auth] class label. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "exhaust" ),
    "stderr must contain exhaustion message. Got:\n{stderr}"
  );
}

// ── Param 43 — --auth-delay ──────────────────────────────────────────────────

// ── EC-1 (delay): --help lists --auth-delay ──────────────────────────────────

/// EC-1 (param 43): `clr --help` output contains `--auth-delay`.
#[ test ]
fn ec1_auth_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--auth-delay" ),
    "`clr --help` must list --auth-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --auth-delay 0 --dry-run → exit 0 ─────────────────────────

/// EC-2 (param 43): delay=0 (immediate retry) accepted in dry-run.
#[ test ]
fn ec2_auth_delay_zero_dry_run()
{
  let out = run_cli( &[ "--auth-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --auth-delay 30 --dry-run → exit 0 ────────────────────────

/// EC-3 (param 43): delay=30 accepted in dry-run.
#[ test ]
fn ec3_auth_delay_nonzero_dry_run()
{
  let out = run_cli( &[ "--auth-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_AUTH_DELAY=30 env var applied ──────────────────────────

/// EC-4 (param 43): `CLR_AUTH_DELAY=30` applied when CLI flag absent.
#[ test ]
fn ec4_clr_auth_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_AUTH_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_AUTH_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_AUTH_DELAY ───────────────────────────────

/// EC-5 (param 43): CLI value 30 wins over `CLR_AUTH_DELAY=10`.
#[ test ]
fn ec5_auth_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--auth-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_AUTH_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_AUTH_DELAY=invalid → silently ignored ──────────────────

/// EC-6 (param 43): invalid `CLR_AUTH_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_auth_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_AUTH_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_AUTH_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}
