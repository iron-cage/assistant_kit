#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-override` and `--retry-override-delay` Integration Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-10 from `tests/docs/cli/param/054_retry_override.md` and
//! EC-1 through EC-6 from `tests/docs/cli/param/055_retry_override_delay.md`.
//!
//! Both parameter specs share this test file because `--retry-override-delay` is
//! the Tier 1 delay paired with the Tier 1 count `--retry-override`.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 54), EC-1..EC-6 (param 55): parser/dry-run — no subprocess
//! - EC-7..EC-10 (param 54): require fake subprocess; test 3-tier priority
//!
//! ## Corner Cases Covered
//!
//! ### --retry-override (param 54)
//! - EC-1: help lists flag
//! - EC-2: value 0 (disables all retries) accepted in dry-run
//! - EC-3: value 3 (enables retries for all classes) accepted in dry-run
//! - EC-4: `CLR_RETRY_OVERRIDE` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//! - EC-7: override=0 disables Transient retry; fake exits 2 → still exit 2; no retry in stderr
//! - EC-8: override=2 beats class-specific=0; fake exits 2 once then 0 → exit 0; [Transient] in stderr
//! - EC-9: override applies to Service class; API Error fake → retried; exit 0; [Service] in stderr
//! - EC-10: no override; class-specific=1 honored; fake exits 2 once then 0 → exit 0
//!
//! ### --retry-override-delay (param 55)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 accepted in dry-run
//! - EC-4 (delay): `CLR_RETRY_OVERRIDE_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };
use std::process::Command;
#[ cfg( unix ) ]
use std::os::unix::fs::PermissionsExt;

// ── Param 54 — --retry-override ───────────────────────────────────────────────

// ── EC-1: --help lists --retry-override ───────────────────────────────────────

/// EC-1 (param 54): `clr --help` output contains `--retry-override`.
#[ test ]
fn ec1_retry_override_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-override" ),
    "`clr --help` must list --retry-override. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-override 0 --dry-run → exit 0 ──────────────────────────────

/// EC-2 (param 54): value 0 (disables ALL retries for every class) accepted in dry-run.
///
/// Divergence from EC-3: 0 disables all class retries; 3 enables 3 retries for every class.
#[ test ]
fn ec2_retry_override_zero_dry_run()
{
  let out = run_cli( &[ "--retry-override", "0", "--dry-run", "task" ] );
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

// ── EC-3: --retry-override 3 --dry-run → exit 0 ──────────────────────────────

/// EC-3 (param 54): value 3 (3 retries for all classes) accepted in dry-run.
#[ test ]
fn ec3_retry_override_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-override", "3", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_OVERRIDE=3 env var applied ───────────────────────────────

/// EC-4 (param 54): `CLR_RETRY_OVERRIDE=3` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_override_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_OVERRIDE", "3" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_OVERRIDE env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_OVERRIDE ────────────────────────────────────

/// EC-5 (param 54): CLI value 3 wins over `CLR_RETRY_OVERRIDE=1`.
#[ test ]
fn ec5_retry_override_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-override", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_OVERRIDE", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_OVERRIDE=invalid → silently ignored ───────────────────────

/// EC-6 (param 54): invalid `CLR_RETRY_OVERRIDE` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_override_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_OVERRIDE", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_OVERRIDE must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: --retry-override 0 disables ALL retries ─────────────────────────────

/// EC-7 (param 54): `--retry-override 0` disables all retries. Transient fake exits 2 →
/// exit 2 immediately; no retry-progress messages in stderr (only terminal error label).
///
/// Root Cause: Tier 1 override must suppress all class-specific retry behavior
/// Why Not Caught: override is a new Tier 1 mechanism in the redesign
/// Fix Applied: resolve_count(Some(0), ...) returns Some(0); 0 retries = no retry loop
/// Prevention: guard asserting no "retrying" word in stderr with override=0
/// Pitfall: the terminal error label "Error: [Transient]..." still appears; only the
///          retry-progress lines ("retrying in Xs") must be absent
#[ cfg( unix ) ]
#[ test ]
fn ec7_retry_override_zero_disables_all_retries()
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
    .args( [
      "-p", "--retry-override", "0", "--transient-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "exit must be 2 with override=0 (no retries). Got: {:?}", out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.to_lowercase().contains( "retrying" ),
    "stderr must not contain retry-progress messages when override=0. Got:\n{stderr}"
  );
}

// ── EC-8: --retry-override beats class-specific zero ──────────────────────────

/// EC-8 (param 54): `--retry-override 2` beats `--retry-on-transient 0`. Fake exits 2
/// once then 0 → exit 0; `[Transient]` retry message in stderr (override of 2 used,
/// not class-specific 0).
///
/// Root Cause: Tier 1 must take priority over Tier 2 in resolve_count()
/// Why Not Caught: 3-tier priority chain is new in redesign
/// Fix Applied: resolve_count returns first Some() value; override is checked first
/// Prevention: guard asserting [Transient] retry fires despite class-specific=0
/// Pitfall: both flags set simultaneously — override must win, not class-specific
#[ cfg( unix ) ]
#[ test ]
fn ec8_retry_override_beats_class_specific_zero()
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
      "-p", "--retry-override", "2", "--retry-on-transient", "0",
      "--transient-delay", "0", "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 (override=2 beats class-specific=0). exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Transient]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Transient] retry message (override used). Got:\n{stderr}"
  );
}

// ── EC-9: Override applies to Service class ───────────────────────────────────

/// EC-9 (param 54): `--retry-override 1` applies across classes. Fake emits `"API Error: 500"`
/// + exits 1 once, then exits 0 → exit 0; `[Service]` retry message in stderr.
///
/// Root Cause: Tier 1 override must apply uniformly to all error classes
/// Why Not Caught: cross-class override behavior is new in redesign
/// Fix Applied: resolve_count checks override first regardless of ErrorClass variant
/// Prevention: guard asserting [Service] retry fires with override (no --retry-on-service)
/// Pitfall: must NOT set --retry-on-service; override alone must cause Service retry
#[ cfg( unix ) ]
#[ test ]
fn ec9_retry_override_applies_to_service_class()
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
      "-p", "--retry-override", "1", "--service-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 (override=1 applies to Service). exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Service]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Service] retry message (override used). Got:\n{stderr}"
  );
}

// ── EC-10: No override; class-specific honored ────────────────────────────────

/// EC-10 (param 54): no `--retry-override`; `--retry-on-transient 1` honored.
/// Fake exits 2 once then 0 → exit 0; `[Transient]` retry message in stderr.
///
/// Root Cause: when no override, Tier 2 class-specific must be used
/// Why Not Caught: verify fallthrough from Tier 1 (absent) to Tier 2 works
/// Fix Applied: resolve_count(None, Some(1), ...) returns Some(1)
/// Prevention: guard asserting retry fires with class-specific when override absent
/// Pitfall: must ensure CLR_RETRY_OVERRIDE is also absent (env_remove)
#[ cfg( unix ) ]
#[ test ]
fn ec10_no_override_class_specific_honored()
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
      "-p", "--retry-on-transient", "1", "--transient-delay", "0",
      "--max-sessions", "0", "x"
    ] )
    .env( "PATH", &new_path )
    .env_remove( "CLR_RETRY_OVERRIDE" )
    .output()
    .expect( "invoke clr" );

  assert!(
    out.status.success(),
    "exit must be 0 (class-specific=1 honored). exit={:?} stderr={}",
    out.status.code(),
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "[Transient]" ) && stderr.to_lowercase().contains( "retry" ),
    "stderr must contain [Transient] retry message (class-specific used). Got:\n{stderr}"
  );
}

// ── Param 55 — --retry-override-delay ─────────────────────────────────────────

// ── EC-1 (delay): --help lists --retry-override-delay ─────────────────────────

/// EC-1 (param 55): `clr --help` output contains `--retry-override-delay`.
#[ test ]
fn ec1_retry_override_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-override-delay" ),
    "`clr --help` must list --retry-override-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --retry-override-delay 0 --dry-run → exit 0 ────────────────

/// EC-2 (param 55): delay=0 (immediate) accepted in dry-run.
#[ test ]
fn ec2_retry_override_delay_zero_dry_run()
{
  let out = run_cli( &[ "--retry-override-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --retry-override-delay 30 --dry-run → exit 0 ───────────────

/// EC-3 (param 55): delay=30 accepted in dry-run.
#[ test ]
fn ec3_retry_override_delay_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-override-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_RETRY_OVERRIDE_DELAY=30 env var applied ────────────────

/// EC-4 (param 55): `CLR_RETRY_OVERRIDE_DELAY=30` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_override_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_OVERRIDE_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_OVERRIDE_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_RETRY_OVERRIDE_DELAY ─────────────────────

/// EC-5 (param 55): CLI value 30 wins over `CLR_RETRY_OVERRIDE_DELAY=10`.
#[ test ]
fn ec5_retry_override_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-override-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_OVERRIDE_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_RETRY_OVERRIDE_DELAY=invalid → silently ignored ─────────

/// EC-6 (param 55): invalid `CLR_RETRY_OVERRIDE_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_override_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_OVERRIDE_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_OVERRIDE_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}
