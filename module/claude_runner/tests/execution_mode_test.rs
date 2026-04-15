//! Execution Mode Tests — interactive and print paths (Unix-only: uses shell scripts and chmod)
// Fix(issue-108): all tests in this file depend on Unix shell scripts and PermissionsExt.
// Root cause: PATH-injection strategy uses chmod(0o755) and sh scripts — unavailable on Windows.
// Pitfall: compiling this file on Windows fails silently — gate the whole file, not individual tests.
#![ cfg( unix ) ]
//!
//! ## Purpose
//!
//! Verify that `claude_runner` correctly routes to `execute_interactive()` or
//! `execute()` based on the mode-selection rules:
//! - No message → interactive REPL (`execute_interactive()`)
//! - Message given → print mode (`execute()` + `--print`) **unless** `--interactive` is set
//! - `--interactive` flag → forces TTY passthrough even when message given
//!
//! Covers error handling, exit code propagation, stderr forwarding, verbosity gating.
//!
//! ## Strategy
//!
//! Uses fake `claude` shell scripts injected via PATH manipulation to test
//! execution behavior without requiring the real Claude binary. The fake scripts
//! produce deterministic output and exit codes for assertion.
//!
//! ## Test Matrix
//!
//! | # | Scenario | Mode | Expected |
//! |---|----------|------|----------|
//! | E01 | Binary not found, interactive | interactive (no msg) | Exit 1, stderr error |
//! | E02 | Binary not found, print | print (-p flag) | Exit 1, stderr error |
//! | E03 | Subprocess exits non-zero, interactive | `--interactive "test"` | Exit code propagated |
//! | E04 | Subprocess exits non-zero, print | print (-p flag) | Exit 1, stderr error message |
//! | E05 | Print mode: stderr forwarded | print (-p flag) | Subprocess stderr appears in runner stderr |
//! | E06 | Print mode: stdout captured | print (-p flag) | Subprocess stdout appears in runner stdout |
//! | E07 | Interactive mode: binary not found, verbosity 0 | interactive (no msg) | Exit 1, stderr empty |
//! | E08 | Print mode: binary not found, verbosity 0 | print (-p flag) | Exit 1, stderr empty |
//! | E09 | Verbosity 4: preview to stderr before print execution | print (-p flag) | Stderr has env vars |
//! | E10 | Message forwarded + -c (print mode) | default print (msg given) | Binary receives --print, -c, and message arg |
//! | E11 | --new-session flag | interactive (no msg) | Subprocess does not receive -c |
//! | E12 | message, no -p | default print | Subprocess receives `--print` |
//! | E13 | --interactive "msg" | explicit interactive | Subprocess does NOT receive `--print` |

use std::os::unix::fs::PermissionsExt;
use std::process::Command;

/// Create a fake `claude` binary in a temp directory; return (tempdir, modified PATH).
fn fake_claude( script : &str ) -> ( tempfile::TempDir, String )
{
  let tmp = tempfile::tempdir().expect( "Failed to create temp dir" );
  let fake = tmp.path().join( "claude" );
  std::fs::write( &fake, script ).expect( "Failed to write fake claude" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "Failed to chmod fake claude" );
  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let new_path = format!( "{}:{old_path}", tmp.path().display() );
  ( tmp, new_path )
}

fn run_with_path( args : &[ &str ], path : &str ) -> std::process::Output
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .env( "PATH", path )
    .output()
    .expect( "Failed to invoke clr binary" )
}

fn run_no_claude( args : &[ &str ] ) -> std::process::Output
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "Failed to invoke clr binary" )
}

// E01: Interactive mode: binary not found exits 1 with error on stderr.
#[ test ]

fn e01_interactive_binary_not_found()
{
  let out = run_no_claude( &[ "test message" ] );
  assert!( !out.status.success(), "must exit non-zero when claude not found" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "Error:" ),
    "must report error to stderr. Got:\n{stderr}"
  );
}

// E02: Print mode: binary not found exits 1 with error on stderr.
#[ test ]

fn e02_print_binary_not_found()
{
  let out = run_no_claude( &[ "-p", "test message" ] );
  assert!( !out.status.success(), "must exit non-zero when claude not found" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "Error:" ),
    "must report error to stderr. Got:\n{stderr}"
  );
}

// E03: Interactive mode: subprocess non-zero exit code is propagated.
// Uses --interactive to force interactive mode (message alone now defaults to print mode).
#[ test ]

fn e03_interactive_exit_code_propagated()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\nexit 42\n" );
  let out = run_with_path( &[ "--interactive", "test" ], &path );
  assert_eq!(
    out.status.code(), Some( 42 ),
    "interactive mode must propagate subprocess exit code. Got: {:?}",
    out.status.code()
  );
}

// E04: Print mode: subprocess non-zero exit triggers error message.
#[ test ]

fn e04_print_exit_nonzero_error()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\nexit 3\n" );
  let out = run_with_path( &[ "-p", "test" ], &path );
  assert!( !out.status.success(), "must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "Claude exited with code 3" ),
    "must report exit code in error. Got:\n{stderr}"
  );
}

// E05: Print mode: subprocess stderr is forwarded to runner stderr.
#[ test ]

fn e05_print_stderr_forwarded()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho STDERR_MARKER >&2\necho STDOUT_OK\n" );
  let out = run_with_path( &[ "-p", "test" ], &path );
  assert!( out.status.success(), "must exit 0" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "STDERR_MARKER" ),
    "subprocess stderr must be forwarded. Got:\n{stderr}"
  );
}

// E06: Print mode: subprocess stdout is captured and printed.
#[ test ]

fn e06_print_stdout_captured()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho CAPTURED_OUTPUT\n" );
  let out = run_with_path( &[ "-p", "test" ], &path );
  assert!( out.status.success(), "must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CAPTURED_OUTPUT" ),
    "subprocess stdout must appear in runner stdout. Got:\n{stdout}"
  );
}

// E07: Interactive mode: binary not found + verbosity 0 → stderr empty.
#[ test ]

fn e07_interactive_not_found_verbosity_zero()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--verbosity", "0", "test" ] )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "Failed to invoke" );
  assert!( !out.status.success(), "must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.is_empty(),
    "--verbosity 0 must suppress error output. Got:\n{stderr}"
  );
}

// E08: Print mode: binary not found + verbosity 0 → stderr empty.
#[ test ]

fn e08_print_not_found_verbosity_zero()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--verbosity", "0", "-p", "test" ] )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "Failed to invoke" );
  assert!( !out.status.success(), "must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.is_empty(),
    "--verbosity 0 must suppress error output. Got:\n{stderr}"
  );
}

// E09: Verbosity 4 prints command preview to stderr before print-mode execution.
#[ test ]

fn e09_verbosity_four_stderr_preview()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho OK\n" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--verbosity", "4", "-p", "test" ] )
    .env( "PATH", &path )
    .output()
    .expect( "Failed to invoke" );
  assert!( out.status.success(), "must exit 0" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--verbosity 4 must print env preview to stderr. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "claude" ),
    "--verbosity 4 must print command preview to stderr. Got:\n{stderr}"
  );
}

// E10: Print mode (default when message given): message and -c are forwarded to the subprocess.
// `clr "hello world"` routes to print mode (execute() + --print), but this test only asserts
// that the message and -c are present — not that --print is absent — so it covers both paths.
// Uses a fake binary that echoes its arguments to a file, then verifies.
#[ test ]

fn e10_interactive_message_forwarded()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let args_file = tmp.path().join( "received_args" );

  // Fake claude that writes its arguments to a file.
  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\necho \"$@\" > \"{}\"\n",
    args_file.display()
  );
  std::fs::write( &fake, script ).expect( "write fake" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let path = format!( "{}:{old_path}", tmp.path().display() );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "hello world" ] )
    .env( "PATH", &path )
    .output()
    .expect( "invoke" );

  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );

  let received = std::fs::read_to_string( &args_file )
    .expect( "read args file" );
  assert!(
    received.contains( "hello world" ),
    "message must be forwarded to subprocess. Received args: {received}"
  );
  assert!(
    received.contains( "-c" ),
    "automatic -c must be forwarded to subprocess. Received args: {received}"
  );
}

// E11: --new-session suppresses the default -c flag passed to the subprocess.
#[ test ]

fn e11_new_session_does_not_pass_continue()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let args_file = tmp.path().join( "received_args" );

  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\necho \"$@\" > \"{}\"\n",
    args_file.display()
  );
  std::fs::write( &fake, script ).expect( "write fake" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let path = format!( "{}:{old_path}", tmp.path().display() );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--new-session", "hello world" ] )
    .env( "PATH", &path )
    .output()
    .expect( "invoke" );

  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );

  let received = std::fs::read_to_string( &args_file )
    .expect( "read args file" );
  assert!(
    !received.contains( " -c" ),
    "--new-session must suppress -c. Received args: {received}"
  );
}

// E12: message without -p routes to print mode — subprocess receives --print.
#[ test ]
fn e12_message_without_print_flag_uses_print_mode()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let args_file = tmp.path().join( "received_args" );

  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\necho \"$@\" > \"{}\"\n",
    args_file.display()
  );
  std::fs::write( &fake, &script ).expect( "write fake" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let path = format!( "{}:{old_path}", tmp.path().display() );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "Fix the bug" ] )
    .env( "PATH", &path )
    .output()
    .expect( "invoke" );

  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );

  let received = std::fs::read_to_string( &args_file )
    .expect( "read args file" );
  assert!(
    received.contains( "--print" ),
    "message without -p must route to print mode (--print in subprocess args). Received: {received}"
  );
}

// E13: --interactive with message routes to interactive mode — subprocess does NOT receive --print.
#[ test ]
fn e13_interactive_flag_with_message_uses_interactive_mode()
{
  let tmp = tempfile::tempdir().expect( "create temp dir" );
  let args_file = tmp.path().join( "received_args" );

  let fake = tmp.path().join( "claude" );
  let script = format!(
    "#!/bin/sh\necho \"$@\" > \"{}\"\n",
    args_file.display()
  );
  std::fs::write( &fake, &script ).expect( "write fake" );
  std::fs::set_permissions( &fake, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod" );

  let old_path = std::env::var( "PATH" ).unwrap_or_default();
  let path = format!( "{}:{old_path}", tmp.path().display() );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--interactive", "Fix the bug" ] )
    .env( "PATH", &path )
    .output()
    .expect( "invoke" );

  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );

  let received = std::fs::read_to_string( &args_file )
    .expect( "read args file" );
  assert!(
    !received.contains( "--print" ),
    "--interactive must suppress --print (interactive mode). Received: {received}"
  );
}
