//! Dry-Run Output Structure Tests
//!
//! ## Purpose
//!
//! Verify that `--dry-run` mode produces correctly structured output:
//! environment variable lines followed by the command line.
//! Tests inspect the output format without executing Claude Code.
//!
//! ## Strategy
//!
//! Each test invokes `clr --dry-run` with specific flags and
//! asserts that the printed output reflects the expected builder configuration.
//! This validates the round-trip: `--flag value` CLI → builder call → describe output.
//!
//! ## Corner Cases Covered
//!
//! - Default env vars appear (`CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000`)
//! - Default `-c` appears in dry-run output when session storage is non-empty (automatic session continuation)
//! - Empty session storage (fresh `CLAUDE_HOME`) suppresses `-c` even without `--new-session` (BUG-214 regression guard)
//! - `--new-session` suppresses `-c` from dry-run output
//! - `--dir` emits `cd <path>` prefix line
//! - `--max-tokens N` overrides the default token env var
//! - `--model NAME` appears in command args
//! - `--session-dir PATH` is deprecated and inert (BUG-493) — no env var, warns on stderr
//! - Combined flags produce correct combined output (no explicit `-c` needed)
//! - Message becomes quoted in command output — FR-1
//! - Message with embedded double quotes is properly escaped
//! - `--dir` with spaces: `cd` output is unquoted (human-readable per FR-21, not shell-safe)
//! - All 5 Tier-1 default env vars appear in output (not just max-tokens)
//! - No message provided: `--dry-run` routes to print mode (non-TTY stdin) with no message arg and no `-c` (BUG-425/BUG-426)
//! - `--dry-run --quiet` still shows output (--quiet does not gate dry-run; bug reproducer)
//! - `--system-prompt TEXT` appears in command args (param 15 round-trip)
//! - `--append-system-prompt TEXT` appears in command args (param 16 round-trip)
//! - Both system-prompt flags can appear together in a single invocation
//! - `--help` output lists both `--system-prompt` and `--append-system-prompt`
//! - `"\n\nultrathink"` suffix applied to message by default
//! - `--no-ultrathink` suppresses `"\n\nultrathink"` suffix in dry-run output
//! - Idempotent guard: message ending with `"ultrathink"` is not double-suffixed
//! - `--trace --dry-run` emits nothing to stderr (dry-run returns before trace fires)
//! - `""` empty positional arg ignored — bare command, no message, no degenerate ultrathink suffix
//! - T-A: `--interactive --from <session with history>` injects `-c` (BUG-435 fix)
//! - T-B: `--interactive --from <no session>` does NOT inject `-c` (no session → no -c)
//! - T-C: bare `--dry-run` with a prior session in storage (print mode, no message) does NOT inject `-c` (D-10 preserved)

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ make_session_for, run_cli, run_cli_with_env, run_dry, stdout_str };
use std::process::Command;

#[ test ]
fn default_env_vars_appear_in_output()
{
  let output = run_dry( &[ "test" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000" ),
    "Default 128K token limit must appear in env output. Got:\n{output}"
  );
}

#[ test ]
fn working_dir_emits_cd_prefix()
{
  let output = run_dry( &[ "--dir", "/tmp/work", "test" ] );
  assert!(
    output.contains( "cd /tmp/work" ),
    "--dir must produce 'cd <path>' prefix. Got:\n{output}"
  );
}

#[ test ]
fn max_tokens_override_updates_env_var()
{
  let output = run_dry( &[ "--max-tokens", "100000", "test" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=100000" ),
    "--max-tokens must override default. Got:\n{output}"
  );
  assert!(
    !output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000" ),
    "Default 128K must be replaced. Got:\n{output}"
  );
}

#[ test ]
fn model_param_appears_in_command()
{
  let output = run_dry( &[ "--model", "claude-opus-4-8", "test" ] );
  assert!(
    output.contains( "claude-opus-4-8" ),
    "--model must appear in command line. Got:\n{output}"
  );
}

#[ test ]
fn session_dir_no_longer_sets_env_var()
{
  let out = run_cli( &[ "--dry-run", "--session-dir", "/tmp/sessions", "test" ] );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "--session-dir is deprecated and inert — must never set the env var. Got:\n{stdout}"
  );
  assert!(
    stderr.contains( "deprecated" ) && stderr.contains( "/tmp/sessions" ),
    "--session-dir must emit a deprecation warning naming the value. Got:\n{stderr}"
  );
}

#[ test ]
fn message_appears_in_command_quoted()
{
  let output = run_dry( &[ "hello world" ] );
  assert!(
    output.contains( "\"hello world\n\nultrathink\"" ),
    "Message must appear with ultrathink suffix and quoted. Got:\n{output}"
  );
}

#[ test ]
fn combined_flags_all_appear()
{
  // --dangerously-skip-permissions appears automatically (default-on; no explicit flag needed).
  // Note: -c is NOT checked here — /tmp has no prior Claude session so session_exists() returns
  // `None`. The -c default is covered by default_continuation_always_present (same cwd as project).
  let output = run_dry( &[
    "--dir", "/tmp", "fix it",
  ] );
  assert!( output.contains( "cd /tmp" ), "Must have cd line" );
  assert!( output.contains( "--dangerously-skip-permissions" ), "Must have skip-permissions (default)" );
  // Note: -c is omitted because /tmp has no prior Claude session; session_exists() uses
  // project-specific storage ($HOME/.claude/projects/{encoded(/tmp)}/), not the global dir.
  // Use a temp dir with a dummy session file if -c injection needs to be tested (see t10).
  assert!( output.contains( "--effort max" ), "Must have --effort max (default). Got:\n{output}" );
  assert!( output.contains( "\"fix it\n\nultrathink\"" ), "Must have ultrathink-suffixed quoted message" );
}

#[ test ]
fn dry_run_does_not_invoke_claude_binary()
{
  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
  .args( [ "--dry-run", "test" ] )
  .output()
  .expect( "Failed to invoke binary" );
  assert!(
    out.status.success(),
    "Dry-run must not fail due to missing claude binary"
  );
}

// FR-1: message text appears quoted in command output.
#[ test ]
fn message_param_appears_in_command()
{
  let output = run_dry( &[ "Hello there" ] );
  assert!(
    output.contains( "\"Hello there\n\nultrathink\"" ),
    "message must appear with ultrathink suffix and quoted. Got:\n{output}"
  );
}

// FR-1: message containing double quotes must be escaped in describe() output.
#[ test ]
fn message_with_embedded_quotes_is_escaped()
{
  let output = run_dry( &[ r#"say "hi""# ] );
  assert!(
    output.contains( r#"\"hi\""# ),
    "Embedded double quotes must be escaped. Got:\n{output}"
  );
}

#[ test ]
fn dir_param_produces_cd_prefix()
{
  let output = run_dry( &[ "--dir", "/tmp/mydir", "test" ] );
  assert!(
    output.contains( "cd /tmp/mydir" ),
    "--dir must produce 'cd <path>' prefix. Got:\n{output}"
  );
}

// FR-21: dir with spaces: cd output is unquoted (human-readable, not shell-safe).
#[ test ]
fn dir_with_spaces_produces_unquoted_cd_line()
{
  let output = run_dry( &[ "--dir", "/path/with spaces", "test" ] );
  assert!(
    output.contains( "cd /path/with spaces" ),
    "Path with spaces must appear unquoted in cd line (FR-21). Got:\n{output}"
  );
}

// No-message case: --dry-run with no message routes to print mode under this harness's
// non-TTY subprocess stdin, but still WITHOUT -c because the fresh, empty CLAUDE_HOME
// guarantees session_exists() returns `None`. Fix(BUG-246): describe() now starts with
// "env -u CLAUDECODE" (default unset_claudecode=true).
// Fix(BUG-538): isolation migrated from the inert --session-dir override (BUG-493) to an
//   empty temp CLAUDE_HOME — the no-session guarantee must live in the storage
//   session_exists() actually reads. Do NOT seed a session here (make_session_for());
//   that would inject -c and break the "no -c" assertion.
//
// Fix(BUG-425): corrected from asserting a bare/no-`--print` command to asserting the
//   post-fix print-mode-routed command — this test's own subprocess stdin is non-TTY
//   (no PTY simulation in this harness), and BUG-425's fix makes non-TTY the deciding
//   term for a bare invocation with no message and no `--interactive`.
// Root cause: this test was written before BUG-425's TTY-check term existed, when
//   "no message" alone meant the bare/interactive-REPL command shape.
// Pitfall: `--chrome` also drops from the composed command here — print mode suppresses
//   it unconditionally (Fix(BUG-304)), not just when explicitly requested via --no-chrome.
#[ test ]
fn dry_run_without_message_shows_bare_command()
{
  let claude_home = tempfile::TempDir::new().expect( "create empty claude home" );
  let claude_home_str = claude_home.path().to_str().expect( "claude home path valid utf-8" );
  // Fix(BUG-008) isolation: `CLAUDE_HOME` alone does not isolate this assertion — the composed
  //   command's `--model` term resolves out of `$HOME/.claude.json`, not `$CLAUDE_HOME`. Pointing
  //   HOME at the same empty temp dir is the pattern already used at the `--continue` test below.
  // Root cause: on a host whose `~/.claude.json` carries a `"model"` key, the bare invocation
  //   gains `--model <id>` between `--effort` and `--print`, and this exact-string assertion fails.
  // Pitfall: the container hides this — its `$HOME` has no such key, so the gap stays invisible
  //   until someone runs the suite on a real workstation via the `VERB_LAYER=l0` escape hatch.
  let out = run_cli_with_env(
    &[ "--dry-run" ],
    &[ ( "CLAUDE_HOME", claude_home_str ), ( "HOME", claude_home_str ) ],
  );
  let output = stdout_str( &out );
  let last_line = output.trim_end().lines().last().unwrap_or_default();
  assert_eq!(
    last_line, "env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude --dangerously-skip-permissions --effort max --print --output-format json",
    "Bare --dry-run under non-TTY stdin must route to print mode (no message, no -c with no prior session). Got:\n{output}"
  );
}

// --new-session suppresses -c from dry-run output.
#[ test ]
fn new_session_suppresses_continue_flag()
{
  let output = run_dry( &[ "--new-session", "test" ] );
  assert!(
    !output.contains( " -c" ),
    "--new-session must suppress -c in dry-run output. Got:\n{output}"
  );
}

// Continuation: --dry-run shows -c when the --from source storage has a prior session.
// Canonical reproducer for BUG-538 — the migration of the 13 test call sites that BUG-493's
// execution left relying on the inert --session-dir override (6 of them suite-failing).
// test_kind: bug_reproducer(BUG-538)
//
// ## Root Cause
// BUG-493 made --session-dir fully inert — session_exists() stopped scanning the override
// dir and now reads only scope_for(--from|cwd).claude_session_dir, resolved under
// CLAUDE_HOME. This test and 12 sibling call sites (cli_args_test T04/T10, the BUG-428
// quintet in execution_mode_ext_test, three user_story fixtures, two empty-dir isolation
// fixtures here) still relied on --session-dir as a fixture — seeding a raw temp directory,
// or passing an empty one as a no-session guarantee: the six that assert -c-injection or
// resume behavior pinned the REMOVED contract and failed against the correct post-493
// binary; the other seven passed by accident with dead or ambient-dependent fixtures.
//
// ## Why Not Caught
// BUG-493's execution migrated the tests that assert the deprecation contract itself and
// retitled the user-story -c tests, but no mechanical sweep enumerated every
// "--session-dir as fixture trigger" call site — and the full claude_runner suite was not
// re-run in that window, so the six failures only surfaced at the next full gate
// (verb/-0040_longrun.log, exit 1, 1380/1386).
//
// ## Fix Applied
// Every remaining fixture call site migrated to the documented successor mechanism:
// make_session_for()/make_zero_turn_session_for() seed
// <CLAUDE_HOME>/projects/<encoded src>/, and the invocation carries --from <src> plus a
// CLAUDE_HOME override pointing at the temp home. The caller-less make_session_dir()/
// make_zero_turn_session_dir() helpers are deleted (Delete, Don't Archive).
//
// ## Prevention
// When a parameter is deprecated to inert, grep the test tree for the flag string and
// triage EVERY call site into "asserts the deprecation contract" vs "uses the flag as a
// fixture trigger" — the second class breaks (or silently de-scopes coverage) only later,
// at full-suite time, because nothing at migration time links it to the contract change.
//
// ## Pitfall
// A test asserting -c injection must seed the storage session_exists() actually reads
// (CLAUDE_HOME + --from|cwd encoding) — seeding an arbitrary directory and passing its raw
// path is exactly the inert mechanism BUG-493 removed, and it fails silently: the dead
// fixture still "looks like" session setup while contributing nothing.
#[ test ]
fn continuation_present_when_prior_session_exists()
{
  let claude_home = tempfile::tempdir().expect( "create temp claude home" );
  let src = "/tmp/bug538-continuation-src";
  let _jsonl = make_session_for( claude_home.path(), src, "00000000-0000-0000-0000-000000000000" );
  let claude_home_str = claude_home.path().to_str().expect( "claude home path is valid utf-8" );
  let out = run_cli_with_env(
    &[ "--dry-run", "--from", src, "test" ],
    &[ ( "CLAUDE_HOME", claude_home_str ) ],
  );
  let output = stdout_str( &out );
  assert!(
    output.contains( " -c" ),
    "prior session in the --from source storage must inject -c. Got:\n{output}"
  );
}

// Tier-1 automation defaults: all four remaining env vars must appear alongside max-tokens.
#[ test ]
fn tier1_default_env_vars_all_appear()
{
  let output = run_dry( &[ "test" ] );
  for var in &[
    "CLAUDE_CODE_BASH_TIMEOUT=3600000",
    "CLAUDE_CODE_BASH_MAX_TIMEOUT=7200000",
    "CLAUDE_CODE_AUTO_CONTINUE=true",
    "CLAUDE_CODE_TELEMETRY=false",
  ]
  {
    assert!(
      output.contains( var ),
      "Tier-1 default env var missing: {var}. Got:\n{output}"
    );
  }
}

// --print flag appears in dry-run output.
#[ test ]
fn print_flag_appears_in_dry_run()
{
  let output = run_dry( &[ "-p", "test" ] );
  assert!(
    output.contains( "--print" ),
    "-p must add --print to command in dry-run output. Got:\n{output}"
  );
}

// --verbose flag appears in dry-run output (passed through to claude).
#[ test ]
fn verbose_flag_appears_in_dry_run()
{
  let output = run_dry( &[ "--verbose", "test" ] );
  assert!(
    output.contains( "--verbose" ),
    "--verbose must appear in command in dry-run output. Got:\n{output}"
  );
}

// Message without -p defaults to --print (print mode is default when message given).
#[ test ]
fn message_without_print_flag_defaults_to_print_mode()
{
  let output = run_dry( &[ "Fix the bug" ] );
  assert!(
    output.contains( "--print" ),
    "message without -p must default to --print in dry-run output. Got:\n{output}"
  );
}

// --interactive with message suppresses the default --print.
#[ test ]
fn interactive_flag_suppresses_default_print()
{
  let output = run_dry( &[ "--interactive", "Fix the bug" ] );
  assert!(
    !output.contains( "--print" ),
    "--interactive must suppress --print default in dry-run output. Got:\n{output}"
  );
}

// Fix(BUG-425): retitled from `bare_dry_run_no_message_has_no_print` and inverted the
//   assertion — this test's subprocess stdin is non-TTY (no PTY simulation in this
//   harness), and BUG-425's fix makes non-TTY the deciding term for a bare invocation
//   with no message and no `--interactive`, same as a piped invocation.
// Root cause: this test predates BUG-425's TTY-check term, when "no message" alone
//   meant the bare/interactive-REPL command shape regardless of TTY presence.
// Pitfall: a genuine TTY (not reachable in this harness) would still route to
//   run_interactive() for this same bare invocation — this test only covers the
//   non-TTY case, matching every other subprocess-spawning test in this suite.
#[ test ]
fn bare_dry_run_no_message_routes_to_print()
{
  let output = run_dry( &[] );
  assert!(
    output.contains( "--print" ),
    "bare --dry-run (no message) under non-TTY stdin must add --print. Got:\n{output}"
  );
}

// Bug reproducer: --dry-run output must appear even with --quiet.
//
// ## Root Cause
//
// `handle_dry_run()` previously gated output on `verbosity.shows_progress()` (level ≥ 3).
// At low verbosity, the entire output block was skipped — `--dry-run` produced empty
// stdout with no indication that anything had been previewed.
//
// ## Fix Applied
//
// Removed the verbosity guard from `handle_dry_run`. `--quiet` controls runner
// *diagnostics* (retry warnings, gate-wait messages); `--dry-run` output is core
// functionality that the user explicitly requested and must never be suppressed.
//
// ## Pitfall
//
// Do not confuse runner diagnostic suppression with feature output. `--quiet`
// suppresses runner messages; it must never suppress the command the user asked to see.
#[ test ]
fn dry_run_output_appears_with_quiet()
{
  let output = run_dry( &[ "--quiet", "test" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--dry-run --quiet must still show env+command output. Got:\n{output}"
  );
  assert!(
    output.contains( "claude " ),
    "--dry-run --quiet must still show the claude command line. Got:\n{output}"
  );
}

// --system-prompt value round-trips through dry-run output.
// The flag and its text must appear verbatim in the assembled command.
#[ test ]
fn system_prompt_flag_round_trip()
{
  let output = run_dry( &[ "--system-prompt", "Be concise.", "test" ] );
  assert!(
    output.contains( "--system-prompt" ),
    "--system-prompt must appear in dry-run command output. Got:\n{output}"
  );
  assert!(
    output.contains( "Be concise." ),
    "--system-prompt value must appear in dry-run output. Got:\n{output}"
  );
}

// --append-system-prompt value round-trips through dry-run output.
#[ test ]
fn append_system_prompt_flag_round_trip()
{
  let output = run_dry( &[ "--append-system-prompt", "Always respond in JSON.", "test" ] );
  assert!(
    output.contains( "--append-system-prompt" ),
    "--append-system-prompt must appear in dry-run command output. Got:\n{output}"
  );
  assert!(
    output.contains( "Always respond in JSON." ),
    "--append-system-prompt value must appear in dry-run output. Got:\n{output}"
  );
}

// Both system-prompt flags may appear together in a single invocation.
#[ test ]
fn both_system_prompt_flags_together()
{
  let output = run_dry( &[
    "--system-prompt", "You are a Rust expert.",
    "--append-system-prompt", "Be concise.",
    "test",
  ] );
  assert!(
    output.contains( "--system-prompt" ),
    "--system-prompt must appear when both flags given. Got:\n{output}"
  );
  assert!(
    output.contains( "--append-system-prompt" ),
    "--append-system-prompt must appear when both flags given. Got:\n{output}"
  );
}

// --help output must list --system-prompt and --append-system-prompt.
#[ test ]
fn help_shows_system_prompt_flags()
{
  let output = stdout_str( &run_cli( &[ "--help" ] ) );
  assert!(
    output.contains( "--system-prompt" ),
    "--help must mention --system-prompt. Got:\n{output}"
  );
  assert!(
    output.contains( "--append-system-prompt" ),
    "--help must mention --append-system-prompt. Got:\n{output}"
  );
}

// Default "\n\nultrathink" suffix is applied to every message in dry-run output.
#[ test ]
fn ultrathink_suffix_default_on()
{
  let output = run_dry( &[ "fix the bug" ] );
  assert!(
    output.contains( "\"fix the bug\n\nultrathink\"" ),
    "message must be suffixed with \"\\n\\nultrathink\" by default. Got:\n{output}"
  );
}

// --no-ultrathink flag suppresses the default "\n\nultrathink" suffix.
#[ test ]
fn no_ultrathink_flag_suppresses_suffix()
{
  let out = run_cli( &[ "--dry-run", "--no-ultrathink", "fix the bug" ] );
  assert!(
    out.status.success(),
    "--no-ultrathink must be accepted (exit 0). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "\"fix the bug\"" ),
    "message must appear verbatim when --no-ultrathink given. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "ultrathink" ),
    "ultrathink suffix must be suppressed. Got:\n{stdout}"
  );
}

// Idempotent guard: message already ending with "ultrathink" is not double-suffixed.
#[ test ]
fn ultrathink_idempotent_guard()
{
  let output = run_dry( &[ "fix it ultrathink" ] );
  assert!(
    output.contains( "\"fix it ultrathink\"" ),
    "message must appear verbatim when already ending with ultrathink. Got:\n{output}"
  );
  assert!(
    !output.contains( "ultrathink\n\nultrathink" ),
    "double ultrathink suffix must not appear. Got:\n{output}"
  );
}

// --trace combined with --dry-run: dry-run wins; nothing appears on stderr.
//
// `handle_dry_run` returns before the trace output block fires, so stderr must be empty.
// Regression guard: if the control flow order is changed (trace moved before dry-run check),
// this catches the breakage.
#[ test ]
fn trace_with_dry_run_emits_no_stderr()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--trace", "--dry-run", "test" ] )
    .output()
    .expect( "Failed to invoke clr binary" );
  assert!( out.status.success(), "--trace --dry-run must exit 0" );
  assert!(
    out.stderr.is_empty(),
    "--trace must not emit to stderr when --dry-run wins. Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "claude " ),
    "--dry-run stdout output must still appear. Got:\n{stdout}"
  );
}

// BUG-214 reopen: bare `clr --dry-run` in a fresh directory (no --session-dir) injects -c
// because session_exists(None) fell back to $HOME/.claude/ which is always non-empty.
//
// ## Root Cause (bug_reproducer(BUG-214))
//
// The None branch of session_exists() checked $HOME/.claude/ (Claude's global config dir).
// That directory always has entries (credentials.json, projects/, etc.) regardless of whether
// the CURRENT project directory has any Claude session history.  Result: -c was unconditionally
// injected for every default invocation, causing "No conversation found to continue" in any
// directory without a prior session.
//
// ## Why Not Caught
//
// The existing BUG-214 MRE test always supplied --session-dir pointing to an empty temp dir.
// That case correctly exercises the Some(dir) branch which checks the custom dir directly.
// The None (no --session-dir) branch was never tested in isolation in a fresh cwd.
//
// ## Fix Applied
//
// session_exists(None, effective_dir) now calls
// claude_storage_core::continuation::most_recent_session_id(&cwd) which looks up
// $HOME/.claude/projects/{encoded(cwd)}/ — the project-specific storage — instead
// of the global $HOME/.claude/ directory.
//
// ## Prevention
//
// Test bare --dry-run in a fresh temp directory as the cwd; assert no -c.
// The session check must always use the project-specific path, not the global claude home.
//
// ## Pitfall
//
// $HOME/.claude/ is Claude's global config directory, not per-project session storage.
// Per-project sessions live at $HOME/.claude/projects/{encoded(project_dir)}/.
// Any check for "has prior session" must look at the encoded project path, not the global home.
//
// CLR_DIR env var (if set in the ambient shell) overrides the working directory used for session
// detection — it is inherited by subprocesses unless explicitly removed.  Always unset CLR_DIR
// and CLR_SESSION_DIR when spawning clr in tests that assert -c is NOT injected; otherwise the
// test fails whenever the host shell has CLR_DIR pointing to a directory with a prior session.
//
// test_kind: bug_reproducer(BUG-214)
#[ test ]
fn bug_reproducer_214_no_session_dir_fresh_cwd_no_continue_flag()
{
  // Run --dry-run from a fresh temp dir that has NO prior Claude session.
  // The session check must look at $HOME/.claude/projects/{encoded(tmp_dir)}/ which does not
  // exist, so -c must NOT appear in the output.
  //
  // CLR_DIR and CLR_SESSION_DIR are removed so the subprocess uses current_dir (tmp_dir)
  // for session detection instead of any ambient shell value.
  let tmp = tempfile::TempDir::new().expect( "create temp dir" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--dry-run", "Fix bug" ] )
    .current_dir( tmp.path() )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .output()
    .expect( "invoke clr --dry-run" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( " -c" ),
    "fresh cwd with no prior session must not inject -c (BUG-214 reopen). Got:\n{stdout}"
  );
}

// Empty positional arg `""` is ignored — bare command, no message, no degenerate ultrathink.
//
// Bug reproducer: before the fix, `clr ""` produced `"ultrathink "` (trailing space)
// as the message because the empty token was pushed to positional, joined to Some(""),
// and the ultrathink prefix fired unconditionally. See cli_args_test.rs T54 for the
// canonical reproducer with 5-section documentation.
// test_kind: bug_reproducer(BUG-219)
#[ test ]
fn empty_positional_arg_produces_bare_command()
{
  // Fresh empty CLAUDE_HOME → no prior session → no -c (session_exists returns `None`).
  // Fix(BUG-246): last_line now starts with "env -u CLAUDECODE" (default unset_claudecode=true).
  // Fix(BUG-538): isolation migrated from the inert --session-dir override (BUG-493) to
  //   an empty temp CLAUDE_HOME — the storage session_exists() actually reads.
  let claude_home = tempfile::TempDir::new().expect( "create empty claude home" );
  let claude_home_str = claude_home.path().to_str().expect( "claude home path valid utf-8" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--dry-run", "" ] )
    .env( "HOME", "/tmp/clr-isolated-home" ) // Fix(BUG-008) isolation: prevent host prefs from injecting --model
    .env( "CLAUDE_HOME", claude_home_str )
    .output()
    .expect( "Failed to invoke clr binary" );
  assert!( out.status.success(), "empty positional arg must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let last_line = stdout.trim_end().lines().last().unwrap_or_default();
  // Fix(BUG-425): "bare command" here means no message/-c content, not print-mode-free —
  //   this subprocess's stdin is non-TTY (no PTY simulation in this harness), so BUG-425's
  //   fix routes it to print mode same as any other bare invocation. The test's actual
  //   differentiator (empty positional must not leak as a degenerate "ultrathink " message,
  //   BUG-219) is the assertion immediately below, unaffected by this correction.
  assert_eq!(
    last_line, "env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude --dangerously-skip-permissions --effort max --print --output-format json",
    "empty positional arg must produce no-message command (no -c with no prior session). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "\"ultrathink \"" ),
    "empty positional must NOT produce 'ultrathink ' degenerate message. Got:\n{stdout}"
  );
}

// T-A: `--interactive --from <session with history>` injects `-c`
//
// ## Root Cause (BUG-435)
//
// D-10's -c injection guard (BUG-426 fix) required `cli.interactive` (the explicit
// `--interactive` flag) to be true when no message, print mode, file, or stdin-content
// was present.  Bare `clr` on TTY enters interactive mode via TTY detection (BUG-425
// fix), never by setting the explicit flag — so the guard excluded bare interactive
// invocations and sessions were never resumed.
//
// ## Why Not Caught
//
// D-10 was designed for BUG-426 (invalid `claude -c` with no message in print mode).
// In the test harness (non-TTY subprocess), every `--interactive` test that exercised
// -c injection used the explicit `--interactive` flag, which set `cli.interactive=true`
// and passed the existing guard.  Bare TTY interactive (no flag) was never tested.
//
// ## Fix Applied
//
// Added `!use_print` as the first inner term of the -c injection condition.  When
// `use_print=false` (interactive mode — either via explicit `--interactive` or via TTY
// detection), -c is always injected if `expected_id.is_some()`.
//
// ## Prevention
//
// Assert that `--interactive --from <src with history>` produces ` -c` in dry-run
// output.  In test context `--interactive` forces `use_print=false` (the same path
// real bare TTY invocations take via `!is_tty`).
//
// ## Pitfall
//
// In the test harness (non-TTY subprocess), bare `--dry-run` without `--interactive`
// routes to print mode (`use_print=true`); `!use_print=false`.  `--interactive` is
// required to reach the interactive path in the test harness.  Fix(BUG-493):
// `expected_id` now comes exclusively from `--from`'s resolved storage under
// `CLAUDE_HOME` — seed via `make_session_for()`, never raw `--session-dir` (inert).
// test_kind: bug_reproducer(BUG-435)
#[ test ]
fn ta_interactive_with_session_injects_continue_flag()
{
  let claude_home = tempfile::TempDir::new().expect( "create claude home" );
  let src = "/tmp/ta-interactive-with-session-src";
  let _jsonl = cli_binary_test_helpers::make_session_for( claude_home.path(), src, "11111111-1111-1111-1111-111111111111" );
  let claude_home_str = claude_home.path().to_str().expect( "claude home path valid utf-8" );
  let out = run_cli_with_env(
    &[ "--dry-run", "--interactive", "--from", src ],
    &[ ( "CLAUDE_HOME", claude_home_str ) ],
  );
  assert!(
    out.status.success(),
    "--interactive --from <session with history> dry-run must succeed. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( " -c" ),
    "--interactive with existing session must inject -c (BUG-435 fix). Got:\n{stdout}"
  );
}

// T-B: `--interactive --from <no session>` does NOT inject `-c`
//
// ## Root Cause (BUG-435)
//
// See T-A.  This companion test guards the no-session path: the fix must allow -c only
// when a qualifying session exists, never unconditionally in interactive mode.
//
// ## Why Not Caught
//
// The no-session path in interactive mode was passively correct before the fix (since
// `expected_id=None` regardless of mode), but was never explicitly asserted.  Adding
// T-A without T-B would leave a regression gap for the outer `expected_id.is_some()`
// guard.
//
// ## Fix Applied
//
// The `expected_id.is_some()` outer guard remains unchanged; `!use_print` only gates
// the inner condition.  With no seeded session, `session_exists()` returns `None`
// → -c is never injected.
//
// ## Prevention
//
// Assert that `--interactive --from <src with no history>` produces no ` -c` in
// dry-run output.
//
// ## Pitfall
//
// Always use a fresh, unseeded `CLAUDE_HOME` temp dir for the no-session case —
// never rely on the ambient host `~/.claude`, which may have prior sessions.
// Fix(BUG-493): `--session-dir` no longer provides isolation; `CLAUDE_HOME` does.
// test_kind: bug_reproducer(BUG-435)
#[ test ]
fn tb_interactive_without_session_no_continue_flag()
{
  let claude_home = tempfile::TempDir::new().expect( "create empty claude home" );
  let claude_home_str = claude_home.path().to_str().expect( "claude home path valid utf-8" );
  let out = run_cli_with_env(
    &[ "--dry-run", "--interactive", "--from", "/tmp/tb-interactive-no-session-src" ],
    &[ ( "CLAUDE_HOME", claude_home_str ) ],
  );
  assert!(
    out.status.success(),
    "--interactive --from <no session> dry-run must succeed. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( " -c" ),
    "--interactive with no existing session must NOT inject -c. Got:\n{stdout}"
  );
}

// T-C: bare print-mode dry-run with session → D-10 protection preserved (no -c)
//
// ## Root Cause (BUG-426, D-10 regression guard)
//
// Before D-10's BUG-426 fix, `claude -c` was injected even when no message would
// follow — causing the claude binary to fail.  D-10 guarded -c on message-presence
// (or other qualifying terms).  The BUG-435 fix adds `!use_print` as the first term;
// a mistake in that addition could accidentally let print mode with no message inject
// -c again, reintroducing BUG-426.
//
// ## Why Not Caught
//
// This test was not filed when D-10 landed; it is added now alongside the BUG-435 fix
// to ensure the `!use_print` addition cannot accidentally regress D-10.
//
// ## Fix Applied
//
// `!use_print` is false in print mode (`use_print=true`), so the existing
// message-presence terms still control the guard.  D-10's protection is unchanged.
//
// ## Prevention
//
// Assert that bare `--dry-run --from <src with history>` (no `--interactive`, no
// message) does NOT inject -c in dry-run output.
//
// ## Pitfall
//
// In test context (non-TTY subprocess), bare `--dry-run --from <src>` routes to
// print mode (`use_print=true`), so `!use_print=false`.  Do NOT add `--interactive`
// here — that changes the mode to interactive and would test T-A instead.
#[ test ]
fn tc_print_mode_with_session_no_continue_flag()
{
  let claude_home = tempfile::TempDir::new().expect( "create claude home" );
  let src = "/tmp/tc-print-mode-with-session-src";
  let _jsonl = cli_binary_test_helpers::make_session_for( claude_home.path(), src, "33333333-3333-3333-3333-333333333333" );
  let claude_home_str = claude_home.path().to_str().expect( "claude home path valid utf-8" );
  let out = run_cli_with_env(
    &[ "--dry-run", "--from", src ],
    &[ ( "CLAUDE_HOME", claude_home_str ) ],
  );
  assert!(
    out.status.success(),
    "bare --dry-run --from <session with history> must succeed. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( " -c" ),
    "print mode with no message must NOT inject -c even with existing session (D-10). Got:\n{stdout}"
  );
}
