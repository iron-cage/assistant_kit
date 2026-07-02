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
//! - Default env vars appear (`CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`)
//! - Default `-c` appears in dry-run output when session storage is non-empty (automatic session continuation)
//! - Empty `--session-dir` suppresses `-c` even without `--new-session` (BUG-214 regression guard)
//! - `--new-session` suppresses `-c` from dry-run output
//! - `--dir` emits `cd <path>` prefix line
//! - `--max-tokens N` overrides the default token env var
//! - `--model NAME` appears in command args
//! - `--session-dir PATH` appears as `CLAUDE_CODE_SESSION_DIR` env var
//! - Combined flags produce correct combined output (no explicit `-c` needed)
//! - Message becomes quoted in command output — FR-1
//! - Message with embedded double quotes is properly escaped
//! - `--dir` with spaces: `cd` output is unquoted (human-readable per FR-21, not shell-safe)
//! - All 5 Tier-1 default env vars appear in output (not just max-tokens)
//! - No message provided: `--dry-run` outputs bare `claude --dangerously-skip-permissions --chrome --effort max -c` command with no message arg (when default session dir has sessions)
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

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_dry, stdout_str };
use std::process::Command;

#[ test ]
fn default_env_vars_appear_in_output()
{
  let output = run_dry( &[ "test" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000" ),
    "Default 200K token limit must appear in env output. Got:\n{output}"
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
    !output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000" ),
    "Default 200K must be replaced. Got:\n{output}"
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
fn session_dir_appears_as_env_var()
{
  let output = run_dry( &[ "--session-dir", "/tmp/sessions", "test" ] );
  assert!(
    output.contains( "CLAUDE_CODE_SESSION_DIR=/tmp/sessions" ),
    "--session-dir must set CLAUDE_CODE_SESSION_DIR. Got:\n{output}"
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

// No-message case: --dry-run with no message produces the bare command with all defaults
// but WITHOUT -c because the session dir is empty → session_exists() returns `None`.
// Fix(BUG-246): describe() now starts with "env -u CLAUDECODE" (default unset_claudecode=true).
// Do NOT use make_session_dir() here — that writes a dummy .jsonl making session_exists() return `Some(SessionId)`,
// which would inject -c and break the "no -c" assertion.
#[ test ]
fn dry_run_without_message_shows_bare_command()
{
  let empty_dir = tempfile::TempDir::new().expect( "create empty session dir" );
  let session_path = empty_dir.path().to_str().expect( "session dir path valid utf-8" );
  let output = run_dry( &[ "--session-dir", session_path ] );
  let last_line = output.trim_end().lines().last().unwrap_or_default();
  assert_eq!(
    last_line, "env -u CLAUDECODE claude --dangerously-skip-permissions --chrome --effort max",
    "Bare --dry-run must end with default bypass and effort max (no message, no -c in empty session dir). Got:\n{output}"
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

// Continuation: --dry-run shows -c when --session-dir has a qualifying .jsonl file.
// session_exists(Some(dir)) scans for .jsonl files via most_recent_session_in_dir().
#[ test ]
fn continuation_present_when_session_dir_nonempty()
{
  let session_dir = tempfile::tempdir().expect( "create temp session dir" );
  std::fs::write( session_dir.path().join( "00000000-0000-0000-0000-000000000000.jsonl" ), b"{}" )
    .expect( "write dummy session file" );
  let session_dir_str = session_dir.path().to_str().expect( "session dir path is valid utf-8" );
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "--dry-run", "--session-dir", session_dir_str, "test" ] )
    .output()
    .expect( "invoke clr" );
  let output = String::from_utf8_lossy( &out.stdout );
  assert!(
    output.contains( " -c" ),
    "non-empty --session-dir must inject -c. Got:\n{output}"
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

// Bare --dry-run (no message) does not add --print (no message = interactive REPL).
#[ test ]
fn bare_dry_run_no_message_has_no_print()
{
  let output = run_dry( &[] );
  assert!(
    !output.contains( "--print" ),
    "bare --dry-run (no message) must not add --print. Got:\n{output}"
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
  // Empty session dir → no -c (session_exists returns `None` for empty dir).
  // Fix(BUG-246): last_line now starts with "env -u CLAUDECODE" (default unset_claudecode=true).
  let empty_dir = tempfile::TempDir::new().expect( "create empty session dir" );
  let session_path = empty_dir.path().to_str().expect( "session dir path valid utf-8" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--dry-run", "--session-dir", session_path, "" ] )
    .env( "HOME", "/tmp/clr-isolated-home" ) // Fix(BUG-008) isolation: prevent host prefs from injecting --model
    .output()
    .expect( "Failed to invoke clr binary" );
  assert!( out.status.success(), "empty positional arg must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let last_line = stdout.trim_end().lines().last().unwrap_or_default();
  assert_eq!(
    last_line, "env -u CLAUDECODE claude --dangerously-skip-permissions --chrome --effort max",
    "empty positional arg must produce bare command (no message, no -c in empty session dir). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "\"ultrathink \"" ),
    "empty positional must NOT produce 'ultrathink ' degenerate message. Got:\n{stdout}"
  );
}
