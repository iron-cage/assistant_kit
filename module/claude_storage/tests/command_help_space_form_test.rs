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
//! - T11: `clg ".list help"` (single joined argv element) intercepted — MAAV Tier 5 Round 1 G1 gap
//! - T12: `clg ".list " "help"` / `clg ".list" "help "` (whitespace-corrupted separate argv) intercepted — MAAV Tier 5 Round 3 G1 gap
//! - T13: leading whitespace / tab-separated argv intercepted — MAAV Tier 5 Round 4 non-blocking finding
//! - T14: `clg ".show help"` (single joined argv element) intercepted — MAAV Tier 5 G4 non-blocking finding

mod common;

use std::io::Write;




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
/// dot-suffix baseline (`assert_eq!(common::stdout(&new_form), common::stdout(&dot_form))`),
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

  common::assert_exit( &dot_form, 0 );
  common::assert_exit( &space_form, 0 );
  assert_eq!(
    common::stdout( &space_form ), common::stdout( &dot_form ),
    "`.list help` must render byte-identical to `.list.help`;\n  space-form:\n{}\n  dot-form:\n{}",
    common::stdout( &space_form ), common::stdout( &dot_form )
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
  let s = common::stdout( &out );

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

  common::assert_exit( &dot_form, 0 );
  common::assert_exit( &space_form, 0 );
  assert_eq!(
    common::stdout( &space_form ), common::stdout( &dot_form ),
    "`.show help` must render byte-identical to `.show.help`;\n  space-form:\n{}\n  dot-form:\n{}",
    common::stdout( &space_form ), common::stdout( &dot_form )
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

  common::assert_exit( &out, 1 );
  assert!(
    common::stderr( &out ).contains( "The command '.nonexistent' was not found" ),
    "T04: `.nonexistent help` must remain unintercepted (unregistered command falls through); got stderr:\n{}",
    common::stderr( &out )
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

  common::assert_exit( &out, 1 );
  assert!(
    common::stderr( &out ).contains( "Cannot coerce value for argument 'show_sessions' to Boolean" ),
    "T05: `.list uuid help` must remain unintercepted (three-token positional binding); got stderr:\n{}",
    common::stderr( &out )
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

  common::assert_exit( &out, 1 );
  assert!(
    common::stderr( &out ).contains( "Unexpected token 'HELP'" ),
    "T06: `.list HELP` must remain unintercepted (case-sensitive match); got stderr:\n{}",
    common::stderr( &out )
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

  common::assert_exit( &out, 1 );
  assert!(
    common::stderr( &out ).contains( "Invalid type: helpme" ),
    "T07: `.list helpme` must remain unintercepted (exact-match only); got stderr:\n{}",
    common::stderr( &out )
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
  // Empty root: if the space-form interception regresses, `.list` falls through
  // and enumerates real storage. Isolating means that regression surfaces as the
  // assertion below failing, not as output that varies per machine.
  let root = tempfile::TempDir::new().unwrap();

  let mut cmd = common::clg_cmd();
  cmd.env( "CLAUDE_STORAGE_ROOT", root.path() );
  let out = repl_run( &mut cmd, ".list  help\nexit\n" );

  assert!(
    common::stdout( &out ).contains( "Usage: clg .list" ),
    "T08: `.list  help` (irregular whitespace) must render .list's help text in REPL; got stdout:\n{}",
    common::stdout( &out )
  );
  assert!(
    !common::stderr( &out ).contains( "Invalid type: help" ),
    "T08: must not fall through to the pre-fix dispatch error; got stderr:\n{}",
    common::stderr( &out )
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

  common::assert_exit( &dot_form, 0 );
  common::assert_exit( &space_form, 0 );
  assert_eq!(
    common::stdout( &space_form ), common::stdout( &dot_form ),
    "`.search help` must render byte-identical to `.search.help` (AGG-01 accepted trade-off);\n  space-form:\n{}\n  dot-form:\n{}",
    common::stdout( &space_form ), common::stdout( &dot_form )
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

  common::assert_exit( &out, 0 );
  assert!(
    !common::stdout( &out ).contains( "Usage:" ),
    "T10: `.search query::help` must not render help; got stdout:\n{}",
    common::stdout( &out )
  );
  assert!(
    common::stdout( &out ).contains( "matches" ),
    "T10: `.search query::help` must still perform a real search; got stdout:\n{}",
    common::stdout( &out )
  );
}

/// T11: `clg ".list help"` — a single shell-quoted argv element containing
/// both tokens joined by whitespace (`args.len() == 2`, distinct from T01's
/// two-separate-argv-elements shape) — is intercepted and renders `.list`'s
/// help, byte-for-byte identical to `.list.help` (MAAV Tier 5 Round 1 G1 gap,
/// found during adversarial re-verification of this file's own fix).
///
/// ## Root Cause
/// `execute_oneshot()`'s `args.len() == 2` branch only ever called
/// `try_command_help()` (the dot-suffix matcher) against `args[ 1 ]` as a
/// whole string. `try_command_help_space_form()` was only reachable from the
/// `args.len() == 3` branch (two separate argv elements), so a single quoted
/// argument like `clg ".list help"` fell through unintercepted — the same
/// underlying space-separated-help case T01 covers, reached through a
/// different argv shape.
///
/// ## Why Not Caught
/// T01-T10 invoke the space-form either as two separate `.arg()` calls
/// (`args.len() == 3`) or via REPL's single-line tokenization. None exercised
/// a single `.arg()` call whose own string content contains an embedded
/// space (`args.len() == 2`) — the shape a shell produces for `clg ".list
/// help"` (quoted) as opposed to `clg .list help` (unquoted).
///
/// ## Fix Applied
/// `execute_oneshot()`'s `args.len() == 2` branch now falls back to
/// `args[ 1 ].split_whitespace()` and retries `try_command_help_space_form()`
/// on the resulting tokens when the dot-suffix matcher declines, mirroring
/// how `run_repl()` already tokenizes its own input line.
///
/// ## Prevention
/// Any new argv shape reaching command dispatch (single joined arg, REPL
/// line, separate argv elements) needs its own Test Matrix entry proving
/// byte-identical output to the dot-suffix baseline — argv shape and
/// help-request semantics are orthogonal and must each be covered.
///
/// ## Pitfall
/// Don't assume `args.len() == 2` only ever means "one bare token" — a
/// shell-quoted multi-word argument collapses to a single argv element too,
/// so this branch must re-tokenize `args[ 1 ]` itself rather than treating
/// its length-2 arity as proof the space-form case can't apply here.
#[ test ]
// bug_reproducer(BUG-005)
fn t11_list_help_single_joined_argv_intercepted()
{
  let space_form = common::clg_cmd().arg( ".list help" ).output().unwrap();
  let dot_form = common::clg_cmd().arg( ".list.help" ).output().unwrap();

  common::assert_exit( &dot_form, 0 );
  common::assert_exit( &space_form, 0 );
  assert_eq!(
    common::stdout( &space_form ), common::stdout( &dot_form ),
    "`.list help` as a single joined argv element must render byte-identical to `.list.help`;\n  space-form:\n{}\n  dot-form:\n{}",
    common::stdout( &space_form ), common::stdout( &dot_form )
  );
}

/// T12: `clg ".list " "help"` and `clg ".list" "help "` — two separate argv
/// elements (the `args.len() == 3` shape T01 already covers) but with
/// trailing whitespace baked into one element — are both intercepted
/// and render byte-identical to `.list.help` (MAAV Tier 5 Round 3 G1 gap,
/// found during adversarial re-verification of T11's own fix).
///
/// ## Root Cause
/// `try_command_help_space_form()` compared `tokens[ 1 ] == "help"` and
/// looked up `tokens[ 0 ]` verbatim. The REPL path (`input.trim()` then
/// `split_whitespace()`) and the `args.len() == 2` fallback
/// (`split_whitespace()`) both normalize whitespace before calling it, but
/// the `args.len() == 3` call site passed its two argv elements straight
/// through unnormalized — so a caller building argv from unstripped strings
/// (a config-file line, a string-formatting bug) could carry a stray leading
/// or trailing space into either element and silently defeat the match.
///
/// ## Why Not Caught
/// T01/T04-T07 all use clean, pre-trimmed literal `&str` arguments. T11
/// covers the single-joined-argv shape but not this one. No test constructed
/// an `args.len() == 3` invocation with whitespace-corrupted individual
/// elements — a third, distinct argv shape from both T01's clean form and
/// T11's single-joined form.
///
/// ## Fix Applied
/// `try_command_help_space_form()` now trims both tokens before comparing:
/// `tokens[ 1 ].trim() != "help"` and `registry.command( tokens[ 0 ].trim() )`.
/// Fixed inside the shared matcher (not at the `args.len() == 3` call site)
/// so every call site gets the same tolerance uniformly.
///
/// ## Prevention
/// When multiple call sites feed a shared matcher, don't assume they all
/// normalize their input the same way just because some of them happen to —
/// push normalization into the matcher itself, or add a boundary test per
/// call site proving it does.
///
/// ## Pitfall
/// Trimming must stay whitespace-only, not case- or content-loosening —
/// `t06_list_help_uppercase_unchanged`/`t07_list_helpme_content_near_miss_unchanged`
/// still lock in exact-match-on-case-and-content as separate boundaries.
#[ test ]
// bug_reproducer(BUG-005)
fn t12_three_argv_whitespace_corrupted_intercepted()
{
  let trailing_space_tok0 = common::clg_cmd().arg( ".list " ).arg( "help" ).output().unwrap();
  let trailing_space_tok1 = common::clg_cmd().arg( ".list" ).arg( "help " ).output().unwrap();
  let dot_form = common::clg_cmd().arg( ".list.help" ).output().unwrap();

  common::assert_exit( &dot_form, 0 );
  common::assert_exit( &trailing_space_tok0, 0 );
  common::assert_exit( &trailing_space_tok1, 0 );
  assert_eq!(
    common::stdout( &trailing_space_tok0 ), common::stdout( &dot_form ),
    "`.list ` + `help` (trailing space in first element) must render byte-identical to `.list.help`;\n  got:\n{}\n  dot-form:\n{}",
    common::stdout( &trailing_space_tok0 ), common::stdout( &dot_form )
  );
  assert_eq!(
    common::stdout( &trailing_space_tok1 ), common::stdout( &dot_form ),
    "`.list` + `help ` (trailing space in second element) must render byte-identical to `.list.help`;\n  got:\n{}\n  dot-form:\n{}",
    common::stdout( &trailing_space_tok1 ), common::stdout( &dot_form )
  );
}

/// T13: leading whitespace before the command token, and a tab character in
/// place of a space, are both intercepted and render byte-identical to
/// `.list.help` (MAAV Tier 5 Round 4 non-blocking finding, raised
/// independently by both the Dimension Adversary and a Fresh Challenger:
/// T12 only asserted *trailing* whitespace on each token, leaving leading
/// whitespace and non-space whitespace characters unpinned even though the
/// underlying `.trim()` fix already covers both).
///
/// ## Root Cause
/// Same shared-matcher gap as T12 — before the Round 3 fix,
/// `try_command_help_space_form()` did no trimming at all. The Round 3 fix
/// (`.trim()` on both tokens) already generically covers leading whitespace
/// and any Unicode whitespace character, not just trailing ASCII spaces, but
/// no test asserted that — only trailing-space cases were pinned.
///
/// ## Why Not Caught
/// T12 was written directly from the adversary's two Round 3 counterexamples
/// (both trailing-space), so it pinned exactly those two shapes and no
/// others. Leading whitespace and tab characters are a different corner of
/// the same input space that happened not to be in the original
/// counterexample set.
///
/// ## Fix Applied
/// No code change — `.trim()` (Round 3 fix, `src/cli_main.rs:246`) already
/// strips leading whitespace and any Unicode whitespace character, including
/// tabs, by definition. This test only adds the missing pin.
///
/// ## Prevention
/// When a doc comment or module claim states a general property ("all
/// forms of whitespace"), verify the test suite actually asserts the general
/// case, not just the specific counterexamples that originally motivated it.
///
/// ## Pitfall
/// Keep this whitespace-only — do not fold in case or content variation;
/// those boundaries are T06/T07's responsibility, not this test's.
#[ test ]
fn t13_leading_whitespace_and_tab_intercepted()
{
  let leading_space = common::clg_cmd().arg( " .list" ).arg( "help" ).output().unwrap();
  let tab_separated = common::clg_cmd().arg( ".list" ).arg( "\thelp" ).output().unwrap();
  let dot_form = common::clg_cmd().arg( ".list.help" ).output().unwrap();

  common::assert_exit( &dot_form, 0 );
  common::assert_exit( &leading_space, 0 );
  common::assert_exit( &tab_separated, 0 );
  assert_eq!(
    common::stdout( &leading_space ), common::stdout( &dot_form ),
    "` .list` + `help` (leading space in first element) must render byte-identical to `.list.help`;\n  got:\n{}\n  dot-form:\n{}",
    common::stdout( &leading_space ), common::stdout( &dot_form )
  );
  assert_eq!(
    common::stdout( &tab_separated ), common::stdout( &dot_form ),
    "`.list` + `\\thelp` (tab-prefixed second element) must render byte-identical to `.list.help`;\n  got:\n{}\n  dot-form:\n{}",
    common::stdout( &tab_separated ), common::stdout( &dot_form )
  );
}

/// T14: `clg ".show help"` — the single-joined-argv shape T11 already proves
/// for `.list` — is intercepted and renders byte-identical to `.show.help`
/// (MAAV Tier 5 G4 non-blocking finding, raised in Round 1 by the Dimension
/// Adversary and reconfirmed live in Round 5: T11 only covers `.list`,
/// leaving `.show` — already proven generality for the two-separate-argv
/// shape by this file's own T03 — without an equivalent single-joined-argv
/// test).
///
/// ## Root Cause
/// T11 was written to close the `args.len() == 2` gap found during
/// adversarial re-verification, and used `.list` (this file's baseline
/// command) throughout. `execute_oneshot()`'s `args.len() == 2` fallback
/// re-tokenizes and retries `try_command_help_space_form()` for any
/// registered command, not just `.list`, so `.show` was already covered by
/// the fix — but no test asserted it.
///
/// ## Why Not Caught
/// T03 already proves `.show help` generality for the two-separate-argv
/// shape (`args.len() == 3`), so `.show`'s space-form coverage looked
/// complete at a glance. The single-joined-argv shape (`args.len() == 2`)
/// is a distinct argv path from `args.len() == 3`, and T03 predates T11 by
/// several MAAV rounds, so the two were never cross-checked against each
/// other.
///
/// ## Fix Applied
/// No code change — `execute_oneshot()`'s `args.len() == 2` fallback branch
/// (`src/cli_main.rs`) already calls `registry.command( tokens[ 0 ] )`
/// generically, with no `.list`-specific logic. This test only adds the
/// missing pin for a second registered command, proving the fix generalizes
/// rather than accidentally special-casing `.list`.
///
/// ## Prevention
/// When a regression test is written against one command to close a gap,
/// add a same-round or near-term test against at least one other registered
/// command exercising the identical code path — a single-command pin cannot
/// distinguish "fixed generically" from "fixed for this command only."
///
/// ## Pitfall
/// Keep this scoped to the single-joined-argv shape only — T03 already owns
/// the two-separate-argv generality proof for `.show`; duplicating that
/// coverage here would be redundant, not additive.
#[ test ]
fn t14_show_help_single_joined_argv_intercepted()
{
  let space_form = common::clg_cmd().arg( ".show help" ).output().unwrap();
  let dot_form = common::clg_cmd().arg( ".show.help" ).output().unwrap();

  common::assert_exit( &dot_form, 0 );
  common::assert_exit( &space_form, 0 );
  assert_eq!(
    common::stdout( &space_form ), common::stdout( &dot_form ),
    "`.show help` as a single joined argv element must render byte-identical to `.show.help`;\n  space-form:\n{}\n  dot-form:\n{}",
    common::stdout( &space_form ), common::stdout( &dot_form )
  );
}
