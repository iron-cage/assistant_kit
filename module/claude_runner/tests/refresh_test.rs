#![ cfg( feature = "enabled" ) ]
//! Integration tests for `clr refresh` subcommand.
//!
//! # Test Matrix
//!
//! | ID | Test | Requires Live Claude |
//! |----|------|----------------------|
//! | IT-2 | `--creds missing.json` → exit 1 | No |
//! | IT-4 | `--timeout 0` → unlimited (no watchdog), exit 0 | No (Unix) |
//! | IT-6 | `--creds <f> --timeout abc` → exit 1, invalid timeout | No |
//! | IT-8 | `clr refresh --help` → exit 0, help text shown | No |
//! | IT-9 | `CLR_JOURNAL=bogus` → exit 1 with error message | No |
//! | IT-10 | `clr refresh --creds <f> "message"` → exit 1, positional arg rejected | No |
//!
//! Tests containing `lim_it` (not present in this file) would require a live
//! OAuth-capable `claude` binary.  All tests here run without live credentials.
//!
//! Source: `tests/docs/cli/command/04_refresh.md`

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, make_creds_file, stderr_str, stdout_str };

// ── helpers ──────────────────────────────────────────────────────────────────

/// Invoke `clr refresh <args>` and return raw output.
///
/// Delegates to the shared `cli_binary_test_helpers::run_cli` binary helper, prepending
/// the `"refresh"` subcommand to the caller-supplied arguments.
fn run_refresh( args : &[ &str ] ) -> std::process::Output
{
  let mut full = vec![ "refresh" ];
  full.extend_from_slice( args );
  cli_binary_test_helpers::run_cli( &full )
}

// ── offline tests (no live claude required) ───────────────────────────────────

/// IT-2: creds file that does not exist → exit 1 with file-not-found message.
///
/// Unix-only: uses `/tmp/` path and checks unix-style error messages.
///
/// Source: tests/docs/cli/command/04_refresh.md#it-2
#[ cfg( unix ) ]
#[ test ]
fn test_it2_creds_file_not_found()
{
  let out = run_refresh( &[ "--creds", "/tmp/clr_refresh_nonexistent_it2.json" ] );
  assert_eq!( exit_code( &out ), 1, "expected exit 1; stderr: {}", stderr_str( &out ) );
  let err = stderr_str( &out );
  assert!(
    err.contains( "not found" ) || err.contains( "No such file" ) || err.contains( "cannot read" ),
    "expected file-not-found message; got: {err}"
  );
}

/// IT-4: `--timeout 0` → unlimited (no watchdog); subprocess runs to natural exit.
///
/// Creates a fake `claude` shell script that sleeps briefly.  With `--timeout 0`,
/// no watchdog is spawned — the subprocess runs until it exits naturally and
/// `clr refresh` forwards the subprocess exit code (0).
///
/// Source: tests/docs/cli/command/04_refresh.md#it-4
#[ cfg( unix ) ]
#[ test ]
fn test_it4_timeout_zero_unlimited()
{
  use std::os::unix::fs::PermissionsExt;

  let dir = tempfile::tempdir().expect( "tmpdir" );
  let script = dir.path().join( "claude" );
  // timeout-0 = unlimited: subprocess runs to completion — use short sleep to keep test fast.
  std::fs::write( &script, "#!/bin/sh\nsleep 1\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &script, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );
  let path_val = format!(
    "{}:{}",
    dir.path().display(),
    std::env::var( "PATH" ).unwrap_or_default(),
  );

  let creds = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().expect( "temp path is valid UTF-8" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "refresh", "--creds", creds_path, "--timeout", "0" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "invoke clr refresh" );

  assert_eq!(
    exit_code( &out ),
    0,
    "IT-4: --timeout 0 must let subprocess run to completion (unlimited); stderr: {}",
    stderr_str( &out ),
  );
}

/// IT-6: `--timeout abc` → exit 1, invalid `--timeout` error.
///
/// Source: tests/docs/cli/command/04_refresh.md#it-6
#[ test ]
fn test_it6_invalid_timeout()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out   = run_refresh( &[ "--creds", path, "--timeout", "abc" ] );
  assert_eq!( exit_code( &out ), 1, "expected exit 1; stderr: {}", stderr_str( &out ) );
  assert!(
    stderr_str( &out ).contains( "invalid --timeout" ),
    "expected 'invalid --timeout' message; got: {}", stderr_str( &out )
  );
}

/// IT-8: `clr refresh --help` exits 0 and prints refresh-specific help text.
///
/// Source: tests/docs/cli/command/04_refresh.md#it-8
#[ test ]
fn test_it8_help_exits_zero()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "refresh", "--help" ] )
    .output()
    .expect( "failed to invoke clr refresh --help" );
  assert_eq!(
    exit_code( &out ),
    0,
    "clr refresh --help must exit 0; got: {:?}\nstderr: {}",
    out.status.code(),
    stderr_str( &out ),
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--creds" ),
    "help text must mention --creds; got:\n{stdout}",
  );
  assert!(
    stdout.contains( "--timeout" ),
    "help text must mention --timeout; got:\n{stdout}",
  );
  assert!(
    stdout.contains( "--trace" ),
    "help text must mention --trace; got:\n{stdout}",
  );
  assert!(
    stdout.contains( "--help" ),
    "help text must mention --help; got:\n{stdout}",
  );
}

// ── IT-9: CLR_JOURNAL=invalid → exit 1 ───────────────────────────────────────

/// IT-9: `CLR_JOURNAL=bogus` env var with `clr refresh` exits 1 and names the
/// invalid value in the error message.
///
/// ## Root Cause
///
/// `apply_refresh_env_vars()` applied `CLR_JOURNAL` directly via `env_str()` without
/// validation — an invalid value was silently accepted, unlike the `run`/`ask` path.
///
/// ## Why Not Caught
///
/// No test asserted that `CLR_JOURNAL=bogus` would be rejected by the refresh path.
///
/// ## Fix Applied
///
/// `apply_refresh_env_vars()` now validates `CLR_JOURNAL` against `"full" | "meta" | "off"`
/// and returns `Err` with message `"CLR_JOURNAL: invalid value '…'"`.
///
/// ## Prevention
///
/// Assert `CLR_JOURNAL=bogus clr refresh …` exits 1 and names the env var in stderr.
///
/// ## Pitfall
///
/// The `--creds` flag must point to a readable file so the env var error is the first
/// exit point (fired before the creds-path empty-string check).
///
/// Source: tests/docs/cli/command/04_refresh.md#it-9
#[ test ]
fn test_it9_clr_journal_invalid_value_exits_1()
{
  let creds   = make_creds_file( "{}" );
  let creds_s = creds.path().to_str().expect( "utf-8" );
  let bin     = env!( "CARGO_BIN_EXE_clr" );
  let out     = std::process::Command::new( bin )
    .args( [ "refresh", "--creds", creds_s ] )
    .env( "CLR_JOURNAL", "bogus" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .output()
    .expect( "failed to invoke clr refresh" );
  assert_eq!(
    exit_code( &out ),
    1,
    "CLR_JOURNAL=bogus must cause refresh to exit 1. Got: {:?}\nstderr: {}",
    out.status.code(),
    stderr_str( &out ),
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "CLR_JOURNAL" ),
    "error must mention CLR_JOURNAL. Got:\n{stderr}"
  );
  assert!(
    stderr.to_lowercase().contains( "invalid" ),
    "error must describe the value as invalid. Got:\n{stderr}"
  );
}

// ── IT-10: positional MESSAGE rejected ───────────────────────────────────────

/// IT-10: `clr refresh --creds <f> "Fix the bug"` exits 1 — positional MESSAGE rejected.
///
/// Parity PC-5: `refresh` has no `MESSAGE` parameter — it always uses a hardcoded `"."`
/// prompt. Passing a positional argument must be rejected at parse time, not silently
/// ignored. Before this fix, `parse_refresh_args()` had a wildcard arm `_ => {}` that
/// silently discarded any unrecognised token.
///
/// Source: tests/docs/cli/parity/02_isolated_refresh.md#pc-5
#[ test ]
fn test_it10_refresh_rejects_positional_message()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_refresh( &[ "--creds", path, "Fix the bug" ] );
  assert_eq!(
    exit_code( &out ),
    1,
    "refresh must reject a positional MESSAGE argument (exit 1); got {:?}; stderr: {}",
    out.status.code(),
    stderr_str( &out ),
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "unexpected argument" ) || err.contains( "positional" ),
    "stderr must mention unexpected/positional argument; got: {err}"
  );
}
