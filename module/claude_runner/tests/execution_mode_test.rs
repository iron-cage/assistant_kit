//! Execution Mode Tests — interactive and print paths (Unix-only: uses shell scripts and chmod)
#![ cfg( unix ) ]
// Fix(BUG-226): all tests in this file depend on Unix shell scripts and PermissionsExt.
// Root cause: PATH-injection strategy uses chmod(0o755) and sh scripts — unavailable on Windows.
// Pitfall: compiling this file on Windows fails silently — gate the whole file, not individual tests.
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
//! | E04 | Subprocess exits non-zero + stderr populated | print (-p flag) | Subprocess stderr forwarded, exit non-zero |
//! | E05 | Print mode: stderr forwarded | print (-p flag) | Subprocess stderr appears in runner stderr |
//! | E06 | Print mode: stdout captured | print (-p flag) | Subprocess stdout appears in runner stdout |
//! | E07 | Interactive mode: binary not found, verbosity 0 | interactive (no msg) | Exit 1, stderr empty |
//! | E08 | Print mode: binary not found, verbosity 0 | print (-p flag) | Exit 1, stderr empty |
//! | E09 | Verbosity 4: preview to stderr before print execution | print (-p flag) | Stderr has env vars |
//! | E10 | Message forwarded + -c (print mode) | default print (msg given) | Binary receives --print, -c, and message arg |
//! | E11 | --new-session flag | interactive (no msg) | Subprocess does not receive -c |
//! | E12 | message, no -p | default print | Subprocess receives `--print` |
//! | E13 | --interactive "msg" | explicit interactive | Subprocess does NOT receive `--print` |
//! | E14 | Empty stderr + non-zero exit (rate limit) | print (-p flag) | Rate-limit diagnostic emitted |

use std::process::Command;

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude, fake_claude_dir, run_with_path };

// E01: Interactive mode: binary not found exits 1 with error on stderr.
#[ test ]
fn e01_interactive_binary_not_found()
{
  let out = run_with_path( &[ "test message" ], "/nonexistent" );
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
  let out = run_with_path( &[ "-p", "test message" ], "/nonexistent" );
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

// E04: Print mode: subprocess exits non-zero with populated stderr → stderr forwarded, exit non-zero.
// (Empty-stderr + non-zero case is covered by E14 — the rate-limit silent failure path.)
#[ test ]
fn e04_print_exit_nonzero_stderr_forwarded()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho 'claude error detail' >&2\nexit 3\n" );
  let out = run_with_path( &[ "-p", "test" ], &path );
  assert!( !out.status.success(), "must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "claude error detail" ),
    "subprocess stderr must be forwarded on failure. Got:\n{stderr}"
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
// Fix(BUG-213): env_remove CLR_TRACE prevents dev-shell trace activation from
//   printing the command preview to stderr regardless of --verbosity 0.
#[ test ]
fn e07_interactive_not_found_verbosity_zero()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--verbosity", "0", "test" ] )
    .env( "PATH", "/nonexistent" )
    .env_remove( "CLR_TRACE" ) // Fix(BUG-213)
    .output()
    .expect( "Failed to invoke" );
  assert!( !out.status.success(), "must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  // Fix(BUG-240): fatal spawn errors must be visible at verbosity 0.
  // Root cause: prior code gated the Err branch on shows_errors(); at verbosity 0
  //   spawn failures produced zero stderr output — a perfectly silent failure.
  // Pitfall: verbosity 0 suppresses runner diagnostics, never fatal errors.
  assert!(
    !stderr.is_empty(),
    "--verbosity 0 must still emit fatal spawn errors (BUG-240 fix). Got empty stderr"
  );
}

// E08: Print mode: binary not found + verbosity 0 → stderr empty.
// Fix(BUG-213): env_remove CLR_TRACE prevents dev-shell trace activation from
//   printing the command preview to stderr regardless of --verbosity 0.
#[ test ]
fn e08_print_not_found_verbosity_zero()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--verbosity", "0", "-p", "test" ] )
    .env( "PATH", "/nonexistent" )
    .env_remove( "CLR_TRACE" ) // Fix(BUG-213)
    .output()
    .expect( "Failed to invoke" );
  assert!( !out.status.success(), "must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  // Fix(BUG-240): fatal spawn errors must be visible at verbosity 0.
  // Root cause: prior code gated the Err branch on shows_errors(); at verbosity 0
  //   spawn failures produced zero stderr output — a perfectly silent failure.
  // Pitfall: verbosity 0 suppresses runner diagnostics, never fatal errors.
  assert!(
    !stderr.is_empty(),
    "--verbosity 0 must still emit fatal spawn errors (BUG-240 fix). Got empty stderr"
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
  let args_file      = tempfile::NamedTempFile::new().expect( "create args file" );
  let args_path      = args_file.path().display().to_string();
  let script         = format!( "echo \"$@\" > \"{args_path}\"\n" );
  let ( _tmp, path ) = fake_claude_dir( &script );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "hello world" ] )
    .env( "PATH", &path )
    .output()
    .expect( "invoke" );

  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );

  let received = std::fs::read_to_string( args_file.path() )
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
  let args_file      = tempfile::NamedTempFile::new().expect( "create args file" );
  let args_path      = args_file.path().display().to_string();
  let script         = format!( "echo \"$@\" > \"{args_path}\"\n" );
  let ( _tmp, path ) = fake_claude_dir( &script );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--new-session", "hello world" ] )
    .env( "PATH", &path )
    .output()
    .expect( "invoke" );

  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );

  let received = std::fs::read_to_string( args_file.path() )
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
  let args_file      = tempfile::NamedTempFile::new().expect( "create args file" );
  let args_path      = args_file.path().display().to_string();
  let script         = format!( "echo \"$@\" > \"{args_path}\"\n" );
  let ( _tmp, path ) = fake_claude_dir( &script );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "Fix the bug" ] )
    .env( "PATH", &path )
    .output()
    .expect( "invoke" );

  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );

  let received = std::fs::read_to_string( args_file.path() )
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
  let args_file      = tempfile::NamedTempFile::new().expect( "create args file" );
  let args_path      = args_file.path().display().to_string();
  let script         = format!( "echo \"$@\" > \"{args_path}\"\n" );
  let ( _tmp, path ) = fake_claude_dir( &script );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--interactive", "Fix the bug" ] )
    .env( "PATH", &path )
    .output()
    .expect( "invoke" );

  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );

  let received = std::fs::read_to_string( args_file.path() )
    .expect( "read args file" );
  assert!(
    !received.contains( "--print" ),
    "--interactive must suppress --print (interactive mode). Received: {received}"
  );
}

// E14: Print mode: empty stderr + non-zero exit emits labeled error diagnostic.
// test_kind: bug_reproducer(BUG-037)
//
// ## Root Cause
// `run_print_mode()` gated its only diagnostic on `!stderr.is_empty()`.  When
// `claude --print` exits non-zero with empty stderr (as 429 rate-limit does),
// the gate was false and no message was emitted — complete silence.
//
// ## Why Not Caught
// E04 used `exit 3` (empty stderr) and asserted "Claude exited with code 3",
// which was the generic fallthrough message.  The *meaningful* rate-limit case
// (empty stderr + non-zero exit → zero output) was never separately exercised.
//
// ## Fix Applied
// BUG-037 block replaced with `classify_error()` — each ErrorKind variant emits
// `"Error: {label} (exit {code})"`.  exit 1 with no pattern → Unknown label.
// The old generic phrase "possible rate limit or quota exhaustion" is fully removed.
//
// ## Prevention
// Every subprocess wrapper that gates diagnostics on `!stderr.is_empty()` must
// also handle the empty-stderr + non-zero-exit case with its own actionable
// message.
//
// ## Pitfall
// `claude --print` on 429 writes the rate-limit reason only to its JSONL
// session file, not to stderr.  Any wrapper relying solely on stderr for
// failure signal will silently swallow rate-limit errors.
#[ test ]
fn e14_print_silent_failure_rate_limit_diagnostic()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\nexit 1\n" );
  let out = run_with_path( &[ "-p", "test" ], &path );
  assert!( !out.status.success(), "must exit non-zero on silent failure" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "Error: unknown error (exit 1)" ),
    "must emit classified diagnostic on silent failure (empty stderr + non-zero exit). Got:\n{stderr}"
  );
  assert!(
    !stderr.contains( "possible rate limit or quota exhaustion" ),
    "generic phrase must be absent after BUG-037 fix. Got:\n{stderr}"
  );
}

// ── S76–S78, S80: New flag execution tests ──────────────────────────────────────

// S76: --strip-fences strips outermost fences from captured output
#[ test ]
fn s76_strip_fences_applied_to_captured_output()
{
  let script = "#!/bin/sh\nprintf '```rust\\nfn main(){}\\n```\\n'\n";
  let ( _tmp, path ) = fake_claude( script );
  let out = run_with_path(
    &[ "--strip-fences", "--no-ultrathink", "t" ],
    &path,
  );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "```" ),
    "fences must be stripped. Got:\n{stdout}",
  );
  assert!(
    stdout.contains( "fn main(){}" ),
    "content must remain. Got:\n{stdout}",
  );
}

// S77: --keep-claudecode preserves CLAUDECODE env var in subprocess
#[ test ]
fn s77_keep_claudecode_preserves_env_in_subprocess()
{
  let script = "#!/bin/sh\necho \"CLAUDECODE=$CLAUDECODE\"\n";
  let ( _tmp, path ) = fake_claude( script );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--keep-claudecode", "--no-ultrathink", "t" ] )
    .env( "PATH", &path )
    .env( "CLAUDECODE", "test_val" )
    .output()
    .expect( "failed to invoke clr" );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDECODE=test_val" ),
    "--keep-claudecode must preserve env var in subprocess. Got:\n{stdout}",
  );
}

// S78: --file pipes file content to subprocess stdin
#[ test ]
fn s78_file_content_piped_to_subprocess_stdin()
{
  let script = "#!/bin/sh\ncat\n";
  let ( _tmp, path ) = fake_claude( script );
  let input_file = tempfile::NamedTempFile::new().expect( "create temp" );
  std::fs::write( input_file.path(), "piped_content_s78" ).expect( "write" );
  let out = run_with_path(
    &[ "--no-ultrathink", "--file", input_file.path().to_str().unwrap(), "t" ],
    &path,
  );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "piped_content_s78" ),
    "--file must pipe content to subprocess stdin. Got:\n{stdout}",
  );
}

// S80: --file with nonexistent path → exit non-zero, stderr contains path
#[ test ]
fn s80_file_nonexistent_path_errors()
{
  let ( _tmp, path ) = fake_claude( "#!/bin/sh\necho ok\n" );
  let out = run_with_path(
    &[ "--no-ultrathink", "--file", "/tmp/nonexistent_99999.txt", "t" ],
    &path,
  );
  assert!( !out.status.success(), "--file with nonexistent path must fail" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "/tmp/nonexistent_99999.txt" ),
    "stderr must contain the file path. Got: {stderr}",
  );
}
