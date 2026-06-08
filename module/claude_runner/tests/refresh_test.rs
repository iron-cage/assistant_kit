#![ cfg( feature = "enabled" ) ]
//! Integration tests for `clr refresh` subcommand.
//!
//! # Test Matrix
//!
//! | ID | Test | Requires Live Claude |
//! |----|------|----------------------|
//! | IT-2 | `--creds missing.json` → exit 1 | No |
//! | IT-4 | `--timeout 0` with fake sleeping claude → exit 2 | No (Unix) |
//! | IT-6 | `--creds <f> --timeout abc` → exit 1, invalid timeout | No |
//! | IT-8 | `clr refresh --help` → exit 0, help text shown | No |
//!
//! Tests containing `lim_it` (not present in this file) would require a live
//! OAuth-capable `claude` binary.  All tests here run without live credentials.
//!
//! Source: `tests/docs/cli/command/04_refresh.md`

use std::io::Write as _;
use tempfile::NamedTempFile;

mod cli_binary_test_helpers;

// ── helpers ──────────────────────────────────────────────────────────────────

fn exit_code( o : &std::process::Output ) -> i32 { o.status.code().unwrap_or( -1 ) }
fn stderr_str( o : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &o.stderr ).to_string()
}
fn stdout_str( o : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &o.stdout ).to_string()
}

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

/// Write `content` to a new `NamedTempFile` and return it.
///
/// The caller must keep the returned file alive for the duration of the test;
/// dropping it deletes the file on disk.
fn make_creds_file( content : &str ) -> NamedTempFile
{
  let mut f = NamedTempFile::new().expect( "failed to create temp creds file" );
  f.write_all( content.as_bytes() ).expect( "failed to write creds content" );
  f
}

// ── offline tests (no live claude required) ───────────────────────────────────

/// IT-2: creds file that does not exist → exit 1 with file-not-found message.
///
/// Source: tests/docs/cli/command/04_refresh.md#it-2
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

/// IT-4: `--timeout 0` with fake sleeping claude → not exit 2 (watchdog disabled, unlimited).
///
/// Creates a fake `claude` shell script that sleeps briefly then exits.  With `--timeout 0`,
/// the watchdog is disabled entirely (deadline = None); the subprocess runs to completion
/// and `clr refresh` exits with the subprocess exit code — never with 2 (timeout).
///
/// Source: tests/docs/cli/command/04_refresh.md#it-4
#[ cfg( unix ) ]
#[ test ]
fn test_it4_timeout_zero_exits_two()
{
  use std::os::unix::fs::PermissionsExt;

  let dir = tempfile::tempdir().expect( "tmpdir" );
  let script = dir.path().join( "claude" );
  // Short sleep (0.3s) so the test completes quickly; the subprocess exits naturally.
  std::fs::write( &script, "#!/bin/sh\nsleep 0.3\nexit 0\n" ).expect( "write fake claude" );
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

  assert_ne!(
    exit_code( &out ),
    2,
    "IT-4: --timeout 0 must disable watchdog (unlimited) — exit 2 means timeout fired; stderr: {}",
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
}
