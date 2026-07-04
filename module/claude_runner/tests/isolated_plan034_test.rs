//! Integration tests for `clr isolated` subcommand — Plan 034.
//!
//! # Test Matrix
//!
//! | ID | Test | Requires Live Claude |
//! |----|------|----------------------|
//! | IT-12 | `--dry-run` exits 0 without spawning subprocess | No |
//! | IT-13 | `--dry-run "msg"` exits 0, preview includes `--print` | No |
//! | IT-14 | `--dry-run --dir /tmp` exits 0, preview includes `--dir` | No |
//! | IT-15 | `--dry-run --add-dir /tmp` exits 0, preview includes `--add-dir` | No |
//! | IT-16 | `--dir /tmp` injected into subprocess argv | No (fake claude) |
//! | IT-17 | `--dir /nonexistent` exits 1 before subprocess spawn | No |
//! | IT-18 | `--add-dir /tmp` injected into subprocess argv | No (fake claude) |
//! | IT-19 | `--dir /tmp --add-dir /var` — both flags in subprocess argv | No (fake claude) |
//! | IT-20 | `CLR_DIR=/tmp` env var fallback when `--dir` absent | No |
//! | IT-21 | `--file <path>` pipes file content to subprocess stdin | No (fake claude) |
//! | IT-22 | `--file /nonexistent` exits 1 before subprocess spawn | No |
//! | IT-23 | `--file <path> "msg"` file stdin combined with message | No (fake claude) |
//! | IT-24 | `--expect "hello"` match → exit 0, output unchanged | No (fake claude) |
//! | IT-25 | `--expect "hello" --expect-strategy fail` mismatch → exit 3 | No (fake claude) |
//! | IT-26 | `--expect "hello" --expect-strategy default:no` mismatch → exit 0 | No (fake claude) |
//! | IT-27 | `--expect-strategy retry` unsupported for isolated → exit 1 | No (fake claude) |
//! | IT-28 | `--file` with >64 KiB stdout no deadlock | No (fake claude) |

#![ cfg( feature = "enabled" ) ]

use std::io::Write as _;
use tempfile::NamedTempFile;

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, make_creds_file, run_isolated, run_with_path, stderr_str, stdout_str };
#[ cfg( unix ) ]
use cli_binary_test_helpers::fake_claude_dir;

// ── IT-12 through IT-15: --dry-run tests (offline) ──────────────────────────

/// IT-12: `--dry-run` exits 0 without spawning a subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-12
#[ test ]
fn it12_dry_run_exits_zero()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0 from --dry-run; stderr: {}", stderr_str( &out )
  );
  assert!(
    !stdout_str( &out ).is_empty(),
    "--dry-run must print command preview to stdout"
  );
}

/// IT-13: `--dry-run "msg"` exits 0 and preview contains `--print` with the message.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-13
#[ test ]
fn it13_dry_run_includes_message()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "say hello" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--print" ),
    "--dry-run with message must include --print in preview; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "say hello" ),
    "--dry-run with message must include message text in preview; got:\n{stdout}"
  );
}

/// IT-14: `--dry-run --dir /tmp` exits 0 and preview includes `--dir`.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-14
#[ test ]
fn it14_dry_run_includes_dir()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--dir", "/tmp", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--dir" ),
    "--dry-run with --dir must include --dir in preview; got:\n{}", stdout_str( &out )
  );
}

/// IT-15: `--dry-run --add-dir /tmp` exits 0 and preview includes `--add-dir`.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-15
#[ test ]
fn it15_dry_run_includes_add_dir()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated( &[ "--creds", path, "--dry-run", "--add-dir", "/tmp", "msg" ] );
  assert_eq!(
    exit_code( &out ), 0,
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--add-dir" ),
    "--dry-run with --add-dir must include --add-dir in preview; got:\n{}", stdout_str( &out )
  );
}

// ── IT-16 through IT-20: --dir and --add-dir tests ──────────────────────────

/// IT-16: `--dir /tmp` is injected into the subprocess argv.
///
/// Fake claude echoes all args to stdout so the test can verify `--dir /tmp` arrives.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-16
#[ cfg( unix ) ]
#[ test ]
fn it16_dir_injected_into_subprocess()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo \"$@\"" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--dir", "/tmp", "msg" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "--dir" ),
    "--dir must be injected into subprocess argv; got:\n{}", stdout_str( &out )
  );
}

/// IT-17: `--dir /nonexistent_clr_test_dir_it17` exits 1 before subprocess spawn.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-17
#[ test ]
fn it17_dir_nonexistent_exits_one()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();
  let out   = run_isolated(
    &[ "--creds", path, "--dir", "/nonexistent_clr_test_dir_it17", "msg" ],
  );
  assert_eq!(
    exit_code( &out ), 1,
    "expected exit 1 for nonexistent --dir; stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "does not exist" ) || err.contains( "not found" ),
    "stderr must indicate nonexistent dir; got:\n{err}"
  );
}

/// IT-18: `--add-dir /tmp` is injected into the subprocess argv.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-18
#[ cfg( unix ) ]
#[ test ]
fn it18_add_dir_injected_into_subprocess()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo \"$@\"" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--add-dir", "/tmp", "msg" ],
    &path,
  );
  assert!(
    stdout_str( &out ).contains( "--add-dir" ),
    "--add-dir must be injected into subprocess argv; got:\n{}", stdout_str( &out )
  );
}

/// IT-19: `--dir /tmp --add-dir /var` — both flags appear in subprocess argv.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-19
#[ cfg( unix ) ]
#[ test ]
fn it19_dir_and_add_dir_combined()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo \"$@\"" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--dir", "/tmp", "--add-dir", "/var", "msg" ],
    &path,
  );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--dir" ) && stdout.contains( "--add-dir" ),
    "both --dir and --add-dir must appear in subprocess argv; got:\n{stdout}"
  );
}

/// IT-20: `CLR_DIR=/tmp` env var is applied when `--dir` flag is absent.
///
/// Uses `--dry-run` to verify env var pickup without spawning a subprocess.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-20
#[ test ]
fn it20_clr_dir_env_fallback()
{
  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().unwrap();
  let out        = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", creds_path, "--dry-run", "msg" ] )
    .env( "CLR_DIR", "/tmp" )
    .env_remove( "CLR_ADD_DIR" )
    .output()
    .expect( "invoke clr isolated" );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "expected exit 0; stderr: {}", String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--dir" ),
    "CLR_DIR must appear in --dry-run preview; got:\n{stdout}"
  );
}

// ── IT-21 through IT-23: --file tests ──────────────────────────────────────

/// IT-21: `--file <path>` pipes file content to the subprocess stdin.
///
/// Fake claude runs `cat` to read stdin and emit it to stdout, proving file content arrived.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-21
#[ cfg( unix ) ]
#[ test ]
fn it21_file_piped_as_stdin()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let mut input_file = NamedTempFile::new().expect( "create input file" );
  input_file.write_all( b"file_content_it21" ).expect( "write input file" );
  let input_path     = input_file.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "cat" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--file", input_path, "process this" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "file_content_it21" ),
    "--file must pipe file content to subprocess stdin; got:\n{}", stdout_str( &out )
  );
}

/// IT-22: `--file /nonexistent_clr_it22.txt` exits 1 before subprocess spawn.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-22
#[ test ]
fn it22_file_nonexistent_exits_one()
{
  let out = run_isolated( &[
    "--creds", "/tmp/clr_it22_dummy.json",
    "--file",  "/tmp/clr_it22_nonexistent_input.txt",
    "msg",
  ] );
  assert_eq!(
    exit_code( &out ), 1,
    "expected exit 1 for nonexistent --file; stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "does not exist" ) || err.contains( "not found" ),
    "stderr must indicate nonexistent file; got:\n{err}"
  );
}

/// IT-23: `--file <path> "msg"` — file as stdin combined with prompt message.
///
/// File content arrives via subprocess stdin; message is forwarded via `--print`.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-23
#[ cfg( unix ) ]
#[ test ]
fn it23_file_combined_with_message()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let mut input_file = NamedTempFile::new().expect( "create input file" );
  input_file.write_all( b"combined_input_it23" ).expect( "write input file" );
  let input_path     = input_file.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "cat" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--file", input_path, "process this file" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "combined_input_it23" ),
    "--file + message: file content must reach subprocess stdin; got:\n{}", stdout_str( &out )
  );
}

// ── IT-24 through IT-27: --expect and --expect-strategy tests ───────────────

/// IT-24: `--expect "hello"` — output matches → exit 0, output unchanged.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-24
#[ cfg( unix ) ]
#[ test ]
fn it24_expect_match_exits_zero()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'hello'" );
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--expect", "hello", "msg" ],
    &path,
  );
  assert!(
    out.status.success(),
    "expect match must exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "hello" ),
    "output must be unchanged on match; got:\n{}", stdout_str( &out )
  );
}

/// IT-25: `--expect "hello" --expect-strategy fail` — mismatch → exit 3.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-25
#[ cfg( unix ) ]
#[ test ]
fn it25_expect_mismatch_fail_exits_three()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'world'" );
  let out            = run_with_path(
    &[
      "isolated", "--creds", creds_path,
      "--expect", "hello", "--expect-strategy", "fail", "msg",
    ],
    &path,
  );
  assert_eq!(
    exit_code( &out ), 3,
    "expect mismatch with fail strategy must exit 3; stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "expected" ) || err.contains( "Validation" ),
    "stderr must indicate expect mismatch; got:\n{err}"
  );
}

/// IT-26: `--expect "hello" --expect-strategy default:no` — mismatch → exit 0, stdout "no".
///
/// Source: tests/docs/cli/command/03_isolated.md#it-26
#[ cfg( unix ) ]
#[ test ]
fn it26_expect_mismatch_default_fallback()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'world'" );
  let out            = run_with_path(
    &[
      "isolated", "--creds", creds_path,
      "--expect", "hello", "--expect-strategy", "default:no", "msg",
    ],
    &path,
  );
  assert!(
    out.status.success(),
    "default strategy must exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).contains( "no" ),
    "default strategy must replace output with fallback 'no'; got:\n{}", stdout_str( &out )
  );
}

/// IT-27: `--expect-strategy retry` is explicitly unsupported for isolated → exit 1.
///
/// Source: tests/docs/cli/command/03_isolated.md#it-27
#[ cfg( unix ) ]
#[ test ]
fn it27_expect_strategy_retry_unsupported()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let ( _dir, path ) = fake_claude_dir( "echo 'world'" );
  let out            = run_with_path(
    &[
      "isolated", "--creds", creds_path,
      "--expect", "hello", "--expect-strategy", "retry", "msg",
    ],
    &path,
  );
  assert_eq!(
    exit_code( &out ), 1,
    "retry strategy must exit 1 (unsupported); stderr: {}", stderr_str( &out )
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "retry" ),
    "stderr must mention 'retry' as unsupported; got:\n{err}"
  );
}

/// IT-28: `--file` with subprocess producing >64 KiB stdout completes without deadlock.
///
/// Without background reader threads draining the pipes, a subprocess writing more than
/// the Linux kernel pipe buffer (64 KiB) blocks on write. `try_wait()` polling never
/// returns `Some(_)` because the subprocess is stuck, causing the operation to hang
/// until timeout fires. This test proves the reader-thread fix prevents the deadlock.
///
/// The fake claude generates 100,000 bytes of 'B' characters via `head -c` + `tr`,
/// well above the 64 KiB threshold. With the fix, `try_wait()` returns promptly because
/// reader threads drain the pipe continuously.
#[ cfg( unix ) ]
#[ test ]
fn it28_file_large_stdout_no_deadlock()
{
  let creds          = make_creds_file( "{}" );
  let creds_path     = creds.path().to_str().unwrap();
  let mut input_file = NamedTempFile::new().expect( "create input file" );
  input_file.write_all( b"input_it28" ).expect( "write input file" );
  let input_path     = input_file.path().to_str().unwrap();
  // Fake claude emits 100_000 bytes — exceeds 64 KiB pipe buffer.
  let ( _dir, path ) = fake_claude_dir( "head -c 100000 < /dev/zero | tr '\\0' 'B'" );
  let start          = std::time::Instant::now();
  let out            = run_with_path(
    &[ "isolated", "--creds", creds_path, "--file", input_path, "--timeout", "30", "msg" ],
    &path,
  );
  let elapsed = start.elapsed();
  assert!(
    out.status.success(),
    "expected exit 0; stderr: {}", stderr_str( &out )
  );
  assert!(
    stdout_str( &out ).len() >= 100_000,
    "stdout must contain >=100000 bytes; got {} bytes", stdout_str( &out ).len()
  );
  assert!(
    elapsed.as_secs() < 15,
    "must complete promptly (no pipe deadlock); elapsed {}s suggests hang", elapsed.as_secs()
  );
}
