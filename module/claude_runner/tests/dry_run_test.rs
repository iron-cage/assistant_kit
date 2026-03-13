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
//! - Default `-c` always appears in dry-run output (automatic session continuation)
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
//! - No message provided: `--dry-run` outputs bare `claude --dangerously-skip-permissions -c` command with no message arg
//! - `--dry-run --verbosity 0` still shows output (verbosity does not gate dry-run; bug reproducer)
//! - `--system-prompt TEXT` appears in command args (param 14 round-trip)
//! - `--append-system-prompt TEXT` appears in command args (param 15 round-trip)
//! - Both system-prompt flags can appear together in a single invocation
//! - `--help` output lists both `--system-prompt` and `--append-system-prompt`

use std::process::Command;

fn run_dry( args : &[ &str ] ) -> String
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
  .args( args )
  .output()
  .expect( "Failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "dry-run failed (exit {}): {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stderr )
  );
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

#[ test ]
fn default_env_vars_appear_in_output()
{
  let output = run_dry( &[ "--dry-run", "test" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000" ),
    "Default 200K token limit must appear in env output. Got:\n{output}"
  );
}

#[ test ]
fn working_dir_emits_cd_prefix()
{
  let output = run_dry( &[ "--dry-run", "--dir", "/tmp/work", "test" ] );
  assert!(
    output.contains( "cd /tmp/work" ),
    "--dir must produce 'cd <path>' prefix. Got:\n{output}"
  );
}

#[ test ]
fn max_tokens_override_updates_env_var()
{
  let output = run_dry( &[ "--dry-run", "--max-tokens", "100000", "test" ] );
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
  let output = run_dry( &[ "--dry-run", "--model", "claude-opus-4-6", "test" ] );
  assert!(
    output.contains( "claude-opus-4-6" ),
    "--model must appear in command line. Got:\n{output}"
  );
}

#[ test ]
fn session_dir_appears_as_env_var()
{
  let output = run_dry( &[ "--dry-run", "--session-dir", "/tmp/sessions", "test" ] );
  assert!(
    output.contains( "CLAUDE_CODE_SESSION_DIR=/tmp/sessions" ),
    "--session-dir must set CLAUDE_CODE_SESSION_DIR. Got:\n{output}"
  );
}

#[ test ]
fn message_appears_in_command_quoted()
{
  let output = run_dry( &[ "--dry-run", "hello world" ] );
  assert!(
    output.contains( "\"hello world\"" ),
    "Message must appear quoted in command. Got:\n{output}"
  );
}

#[ test ]
fn combined_flags_all_appear()
{
  // --dangerously-skip-permissions appears automatically (default-on; no explicit flag needed).
  // No explicit -c needed — it appears automatically via default continuation.
  let output = run_dry( &[
    "--dry-run", "--dir", "/tmp", "fix it",
  ] );
  assert!( output.contains( "cd /tmp" ), "Must have cd line" );
  assert!( output.contains( "--dangerously-skip-permissions" ), "Must have skip-permissions (default)" );
  assert!( output.contains( " -c" ), "Must have -c (automatic)" );
  assert!( output.contains( "\"fix it\"" ), "Must have quoted message" );
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
  let output = run_dry( &[ "--dry-run", "Hello there" ] );
  assert!(
    output.contains( "\"Hello there\"" ),
    "message must appear quoted in command. Got:\n{output}"
  );
}

// FR-1: message containing double quotes must be escaped in describe() output.
#[ test ]
fn message_with_embedded_quotes_is_escaped()
{
  let output = run_dry( &[ "--dry-run", r#"say "hi""# ] );
  assert!(
    output.contains( r#"\"hi\""# ),
    "Embedded double quotes must be escaped. Got:\n{output}"
  );
}

#[ test ]
fn dir_param_produces_cd_prefix()
{
  let output = run_dry( &[ "--dry-run", "--dir", "/tmp/mydir", "test" ] );
  assert!(
    output.contains( "cd /tmp/mydir" ),
    "--dir must produce 'cd <path>' prefix. Got:\n{output}"
  );
}

// FR-21: dir with spaces: cd output is unquoted (human-readable, not shell-safe).
#[ test ]
fn dir_with_spaces_produces_unquoted_cd_line()
{
  let output = run_dry( &[ "--dry-run", "--dir", "/path/with spaces", "test" ] );
  assert!(
    output.contains( "cd /path/with spaces" ),
    "Path with spaces must appear unquoted in cd line (FR-21). Got:\n{output}"
  );
}

// No-message case: --dry-run with no message produces `claude --dangerously-skip-permissions -c` command.
#[ test ]
fn dry_run_without_message_shows_bare_command()
{
  let output = run_dry( &[ "--dry-run" ] );
  let last_line = output.trim_end().lines().last().unwrap_or_default();
  assert_eq!(
    last_line, "claude --dangerously-skip-permissions -c",
    "Bare --dry-run must end with default bypass and continuation (no message arg). Got:\n{output}"
  );
}

// --new-session suppresses -c from dry-run output.
#[ test ]
fn new_session_suppresses_continue_flag()
{
  let output = run_dry( &[ "--dry-run", "--new-session", "test" ] );
  assert!(
    !output.contains( " -c" ),
    "--new-session must suppress -c in dry-run output. Got:\n{output}"
  );
}

// Default continuation: bare --dry-run always shows -c.
#[ test ]
fn default_continuation_always_present()
{
  let output = run_dry( &[ "--dry-run", "test" ] );
  assert!(
    output.contains( " -c" ),
    "Dry-run output must always contain -c (automatic session continuation). Got:\n{output}"
  );
}

// Tier-1 automation defaults: all four remaining env vars must appear alongside max-tokens.
#[ test ]
fn tier1_default_env_vars_all_appear()
{
  let output = run_dry( &[ "--dry-run", "test" ] );
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
  let output = run_dry( &[ "--dry-run", "-p", "test" ] );
  assert!(
    output.contains( "--print" ),
    "-p must add --print to command in dry-run output. Got:\n{output}"
  );
}

// --verbose flag appears in dry-run output (passed through to claude).
#[ test ]
fn verbose_flag_appears_in_dry_run()
{
  let output = run_dry( &[ "--dry-run", "--verbose", "test" ] );
  assert!(
    output.contains( "--verbose" ),
    "--verbose must appear in command in dry-run output. Got:\n{output}"
  );
}

// Message without -p defaults to --print (print mode is default when message given).
#[ test ]
fn message_without_print_flag_defaults_to_print_mode()
{
  let output = run_dry( &[ "--dry-run", "Fix the bug" ] );
  assert!(
    output.contains( "--print" ),
    "message without -p must default to --print in dry-run output. Got:\n{output}"
  );
}

// --interactive with message suppresses the default --print.
#[ test ]
fn interactive_flag_suppresses_default_print()
{
  let output = run_dry( &[ "--dry-run", "--interactive", "Fix the bug" ] );
  assert!(
    !output.contains( "--print" ),
    "--interactive must suppress --print default in dry-run output. Got:\n{output}"
  );
}

// Bare --dry-run (no message) does not add --print (no message = interactive REPL).
#[ test ]
fn bare_dry_run_no_message_has_no_print()
{
  let output = run_dry( &[ "--dry-run" ] );
  assert!(
    !output.contains( "--print" ),
    "bare --dry-run (no message) must not add --print. Got:\n{output}"
  );
}

// Bug reproducer: --dry-run output must appear regardless of --verbosity level.
//
// ## Root Cause
//
// `handle_dry_run()` gated output on `verbosity.shows_progress()` (level ≥ 3).
// At `--verbosity 0`, `shows_progress()` returned false so the entire output block
// was skipped — `--dry-run --verbosity 0` produced empty stdout and exit 0, with
// no indication that anything had been previewed.
//
// ## Why Not Caught
//
// All existing tests used default verbosity (3) or higher with `--dry-run`.
// No test exercised `--dry-run` at verbosity < 3, so the gate was never hit.
//
// ## Fix Applied
//
// Removed the `shows_progress()` guard from `handle_dry_run`. Verbosity controls
// runner *diagnostics* (progress messages, error reporting); `--dry-run` output
// is core functionality that the user explicitly requested.
//
// ## Prevention
//
// Test `--dry-run` at the extremes (verbosity 0 and verbosity 5). Core mode output
// (`--dry-run`, `--trace`) must never be gated on verbosity.
//
// ## Pitfall
//
// Do not confuse runner diagnostic verbosity with feature output. `--verbosity 0`
// suppresses runner messages; it must never suppress the command the user asked to see.
#[ test ]
fn dry_run_output_appears_regardless_of_verbosity()
{
  for level in [ "0", "1", "2", "3", "4", "5" ]
  {
    let output = run_dry( &[ "--dry-run", "--verbosity", level, "test" ] );
    assert!(
      output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
      "--dry-run --verbosity {level} must show env+command output regardless of verbosity. Got:\n{output}"
    );
    assert!(
      output.contains( "claude " ),
      "--dry-run --verbosity {level} must show the claude command line. Got:\n{output}"
    );
  }
}

// --system-prompt value round-trips through dry-run output.
// The flag and its text must appear verbatim in the assembled command.
#[ test ]
fn system_prompt_flag_round_trip()
{
  let output = run_dry( &[ "--dry-run", "--system-prompt", "Be concise.", "test" ] );
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
  let output = run_dry( &[ "--dry-run", "--append-system-prompt", "Always respond in JSON.", "test" ] );
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
    "--dry-run",
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
  let output = run_dry( &[ "--help" ] );
  assert!(
    output.contains( "--system-prompt" ),
    "--help must mention --system-prompt. Got:\n{output}"
  );
  assert!(
    output.contains( "--append-system-prompt" ),
    "--help must mention --append-system-prompt. Got:\n{output}"
  );
}
