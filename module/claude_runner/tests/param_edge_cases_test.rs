//! Parameter Edge Case Tests
//!
//! ## Purpose
//!
//! Cover spec test cases from `tests/docs/cli/param/` and `tests/docs/cli/command/`
//! that are not already exercised by `cli_args_test.rs`, `dry_run_test.rs`,
//! `ultrathink_args_test.rs`, `effort_args_test.rs`, or `verbosity_test.rs`.
//!
//! ## Strategy
//!
//! All tests invoke the compiled binary via `env!("CARGO_BIN_EXE_clr")`.
//! Most tests use `--dry-run` to inspect command assembly without executing claude.
//! Trace-mode tests set PATH to `/nonexistent` so execution fails deterministically.
//!
//! ## Spec Coverage
//!
//! help:
//! - S01: `--help` stdout non-empty, stderr empty (`02_help.md` IT-6)
//! - S02: `-h` output byte-identical to `--help` output (`02_help.md` IT-7)
//! - S03: `--help` output stable across three invocations (`02_help.md` IT-8)
//!
//! trace:
//! - S04: `--trace "msg"` without `--dry-run` → stderr has command, exit 1 (`01_run.md` IT-5, `13_trace.md` EC-1, `11_dry_run.md` EC-2)
//! - S05: `--trace --dry-run` no message → stdout preview, stderr empty (`13_trace.md` EC-4)
//! - S06: `--trace "msg"` stderr contains env vars and command (`13_trace.md` EC-6)
//!
//! --model:
//! - S07: positional then `--model` at end of argv → exit 1 (`03_model.md` EC-3)
//! - S08: without `--model` → `--model` absent from command (`03_model.md` EC-7)
//!
//! --verbose:
//! - S09: without `--verbose` → absent from command (`04_verbose.md` EC-2)
//! - S10: `--verbose` without message → accepted, forwarded (`04_verbose.md` EC-4)
//! - S11: `--verbose` specified twice → appears at most once (`04_verbose.md` EC-6)
//!
//! --no-skip-permissions:
//! - S12: `--no-skip-permissions "msg"` → message still forwarded (`05_no_skip_permissions.md` EC-3)
//! - S13: `--no-skip-permissions` alone → exit 0 (`05_no_skip_permissions.md` EC-4)
//! - S14: `--no-skip-permissions --verbose "msg"` → both coexist (`05_no_skip_permissions.md` EC-6)
//!
//! --interactive:
//! - S15: `--interactive --verbose "msg"` → --verbose present, --print absent (`06_interactive.md` EC-6)
//!
//! --new-session:
//! - S16: `--new-session` without message → exit 0, no `-c` (`07_new_session.md` EC-3)
//! - S17: `--new-session --session-dir /path "msg"` → both accepted (`07_new_session.md` EC-6)
//!
//! --dir:
//! - S18: without `--dir` → no `cd` line in output (`08_dir.md` EC-3)
//! - S19: `--dir /no/such/path` → accepted without validation (`08_dir.md` EC-4)
//!
//! --session-dir:
//! - S20: without `--session-dir` → `CLAUDE_CODE_SESSION_DIR` absent (`10_session_dir.md` EC-3)
//! - S21: `--session-dir --new-session` → both accepted (`10_session_dir.md` EC-4)
//! - S22: `--session-dir /no/such/dir` → accepted without validation (`10_session_dir.md` EC-6)
//!
//! --dry-run:
//! - S23: `--dry-run --no-ultrathink --no-effort-max --verbose "msg"` → all flags visible (`11_dry_run.md` EC-6)
//!
//! --verbosity:
//! - S24: `--verbosity high "msg"` → exit 1, error on stderr (`12_verbosity.md` EC-6)
//!
//! --print:
//! - S25: `clr --dry-run -p "msg"` and `clr --dry-run --print "msg"` byte-identical (`02_print.md` EC-2)
//! - S26: `--print "msg"` (long form) → `--print` in output (`02_print.md` EC-5)
//!
//! --system-prompt:
//! - S27: `--system-prompt ""` → forwarded, exit 0 (`15_system_prompt.md` EC-3)
//! - S28: `--system-prompt "Be concise and accurate."` → value forwarded as single arg (`15_system_prompt.md` EC-6)
//!
//! --append-system-prompt:
//! - S29: `--append-system-prompt ""` → forwarded, exit 0 (`16_append_system_prompt.md` EC-3)
//! - S30: `--append-system-prompt "Always respond in JSON."` → single arg (`16_append_system_prompt.md` EC-6)
//!
//! --no-effort-max:
//! - S31: `--no-effort-max` without message → exit 0, no --effort (`18_no_effort_max.md` EC-2)
//! - S32: `--no-effort-max --new-session "msg"` → no --effort, no -c (`18_no_effort_max.md` EC-6)
//!
//! invariant:
//! - S33: all opt-outs together remove all suppressible defaults (`invariant/001_default_flags.md` IT-6)
//!
//! --no-chrome:
//! - S34: `--no-chrome "msg"` → no `--chrome` in assembled command (`21_no_chrome.md` EC-1)
//! - S35: default (no `--no-chrome`) → `--chrome` present in assembled command (`21_no_chrome.md` EC-2)
//! - S36: `--no-chrome` without message → exit 0; no `--chrome` in preview (`21_no_chrome.md` EC-3)
//! - S37: `--help` output contains `--no-chrome` (`21_no_chrome.md` EC-4)
//! - S38: `--no-chrome` + `--no-skip-permissions` → neither default flag in assembled command (`21_no_chrome.md` EC-5)
//! - S39: `--no-chrome` + `--dry-run` → preview without `--chrome`; stderr empty (`21_no_chrome.md` EC-6)
//!
//! --no-persist:
//! - S40: `--no-persist "msg"` → `--no-session-persistence` in assembled command (`22_no_persist.md` EC-1)
//! - S41: default (no `--no-persist`) → no `--no-session-persistence` in assembled command (`22_no_persist.md` EC-2)
//! - S42: `--no-persist` without message → exit 0 (`22_no_persist.md` EC-3)
//! - S43: `--help` output contains `--no-persist` (`22_no_persist.md` EC-4)
//! - S44: `--no-persist` + `--new-session` → both accepted; `--no-session-persistence` present, no `-c` (`22_no_persist.md` EC-5)
//! - S45: `--no-persist` + `--dry-run` → preview shows `--no-session-persistence`; stderr empty (`22_no_persist.md` EC-6)
//!
//! --json-schema:
//! - S46: `--json-schema <val> "msg"` → forwarded in assembled command (`23_json_schema.md` EC-1)
//! - S47: default (no `--json-schema`) → no `--json-schema` in assembled command (`23_json_schema.md` EC-2)
//! - S48: complex schema → forwarded verbatim (`23_json_schema.md` EC-3)
//! - S49: `--help` output contains `--json-schema` (`23_json_schema.md` EC-4)
//! - S50: `--json-schema` + `--model` → both forwarded (`23_json_schema.md` EC-5)
//! - S51: `--json-schema` without message → exit 0; schema in assembled command (`23_json_schema.md` EC-6)
//!
//! --mcp-config:
//! - S52: single `--mcp-config <path> "msg"` → forwarded in assembled command (`24_mcp_config.md` EC-1)
//! - S53: default (no `--mcp-config`) → no `--mcp-config` in assembled command (`24_mcp_config.md` EC-2)
//! - S54: multiple `--mcp-config` flags → all forwarded individually (`24_mcp_config.md` EC-3)
//! - S55: `--help` output contains `--mcp-config` (`24_mcp_config.md` EC-4)
//! - S56: `--mcp-config` + `--model` → both forwarded (`24_mcp_config.md` EC-5)
//! - S57: `--mcp-config` without message → exit 0; path in assembled command (`24_mcp_config.md` EC-6)

mod common;
use common::run_cli;
use std::process::Command;

fn run_no_claude( args : &[ &str ] ) -> std::process::Output
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  Command::new( bin )
    .args( args )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "Failed to invoke clr binary" )
}

// S01: --help stdout non-empty; stderr empty
#[ test ]
fn s01_help_flag_stderr_empty()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "--help must exit 0" );
  assert!(
    !out.stdout.is_empty(),
    "--help stdout must be non-empty"
  );
  assert!(
    out.stderr.is_empty(),
    "--help must produce no stderr output. Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S02: -h output byte-identical to --help output
#[ test ]
fn s02_h_output_byte_identical_to_help()
{
  let short = run_cli( &[ "-h" ] );
  let long = run_cli( &[ "--help" ] );
  assert!( short.status.success(), "-h must exit 0" );
  assert!( long.status.success(), "--help must exit 0" );
  assert_eq!(
    short.stdout, long.stdout,
    "-h output must be byte-identical to --help output"
  );
}

// S03: --help output is stable across three invocations
#[ test ]
fn s03_help_output_stable_across_invocations()
{
  let first = run_cli( &[ "--help" ] ).stdout;
  let second = run_cli( &[ "--help" ] ).stdout;
  let third = run_cli( &[ "--help" ] ).stdout;
  assert_eq!( first, second, "--help output must be identical on run 1 vs 2" );
  assert_eq!( second, third, "--help output must be identical on run 2 vs 3" );
}

// S04: --trace without --dry-run → stderr has command; exit 1 (claude absent)
#[ test ]
fn s04_trace_without_dry_run_echoes_command_to_stderr()
{
  let out = run_no_claude( &[ "--trace", "Fix bug" ] );
  assert!(
    !out.status.success(),
    "--trace without --dry-run must fail (claude not found)"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "claude" ),
    "--trace must echo assembled command to stderr before invocation attempt. Got:\n{stderr}"
  );
}

// S05: --trace --dry-run without message → stdout has preview; stderr is empty
#[ test ]
fn s05_trace_with_dry_run_no_message_stderr_empty()
{
  let out = run_cli( &[ "--trace", "--dry-run" ] );
  assert!( out.status.success(), "--trace --dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "claude" ),
    "--dry-run output must appear on stdout. Got:\n{stdout}"
  );
  assert!(
    out.stderr.is_empty(),
    "--trace must not fire when --dry-run wins (stderr must be empty). Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S06: --trace (no --dry-run) stderr includes env vars and command
#[ test ]
fn s06_trace_stderr_includes_env_vars_and_command()
{
  let out = run_no_claude( &[ "--trace", "Fix bug" ] );
  assert!( !out.status.success(), "must fail (claude absent)" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "claude" ),
    "trace stderr must include assembled command. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "trace stderr must include env vars. Got:\n{stderr}"
  );
}

// S07: positional before --model at end of argv → exit 1 (missing value)
#[ test ]
fn s07_model_at_end_of_argv_rejected()
{
  let out = run_cli( &[ "Fix bug", "--model" ] );
  assert!( !out.status.success(), "--model at end of argv must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "requires a value" ),
    "--model without value must report 'requires a value'. Got:\n{stderr}"
  );
}

// S08: without --model → --model absent from assembled command
#[ test ]
fn s08_model_absent_from_default_command()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--model" ),
    "without --model, assembled command must not contain --model. Got:\n{stdout}"
  );
}

// S09: without --verbose → --verbose absent from assembled command
#[ test ]
fn s09_verbose_absent_from_default_command()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--verbose" ),
    "without --verbose, assembled command must not contain --verbose. Got:\n{stdout}"
  );
}

// S10: --verbose without message → forwarded, exit 0
#[ test ]
fn s10_verbose_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--verbose" ] );
  assert!( out.status.success(), "--verbose without message must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--verbose" ),
    "--verbose must appear in assembled command even without message. Got:\n{stdout}"
  );
}

// S11: --verbose specified twice → appears at most once (no duplication)
#[ test ]
fn s11_verbose_specified_twice_not_duplicated()
{
  let out = run_cli( &[ "--dry-run", "--verbose", "--verbose", "Fix bug" ] );
  assert!( out.status.success(), "double --verbose must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let count = stdout.matches( "--verbose" ).count();
  assert!(
    count <= 1,
    "--verbose must appear at most once (not duplicated). Got {count} occurrences in:\n{stdout}"
  );
}

// S12: --no-skip-permissions + message → message still forwarded
#[ test ]
fn s12_no_skip_permissions_with_message_forwards_message()
{
  let out = run_cli( &[ "--dry-run", "--no-skip-permissions", "Explain this" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Explain this" ),
    "message must be forwarded when --no-skip-permissions is set. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--dangerously-skip-permissions" ),
    "--dangerously-skip-permissions must be absent. Got:\n{stdout}"
  );
}

// S13: --no-skip-permissions alone → exit 0 (no error)
#[ test ]
fn s13_no_skip_permissions_alone_accepted()
{
  let out = run_cli( &[ "--dry-run", "--no-skip-permissions" ] );
  assert!(
    out.status.success(),
    "--no-skip-permissions alone must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S14: --no-skip-permissions + --verbose → both coexist
#[ test ]
fn s14_no_skip_permissions_with_verbose_coexist()
{
  let out = run_cli( &[ "--dry-run", "--no-skip-permissions", "--verbose", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--verbose" ),
    "--verbose must appear with --no-skip-permissions. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--dangerously-skip-permissions" ),
    "--dangerously-skip-permissions must be absent. Got:\n{stdout}"
  );
}

// S15: --interactive + --verbose → --verbose present, --print absent
#[ test ]
fn s15_interactive_with_verbose_both_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--interactive", "--verbose", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--verbose" ),
    "--verbose must appear with --interactive. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--print" ),
    "--print must be absent with --interactive. Got:\n{stdout}"
  );
}

// S16: --new-session without message → exit 0, no -c
#[ test ]
fn s16_new_session_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--new-session" ] );
  assert!(
    out.status.success(),
    "--new-session without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{stdout}"
  );
}

// S17: --new-session + --session-dir → both accepted; CLAUDE_CODE_SESSION_DIR present, no -c
#[ test ]
fn s17_new_session_with_session_dir_accepted()
{
  let out = run_cli( &[ "--dry-run", "--new-session", "--session-dir", "/tmp/sessions", "Fix bug" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE_CODE_SESSION_DIR=/tmp/sessions" ),
    "--session-dir must set env var. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{stdout}"
  );
}

// S18: without --dir → no `cd` prefix line in output
#[ test ]
fn s18_dir_absent_from_default_output()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "cd " ),
    "without --dir, output must not contain a 'cd' prefix line. Got:\n{stdout}"
  );
}

// S19: --dir /no/such/path → accepted without validation
#[ test ]
fn s19_dir_nonexistent_path_accepted()
{
  let out = run_cli( &[ "--dry-run", "--dir", "/no/such/path/xyz", "Fix bug" ] );
  assert!(
    out.status.success(),
    "--dir with non-existent path must exit 0 (no path validation at runner layer). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "cd /no/such/path/xyz" ),
    "--dir must emit cd line even for non-existent path. Got:\n{stdout}"
  );
}

// S20: without --session-dir → CLAUDE_CODE_SESSION_DIR absent from env block
#[ test ]
fn s20_session_dir_absent_from_default_output()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "without --session-dir, CLAUDE_CODE_SESSION_DIR must be absent. Got:\n{stdout}"
  );
}

// S21: --session-dir + --new-session → both accepted; env var present, no -c
#[ test ]
fn s21_session_dir_with_new_session_accepted()
{
  let out = run_cli( &[ "--dry-run", "--session-dir", "/tmp/s", "--new-session", "Fix bug" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "CLAUDE_CODE_SESSION_DIR=/tmp/s" ),
    "--session-dir must set env var. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{stdout}"
  );
}

// S22: --session-dir /no/such/dir → accepted without validation
#[ test ]
fn s22_session_dir_nonexistent_path_accepted()
{
  let out = run_cli( &[ "--dry-run", "--session-dir", "/no/such/dir/xyz", "Fix bug" ] );
  assert!(
    out.status.success(),
    "--session-dir with non-existent path must exit 0 (no validation). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S23: --dry-run with many opt-out flags → all flags visible in preview
#[ test ]
fn s23_dry_run_with_multiple_opt_out_flags_full_preview()
{
  let out = run_cli( &[
    "--dry-run", "--no-ultrathink", "--no-effort-max", "--verbose", "Fix bug",
  ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--verbose" ), "--verbose must appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "--effort" ), "--effort must be absent (--no-effort-max). Got:\n{stdout}" );
  assert!( !stdout.contains( "ultrathink" ), "ultrathink suffix must be absent (--no-ultrathink). Got:\n{stdout}" );
}

// S24: --verbosity with non-numeric value → exit 1
#[ test ]
fn s24_verbosity_non_numeric_value_rejected()
{
  let out = run_cli( &[ "--verbosity", "high", "--dry-run", "Fix bug" ] );
  assert!( !out.status.success(), "--verbosity with non-numeric value must exit non-zero" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.is_empty(),
    "error must go to stderr for invalid --verbosity value. Got:\n{stderr}"
  );
}

// S25: -p and --print produce byte-identical output
#[ test ]
fn s25_p_alias_output_byte_identical_to_print_long_form()
{
  let short = run_cli( &[ "--dry-run", "-p", "Fix bug" ] );
  let long = run_cli( &[ "--dry-run", "--print", "Fix bug" ] );
  assert!( short.status.success(), "-p must exit 0" );
  assert!( long.status.success(), "--print must exit 0" );
  assert_eq!(
    short.stdout, long.stdout,
    "-p and --print must produce byte-identical dry-run output"
  );
}

// S26: --print (explicit long form) + message → --print in assembled command
#[ test ]
fn s26_print_explicit_long_form_adds_flag()
{
  let out = run_cli( &[ "--dry-run", "--print", "Fix bug" ] );
  assert!( out.status.success(), "--print with message must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--print" ),
    "--print must appear in assembled command when specified explicitly. Got:\n{stdout}"
  );
}

// S27: --system-prompt "" → forwarded without rejection
#[ test ]
fn s27_system_prompt_empty_string_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--system-prompt", "", "test" ] );
  assert!(
    out.status.success(),
    "--system-prompt with empty string must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S28: --system-prompt with spaces → forwarded as single argument
#[ test ]
fn s28_system_prompt_with_spaces_as_single_arg()
{
  let out = run_cli( &[ "--dry-run", "--system-prompt", "Be concise and accurate.", "test" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Be concise and accurate." ),
    "--system-prompt value with spaces must be forwarded as single arg. Got:\n{stdout}"
  );
}

// S29: --append-system-prompt "" → forwarded without rejection
#[ test ]
fn s29_append_system_prompt_empty_string_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--append-system-prompt", "", "test" ] );
  assert!(
    out.status.success(),
    "--append-system-prompt with empty string must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S30: --append-system-prompt with spaces → forwarded as single argument
#[ test ]
fn s30_append_system_prompt_with_spaces_as_single_arg()
{
  let out = run_cli( &[ "--dry-run", "--append-system-prompt", "Always respond in JSON.", "test" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Always respond in JSON." ),
    "--append-system-prompt value with spaces must be forwarded as single arg. Got:\n{stdout}"
  );
}

// S31: --no-effort-max without message → exit 0, no --effort
#[ test ]
fn s31_no_effort_max_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--no-effort-max" ] );
  assert!(
    out.status.success(),
    "--no-effort-max without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--effort" ),
    "--no-effort-max must suppress all --effort tokens. Got:\n{stdout}"
  );
}

// S32: --no-effort-max + --new-session → no --effort, no -c; both coexist
#[ test ]
fn s32_no_effort_max_with_new_session_accepted()
{
  let out = run_cli( &[ "--dry-run", "--no-effort-max", "--new-session", "Fix bug" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--effort" ),
    "--no-effort-max must suppress --effort. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{stdout}"
  );
}

// S33: all suppressible opt-outs together remove all suppressible defaults
//
// --new-session removes -c, --no-skip-permissions removes --dangerously-skip-permissions,
// --no-ultrathink removes ultrathink suffix, --no-effort-max removes --effort max.
// --chrome is injected by default; --no-chrome suppresses it (see S34–S39).
// This test does not use --no-chrome, so --chrome remains in the assembled command.
#[ test ]
fn s33_all_opt_outs_together_remove_all_suppressible_defaults()
{
  let out = run_cli( &[
    "--dry-run",
    "--new-session",
    "--no-skip-permissions",
    "--no-ultrathink",
    "--no-effort-max",
    "Fix bug",
  ] );
  assert!( out.status.success(), "all opt-outs must be accepted. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( !stdout.contains( " -c" ),
    "-c must be suppressed by --new-session. Got:\n{stdout}" );
  assert!( !stdout.contains( "--dangerously-skip-permissions" ),
    "--dangerously-skip-permissions must be suppressed by --no-skip-permissions. Got:\n{stdout}" );
  assert!( !stdout.contains( "--effort" ),
    "--effort must be suppressed by --no-effort-max. Got:\n{stdout}" );
  assert!( !stdout.contains( "ultrathink" ),
    "ultrathink suffix must be suppressed by --no-ultrathink. Got:\n{stdout}" );
}

// S34: `--no-chrome` suppresses default `--chrome` injection (`21_no_chrome.md` EC-1)
#[ test ]
fn s34_no_chrome_suppresses_chrome_flag()
{
  let out = run_cli( &[ "--dry-run", "--no-chrome", "Fix bug" ] );
  assert!( out.status.success(), "--no-chrome must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "--no-chrome must suppress --chrome. Got:\n{stdout}"
  );
}

// S35: default (no `--no-chrome`) → `--chrome` present in assembled command (`21_no_chrome.md` EC-2)
#[ test ]
fn s35_default_chrome_injected()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--chrome" ),
    "default assembled command must contain --chrome. Got:\n{stdout}"
  );
}

// S36: `--no-chrome` without message → exit 0; no `--chrome` in preview (`21_no_chrome.md` EC-3)
#[ test ]
fn s36_no_chrome_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--no-chrome" ] );
  assert!(
    out.status.success(),
    "--no-chrome without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "--no-chrome must suppress --chrome even without a message. Got:\n{stdout}"
  );
}

// S37: `--help` output lists `--no-chrome` (`21_no_chrome.md` EC-4)
#[ test ]
fn s37_help_lists_no_chrome()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-chrome" ),
    "--help must mention --no-chrome. Got:\n{stdout}"
  );
}

// S38: `--no-chrome` + `--no-skip-permissions` → neither `--chrome` nor
// `--dangerously-skip-permissions` in assembled command (`21_no_chrome.md` EC-5)
#[ test ]
fn s38_no_chrome_with_no_skip_permissions_both_suppressed()
{
  let out = run_cli( &[ "--dry-run", "--no-chrome", "--no-skip-permissions", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "--no-chrome must suppress --chrome. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--dangerously-skip-permissions" ),
    "--no-skip-permissions must suppress --dangerously-skip-permissions. Got:\n{stdout}"
  );
}

// S39: `--no-chrome` + `--dry-run` → preview without `--chrome`; stderr empty (`21_no_chrome.md` EC-6)
#[ test ]
fn s39_no_chrome_with_dry_run_preview_clean()
{
  let out = run_cli( &[ "--dry-run", "--no-chrome", "Fix bug" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "--no-chrome must suppress --chrome in dry-run preview. Got:\n{stdout}"
  );
  assert!(
    out.stderr.is_empty(),
    "--dry-run --no-chrome must produce no stderr. Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S40: `--no-persist` → `--no-session-persistence` in assembled command (`22_no_persist.md` EC-1)
#[ test ]
fn s40_no_persist_forwards_no_session_persistence()
{
  let out = run_cli( &[ "--dry-run", "--no-persist", "Fix bug" ] );
  assert!( out.status.success(), "--no-persist must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "--no-persist must forward --no-session-persistence. Got:\n{stdout}"
  );
}

// S41: default (no `--no-persist`) → no `--no-session-persistence` in assembled command (`22_no_persist.md` EC-2)
#[ test ]
fn s41_default_no_session_persistence_absent()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--no-session-persistence" ),
    "default command must not contain --no-session-persistence. Got:\n{stdout}"
  );
}

// S42: `--no-persist` without message → exit 0 (`22_no_persist.md` EC-3)
#[ test ]
fn s42_no_persist_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--no-persist" ] );
  assert!(
    out.status.success(),
    "--no-persist without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "--no-persist must forward --no-session-persistence. Got:\n{stdout}"
  );
}

// S43: `--help` output lists `--no-persist` (`22_no_persist.md` EC-4)
#[ test ]
fn s43_help_lists_no_persist()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-persist" ),
    "--help must mention --no-persist. Got:\n{stdout}"
  );
}

// S44: `--no-persist` + `--new-session` → `--no-session-persistence` present, no `-c` (`22_no_persist.md` EC-5)
#[ test ]
fn s44_no_persist_with_new_session_accepted()
{
  let out = run_cli( &[ "--dry-run", "--no-persist", "--new-session", "Fix bug" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "--no-persist must forward --no-session-persistence. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{stdout}"
  );
}

// S45: `--no-persist` + `--dry-run` → preview shows `--no-session-persistence`; stderr empty (`22_no_persist.md` EC-6)
#[ test ]
fn s45_no_persist_with_dry_run_preview_shows_flag()
{
  let out = run_cli( &[ "--dry-run", "--no-persist", "Fix bug" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "--no-persist must appear in dry-run preview. Got:\n{stdout}"
  );
  assert!(
    out.stderr.is_empty(),
    "--dry-run --no-persist must produce no stderr. Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// S46: `--json-schema <val>` forwarded in assembled command (`23_json_schema.md` EC-1)
#[ test ]
fn s46_json_schema_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--json-schema", r#"{"type":"object"}"#, "task" ] );
  assert!( out.status.success(), "--json-schema must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "--json-schema must appear in assembled command. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( r#"{"type":"object"}"# ),
    "--json-schema value must be forwarded. Got:\n{stdout}"
  );
}

// S47: default (no `--json-schema`) → absent from assembled command (`23_json_schema.md` EC-2)
#[ test ]
fn s47_default_json_schema_absent()
{
  let out = run_cli( &[ "--dry-run", "task" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--json-schema" ),
    "default command must not contain --json-schema. Got:\n{stdout}"
  );
}

// S48: complex nested schema forwarded verbatim (`23_json_schema.md` EC-3)
#[ test ]
fn s48_json_schema_complex_forwarded_verbatim()
{
  let schema = r#"{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}"#;
  let out = run_cli( &[ "--dry-run", "--json-schema", schema, "task" ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( schema ),
    "complex schema must be forwarded verbatim. Got:\n{stdout}"
  );
}

// S49: `--help` output lists `--json-schema` (`23_json_schema.md` EC-4)
#[ test ]
fn s49_help_lists_json_schema()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "--help must mention --json-schema. Got:\n{stdout}"
  );
}

// S50: `--json-schema` + `--model` → both forwarded (`23_json_schema.md` EC-5)
#[ test ]
fn s50_json_schema_with_model_both_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--json-schema", r#"{"type":"string"}"#, "--model", "sonnet", "task" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "--json-schema must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "--model sonnet" ),
    "--model must appear. Got:\n{stdout}"
  );
}

// S51: `--json-schema` without message → exit 0; schema in assembled command (`23_json_schema.md` EC-6)
#[ test ]
fn s51_json_schema_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--json-schema", r#"{"type":"string"}"# ] );
  assert!(
    out.status.success(),
    "--json-schema without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "--json-schema must appear in assembled command. Got:\n{stdout}"
  );
}

// S52: single `--mcp-config <path>` forwarded in assembled command (`24_mcp_config.md` EC-1)
#[ test ]
fn s52_mcp_config_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--mcp-config", "/tmp/mcp.json", "task" ] );
  assert!( out.status.success(), "--mcp-config must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config /tmp/mcp.json" ),
    "--mcp-config value must appear in assembled command. Got:\n{stdout}"
  );
}

// S53: default (no `--mcp-config`) → absent from assembled command (`24_mcp_config.md` EC-2)
#[ test ]
fn s53_default_mcp_config_absent()
{
  let out = run_cli( &[ "--dry-run", "task" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--mcp-config" ),
    "default command must not contain --mcp-config. Got:\n{stdout}"
  );
}

// S54: multiple `--mcp-config` flags → all forwarded as separate occurrences (`24_mcp_config.md` EC-3)
#[ test ]
fn s54_mcp_config_multiple_forwarded_individually()
{
  let out = run_cli( &[
    "--dry-run", "--mcp-config", "/tmp/s1.json", "--mcp-config", "/tmp/s2.json", "task",
  ] );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let count = stdout.matches( "--mcp-config" ).count();
  assert!(
    count >= 2,
    "multiple --mcp-config flags must each appear in assembled command (found {count}). Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "/tmp/s1.json" ),
    "first mcp-config path must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "/tmp/s2.json" ),
    "second mcp-config path must appear. Got:\n{stdout}"
  );
}

// S55: `--help` output lists `--mcp-config` (`24_mcp_config.md` EC-4)
#[ test ]
fn s55_help_lists_mcp_config()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config" ),
    "--help must mention --mcp-config. Got:\n{stdout}"
  );
}

// S56: `--mcp-config` + `--model` → both forwarded (`24_mcp_config.md` EC-5)
#[ test ]
fn s56_mcp_config_with_model_both_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--mcp-config", "/tmp/mcp.json", "--model", "sonnet", "task" ] );
  assert!( out.status.success() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config" ),
    "--mcp-config must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "--model sonnet" ),
    "--model must appear. Got:\n{stdout}"
  );
}

// S57: `--mcp-config` without message → exit 0; path in assembled command (`24_mcp_config.md` EC-6)
#[ test ]
fn s57_mcp_config_without_message_accepted()
{
  let out = run_cli( &[ "--dry-run", "--mcp-config", "/tmp/mcp.json" ] );
  assert!(
    out.status.success(),
    "--mcp-config without message must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config /tmp/mcp.json" ),
    "--mcp-config must appear in assembled command. Got:\n{stdout}"
  );
}
