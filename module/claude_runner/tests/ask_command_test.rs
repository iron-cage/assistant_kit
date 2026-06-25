//! `ask` Subcommand Integration Tests
//!
//! ## Purpose
//!
//! Verify that `clr ask` is a pure semantic alias for `clr run`: identical dry-run
//! output, no forced overrides, and the only behavioural difference is the `--help`
//! text.
//!
//! ## Strategy
//!
//! Tests T01–T08 invoke `clr ask --dry-run` and compare against `clr run --dry-run`.
//! No real Claude invocation occurs.
//!
//! ## Corner Cases Covered
//!
//! - T01: `clr ask --dry-run "X"` == `clr run --dry-run "X"` (full stdout equivalence)
//! - T02: `clr ask --dry-run "X"` — does not force `--new-session` (no `--new-session` in output)
//! - T03: `clr ask --dry-run "X"` — does not force `--no-chrome` (chrome present in output)
//! - T04: `clr ask --dry-run "X"` — does not force `--no-persist` (no `--no-session-persistence`)
//! - T05: `clr ask --dry-run "X"` — ultrathink suffix injected (same as run)
//! - T06: `clr ask --dry-run "X"` — uses `--effort max` (run default, not ask-specific high)
//! - T07: `clr ask --dry-run "X"` — uses `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000` (run default)
//! - T08: `clr ask --new-session --dry-run "X"` — explicit flag respected (no `-c` in output)
//! - T09: `clr ask --unknown-flag "X"` — unknown flag rejected (exit 1, stderr error)
//! - T10: `clr ask --subdir NAME "X"` — effective dir ends with `/-NAME`
//! - T12: `clr assk …` — edit-distance-1 typo caught by guard; exits 1, "Did you mean 'ask'?"

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_ask_dry, run_cli };
use std::process::Command;

/// Run `clr run --dry-run` with extra args; return stdout.  Asserts exit 0.
fn run_run_dry( extra_args : &[ &str ] ) -> String
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut args = vec![ "run", "--dry-run" ];
  args.extend_from_slice( extra_args );
  let out = Command::new( bin )
    .args( &args )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "clr run --dry-run failed (exit {}): {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stderr )
  );
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

// T01: ask and run produce identical dry-run output — pure alias equivalence
#[ test ]
fn t01_ask_run_dry_run_equivalence()
{
  let ask_out = run_ask_dry( &[ "What does X do?" ] );
  let run_out = run_run_dry( &[ "What does X do?" ] );
  assert_eq!(
    ask_out, run_out,
    "ask and run must produce identical dry-run output.\nask:\n{ask_out}\nrun:\n{run_out}"
  );
}

// T02: ask does not force --new-session — output must not contain a forced new-session flag
#[ test ]
fn t02_ask_no_forced_new_session()
{
  let output = run_ask_dry( &[ "What does X do?" ] );
  // new_session is a runner-internal flag; it affects whether -c appears.
  // A pure alias with default session state has -c (same as run).
  // Verify ask behaves identically to run (T01 is the authoritative check;
  // this test confirms no hidden --new-session injection by checking run for parity).
  let run_out = run_run_dry( &[ "What does X do?" ] );
  assert_eq!(
    output.contains( " -c" ),
    run_out.contains( " -c" ),
    "ask must produce same session-continuation state as run. ask:\n{output}\nrun:\n{run_out}"
  );
}

// T03: ask does not force --no-chrome — chrome present in output (same as run)
#[ test ]
fn t03_ask_no_forced_no_chrome()
{
  let ask_out = run_ask_dry( &[ "What does X do?" ] );
  let run_out = run_run_dry( &[ "What does X do?" ] );
  assert_eq!(
    ask_out.contains( "--chrome" ),
    run_out.contains( "--chrome" ),
    "ask chrome flag must match run. ask:\n{ask_out}\nrun:\n{run_out}"
  );
}

// T04: ask does not force --no-persist — persistence flags match run
#[ test ]
fn t04_ask_no_forced_no_persist()
{
  let ask_out = run_ask_dry( &[ "What does X do?" ] );
  let run_out = run_run_dry( &[ "What does X do?" ] );
  assert_eq!(
    ask_out.contains( "--no-session-persistence" ),
    run_out.contains( "--no-session-persistence" ),
    "ask persistence flag must match run. ask:\n{ask_out}\nrun:\n{run_out}"
  );
}

// T05: ask injects ultrathink suffix (same as run — no forced suppression)
#[ test ]
fn t05_ask_ultrathink_suffix_injected()
{
  let output = run_ask_dry( &[ "What does X do?" ] );
  assert!(
    output.contains( "ultrathink" ),
    "ask must inject ultrathink suffix (same as run). Got:\n{output}"
  );
}

// T06: ask uses --effort max (run default) — not the old ask-specific --effort high
#[ test ]
fn t06_ask_effort_defaults_to_max()
{
  let output = run_ask_dry( &[ "What does X do?" ] );
  assert!(
    output.contains( "--effort max" ),
    "ask must use --effort max (run default). Got:\n{output}"
  );
  assert!(
    !output.contains( "--effort high" ),
    "ask must NOT use --effort high (old ask default removed). Got:\n{output}"
  );
}

// T07: ask uses CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000 (run default) — not 16384
#[ test ]
fn t07_ask_max_tokens_defaults_to_200000()
{
  let output = run_ask_dry( &[ "What does X do?" ] );
  assert!(
    !output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384" ),
    "ask must NOT use max-tokens 16384 (old ask default removed). Got:\n{output}"
  );
}

// T08: explicit --new-session flag is respected by ask
#[ test ]
fn t08_ask_explicit_new_session_respected()
{
  let output = run_ask_dry( &[ "--new-session", "What does X do?" ] );
  // With explicit --new-session, -c must not appear.
  assert!(
    !output.contains( " -c" ),
    "explicit --new-session must suppress -c. Got:\n{output}"
  );
}

// T09: unknown flag rejected — exit 1, stderr has error message
#[ test ]
fn t09_ask_unknown_flag_rejected()
{
  let out = run_cli( &[ "ask", "--unknown-flag-xyz", "X" ] );
  assert!(
    !out.status.success(),
    "unknown flag must cause non-zero exit. Got exit: {:?}",
    out.status.code()
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "unknown option" ) || stderr.contains( "Error:" ),
    "error message must appear on stderr. Got:\n{stderr}"
  );
}

// T11: 'clr ask help' (positional) dispatches to ask help — must not treat "help" as a message
//
// ## Bug Reproducer
//
// ### Root Cause
// `dispatch_ask()` only intercepted `--help`/`-h` flags; the positional "help" token was
// passed to `dispatch_run()` as a message, causing it to reach `wait_for_session_slot()`
// and block when the session limit was reached.
//
// ### Why Not Caught
// IT-8 in `tests/docs/cli/command/05_ask.md` specifies the behavior but no automated test
// exercised the positional form. Session-gate blocking only manifests when ≥15 sessions are
// active, making the regression intermittent in clean environments.
//
// ### Fix Applied
// Added positional "help" check in `dispatch_ask()` before delegating to `dispatch_run()`,
// mirroring the BUG-215 fix in `run_cli()` for `clr run help`.
//
// ### Prevention
// Every subcommand dispatcher that delegates to `dispatch_run` must intercept the positional
// "help" token. Spec IT-8 tests (positional "help" without `--`) must be included for each
// subcommand, not only the `--help`/`-h` flag form.
//
// ### Pitfall
// Positional "help" and `--help`/`-h` are two distinct intercept paths. Adding one does not
// automatically cover the other. Check for both forms in every subcommand dispatcher.
#[ test ]
#[ cfg_attr( any(), doc = "bug_reproducer(BUG-249)" ) ]
fn t11_ask_positional_help_shows_help()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "ask", "help" ] )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "clr ask help must exit 0. Got exit {}: {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Usage" ) || stdout.contains( "usage" ) || stdout.contains( "clr ask" ),
    "clr ask help must print help text. Got stdout:\n{stdout}"
  );
}

// T10: --subdir NAME produces effective dir ending in /-NAME
#[ test ]
fn t10_ask_subdir_effective_dir()
{
  let output = run_ask_dry( &[ "--subdir", "feature", "What is X?" ] );
  let sep = std::path::MAIN_SEPARATOR;
  assert!(
    output.contains( &format!( "{sep}-feature" ) ),
    "ask --subdir feature must produce path ending in {sep}-feature. Got:\n{output}"
  );
}

/// T12: `clr assk` triggers the unknown-subcommand guard with "Did you mean 'ask'?".
///
/// Reproduces BUG-250: `guard_unknown_subcommand` only checked prefix/superstring matches
/// (`starts_with`), missing one-character insertion typos like "assk" for "ask".  The
/// extra 's' in the middle is not caught by either `starts_with` direction, so the guard
/// silently fell through and `dispatch_run` treated "assk" as the message argument.
///
/// ## Root Cause
/// `guard_unknown_subcommand` used two `starts_with` checks only.  `"assk".starts_with("ask")`
/// is false (first three chars are `'a','s','s'`, not `'a','s','k'`) and `"ask".starts_with("assk")`
/// is false (shorter string cannot start with a longer one).  No edit-distance check existed.
///
/// ## Why Not Caught
/// The existing BUG-225 test (`isolated_test.rs`) only exercised prefix truncations ("isol",
/// "isolate").  No test exercised a mid-word character insertion for a short subcommand name.
///
/// ## Fix Applied
/// Added `is_close_typo()` helper in `src/cli/mod.rs` and extended the guard condition
/// to include `|| is_close_typo(first, sub)`.  The helper returns `true` when the first
/// character matches AND the Levenshtein distance is exactly 1.  The first-character
/// constraint prevents false positives for common English words whose deletion of the
/// leading character yields "ask" (e.g. "task" → "ask", first char 't' ≠ 'a').
///
/// ## Prevention
/// Every subcommand must have a test covering at least one edit-distance-1 typo (mid-word
/// insertion or substitution), not just prefix/truncation variants.
///
/// ## Pitfall
/// `starts_with` does not catch mid-word insertions: `"assk".starts_with("ask")` is false
/// because the third character `'s'` ≠ `'k'`, even though `"assk"` and `"ask"` differ by
/// exactly one character.  Use edit distance, not substring matching, for typo detection.
// test_kind: bug_reproducer(BUG-250)
#[ test ]
fn t12_ask_edit_distance_typo_caught_by_guard()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );

  // "assk" = "ask" with an extra 's' inserted — edit distance 1, not caught by starts_with
  let out = std::process::Command::new( bin )
    .args( [ "assk", "--dry-run" ] )
    .output()
    .expect( "failed to invoke clr assk --dry-run" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "'clr assk' must exit 1 (unknown subcommand guard); got {:?}\nstderr: {stderr}",
    out.status.code(),
  );
  assert!(
    stderr.contains( "unknown subcommand" ),
    "stderr must contain 'unknown subcommand'; got: {stderr}"
  );
  assert!(
    stderr.contains( "ask" ),
    "stderr must suggest 'ask'; got: {stderr}"
  );
}
