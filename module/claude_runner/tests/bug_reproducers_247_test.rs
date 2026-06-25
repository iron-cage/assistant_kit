//! Bug reproducer for BUG-247: stdout content silently discarded on subprocess failure.
//!
//! # Root Cause (BUG-247)
//!
//! `run_print_mode()` called `std::process::exit(output.exit_code)` before reaching the
//! `print!("{out}")` statement on the success path.  When the subprocess exited non-zero,
//! any captured stdout was silently discarded — only the classified error label reached
//! the caller via stderr.
//!
//! # Why Not Caught
//!
//! Prior failure-path tests only checked exit codes and stderr labels.  No test verified
//! that stdout content from a failing subprocess appeared anywhere in clr's output.
//!
//! # Fix Applied
//!
//! Added stdout-to-stderr forward immediately before `std::process::exit(output.exit_code)`:
//! when `exit_code != 0 && !output.stdout.is_empty()`, content is forwarded via
//! `eprint!("{}", output.stdout)`.  Unconditional — mirrors the existing stderr forward.
//!
//! # Prevention
//!
//! Every exit-before-print path must be audited for content that would be discarded.
//! Use the `Fix(BUG-247)` comment as a sentinel for this pattern.
//!
//! # Pitfall
//!
//! No verbosity gate — raw subprocess stdout is always forwarded on failure, mirroring
//! the unconditional stderr forward.  Verbosity gates runner diagnostics, not subprocess
//! output content.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `t01_stdout_forwarded_to_stderr_on_exit_1` | subprocess exits 1 with stdout content | content appears in clr stderr |
//! | `t02_stdout_and_stderr_both_forwarded_on_exit_1` | exits 1 with stdout + stderr | both appear in clr stderr |
//! | `t03_empty_stdout_no_spurious_output_on_exit_1` | exits 1 with empty stdout | no extra blank line in stderr |
//! | `t04_stdout_on_success_path_goes_to_clr_stdout` | exits 0 with stdout content | content in clr stdout |
//! | `t05_stdout_forwarded_on_exit_2_rate_limit` | exits 2 with stdout content | content in clr stderr; clr exits 2 |

#![ cfg( unix ) ]
// Fix(BUG-316): Root cause: Feature-064 pull did not gate this file; fake_claude_dir is
// #[cfg(unix)]-only so its unconditional import fails on Windows (E0432 "configured out").
// Pitfall: when a test file imports ANY unix-only helper, the whole file needs #![cfg(unix)];
// unresolved imports are crate-level errors, not item-level — #[cfg(unix)] on test fns alone
// does not prevent the import from being resolved.
mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, run_cli_with_env };

// ── T01 ──────────────────────────────────────────────────────────────────────

/// T01: subprocess exits 1 with content on stdout → content appears on clr's stderr.
///
/// Before fix: `std::process::exit(1)` fired before `print!("{out}")` — stdout was lost.
/// After fix: content forwarded via `eprint!("{}", output.stdout)` before exit call.
#[ test ]
#[ doc = "bug_reproducer(BUG-247)" ]
fn t01_stdout_forwarded_to_stderr_on_exit_1()
{
  let ( _dir, path ) = fake_claude_dir(
    "echo 'API Error: 529 overloaded'; exit 1"
  );
  let out = run_cli_with_env(
    &[ "-p", "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "API Error: 529 overloaded" ),
    "BUG-247 T01: stdout from failing subprocess must appear in clr stderr.\nstderr: {stderr}",
  );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "BUG-247 T01: exit code must propagate as 1. Got: {:?}", out.status.code(),
  );
}

// ── T02 ──────────────────────────────────────────────────────────────────────

/// T02: subprocess exits 1 with both stdout and stderr content → both appear in clr's stderr.
///
/// Verifies that the stdout forward coexists with the existing stderr forward.
#[ test ]
#[ doc = "bug_reproducer(BUG-247)" ]
fn t02_stdout_and_stderr_both_forwarded_on_exit_1()
{
  let ( _dir, path ) = fake_claude_dir(
    "echo 'STDOUT: diagnostic text'; echo 'STDERR: error detail' >&2; exit 1"
  );
  let out = run_cli_with_env(
    &[ "-p", "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "STDOUT: diagnostic text" ),
    "BUG-247 T02: stdout content must appear in clr stderr.\nstderr: {stderr}",
  );
  assert!(
    stderr.contains( "STDERR: error detail" ),
    "BUG-247 T02: stderr content must also appear in clr stderr.\nstderr: {stderr}",
  );
}

// ── T03 ──────────────────────────────────────────────────────────────────────

/// T03: subprocess exits 1 with empty stdout → no spurious blank line from the forward guard.
///
/// The guard `!output.stdout.is_empty()` must prevent forwarding an empty string.
/// Before fix this was moot (exit before any output); after fix, empty stdout must be silent.
#[ test ]
#[ doc = "bug_reproducer(BUG-247)" ]
fn t03_empty_stdout_no_spurious_output_on_exit_1()
{
  let ( _dir, path ) = fake_claude_dir(
    "echo 'stderr-only diagnostic' >&2; exit 1"
  );
  let out = run_cli_with_env(
    &[ "-p", "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );

  let stderr = String::from_utf8_lossy( &out.stderr );
  // stderr-only subprocess content must still arrive (existing stderr forward)
  assert!(
    stderr.contains( "stderr-only diagnostic" ),
    "BUG-247 T03: stderr content must still appear.\nstderr: {stderr}",
  );
  // No stdout → no forward of stdout; stderr must not contain an extra blank line
  // from an empty eprint!().  A blank line would look like "\n\n" or start with "\n".
  assert!(
    !stderr.starts_with( '\n' ),
    "BUG-247 T03: empty stdout must not produce leading blank line in stderr.\nstderr: {stderr:?}",
  );
}

// ── T04 ──────────────────────────────────────────────────────────────────────

/// T04: subprocess exits 0 with content on stdout → content appears on clr's STDOUT (not stderr).
///
/// Confirms the fix does not alter the success path — stdout goes to clr's stdout, not stderr.
#[ test ]
#[ doc = "bug_reproducer(BUG-247)" ]
fn t04_stdout_on_success_path_goes_to_clr_stdout()
{
  let ( _dir, path ) = fake_claude_dir(
    "echo 'SUCCESS: response text'; exit 0"
  );
  let out = run_cli_with_env(
    &[ "-p", "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );

  let stdout = String::from_utf8_lossy( &out.stdout );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    out.status.success(),
    "BUG-247 T04: subprocess exits 0 → clr must exit 0.\nstderr: {stderr}",
  );
  assert!(
    stdout.contains( "SUCCESS: response text" ),
    "BUG-247 T04: success-path stdout must appear on clr's stdout.\nstdout: {stdout}",
  );
  assert!(
    !stderr.contains( "SUCCESS: response text" ),
    "BUG-247 T04: success-path stdout must NOT appear on stderr.\nstderr: {stderr}",
  );
}

// ── T05 ──────────────────────────────────────────────────────────────────────

/// T05: subprocess exits 2 (rate-limit code) with stdout content → content forwarded; exit 2 propagated.
///
/// Exit code 2 is the conventional rate-limit/overloaded code; must propagate unchanged.
#[ test ]
#[ doc = "bug_reproducer(BUG-247)" ]
fn t05_stdout_forwarded_on_exit_2_rate_limit()
{
  let ( _dir, path ) = fake_claude_dir(
    "echo 'Rate limit: please retry in 60s'; exit 2"
  );
  let out = run_cli_with_env(
    &[ "-p", "--max-sessions", "0", "task" ],
    &[ ( "PATH", &path ) ],
  );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "Rate limit: please retry in 60s" ),
    "BUG-247 T05: stdout content must be forwarded to clr stderr on exit 2.\nstderr: {stderr}",
  );
  assert_eq!(
    out.status.code(),
    Some( 2 ),
    "BUG-247 T05: exit code 2 must propagate unchanged. Got: {:?}", out.status.code(),
  );
}
