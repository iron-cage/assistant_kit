//! Integration tests for `clr kill` — the session termination command.
//!
//! Test spec: [`tests/docs/cli/command/07_kill.md`](docs/cli/command/07_kill.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                              | Category         |
//! |------|---------------------------------------------------|------------------|
//! | IT-1 | No PID → exit 1, "missing PID" on stderr          | Missing PID      |
//! | IT-2 | Non-numeric PID → exit 1, "invalid PID" on stderr | Invalid PID      |
//! | IT-3 | PID not a Claude process → exit 1, not-session    | Not Claude       |
//! | IT-4 | Valid running claude PID → exit 0, "Sent SIGTERM" | Successful kill  |
//! | IT-5 | `clr kill --help` → exit 0, help text             | Help flag        |
//! | IT-6 | `clr kill -h` → exit 0, help text                 | Help short flag  |
//! | IT-7 | `clr --help` lists `kill` subcommand              | Help listing     |
//! | IT-8 | `clr kil` (typo) → exit 1, "Did you mean"         | Typo guard       |
//! | IT-9 | `clr kill 1234 extra` → exit 1, unexpected arg    | Extra argument   |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, make_proc_dir, spawn_fake_claude };

// ── IT-1: No PID ─────────────────────────────────────────────────────────────

/// IT-1: `clr kill` with no arguments → exit 1, stderr mentions "missing PID".
#[ test ]
fn it_01_no_pid_argument()
{
  let out    = run_cli( &[ "kill" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "missing PID" ),
    "stderr must mention 'missing PID', got: {stderr}"
  );
}

// ── IT-2: Non-numeric PID ────────────────────────────────────────────────────

/// IT-2: `clr kill abc` with a non-numeric PID → exit 1, stderr mentions "invalid PID".
#[ test ]
fn it_02_non_numeric_pid()
{
  let out    = run_cli( &[ "kill", "abc" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "invalid PID" ),
    "stderr must mention 'invalid PID', got: {stderr}"
  );
}

// ── IT-3: PID not a Claude process ───────────────────────────────────────────

/// IT-3: `clr kill 999999` with a PID that is certainly not a running `claude`
/// process → exit 1, stderr mentions the PID and "not a running Claude Code session".
#[ cfg( unix ) ]
#[ test ]
fn it_03_pid_not_a_claude_process()
{
  let proc     = make_proc_dir( &[] );
  let proc_dir = proc.path().to_str().expect( "proc dir UTF-8" );
  let out    = run_cli_with_env( &[ "kill", "999999" ], &[ ( "CLR_PROC_DIR", proc_dir ) ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "999999" ),
    "stderr must include the rejected PID, got: {stderr}"
  );
  assert!(
    stderr.contains( "not a running Claude Code session" ),
    "stderr must explain the PID is not a Claude session, got: {stderr}"
  );
}

// ── IT-4: Successful kill ─────────────────────────────────────────────────────

/// IT-4: with a fake `claude` process running, `clr kill <pid>` exits 0 and
/// stdout contains "Sent SIGTERM".
///
/// Uses `fake_claude_binary_dir()` to create a real ELF binary named `claude`
/// (a copy of `/bin/sleep`) so it appears in `/proc/{pid}/cmdline` as `claude`
/// and is therefore visible to `find_claude_processes()`.
#[ cfg( unix ) ]
#[ test ]
fn it_04_successful_sigterm_delivery()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let pid = bg.id();
  let proc = make_proc_dir( &[ pid ] );

  let bin    = env!( "CARGO_BIN_EXE_clr" );
  let result = std::process::Command::new( bin )
    .args( [ "kill", &pid.to_string() ] )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output();
  // Always reap the child before unwrapping the result to avoid zombies.
  let _ = bg.kill();
  let _ = bg.wait();
  let out = result.expect( "run clr kill" );

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}: {}", out.status.code(), stderr_str( &out ) );
  assert!(
    stdout.contains( "Sent SIGTERM" ),
    "stdout must contain 'Sent SIGTERM', got: {stdout}"
  );
  assert!(
    stdout.contains( &pid.to_string() ),
    "stdout must include the terminated PID, got: {stdout}"
  );
}

// ── IT-5: --help flag ─────────────────────────────────────────────────────────

/// IT-5: `clr kill --help` → exit 0, stdout contains help text with "SIGTERM" and "<PID>".
#[ test ]
fn it_05_help_flag()
{
  let out    = run_cli( &[ "kill", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "SIGTERM" ), "help must mention SIGTERM: {stdout}" );
  assert!( stdout.contains( "<PID>" ), "help must mention <PID>: {stdout}" );
}

// ── IT-6: -h short flag ───────────────────────────────────────────────────────

/// IT-6: `clr kill -h` → exit 0, stdout contains help text.
#[ test ]
fn it_06_help_short_flag()
{
  let out    = run_cli( &[ "kill", "-h" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "SIGTERM" ), "help must mention SIGTERM: {stdout}" );
}

// ── IT-7: clr --help lists kill ───────────────────────────────────────────────

/// IT-7: `clr --help` includes `kill` in the subcommands list.
#[ test ]
fn it_07_help_lists_kill()
{
  let out    = run_cli( &[ "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "kill" ),
    "help output must mention 'kill' subcommand, got: {stdout}"
  );
}

// ── IT-8: typo guard `clr kil` ───────────────────────────────────────────────

/// IT-8: `clr kil` (one-character truncation) → exit 1, stderr: "Did you mean 'kill'?".
#[ test ]
fn it_08_typo_clr_kil()
{
  let out    = run_cli( &[ "kil" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "Did you mean" ),
    "stderr must contain 'Did you mean', got: {stderr}"
  );
}

// ── IT-9: extra argument ─────────────────────────────────────────────────────

/// IT-9: `clr kill 1234 extra` → exit 1, stderr: "unexpected argument".
#[ test ]
fn it_09_extra_argument()
{
  let out    = run_cli( &[ "kill", "1234", "extra" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "unexpected argument" ),
    "stderr must contain 'unexpected argument', got: {stderr}"
  );
}
