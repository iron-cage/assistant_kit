//! User-story-level integration tests for `clr kill` (Session Termination).
//!
//! Test spec: [`tests/docs/cli/user_story/27_session_termination.md`](docs/cli/user_story/27_session_termination.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                              | AC     |
//! |------|---------------------------------------------------|--------|
//! | US-1 | Successful kill: exit 0, "Sent SIGTERM" message   | AC-001 |
//! | US-2 | Non-Claude PID: exit 1, not-a-session error       | AC-002 |
//! | US-3 | No PID: exit 1, missing PID error                 | AC-003 |
//! | US-4 | `clr kill --help` shows usage with SIGTERM        | AC-004 |
//! | US-5 | `clr --help` lists `kill` subcommand              | AC-005 |
//! | US-6 | Typo `clr kil` triggers guard                     | AC-006 |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, make_proc_dir, spawn_fake_claude };

// ── US-1: Successful kill ─────────────────────────────────────────────────────

/// US-1 (AC-001): `clr kill <PID>` with a valid running Claude process PID
/// exits 0 and prints `"Sent SIGTERM to Claude Code session <PID>."`.
#[ cfg( unix ) ]
#[ test ]
fn us_01_successful_kill()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg  = spawn_fake_claude( &path_val );
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
    "US-1: stdout must confirm SIGTERM delivery. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &pid.to_string() ),
    "US-1: stdout must include the terminated PID. Got:\n{stdout}"
  );
}

// ── US-2: Non-Claude PID rejected ────────────────────────────────────────────

/// US-2 (AC-002): `clr kill 999999` with a PID that is not a running Claude
/// process exits 1 and stderr identifies the PID and explains it is not a
/// Claude session.
#[ cfg( unix ) ]
#[ test ]
fn us_02_non_claude_pid_rejected()
{
  let proc     = make_proc_dir( &[] );
  let proc_dir = proc.path().to_str().expect( "proc dir UTF-8" );
  let out    = run_cli_with_env( &[ "kill", "999999" ], &[ ( "CLR_PROC_DIR", proc_dir ) ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "999999" ),
    "US-2: stderr must include the rejected PID. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "not a running Claude Code session" ),
    "US-2: stderr must explain PID is not a Claude session. Got:\n{stderr}"
  );
}

// ── US-3: Missing PID ────────────────────────────────────────────────────────

/// US-3 (AC-003): `clr kill` with no argument exits 1 and stderr contains
/// `"missing PID"`.
#[ test ]
fn us_03_missing_pid()
{
  let out    = run_cli( &[ "kill" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "missing PID" ),
    "US-3: stderr must mention 'missing PID'. Got:\n{stderr}"
  );
}

// ── US-4: kill --help ─────────────────────────────────────────────────────────

/// US-4 (AC-004): `clr kill --help` exits 0 and help text includes "SIGTERM"
/// and "<PID>".
#[ test ]
fn us_04_kill_help()
{
  let out    = run_cli( &[ "kill", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected" );
  assert!( stdout.contains( "SIGTERM" ), "US-4: help must mention SIGTERM. Got:\n{stdout}" );
  assert!( stdout.contains( "<PID>" ), "US-4: help must mention <PID>. Got:\n{stdout}" );
}

// ── US-5: Help lists kill ─────────────────────────────────────────────────────

/// US-5 (AC-005): `clr --help` includes `kill` in the subcommand list.
#[ test ]
fn us_05_help_lists_kill()
{
  let out    = run_cli( &[ "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "kill" ), "US-5: help must list 'kill'. Got:\n{stdout}" );
}

// ── US-6: Typo guard ──────────────────────────────────────────────────────────

/// US-6 (AC-006): `clr kil` (truncation typo) → exit 1, stderr: "Did you mean".
#[ test ]
fn us_06_typo_guard()
{
  let out    = run_cli( &[ "kil" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "Did you mean" ),
    "US-6: stderr must contain 'Did you mean'. Got:\n{stderr}"
  );
}
