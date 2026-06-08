//! Isolated/Refresh Subprocess Correctness Tests
//!
//! ## Purpose
//!
//! Verify Task 022: five correctness gaps in isolated/refresh are fixed (S2-S6).
//!
//! ## Test Matrix
//!
//! | ID | Scenario | Requires Live Claude |
//! |----|----------|---------------------|
//! | CT-1 | isolated trace contains --no-session-persistence | No |
//! | CT-2 | isolated trace contains --dangerously-skip-permissions when message present | No |
//! | CT-3 | refresh trace contains --no-chrome | No |
//! | CT-4 | isolated trace without message: --dangerously-skip-permissions absent | No |
//! | CT-5 | --timeout 0 (unlimited): fake subprocess NOT killed within 2s | No (fake binary) |
//! | CT-6 | CLAUDE.md written to temp HOME before subprocess spawn | No (fake binary) |

#![ cfg( feature = "enabled" ) ]
#![ cfg( unix ) ]
#![ allow( clippy::doc_markdown ) ]

use std::os::unix::fs::PermissionsExt;

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ make_creds_file, stderr_str, stdout_str };

// ── CT-1 / CT-2 / CT-3 / CT-4: flag injection trace checks ──────────────────

/// CT-1: `clr isolated --trace "x"` → stderr contains `--no-session-persistence`.
///
/// Root Cause: `run_isolated_command()` did not inject --no-session-persistence;
///   session files were written to temp HOME that is discarded after every run
///   (pure I/O waste per gap I3 in command_defaults.md).
/// Why Not Caught: no test for injected flags in isolated trace.
/// Fix Applied: Task 022 S3 prepends --no-session-persistence in `run_isolated_command()`.
/// Prevention: this test; trace checked for flag presence.
/// Pitfall: injecting after --print instead of before breaks passthrough override order.
#[ test ]
fn ct1_isolated_trace_has_no_session_persistence()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", path, "--trace", "x" ] )
    .env( "PATH", "/nonexistent" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  let err = stderr_str( &out );
  assert!(
    err.contains( "--no-session-persistence" ),
    "trace must contain --no-session-persistence; got:\n{err}"
  );
}

/// CT-2: `clr isolated --trace "x"` → stderr contains `--dangerously-skip-permissions`.
///
/// Root Cause: `run_isolated_command()` did not inject --dangerously-skip-permissions;
///   isolated tasks with tool use blocked at every tool call waiting for interactive
///   permission prompt (gap I5 in command_defaults.md).
/// Why Not Caught: no test for injected flags; live execution blocks silently.
/// Fix Applied: Task 022 S5 injects --dangerously-skip-permissions when message present.
/// Prevention: this test; trace checked for flag when message is non-empty.
/// Pitfall: injecting unconditionally (even without a message) would affect no-message
///   interactive mode; S5 condition is message.is_some().
#[ test ]
fn ct2_isolated_trace_has_skip_permissions_when_message_present()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", path, "--trace", "x" ] )
    .env( "PATH", "/nonexistent" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  let err = stderr_str( &out );
  assert!(
    err.contains( "--dangerously-skip-permissions" ),
    "trace must contain --dangerously-skip-permissions when message present; got:\n{err}"
  );
}

/// CT-3: `clr refresh --trace` → stderr contains `--no-chrome`.
///
/// Root Cause: `run_refresh_command()` used `ClaudeCommand::new()` defaults which include
///   --chrome; refresh is an HTTP-only OAuth ping and does not need browser context
///   (gap I4 in command_defaults.md).
/// Why Not Caught: no trace test for refresh flag injection.
/// Fix Applied: Task 022 S4 adds "--no-chrome" to refresh passthrough args.
/// Prevention: this test; trace checked for --no-chrome in refresh.
/// Pitfall: --chrome is injected by `ClaudeCommand::new()`; --no-chrome must appear
///   after --chrome in the arg list so last-wins semantics apply correctly.
#[ test ]
fn ct3_refresh_trace_has_no_chrome()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "refresh", "--creds", path, "--trace" ] )
    .env( "PATH", "/nonexistent" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr refresh" );

  let err = stderr_str( &out );
  assert!(
    err.contains( "--no-chrome" ),
    "refresh trace must contain --no-chrome; got:\n{err}"
  );
}

/// CT-4: `clr isolated --trace` without a message → `--dangerously-skip-permissions` absent.
///
/// Root Cause: not a bug — this tests the S5 guard condition (message.is_some()).
/// Why Not Caught: S5 condition not tested; injecting unconditionally would over-grant.
/// Fix Applied: Task 022 S5 injection is gated on message.is_some().
/// Prevention: this test; trace checked for absence when no message.
/// Pitfall: removing the is_some() guard would inject skip-perms for interactive mode too.
#[ test ]
fn ct4_isolated_no_message_no_skip_permissions()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().unwrap();

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", path, "--trace" ] )
    .env( "PATH", "/nonexistent" )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  let err = stderr_str( &out );
  assert!(
    !err.contains( "--dangerously-skip-permissions" ),
    "trace must NOT contain --dangerously-skip-permissions when no message; got:\n{err}"
  );
}

// ── CT-5: timeout=0 unlimited semantics ──────────────────────────────────────

/// CT-5: `--timeout 0` → subprocess NOT killed; runs to natural completion.
///
/// Root Cause: `run_isolated()` set deadline = `Instant::now()` + `Duration::from_secs(0)`
///   unconditionally; with timeout_secs==0 the deadline was already expired on first
///   poll (50ms later), killing the subprocess immediately (gap I2 in command_defaults.md).
/// Why Not Caught: behavior diverges from run/ask (where 0 = unlimited) but no test.
/// Fix Applied: Task 022 S2 gates deadline computation on timeout_secs > 0.
/// Prevention: this test; fake subprocess sleeps 3s; timeout=0 must not kill it.
/// Pitfall: reverting the guard causes deadline to fire on first 50ms poll.
#[ test ]
fn ct5_timeout_zero_is_unlimited()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: sleeps 3s then exits 0 — watchdog must not fire.
  std::fs::write( &fake, b"#!/bin/sh\nsleep 3\nexit 0\n" ).expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let creds    = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().unwrap();
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );

  let start = std::time::Instant::now();
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", creds_path, "--timeout", "0", "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );
  let elapsed = start.elapsed();

  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "--timeout 0 must allow subprocess to exit naturally (unlimited); got exit {:?}; stderr: {}",
    out.status.code(),
    stderr_str( &out )
  );
  assert!(
    elapsed.as_secs() >= 2,
    "subprocess slept 3s; if elapsed < 2s the watchdog fired prematurely (timeout=0 must be unlimited)"
  );
  assert!(
    !stderr_str( &out ).to_lowercase().contains( "timeout" ),
    "no timeout error expected when subprocess exits naturally; stderr: {}", stderr_str( &out )
  );
}

// ── CT-6: CLAUDE.md provisioning ─────────────────────────────────────────────

/// CT-6: CLAUDE.md written to temp HOME before subprocess spawn.
///
/// Root Cause: `run_isolated()` did not write CLAUDE.md to the temp HOME; the subprocess
///   had no user-level behavioral instructions, potentially asking clarifying questions
///   or requesting interactive confirmation (gap I6 in command_defaults.md).
/// Why Not Caught: no test for CLAUDE.md presence; subprocess blocking is silent in tests.
/// Fix Applied: Task 022 S6 writes CLAUDE.md to claude_dir before spawn.
/// Prevention: this test; fake claude reads and outputs CLAUDE.md content to stdout.
/// Pitfall: cleanup removes the temp dir after execution; CLAUDE.md must be written
///   BEFORE spawn (not after) to be visible to the subprocess.
#[ test ]
fn ct6_claude_md_written_to_temp_home()
{
  let tmp  = tempfile::tempdir().expect( "create temp dir" );
  let fake = tmp.path().join( "claude" );

  // Script: reads $HOME/.claude/CLAUDE.md and prints it; outputs "MISSING" if absent.
  std::fs::write(
    &fake,
    b"#!/bin/sh\ncat \"$HOME/.claude/CLAUDE.md\" 2>/dev/null || printf 'MISSING'\n",
  )
  .expect( "write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod fake claude" );

  let creds      = make_creds_file( "{}" );
  let creds_path = creds.path().to_str().unwrap();
  let old_path   = std::env::var( "PATH" ).unwrap_or_default();
  let new_path   = format!( "{}:{old_path}", tmp.path().display() );

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "--creds", creds_path, "x" ] )
    .env( "PATH", &new_path )
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "invoke clr isolated" );

  let stdout = stdout_str( &out );
  assert!(
    !stdout.contains( "MISSING" ),
    "CLAUDE.md must be present in temp HOME; fake claude got MISSING"
  );
  assert!(
    stdout.contains( "# Isolated subprocess" ),
    "CLAUDE.md must start with '# Isolated subprocess'; got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Execute the given task immediately" ),
    "CLAUDE.md must contain 'Execute the given task immediately'; got:\n{stdout}"
  );
}
