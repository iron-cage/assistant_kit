//! Integration tests for space-form `<command> help` interception.
//!
//! ## Source
//!
//! - Bug: `task/claude_storage/bug/completed/005_space_form_command_help_misparsed.md`
//! - Task: `task/claude_storage/verified/476_intercept_spaced_command_help_dispatch.md`
//! - AGG-01 (T09/T10): `-plan/verified/002_intercept_spaced_command_help_dispatch.plan.md § Known Deferred Risk`
//!
//! Fix(BUG-006)
//! Root cause: T04/T05 originally read expected output from `-baseline/<name>`, a
//! hyphen-prefixed (gitignored) path that was never committed — a permanent test
//! depending on a temporary fixture tier, violating CLAUDE.md storage tiers.
//! Pitfall: never let a permanent test read its expected value from a path this
//! project's own convention marks ephemeral (`-*`); bake the expected literal (or a
//! stable substring, as T06-T10 below already do) into the test source instead.
//!
//! ## Coverage
//!
//! - T01: `.list help` one-shot intercepted, byte-identical to `.list.help`
//! - T02: `.list help` in REPL intercepted, loop continues to the next command
//! - T03: `.show help` one-shot intercepted (generality proof, distinct failure mode)
//! - T04: `.nonexistent help` unchanged (unregistered command boundary)
//! - T05: `.list uuid help` unchanged (three-token boundary)
//! - T06: `.list HELP` unchanged (case-sensitivity boundary)
//! - T07: `.list helpme` unchanged (content near-miss boundary)
//! - T08: REPL irregular whitespace (`.list  help`) intercepted
//! - T09: `.search help` intercepted — AGG-01 accepted trade-off (reserved word)
//! - T10: `.search query::help` unaffected — AGG-01 boundary (named param still searches)

mod common;

use std::io::Write;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

/// Run the binary in `--repl` mode with piped stdin, returning captured output.
///
/// Callers MUST end `input` with an explicit `"exit\n"`/`"quit\n"`/`"q\n"` line.
/// `run_repl`'s read loop busy-spins on a closed stdin: `read_line` returns
/// `Ok(0)` forever on EOF (never an `Err`), so an unterminated pipe hangs
/// `wait_with_output()` forever rather than returning.
fn repl_run( cmd : &mut std::process::Command, input : &str ) -> std::process::Output
{
  assert!(
    input.ends_with( "exit\n" ) || input.ends_with( "quit\n" ) || input.ends_with( "q\n" ),
    "repl_run input must end with an explicit exit command to avoid hanging on the REPL's EOF busy-loop; got: {input:?}"
  );

  let mut child = cmd
    .arg( "--repl" )
    .stdin( std::process::Stdio::piped() )
    .stdout( std::process::Stdio::piped() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .unwrap();

  child.stdin.take().unwrap().write_all( input.as_bytes() ).unwrap();

  child.wait_with_output().unwrap()
}

/// T01: `clg .list help` one-shot is intercepted and renders `.list`'s styled
/// detail help, byte-for-byte identical to `clg .list.help` (BUG-005).
///
/// ## Root Cause
/// unilang's argument binder treats a trailing bare token as a positional
/// argument once it fails to match a flag or one of the pre-existing
/// special-cased help forms (`.help`, `--help`, `-h`, the `.command.help`
/// dot-suffix). Space-separated `<command> help` was never one of those
/// special cases, so the second token (`help`) was bound positionally
/// against `.list`'s `show_sessions` boolean parameter instead, producing
/// `Invalid type: help. Valid values: uuid, path, all` rather than rendering
/// help.
///
/// ## Why Not Caught
/// The existing suite exercised the dot-suffix form (`.list.help`) and the
/// global bare `help` token, but no test exercised the two-token
/// space-separated form `<command> help` — the most intuitive form for a
/// user unfamiliar with the dot-suffix convention. `try_command_help()`'s
/// own scope only ever covered `token.strip_suffix(".help")`, leaving this
/// sibling form entirely unvalidated.
///
/// ## Fix Applied
/// Added `try_command_help_space_form()` in `src/cli_main.rs`, a sibling
/// interceptor checked at the same two call sites as `try_command_help()`
/// (`execute_oneshot()` and `run_repl()`), matching exactly
/// `tokens.len() == 2 && tokens[ 1 ] == "help"` against a registered command
/// name and rendering via the same `print_command_help()` helper —
/// guaranteeing byte-identical output to the dot-suffix form since both
/// paths converge on the same renderer.
///
/// ## Prevention
/// Any new help-invocation syntax must add both a matcher function and a
/// Test Matrix entry proving its output is byte-identical to the existing
/// dot-suffix baseline (`assert_eq!(stdout(&new_form), stdout(&dot_form))`),
/// not merely "renders something."
///
/// ## Pitfall
/// Don't special-case `<command> help` inside the generic argument-parsing/
/// binding path (e.g., pre-filtering `"help"` out of positional args before
/// they reach unilang) — that would silently swallow a legitimate
/// positional value literally named `help` for some other command. The
/// interception must happen as a dedicated pre-dispatch check keyed on the
/// literal two-token shape, before the pipeline ever sees the tokens,
/// exactly mirroring how `try_command_help()` intercepts the dot-suffix
/// form.
#[ test ]
// bug_reproducer(BUG-005)
fn t01_list_help_one_shot_intercepted()
{
  let space_form = common::clg_cmd().arg( ".list" ).arg( "help" ).output().unwrap();
  let dot_form = common::clg_cmd().arg( ".list.help" ).output().unwrap();

  assert_exit( &dot_form, 0 );
  assert_exit( &space_form, 0 );
  assert_eq!(
    stdout( &space_form ), stdout( &dot_form ),
    "`.list help` must render byte-identical to `.list.help`;\n  space-form:\n{}\n  dot-form:\n{}",
    stdout( &space_form ), stdout( &dot_form )
  );
}

/// T02: `.list help` in REPL mode is intercepted and the loop `continue`s to
/// process the next line rather than falling through or terminating.
///
/// ## Purpose
/// Prove the fix works in the REPL entry point (not just one-shot) and that
/// interception properly `continue`s the loop — a missing `continue` would
/// ship undetected, since the user would still see help text once,
/// immediately followed by a confusing fall-through dispatch error for the
/// SAME line rather than moving on to the next one.
///
/// ## Coverage
/// REPL input `.list help\n.status\nexit\n`; pre-fix this shows a dispatch
/// error for `.list help` (no help text) before `.status`'s own output;
/// post-fix it must show `.list`'s help text before `.status`'s own output.
///
/// ## Validation Strategy
/// Pipe two commands plus `exit` into `--repl` with `CLAUDE_STORAGE_ROOT`
/// pointed at an empty temp dir (deterministic `.status` output). Assert
/// both a `.list` help marker and `.status`'s own "Storage:" marker appear
/// in stdout.
///
/// ## Related Requirements
/// `task/claude_storage/bug/completed/005_space_form_command_help_misparsed.md`
#[ test ]
fn t02_list_help_repl_intercepted()
{
  let root = tempfile::TempDir::new().unwrap();
  let mut cmd = common::clg_cmd();
  cmd.env( "CLAUDE_STORAGE_ROOT", root.path() );

  let out = repl_run( &mut cmd, ".list help\n.status\nexit\n" );
  let s = stdout( &out );

  assert!(
    s.contains( "Usage: clg .list" ),
    "T02: `.list help` must render .list's help text in REPL; got stdout:\n{s}"
  );
  assert!(
    s.contains( "Storage:" ),
    "T02: `.status` must still execute after `.list help` (loop must `continue`, not fall through or stop); got stdout:\n{s}"
  );
}

/// T03: `clg .show help` one-shot is intercepted and renders `.show`'s
/// styled detail help, byte-for-byte identical to `clg .show.help` — proves
/// the fix generalizes beyond `.list`.
///
/// ## Purpose
/// Prove the interceptor is command-agnostic, not `.list`-specific. `.show`
/// is chosen deliberately: its first positional argument is
/// `session_id::String`, so pre-fix, `help` silently binds as a session ID
/// to search for instead of producing an obvious type/parse error — the
/// SILENT-misbehavior failure mode named in the Executive Summary (contrast
/// T01's `.list`, which demonstrates the type-coercion-error failure mode).
///
/// Pre-fix live capture: `clg .show help` → exit 1, stderr
/// `Session 'help' not found in current directory projects`.
///
/// ## Coverage
/// `clg .show help` must match `clg .show.help` byte-for-byte on stdout and
/// exit code.
///
/// ## Validation Strategy
/// Run both `.show help` and `.show.help` as separate one-shot invocations;
/// pin both exit codes to 0 and assert stdout is identical between the two.
///
/// ## Related Requirements
/// `task/claude_storage/bug/completed/005_space_form_command_help_misparsed.md`
#[ test ]
fn t03_show_help_one_shot_generality()
{
  let space_form = common::clg_cmd().arg( ".show" ).arg( "help" ).output().unwrap();
  let dot_form = common::clg_cmd().arg( ".show.help" ).output().unwrap();

  assert_exit( &dot_form, 0 );
  assert_exit( &space_form, 0 );
  assert_eq!(
    stdout( &space_form ), stdout( &dot_form ),
    "`.show help` must render byte-identical to `.show.help`;\n  space-form:\n{}\n  dot-form:\n{}",
    stdout( &space_form ), stdout( &dot_form )
  );
}

/// T04: `clg .nonexistent help` (unregistered command) is unaffected by the
/// interceptor and remains byte-for-byte identical to the Phase 0 baseline.
///
/// ## Purpose
/// Prove the interceptor does not over-match: an unregistered command name
/// followed by `help` must still fall through to unilang's own
/// command-not-found handling, unchanged by this fix.
///
/// ## Coverage
/// `clg .nonexistent help` — two tokens, first token not a registered
/// command.
///
/// ## Validation Strategy
/// Live pre-fix capture: `clg .nonexistent help` → exit 1, stderr `Command
/// Error: The command '.nonexistent' was not found. Use '.' to see all
/// available commands or check for typos.` Assert this pre-existing message
/// is unchanged (never intercepted).
///
/// ## Related Requirements
/// `task/claude_storage/bug/completed/005_space_form_command_help_misparsed.md`
#[ test ]
fn t04_nonexistent_help_unchanged()
{
  let out = common::clg_cmd().arg( ".nonexistent" ).arg( "help" ).output().unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "The command '.nonexistent' was not found" ),
    "T04: `.nonexistent help` must remain unintercepted (unregistered command falls through); got stderr:\n{}",
    stderr( &out )
  );
}

/// T05: `clg .list uuid help` (three tokens) is unaffected by the
/// interceptor and remains byte-for-byte identical to the Phase 0 baseline.
///
/// ## Purpose
/// Prove the interceptor does not over-match: a three-token invocation
/// (command + positional value + `help`) must still fall through to
/// unilang's ordinary positional-argument binding, unchanged by this fix —
/// the interceptor only fires on the exact two-token `[<command>, "help"]`
/// shape.
///
/// ## Coverage
/// `clg .list uuid help` — three tokens.
///
/// ## Validation Strategy
/// Live pre-fix capture: `clg .list uuid help` → exit 1, stderr `Argument
/// Error: Cannot coerce value for argument 'show_sessions' to Boolean.
/// Invalid boolean value` (the third token binds positionally to the next
/// unfilled parameter rather than triggering help). Assert this pre-existing
/// message is unchanged (never intercepted).
///
/// ## Related Requirements
/// `task/claude_storage/bug/completed/005_space_form_command_help_misparsed.md`
#[ test ]
fn t05_list_uuid_help_three_token_unchanged()
{
  let out = common::clg_cmd().arg( ".list" ).arg( "uuid" ).arg( "help" ).output().unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "Cannot coerce value for argument 'show_sessions' to Boolean" ),
    "T05: `.list uuid help` must remain unintercepted (three-token positional binding); got stderr:\n{}",
    stderr( &out )
  );
}

/// T06: `clg .list HELP` (uppercase) is unaffected by the interceptor — the
/// match against the literal `"help"` token is case-sensitive by design.
///
/// ## Purpose
/// Prove the interceptor's token match is exact and case-sensitive, not a
/// case-insensitive `eq_ignore_ascii_case` — `HELP` must never trigger help
/// rendering.
///
/// ## Coverage
/// `clg .list HELP` — two tokens, second token differs only in case from the
/// trigger literal.
///
/// ## Validation Strategy
/// Live pre-fix capture: `clg .list HELP` → exit 1, stderr `Parse error:
/// Syntax("Unexpected token 'HELP' in arguments") at StrSpan { start: 6, end:
/// 10 }`. This differs from the lowercase case's `Invalid type: help...`
/// because unilang treats `HELP` as an unrecognized token at the parse
/// stage, not a value-coercion attempt. Assert this pre-existing message is
/// unchanged (never intercepted).
///
/// ## Related Requirements
/// `task/claude_storage/bug/completed/005_space_form_command_help_misparsed.md`
#[ test ]
fn t06_list_help_uppercase_unchanged()
{
  let out = common::clg_cmd().arg( ".list" ).arg( "HELP" ).output().unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "Unexpected token 'HELP'" ),
    "T06: `.list HELP` must remain unintercepted (case-sensitive match); got stderr:\n{}",
    stderr( &out )
  );
}

/// T07: `clg .list helpme` (content near-miss) is unaffected by the
/// interceptor — the match requires the token to equal exactly `"help"`, not
/// merely start with or contain it.
///
/// ## Purpose
/// Prove the interceptor's token match is exact-equality, not a prefix or
/// substring check — `helpme` must never trigger help rendering.
///
/// ## Coverage
/// `clg .list helpme` — two tokens, second token is a near-miss superstring
/// of the trigger literal.
///
/// ## Validation Strategy
/// Live pre-fix capture: `clg .list helpme` → exit 1, stderr `Invalid type:
/// helpme. Valid values: uuid, path, all`. Assert this pre-existing message
/// is unchanged (never intercepted).
///
/// ## Related Requirements
/// `task/claude_storage/bug/completed/005_space_form_command_help_misparsed.md`
#[ test ]
fn t07_list_helpme_content_near_miss_unchanged()
{
  let out = common::clg_cmd().arg( ".list" ).arg( "helpme" ).output().unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "Invalid type: helpme" ),
    "T07: `.list helpme` must remain unintercepted (exact-match only); got stderr:\n{}",
    stderr( &out )
  );
}

/// T08: REPL input with irregular whitespace between command and `help`
/// (double space, e.g. `.list  help`) is intercepted exactly like the
/// single-space form — proves the fix tokenizes via `split_whitespace()`
/// rather than a naive single-`' '` split that would leave an empty token
/// and miss the match.
///
/// ## Purpose
/// Prove the fix is robust to irregular REPL whitespace rather than
/// reproducing this plan's own target bug for whitespace-irregular input.
///
/// ## Coverage
/// REPL input `.list  help\nexit\n` (two spaces). Pre-fix this reproduces
/// the same defect as T01/T02 (`Invalid type: help. Valid values: uuid,
/// path, all` on stderr, confirmed live).
///
/// ## Validation Strategy
/// Pipe the irregular-whitespace line plus `exit` into `--repl`; assert the
/// `.list` help marker appears on stdout and the pre-fix error text does not
/// appear on stderr.
///
/// ## Related Requirements
/// `task/claude_storage/bug/completed/005_space_form_command_help_misparsed.md`
#[ test ]
// bug_reproducer(BUG-005)
fn t08_repl_irregular_whitespace_intercepted()
{
  let mut cmd = common::clg_cmd();
  let out = repl_run( &mut cmd, ".list  help\nexit\n" );

  assert!(
    stdout( &out ).contains( "Usage: clg .list" ),
    "T08: `.list  help` (irregular whitespace) must render .list's help text in REPL; got stdout:\n{}",
    stdout( &out )
  );
  assert!(
    !stderr( &out ).contains( "Invalid type: help" ),
    "T08: must not fall through to the pre-fix dispatch error; got stderr:\n{}",
    stderr( &out )
  );
}

/// T09: `clg .search help` (space-form) is intercepted and renders `.search`'s
/// styled detail help rather than performing a literal search — the accepted
/// AGG-01 trade-off (Option B: reserved-word convention, matching npm/git/
/// kubectl's own treatment of a bare `help` token).
///
/// ## Purpose
/// Lock in AGG-01's resolution as tested, intentional behavior: any
/// registered command's bare `<command> help` form renders help, even for
/// commands (like `.search`) whose first positional argument is an
/// unconstrained string where "help" could otherwise be a legitimate value.
///
/// ## Coverage
/// `clg .search help` — proves the space-form interceptor's match condition
/// applies uniformly across all registered commands, not only `.list`/`.show`.
///
/// ## Validation Strategy
/// Run `.search help` as a one-shot invocation; assert exit 0 and that
/// stdout matches `.search.help`'s own dot-suffix rendering byte-for-byte,
/// the same generality proof T03 uses for `.show`.
///
/// ## Related Requirements
/// `-plan/verified/002_intercept_spaced_command_help_dispatch.plan.md § Known Deferred Risk` (AGG-01)
#[ test ]
fn t09_search_help_one_shot_reserved_word_tradeoff()
{
  let space_form = common::clg_cmd().arg( ".search" ).arg( "help" ).output().unwrap();
  let dot_form = common::clg_cmd().arg( ".search.help" ).output().unwrap();

  assert_exit( &dot_form, 0 );
  assert_exit( &space_form, 0 );
  assert_eq!(
    stdout( &space_form ), stdout( &dot_form ),
    "`.search help` must render byte-identical to `.search.help` (AGG-01 accepted trade-off);\n  space-form:\n{}\n  dot-form:\n{}",
    stdout( &space_form ), stdout( &dot_form )
  );
}

/// T10: `clg .search query::help` (named parameter, not bare positional) is
/// unaffected by the space-form interceptor and still performs a literal
/// search for "help" — the AGG-01 boundary that keeps searching for the word
/// "help" possible via the named-parameter form.
///
/// ## Purpose
/// Prove the interceptor's two-token match is narrow: only a bare second
/// token literally equal to `"help"` triggers interception. `query::help` is
/// one token (`"query::help"`), never equal to `"help"`, so it falls through
/// to ordinary dispatch exactly as before this plan's fix.
///
/// ## Coverage
/// `clg .search query::help` — the boundary case establishing that AGG-01's
/// accepted trade-off does not remove the ability to search for "help", only
/// the bare-positional shorthand for doing so.
///
/// ## Validation Strategy
/// Run in an isolated empty `CLAUDE_STORAGE_ROOT`; assert exit 0 and that
/// stdout reflects a real search result (contains "matches"), not `.search`'s
/// help screen (does not contain the `"Usage:"` marker).
///
/// ## Related Requirements
/// `-plan/verified/002_intercept_spaced_command_help_dispatch.plan.md § Known Deferred Risk` (AGG-01)
#[ test ]
fn t10_search_query_help_named_param_unaffected()
{
  let root = tempfile::TempDir::new().unwrap();
  let out = common::clg_cmd()
    .arg( ".search" )
    .arg( "query::help" )
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    !stdout( &out ).contains( "Usage:" ),
    "T10: `.search query::help` must not render help; got stdout:\n{}",
    stdout( &out )
  );
  assert!(
    stdout( &out ).contains( "matches" ),
    "T10: `.search query::help` must still perform a real search; got stdout:\n{}",
    stdout( &out )
  );
}
