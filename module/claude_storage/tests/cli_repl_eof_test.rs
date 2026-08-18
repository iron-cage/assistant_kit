//! REPL EOF-handling tests for `clg --repl` (task-482 reproducers).
//!
//! ## Coverage
//!
//! T01-T03 per `task/claude_storage/482_repl_eof_infinite_busy_loop.md`'s
//! Test Matrix — stdin EOF must exit the REPL cleanly instead of spinning
//! in a tight `read_line() -> Ok(0) -> continue` busy-loop, and the
//! explicit `exit` path must keep working.
//!
//! Every test runs the real `clg` binary as a subprocess with piped stdin
//! and a hard timeout: a regression to the busy-loop kills the child and
//! fails the assertion loudly instead of hanging the suite.
//!
//! ## Test Case Index
//!
//! | ID | Test Name | Category |
//! |----|-----------|----------|
//! | T01 | `t01_repl_immediate_eof_exits_cleanly` | EOF Handling |
//! | T02 | `t02_repl_eof_after_command_exits_cleanly` | EOF Handling |
//! | T03 | `t03_repl_explicit_exit_still_works` | Regression |

mod common;

use core::time::Duration;

/// Hard ceiling for one REPL subprocess run. The buggy build spins forever;
/// a healthy build exits in well under a second. Generous enough for a
/// loaded CI container, tight enough to fail fast on regression.
const REPL_TIMEOUT : Duration = Duration::from_secs( 10 );

/// Build an `assert_cmd` command for `clg --repl` with piped stdin and an
/// isolated storage root, so no REPL-internal command can touch live state.
///
/// Wraps `common::clg_cmd()` via `assert_cmd::Command::from_std` because
/// only `assert_cmd::Command` offers `.timeout()` — the anti-hang guard
/// this test file exists to provide (AF1).
fn repl_cmd( home : &tempfile::TempDir ) -> assert_cmd::Command
{
  let mut cmd = assert_cmd::Command::from_std( common::clg_cmd() );
  cmd
    .arg( "--repl" )
    .env( "HOME", home.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", home.path().join( "storage" ).to_str().unwrap() )
    .timeout( REPL_TIMEOUT );
  cmd
}

// test_kind: bug_reproducer(task-482)
//
// ## Root Cause
// `run_repl()`'s read loop handled only the `Err` arm of
// `io::stdin().read_line()`. EOF returns `Ok(0)` — not `Err` — so the
// empty buffer trimmed to `""`, hit `if input.is_empty() { continue; }`,
// and looped straight back into `read_line`, which at EOF returns `Ok(0)`
// again immediately without blocking: a tight busy-loop pegging a CPU
// core, escapable only by SIGINT/SIGKILL.
//
// ## Why Not Caught
// No test ever drove the REPL as a subprocess; REPL coverage before this
// file was limited to help-interception logic tested via one-shot argv
// invocations, which never reach the interactive read loop.
//
// ## Fix Applied
// The `if let Err(e)` form became a 3-arm `match`: `Ok(0)` prints a
// newline (closing the unterminated `> ` prompt line) plus the same
// `Goodbye!` farewell as the explicit `exit` path and breaks the loop;
// `Ok(_)` proceeds; `Err(e)` keeps the pre-existing message + continue.
//
// ## Prevention
// Any future REPL input-loop change must keep an explicit `Ok(0)` arm;
// these timeout-guarded subprocess tests fail loudly (child killed at
// REPL_TIMEOUT) if EOF ever falls back into the spin.
//
// ## Pitfall
// `read_line` signals EOF in-band as `Ok(0)`, not out-of-band as `Err`.
// Any `if let Err` / `let-else` handling of `read_line` silently treats
// EOF as "empty input" — in a loop, that is an infinite spin, because
// reads at EOF do not block.

/// T01: `clg --repl` with empty piped stdin (immediate EOF) exits 0
/// promptly with a farewell, instead of spinning forever.
#[ test ]
fn t01_repl_immediate_eof_exits_cleanly()
{
  let home = tempfile::TempDir::new().unwrap();
  let assert = repl_cmd( &home ).write_stdin( "" ).assert();
  let out = assert.get_output().clone();
  assert.success();
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Goodbye!" ),
    "EOF exit must print the farewell; stdout: {stdout}"
  );
}

/// T02: a command followed by EOF is processed, then the REPL exits 0
/// cleanly — EOF after real input takes the same clean-exit path.
#[ test ]
fn t02_repl_eof_after_command_exits_cleanly()
{
  let home = tempfile::TempDir::new().unwrap();
  let assert = repl_cmd( &home ).write_stdin( "help\n" ).assert();
  let out = assert.get_output().clone();
  assert.success();
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "storage explorer" ),
    "the `help` line must render global usage (tagline) before EOF exit; stdout: {stdout}"
  );
  assert!(
    stdout.contains( "Goodbye!" ),
    "EOF after a command must still print the farewell; stdout: {stdout}"
  );
}

/// T03 (regression): the explicit `exit` command keeps working unchanged.
#[ test ]
fn t03_repl_explicit_exit_still_works()
{
  let home = tempfile::TempDir::new().unwrap();
  let assert = repl_cmd( &home ).write_stdin( "exit\n" ).assert();
  let out = assert.get_output().clone();
  assert.success();
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Goodbye!" ),
    "explicit exit must print the farewell; stdout: {stdout}"
  );
}
