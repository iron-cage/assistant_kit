//! CLI Argument Parsing Tests — `--flag value` format (T01–T35)
//!
//! ## Purpose
//!
//! Verify that `claude_runner` correctly parses `--flag value` CLI arguments
//! (mirroring Claude Code's native syntax) and translates them into the
//! underlying `ClaudeCommand` builder calls. Uses `--dry-run` mode to inspect
//! command construction without requiring the Claude binary in PATH.
//!
//! T36–T49, S58–S79, BUG-212, BUG-215, T48 live in `cli_args_ext_test.rs`.
//!
//! ## Strategy
//!
//! All tests invoke the compiled binary via `env!("CARGO_BIN_EXE_clr")`.
//! `--dry-run` outputs the command line that would be executed, allowing
//! assertions against the translation of flags → builder calls.
//!
//! ## Interface
//!
//! All flags use `--flag value` format matching Claude Code's native CLI.
//! Positional arguments form the message. `-p`/`--print` selects non-interactive
//! capture mode; the default is interactive (TTY passthrough).
//!
//! ## Corner Cases Covered
//!
//! - T01: positional message accepted with `--dry-run`
//! - T02: `--model` accepted, appears in command
//! - T03: `--max-tokens` accepted, appears as env var
//! - T04: bare `--dry-run` contains `-c` when session dir is non-empty
//! - T05: `--dangerously-skip-permissions` appears in command by default (no explicit flag needed)
//! - T06: `--verbose` appears in command
//! - T07: `--session-dir` appears as env var
//! - T08: `--dir` produces `cd <path>` prefix
//! - T09: `--dry-run` alone accepted (no message required)
//! - T10: multiple flags combined with session-dir containing files → `-c` injected
//! - T11: unknown flag rejected
//! - T12: `--max-tokens` non-numeric rejected
//! - T13: `--print` without message rejected
//! - T14: `--help` exits zero with USAGE
//! - T15: `-h` works as short help
//! - T16: help lists all documented options (`--new-session` present, `--continue` absent)
//! - T17: errors go to stderr, nothing to stdout
//! - T18: `--max-tokens 0` accepted (`u32::MIN` boundary)
//! - T19: `--max-tokens 4294967295` accepted (`u32::MAX` boundary)
//! - T20: `--max-tokens 4294967296` rejected (u32 overflow)
//! - T21: `--max-tokens -1` rejected (negative)
//! - T22: duplicate `--dir` last-wins
//! - T23: duplicate `--model` last-wins
//! - T24: duplicate `--session-dir` last-wins
//! - T25: duplicate `--max-tokens` last-wins
//! - T26: `--help` after valid flags shows help
//! - T27: `--` separator makes everything after positional
//! - T28: `--verbosity 6` rejected
//! - T29: `--dry-run` does not invoke claude binary
//! - T30: `--print` with message parsed (--print in dry-run output)
//! - T31: flag value missing rejected (`--model` without value)
//! - T32: flag value consumed as flag (`--model --verbose` → model="--verbose")
//! - T33: interleaved positional args and flags
//! - T34: `--model=sonnet` equals syntax rejected
//! - T35: `-pc` combined short flags rejected
//!
//! See `cli_args_ext_test.rs` (T36–T49, S58–S79, BUG-212, BUG-215, T48),
//! `ultrathink_args_test.rs` (T50–T58), and `effort_args_test.rs` (T59–T70).

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;

// T01: positional message accepted with --dry-run
#[ test ]
fn t01_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "hello" ] );
  assert!( out.status.success(), "positional message must be accepted" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "\"hello\n\nultrathink\"" ), "message must be suffixed with \"\\n\\nultrathink\" and appear quoted. Got:\n{stdout}" );
}

// T02: --model accepted, value appears in command
#[ test ]
fn t02_model_flag_accepted()
{
  let out = run_cli( &[ "--dry-run", "--model", "claude-opus-4-6", "test" ] );
  assert!( out.status.success(), "--model must be accepted" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "claude-opus-4-6" ), "model must appear in command. Got:\n{stdout}" );
}

// T03: --max-tokens accepted, appears as env var
#[ test ]
fn t03_max_tokens_flag_accepted()
{
  let out = run_cli( &[ "--dry-run", "--max-tokens", "1000", "test" ] );
  assert!( out.status.success(), "--max-tokens must be accepted" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=1000" ), "token env var must appear. Got:\n{stdout}" );
}

// T04: --dry-run contains -c when --session-dir is non-empty.
// session_exists(Some(dir)) checks the dir directly; a dummy file triggers -c injection.
#[ test ]
fn t04_dry_run_contains_continue_when_sessions_exist()
{
  let session_dir = tempfile::tempdir().expect( "create temp session dir" );
  std::fs::write( session_dir.path().join( "session.json" ), b"{}" )
    .expect( "write dummy session file" );
  let session_dir_str = session_dir.path().to_str().expect( "session dir path is valid utf-8" );
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "--dry-run", "--session-dir", session_dir_str, "test" ] )
    .output()
    .expect( "invoke clr" );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( " -c" ),
    "non-empty --session-dir must inject -c. Got:\n{stdout}"
  );
}

// T05: --dangerously-skip-permissions appears by DEFAULT (always-on — no explicit flag needed)
#[ test ]
fn t05_skip_permissions_default_on()
{
  let out = run_cli( &[ "--dry-run", "test" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--dangerously-skip-permissions" ),
    "Must produce --dangerously-skip-permissions by default. Got:\n{stdout}"
  );
}

// T06: --verbose appears in command passed to claude
#[ test ]
fn t06_verbose_flag_passed_to_claude()
{
  let out = run_cli( &[ "--dry-run", "--verbose", "test" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--verbose" ),
    "--verbose must appear in claude command. Got:\n{stdout}"
  );
}

// T07: --session-dir appears as env var
#[ test ]
fn t07_session_dir_flag()
{
  let out = run_cli( &[ "--dry-run", "--session-dir", "/tmp/sess", "test" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE_CODE_SESSION_DIR=/tmp/sess" ),
    "--session-dir must set env var. Got:\n{stdout}"
  );
}

// T08: --dir produces cd prefix
#[ test ]
fn t08_dir_flag()
{
  let out = run_cli( &[ "--dry-run", "--dir", "/tmp/test-dir", "test" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "cd /tmp/test-dir" ), "--dir must produce cd prefix. Got:\n{stdout}" );
}

// T09: --dry-run alone accepted (no message required)
#[ test ]
fn t09_dry_run_without_message()
{
  let out = run_cli( &[ "--dry-run" ] );
  assert!( out.status.success(), "--dry-run without message must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "claude" ), "dry-run output must contain 'claude'. Got:\n{stdout}" );
}

// T10: multiple flags combined — session-dir with a file triggers -c injection
#[ test ]
fn t10_multiple_flags_combined()
{
  // Create a session dir with one dummy file so session_exists returns true.
  let session_dir = tempfile::tempdir().expect( "create temp session dir" );
  std::fs::write( session_dir.path().join( "session.json" ), b"{}" )
    .expect( "write dummy session file" );
  let session_dir_str = session_dir.path().to_str().expect( "session dir path is valid utf-8" );

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [
      "--dry-run", "--dir", "/tmp",
      "--session-dir", session_dir_str,
      "--model", "claude-sonnet-4-6", "fix it",
    ] )
    .output()
    .expect( "invoke clr" );
  assert!( out.status.success(), "multiple flags must be accepted" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "cd /tmp" ), "Must have cd line. Got:\n{stdout}" );
  assert!( stdout.contains( "--dangerously-skip-permissions" ), "Must have skip-permissions (default-on). Got:\n{stdout}" );
  assert!( stdout.contains( " -c" ), "Must have -c when session-dir is non-empty. Got:\n{stdout}" );
  assert!( stdout.contains( "claude-sonnet-4-6" ), "Must have model. Got:\n{stdout}" );
  assert!( stdout.contains( "\"fix it\n\nultrathink\"" ), "Must have ultrathink-suffixed quoted message. Got:\n{stdout}" );
}

// T11: unknown flag rejected
#[ test ]
fn t11_unknown_flag_rejected()
{
  let out = run_cli( &[ "--unknown-flag-xyz" ] );
  assert!( !out.status.success(), "unknown flag must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "Error:" ), "error must go to stderr. Got: {stderr}" );
}

// T12: --max-tokens non-numeric rejected
#[ test ]
fn t12_max_tokens_non_numeric_rejected()
{
  let out = run_cli( &[ "--dry-run", "--max-tokens", "not-a-number", "test" ] );
  assert!( !out.status.success(), "non-numeric --max-tokens must exit non-zero" );
}

// T13: --print without message rejected
#[ test ]
fn t13_print_without_message_rejected()
{
  let out = run_cli( &[ "--print" ] );
  assert!( !out.status.success(), "--print without message must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "--print requires a message" ),
    "--print without message must give specific error. Got:\n{stderr}"
  );
}

// T14: --help exits zero with USAGE
#[ test ]
fn t14_help_flag_exits_zero_with_usage()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "--help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "RUNNER OPTIONS:" ), "--help must print RUNNER OPTIONS" );
}

// T15: -h works as short help
#[ test ]
fn t15_short_help_flag_works()
{
  let out = run_cli( &[ "-h" ] );
  assert!( out.status.success(), "-h must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "RUNNER OPTIONS:" ), "help output must contain RUNNER OPTIONS:. Got:\n{stdout}" );
}

// T16: help lists all documented options (--new-session present, --continue absent)
#[ test ]
fn t16_help_lists_all_options()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "exit={} stderr={}", out.status.code().unwrap_or( -1 ), String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  for opt in &[
    "--print", "--new-session", "--model", "--verbose",
    "--no-skip-permissions", "--max-tokens", "--session-dir",
    "--dir", "--dry-run", "--verbosity", "--help",
    "--system-prompt", "--append-system-prompt", "--no-ultrathink",
    "--effort", "--no-effort-max", "[<MSG>]",
  ] {
    assert!( stdout.contains( opt ), "--help missing option {opt}. Got:\n{stdout}" );
  }
  assert!(
    !stdout.contains( "--continue" ),
    "--help must NOT list --continue (removed; continuation is automatic). Got:\n{stdout}"
  );
}

// T17: errors go to stderr, nothing to stdout
#[ test ]
fn t17_error_output_goes_to_stderr_not_stdout()
{
  let out = run_cli( &[ "--unknown-flag" ] );
  assert!( !out.status.success(), "unknown flag must fail; exit={}", out.status.code().unwrap_or( -1 ) );
  assert!(
    out.stdout.is_empty(),
    "stdout must be empty on error; got: {}",
    String::from_utf8_lossy( &out.stdout )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "Error:" ), "stderr must contain 'Error:'; got: {stderr}" );
}

// T18: --max-tokens 0 accepted (u32::MIN boundary)
#[ test ]
fn t18_max_tokens_zero_accepted()
{
  let out = run_cli( &[ "--dry-run", "--max-tokens", "0", "test" ] );
  assert!( out.status.success(), "--max-tokens 0 must be accepted" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=0" ), "must set token env to 0. Got:\n{stdout}" );
}

// T19: --max-tokens 4294967295 accepted (u32::MAX boundary)
#[ test ]
fn t19_max_tokens_u32_max_accepted()
{
  let out = run_cli( &[ "--dry-run", "--max-tokens", "4294967295", "test" ] );
  assert!( out.status.success(), "--max-tokens u32::MAX must be accepted" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=4294967295" ), "must set correct token env. Got:\n{stdout}" );
}

// T20: --max-tokens 4294967296 rejected (u32 overflow)
#[ test ]
fn t20_max_tokens_overflow_rejected()
{
  let out = run_cli( &[ "--dry-run", "--max-tokens", "4294967296", "test" ] );
  assert!( !out.status.success(), "--max-tokens u32::MAX+1 must exit non-zero" );
}

// T21: --max-tokens -1 rejected (negative)
#[ test ]
fn t21_max_tokens_negative_rejected()
{
  let out = run_cli( &[ "--dry-run", "--max-tokens", "-1", "test" ] );
  assert!( !out.status.success(), "--max-tokens -1 must exit non-zero" );
}

// T22: duplicate --dir last-wins
#[ test ]
fn t22_duplicate_dir_uses_last_value()
{
  let out = run_cli( &[ "--dry-run", "--dir", "/first", "--dir", "/last", "test" ] );
  assert!( out.status.success(), "duplicate --dir must exit 0 (last wins)" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "cd /last" ), "last --dir value must win. Got:\n{stdout}" );
  assert!( !stdout.contains( "cd /first" ), "first --dir must be overridden. Got:\n{stdout}" );
}

// T23: duplicate --model last-wins
#[ test ]
fn t23_duplicate_model_uses_last_value()
{
  let out = run_cli( &[ "--dry-run", "--model", "first-model", "--model", "last-model", "test" ] );
  assert!( out.status.success(), "duplicate --model must exit 0 (last wins)" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "last-model" ), "last --model value must win. Got:\n{stdout}" );
  assert!( !stdout.contains( "first-model" ), "first --model must be overridden. Got:\n{stdout}" );
}

// T24: duplicate --session-dir last-wins
#[ test ]
fn t24_duplicate_session_dir_uses_last_value()
{
  let out = run_cli( &[ "--dry-run", "--session-dir", "/first", "--session-dir", "/last", "test" ] );
  assert!( out.status.success(), "duplicate --session-dir must exit 0 (last wins)" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "CLAUDE_CODE_SESSION_DIR=/last" ), "last --session-dir must win. Got:\n{stdout}" );
  assert!( !stdout.contains( "CLAUDE_CODE_SESSION_DIR=/first" ), "first must be overridden. Got:\n{stdout}" );
}

// T25: duplicate --max-tokens last-wins
#[ test ]
fn t25_duplicate_max_tokens_uses_last_value()
{
  let out = run_cli( &[ "--dry-run", "--max-tokens", "100", "--max-tokens", "50000", "test" ] );
  assert!( out.status.success(), "duplicate --max-tokens must exit 0 (last wins)" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=50000" ),
    "last --max-tokens must win. Got:\n{stdout}"
  );
}

// T26: --help after valid flags shows help (flags discarded)
#[ test ]
fn t26_help_after_flags_shows_help()
{
  let out = run_cli( &[ "--dir", "/tmp", "--help" ] );
  assert!( out.status.success(), "--help must exit 0 even after valid flags" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "RUNNER OPTIONS:" ),
    "--help must print RUNNER OPTIONS even after valid flags. Got:\n{stdout}"
  );
}

// T27: `--` separator makes everything after positional (message)
#[ test ]
fn t27_double_dash_separator()
{
  let out = run_cli( &[ "--dry-run", "--", "--not-a-flag" ] );
  assert!( out.status.success(), "-- separator must allow --not-a-flag as message" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "\"--not-a-flag\n\nultrathink\"" ),
    "Text after -- must become message with ultrathink suffix. Got:\n{stdout}"
  );
}

// T28: --verbosity 6 rejected
#[ test ]
fn t28_verbosity_six_rejected()
{
  let out = run_cli( &[ "--verbosity", "6", "--dry-run", "test" ] );
  assert!( !out.status.success(), "--verbosity 6 must be rejected" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "verbosity" ), "error must mention verbosity. Got:\n{stderr}" );
}

// T29: --dry-run does not invoke claude binary (succeeds without claude in PATH)
#[ test ]
fn t29_dry_run_does_not_invoke_claude()
{
  let out = run_cli( &[ "--dry-run", "test" ] );
  assert!(
    out.status.success(),
    "--dry-run must not fail due to missing claude binary"
  );
}

// T30: --print with message accepted (validates parse, not execution)
#[ test ]
fn t30_print_with_message_parsed()
{
  let out = run_cli( &[ "--dry-run", "-p", "test" ] );
  assert!( out.status.success(), "-p with message must parse OK" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--print" ),
    "-p must add --print to command. Got:\n{stdout}"
  );
}

// T31: flag value missing rejected (--model without value)
#[ test ]
fn t31_flag_missing_value_rejected()
{
  let out = run_cli( &[ "--dry-run", "--model" ] );
  assert!( !out.status.success(), "--model without value must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "requires a value" ), "must mention missing value. Got:\n{stderr}" );
}

// T32: flag value consumed as flag — `--model --verbose` treats --verbose as model value
#[ test ]
fn t32_flag_value_consumed_as_flag_name()
{
  let out = run_cli( &[ "--dry-run", "--model", "--verbose", "msg" ] );
  assert!( out.status.success(), "must accept --verbose as model value" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  // --verbose becomes the model value, NOT a flag → no --verbose in builder args
  assert!(
    stdout.contains( "--model --verbose" ),
    "--verbose must be the model value, not a flag. Got:\n{stdout}"
  );
}

// T33: interleaved positional args and flags
#[ test ]
fn t33_interleaved_positional_and_flags()
{
  let out = run_cli( &[ "--dry-run", "hello", "--dir", "/tmp", "world" ] );
  assert!( out.status.success(), "interleaved positional must be accepted" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "\"hello world\n\nultrathink\"" ),
    "positional args must join as ultrathink-suffixed message. Got:\n{stdout}"
  );
  assert!( stdout.contains( "cd /tmp" ), "dir flag must still work. Got:\n{stdout}" );
}

// T34: `--model=sonnet` equals syntax rejected
#[ test ]
fn t34_equals_syntax_rejected()
{
  let out = run_cli( &[ "--model=sonnet" ] );
  assert!( !out.status.success(), "--model=sonnet must be rejected" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "unknown option" ), "must report unknown option. Got:\n{stderr}" );
}

// T35: `-pc` combined short flags rejected
#[ test ]
fn t35_combined_short_flags_rejected()
{
  let out = run_cli( &[ "-pc" ] );
  assert!( !out.status.success(), "-pc must be rejected" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "unknown option" ), "must report unknown option. Got:\n{stderr}" );
}
