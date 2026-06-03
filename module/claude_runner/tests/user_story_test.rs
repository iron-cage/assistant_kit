//! User Story Integration Tests
//!
//! ## Purpose
//!
//! End-to-end workflow tests implementing specs from `tests/docs/cli/user_story/`.
//! Each test function maps 1:1 to a spec case (US-N) for traceability.
//!
//! ## Strategy
//!
//! Most cases use `--dry-run` to inspect the assembled command without spawning a
//! Claude subprocess.  Error-path cases invoke real execution against absent or
//! invalid resources.  Credential subcommands (`isolated`, `refresh`) use
//! `--trace` since they lack `--dry-run` support.
//!
//! ## Doc Comment Convention
//!
//! Clippy `doc_markdown` lint flags `SCREAMING_SNAKE_CASE` identifiers and
//! `IDENT=value` patterns in `///` doc comments that are not wrapped in backticks.
//! All `CLR_*` env var names (e.g. `` `CLR_MODEL` ``) and `IDENT=value` patterns
//! (e.g. `` `CLR_NO_ULTRATHINK=true` ``) must use backticks in doc comments.
//! `--flag` patterns do **not** trigger this lint and may appear bare.
//!
//! ## Bool Env Var Semantics
//!
//! `CLR_*` bool env vars use a strict two-value rule: only `"1"` and `"true"`
//! (case-insensitive) are truthy.  `"yes"`, `"0"`, `"false"`, empty string,
//! and absent all evaluate to false (silently ignored).  US-18 US-4 exercises
//! this: `CLR_NO_ULTRATHINK=yes` is rejected — the ultrathink suffix remains
//! present in the assembled command, distinguishing it from the accepted
//! `CLR_NO_ULTRATHINK=true` case (US-18 US-3).
//!
//! ## Test Matrix
//!
//! | Spec | Story | Cases | Method |
//! |------|-------|-------|--------|
//! | 01 | Interactive REPL | US-1..4 (US-5 in param_edge_cases_test.rs) | dry-run / PATH trick |
//! | 02 | Print Mode Capture | US-1..4 | dry-run / PATH trick |
//! | 03 | Interactive With Message | US-1..4 | dry-run |
//! | 04 | Dry-run Preview | US-1..4 | dry-run |
//! | 05 | Project-specific Execution | US-1..4 | dry-run / error path |
//! | 06 | Verbose Debugging | US-1..4 | dry-run |
//! | 07 | Fresh Session | US-1..4 | dry-run |
//! | 08 | Trace Execution | US-1..4 | trace / dry-run / PATH trick |
//! | 09 | Custom System Prompt | US-1..4 | dry-run |
//! | 10 | Credential-isolated Execution | US-1..4 | trace / error path |
//! | 11 | File Input | US-1..4 | dry-run / error path |
//! | 12 | Code Block Extraction | US-1..4 | dry-run |
//! | 13 | Structured JSON Pipeline | US-1..4 | dry-run |
//! | 14 | Credential Refresh | US-1..4 | trace / error path |
//! | 15 | Ask Mode | US-1..4 | dry-run |
//! | 16 | CLI Discoverability | US-1..4 | help / PATH trick |
//! | 17 | Model Selection | US-1..4 | dry-run / env var |
//! | 18 | Env-var Configuration | US-1..4 | dry-run / env var |
//! | 19 | MCP Config Injection | US-1..4 | dry-run / env var |
//! | 20 | Suppress Effort Max | US-1..4 | dry-run / env var |
//! | 21 | Keep ClaudeCode Context | US-1..4 | dry-run / env var |
//! | 22 | Session Isolation via Subdirectory | US-1..5 | dry-run / env var |

#![ cfg( feature = "enabled" ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, make_session_dir };
use std::io::Write as _;
use std::process::Command;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Invoke `clr --dry-run` with extra args and return stdout.  Asserts exit 0.
fn run_dry( args : &[ &str ] ) -> String
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut full = vec![ "--dry-run" ];
  full.extend_from_slice( args );
  let out = Command::new( bin )
    .args( &full )
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

/// Invoke `clr ask --dry-run` with extra args and return stdout.  Asserts exit 0.
fn run_ask_dry( args : &[ &str ] ) -> String
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut full = vec![ "ask", "--dry-run" ];
  full.extend_from_slice( args );
  let out = Command::new( bin )
    .args( &full )
    .output()
    .expect( "Failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "ask dry-run failed (exit {}): {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stderr )
  );
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

/// Write `content` to a new temp file and return it (caller must keep alive).
fn make_creds_file( content : &str ) -> tempfile::NamedTempFile
{
  let mut f = tempfile::NamedTempFile::new().expect( "failed to create temp creds file" );
  f.write_all( content.as_bytes() ).expect( "failed to write creds content" );
  f
}

fn exit_code( o : &std::process::Output ) -> i32 { o.status.code().unwrap_or( -1 ) }
fn stderr_str( o : &std::process::Output ) -> String { String::from_utf8_lossy( &o.stderr ).to_string() }

// ── US01: Interactive REPL ──────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/01_interactive_repl.md

/// US-1: bare clr opens REPL — subprocess args include -c and --dangerously-skip-permissions.
///
/// Validated via --dry-run (no message → REPL route). Print mode is NOT injected.
#[ test ]
fn us01_1_bare_clr_repl_defaults()
{
  let ( _session, session_path ) = make_session_dir();
  let output = run_dry( &[ "--session-dir", &session_path ] );
  assert!(
    output.contains( " -c" ),
    "REPL mode must inject -c (session continuation). Got:\n{output}"
  );
  assert!(
    output.contains( "--dangerously-skip-permissions" ),
    "REPL mode must inject --dangerously-skip-permissions. Got:\n{output}"
  );
  assert!(
    !output.contains( "--print" ),
    "REPL mode (no message) must NOT inject --print. Got:\n{output}"
  );
}

/// US-2: session continuation flag -c present in REPL subprocess args.
#[ test ]
fn us01_2_session_continuation_flag_present()
{
  let ( _session, session_path ) = make_session_dir();
  let output = run_dry( &[ "--session-dir", &session_path ] );
  assert!(
    output.contains( " -c" ),
    "dry-run must show -c for default session continuation. Got:\n{output}"
  );
}

/// US-3: non-interactive environment without message → process exits with error.
///
/// Invoked with PATH=/nonexistent so claude binary is absent. Without a message,
/// clr attempts interactive mode which fails when the subprocess cannot be spawned.
#[ test ]
fn us01_3_non_interactive_no_message_errors()
{
  let out = run_cli_with_env( &[], &[ ( "PATH", "/nonexistent" ) ] );
  assert!(
    !out.status.success(),
    "bare clr without claude must exit non-zero. Got exit: {:?}",
    out.status.code()
  );
}

/// US-4: REPL with --dir changes subprocess working directory.
#[ test ]
fn us01_4_repl_with_custom_dir()
{
  let ( _session, session_path ) = make_session_dir();
  let output = run_dry( &[ "--dir", "/tmp", "--session-dir", &session_path ] );
  assert!(
    output.contains( "cd /tmp" ),
    "--dir must produce 'cd /tmp' prefix. Got:\n{output}"
  );
  assert!(
    output.contains( " -c" ),
    "non-empty --session-dir must inject -c. Got:\n{output}"
  );
}

// ── US02: Print Mode Capture ────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/02_print_mode_capture.md

/// US-1: message argument triggers print mode by default.
#[ test ]
fn us02_1_message_triggers_print_mode()
{
  let output = run_dry( &[ "Explain this function" ] );
  assert!(
    output.contains( "--print" ),
    "message must trigger --print by default. Got:\n{output}"
  );
}

/// US-2: explicit --print with message produces print-mode command.
#[ test ]
fn us02_2_explicit_print_with_message()
{
  let output = run_dry( &[ "-p", "List all files" ] );
  assert!(
    output.contains( "--print" ),
    "-p must add --print to command. Got:\n{output}"
  );
}

/// US-3: --print without message in non-TTY environment → exits non-zero.
///
/// PATH is set to /nonexistent so the subprocess cannot be found. The error
/// path (binary absent) triggers regardless of print-mode state.
#[ test ]
fn us02_3_print_without_message_errors()
{
  let out = run_cli_with_env( &[ "--print" ], &[ ( "PATH", "/nonexistent" ) ] );
  assert!(
    !out.status.success(),
    "--print without message + no claude must exit non-zero. Got exit: {:?}",
    out.status.code()
  );
}

/// US-4: print mode output is capturable — dry-run shows command on stdout.
#[ test ]
fn us02_4_output_capturable_via_stdout()
{
  let output = run_dry( &[ "Generate a greeting" ] );
  assert!(
    output.contains( "claude" ),
    "dry-run stdout must contain assembled command. Got:\n{output}"
  );
  assert!(
    output.contains( "--print" ),
    "message must trigger --print. Got:\n{output}"
  );
}

// ── US03: Interactive With Message ──────────────────────────────────────────
// Source: tests/docs/cli/user_story/03_interactive_with_message.md

/// US-1: --interactive with message keeps interactive mode (no --print).
#[ test ]
fn us03_1_interactive_with_message_no_print()
{
  let output = run_dry( &[ "--interactive", "Fix the bug" ] );
  assert!(
    !output.contains( "--print" ),
    "--interactive must suppress --print even with message. Got:\n{output}"
  );
}

/// US-2: --interactive overrides message-triggers-print default.
#[ test ]
fn us03_2_interactive_overrides_print_default()
{
  let output = run_dry( &[ "--interactive", "Review code" ] );
  assert!(
    !output.contains( "--print" ),
    "--interactive must override print-on-message default. Got:\n{output}"
  );
}

/// US-3: --interactive without TTY — subprocess attempts interactive mode.
///
/// PATH trick ensures deterministic failure; we only assert the process ran
/// (no parse-level error about --interactive requiring a TTY).
#[ test ]
fn us03_3_interactive_without_tty()
{
  let out = run_cli_with_env(
    &[ "--interactive", "Fix it" ],
    &[ ( "PATH", "/nonexistent" ) ],
  );
  let stderr = stderr_str( &out );
  assert!(
    !stderr.contains( "unknown option" ),
    "--interactive must be accepted. Got:\n{stderr}"
  );
}

/// US-4: --interactive with --new-session starts fresh conversation.
#[ test ]
fn us03_4_interactive_with_new_session()
{
  let output = run_dry( &[ "--interactive", "--new-session", "Start analysis" ] );
  assert!(
    !output.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{output}"
  );
  assert!(
    !output.contains( "--print" ),
    "--interactive must suppress --print. Got:\n{output}"
  );
}

// ── US04: Dry-run Preview ───────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/04_dry_run_preview.md

/// US-1: --dry-run prints assembled command without executing.
#[ test ]
fn us04_1_dry_run_prints_command()
{
  let ( _session, session_path ) = make_session_dir();
  let output = run_dry( &[ "--session-dir", &session_path, "test message" ] );
  assert!(
    output.contains( "--dangerously-skip-permissions" ),
    "dry-run must show --dangerously-skip-permissions. Got:\n{output}"
  );
  assert!(
    output.contains( " -c" ),
    "dry-run must show -c. Got:\n{output}"
  );
  assert!(
    output.contains( "--chrome" ),
    "dry-run must show --chrome. Got:\n{output}"
  );
  assert!(
    output.contains( "--effort max" ),
    "dry-run must show --effort max. Got:\n{output}"
  );
}

/// US-2: all injected defaults visible in dry-run output.
#[ test ]
fn us04_2_all_defaults_visible()
{
  let ( _session, session_path ) = make_session_dir();
  let output = run_dry( &[ "--session-dir", &session_path, "test" ] );
  assert!( output.contains( " -c" ), "must have -c. Got:\n{output}" );
  assert!(
    output.contains( "--dangerously-skip-permissions" ),
    "must have --dangerously-skip-permissions. Got:\n{output}"
  );
  assert!( output.contains( "--chrome" ), "must have --chrome. Got:\n{output}" );
  assert!( output.contains( "--effort max" ), "must have --effort max. Got:\n{output}" );
  assert!(
    output.contains( "ultrathink" ),
    "message must have ultrathink suffix. Got:\n{output}"
  );
}

/// US-3: dry-run with model override shows --model in output.
#[ test ]
fn us04_3_dry_run_with_model_override()
{
  let output = run_dry( &[ "--model", "sonnet", "test" ] );
  assert!(
    output.contains( "--model sonnet" ),
    "--model sonnet must appear in dry-run output. Got:\n{output}"
  );
  assert!(
    output.contains( "--dangerously-skip-permissions" ),
    "other defaults must still be present. Got:\n{output}"
  );
}

/// US-4: dry-run always exits 0 regardless of flags.
#[ test ]
fn us04_4_dry_run_exit_always_zero()
{
  let output = run_dry( &[ "--verbose", "--new-session", "test" ] );
  assert!(
    !output.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{output}"
  );
  // exit 0 is asserted by run_dry() itself
}

// ── US05: Project-specific Execution ────────────────────────────────────────
// Source: tests/docs/cli/user_story/05_project_specific_execution.md

/// US-1: --dir sets subprocess working directory.
#[ test ]
fn us05_1_dir_sets_working_directory()
{
  let output = run_dry( &[ "--dir", "/tmp/my_project", "fix tests" ] );
  assert!(
    output.contains( "cd /tmp/my_project" ),
    "--dir must produce cd prefix. Got:\n{output}"
  );
}

/// US-2: --dir with --session-dir for full project isolation.
#[ test ]
fn us05_2_dir_with_session_dir()
{
  let output = run_dry( &[
    "--dir", "/tmp/project_a",
    "--session-dir", "/tmp/sessions_a",
    "analyze",
  ] );
  assert!(
    output.contains( "cd /tmp/project_a" ),
    "cd prefix must appear. Got:\n{output}"
  );
  assert!(
    output.contains( "CLAUDE_CODE_SESSION_DIR=/tmp/sessions_a" ),
    "session dir env var must appear. Got:\n{output}"
  );
}

/// US-3: --dir with non-existent path → exit non-zero.
#[ test ]
fn us05_3_nonexistent_dir_errors()
{
  let out = run_cli( &[
    "--dir", "/tmp/clr_nonexistent_project_us05_3",
    "fix it",
  ] );
  assert!(
    !out.status.success(),
    "--dir with non-existent path must exit non-zero. Got exit: {:?}",
    out.status.code()
  );
}

/// US-4: --dir with --new-session prevents context bleed.
#[ test ]
fn us05_4_dir_with_new_session()
{
  let output = run_dry( &[
    "--dir", "/tmp/my_project",
    "--new-session",
    "start fresh",
  ] );
  assert!(
    !output.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{output}"
  );
  assert!(
    output.contains( "cd /tmp/my_project" ),
    "--dir must still produce cd prefix. Got:\n{output}"
  );
}

// ── US06: Verbose Debugging ─────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/06_verbose_debugging.md

/// US-1: --verbosity 4 with --dry-run shows command on stdout.
#[ test ]
fn us06_1_verbosity_4_shows_command()
{
  let output = run_dry( &[ "--verbosity", "4", "test" ] );
  assert!(
    output.contains( "claude" ),
    "dry-run at verbosity 4 must show command on stdout. Got:\n{output}"
  );
}

/// US-2: --verbosity 0 does not suppress dry-run output.
#[ test ]
fn us06_2_verbosity_0_no_suppress()
{
  let output = run_dry( &[ "--verbosity", "0", "test" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "dry-run at verbosity 0 must still show env+command. Got:\n{output}"
  );
}

/// US-3: --verbosity 5 with --dry-run shows command on stdout.
#[ test ]
fn us06_3_verbosity_5_shows_command()
{
  let output = run_dry( &[ "--verbosity", "5", "test" ] );
  assert!(
    output.contains( "claude" ),
    "dry-run at verbosity 5 must show command on stdout. Got:\n{output}"
  );
}

/// US-4: dry-run always shows output regardless of verbosity level.
#[ test ]
fn us06_4_dry_run_independent_of_verbosity()
{
  let output = run_dry( &[ "--verbosity", "0", "test" ] );
  assert!(
    output.contains( "claude" ),
    "dry-run must show command even at verbosity 0. Got:\n{output}"
  );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "dry-run must show env even at verbosity 0. Got:\n{output}"
  );
}

// ── US07: Fresh Session ─────────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/07_fresh_session.md

/// US-1: --new-session omits -c from assembled command.
#[ test ]
fn us07_1_new_session_omits_continuation()
{
  let output = run_dry( &[ "--new-session", "start fresh" ] );
  assert!(
    !output.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{output}"
  );
}

/// US-2: other defaults preserved when --new-session is active.
#[ test ]
fn us07_2_other_defaults_preserved()
{
  let output = run_dry( &[ "--new-session", "test" ] );
  assert!( !output.contains( " -c" ), "-c must be absent. Got:\n{output}" );
  assert!(
    output.contains( "--dangerously-skip-permissions" ),
    "skip-permissions must remain. Got:\n{output}"
  );
  assert!( output.contains( "--chrome" ), "--chrome must remain. Got:\n{output}" );
  assert!( output.contains( "--effort max" ), "--effort max must remain. Got:\n{output}" );
  assert!(
    output.contains( "ultrathink" ),
    "ultrathink suffix must remain. Got:\n{output}"
  );
}

/// US-3: fresh session in print mode — message triggers --print, no -c.
#[ test ]
fn us07_3_fresh_session_print_mode()
{
  let output = run_dry( &[ "--new-session", "Review this code" ] );
  assert!(
    output.contains( "--print" ),
    "message must trigger --print. Got:\n{output}"
  );
  assert!(
    !output.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{output}"
  );
}

/// US-4: fresh session in interactive mode — --interactive and no -c.
#[ test ]
fn us07_4_fresh_session_interactive()
{
  let output = run_dry( &[ "--new-session", "--interactive", "Begin analysis" ] );
  assert!(
    !output.contains( " -c" ),
    "--new-session must suppress -c. Got:\n{output}"
  );
  assert!(
    !output.contains( "--print" ),
    "--interactive must suppress --print. Got:\n{output}"
  );
}

// ── US08: Trace Execution ───────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/08_trace_execution.md

/// US-1: --trace prints command to stderr before executing.
///
/// PATH=/nonexistent forces exit 1 (claude absent) but trace fires before attempt.
#[ test ]
fn us08_1_trace_prints_command_to_stderr()
{
  let out = run_cli_with_env( &[ "--trace", "test message" ], &[ ( "PATH", "/nonexistent" ) ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "claude" ),
    "--trace must emit assembled command to stderr. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--trace must emit env vars to stderr. Got:\n{stderr}"
  );
}

/// US-2: --trace works on isolated command — stderr has credential trace format.
#[ test ]
fn us08_2_trace_on_isolated()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out = run_cli( &[ "isolated", "--creds", path, "--trace", "test" ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "# clr isolated" ),
    "isolated --trace must emit '# clr isolated'. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "# creds:" ),
    "isolated --trace must emit '# creds:'. Got:\n{stderr}"
  );
}

/// US-3: --trace output independent of --verbosity level.
///
/// Trace is a separate channel from runner verbosity diagnostics.
/// PATH=/nonexistent forces exit 1 but trace fires first.
#[ test ]
fn us08_3_trace_independent_of_verbosity()
{
  let out = run_cli_with_env(
    &[ "--trace", "--verbosity", "0", "test" ],
    &[ ( "PATH", "/nonexistent" ) ],
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "claude" ),
    "--trace must emit to stderr even at --verbosity 0. Got:\n{stderr}"
  );
}

/// US-4: --trace with --dry-run — dry-run wins, stderr empty.
#[ test ]
fn us08_4_trace_with_dry_run()
{
  let out = run_cli( &[ "--trace", "--dry-run", "test" ] );
  assert!( out.status.success(), "--trace --dry-run must exit 0" );
  assert!(
    out.stderr.is_empty(),
    "--trace must not fire when --dry-run wins. Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "claude" ),
    "dry-run stdout must still show assembled command. Got:\n{stdout}"
  );
}

// ── US09: Custom System Prompt ──────────────────────────────────────────────
// Source: tests/docs/cli/user_story/09_custom_system_prompt.md

/// US-1: --system-prompt replaces default system prompt.
#[ test ]
fn us09_1_replace_system_prompt()
{
  let output = run_dry( &[ "--system-prompt", "You are a Python expert", "test" ] );
  assert!(
    output.contains( "--system-prompt" ),
    "--system-prompt must appear in output. Got:\n{output}"
  );
  assert!(
    output.contains( "You are a Python expert" ),
    "prompt text must appear in output. Got:\n{output}"
  );
}

/// US-2: --append-system-prompt extends default system prompt.
#[ test ]
fn us09_2_append_system_prompt()
{
  let output = run_dry( &[ "--append-system-prompt", "Always respond in JSON", "test" ] );
  assert!(
    output.contains( "--append-system-prompt" ),
    "--append-system-prompt must appear in output. Got:\n{output}"
  );
  assert!(
    output.contains( "Always respond in JSON" ),
    "append text must appear in output. Got:\n{output}"
  );
}

/// US-3: both system-prompt flags combined.
#[ test ]
fn us09_3_replace_then_append()
{
  let output = run_dry( &[
    "--system-prompt", "Base prompt",
    "--append-system-prompt", "Extra rule",
    "test",
  ] );
  assert!(
    output.contains( "--system-prompt" ),
    "--system-prompt must appear. Got:\n{output}"
  );
  assert!(
    output.contains( "--append-system-prompt" ),
    "--append-system-prompt must appear. Got:\n{output}"
  );
}

/// US-4: empty string system prompt clears default.
#[ test ]
fn us09_4_empty_system_prompt()
{
  let output = run_dry( &[ "--system-prompt", "", "test" ] );
  assert!(
    output.contains( "--system-prompt" ),
    "--system-prompt must appear even with empty value. Got:\n{output}"
  );
}

// ── US10: Credential-isolated Execution ─────────────────────────────────────
// Source: tests/docs/cli/user_story/10_credential_isolated_execution.md

/// US-1: isolated --creds runs with temp HOME isolation.
///
/// Verified via --trace output (isolated has no --dry-run support).
#[ test ]
fn us10_1_credential_isolation()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out = run_cli( &[ "isolated", "--creds", path, "--trace", "test" ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "# clr isolated" ),
    "isolated --trace must show '# clr isolated'. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "# creds:" ),
    "isolated --trace must show credential path. Got:\n{stderr}"
  );
}

/// US-2: --timeout controls subprocess wait time.
///
/// Custom timeout value visible in trace output.
#[ test ]
fn us10_2_timeout_controls_duration()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out = run_cli( &[ "isolated", "--creds", path, "--timeout", "120", "--trace", "long task" ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "# timeout: 120s" ),
    "isolated --trace must show custom timeout. Got:\n{stderr}"
  );
}

/// US-3: --creds with non-existent file → exit non-zero.
#[ test ]
fn us10_3_nonexistent_creds_errors()
{
  let out = run_cli( &[ "isolated", "--creds", "/tmp/clr_us10_nonexistent.json", "test" ] );
  assert!(
    !out.status.success(),
    "isolated with nonexistent creds must exit non-zero. Got exit: {:?}",
    out.status.code()
  );
}

/// US-4: isolated with `HOME` unset and no `CLR_CREDS` → exit 1, error references `HOME`.
///
/// With `--creds` omitted and `HOME` unset, the 3rd-tier default cannot be resolved.
#[ test ]
fn us10_4_isolated_without_creds_errors()
{
  let out = Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "isolated", "test" ] )
    .env_remove( "HOME" )
    .env_remove( "CLR_CREDS" )
    .output()
    .expect( "invoke clr isolated" );

  assert_eq!( exit_code( &out ), 1, "isolated with HOME unset must exit 1" );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "HOME" ) || stderr.contains( "cannot resolve" ),
    "error must reference HOME or resolution failure; got:\n{stderr}"
  );
}

// ── US11: File Input ────────────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/11_file_input.md

/// US-1: --file pipes file content as subprocess stdin.
///
/// In dry-run output, --file appears as `< /path/to/file`.
#[ test ]
fn us11_1_file_piped_as_stdin()
{
  let tmp = tempfile::NamedTempFile::new().expect( "create temp" );
  std::fs::write( tmp.path(), "test input content" ).expect( "write" );
  let path_str = tmp.path().to_str().unwrap();
  let output = run_dry( &[ "--file", path_str, "Summarize this" ] );
  assert!(
    output.contains( "< " ),
    "dry-run must show stdin redirect for --file. Got:\n{output}"
  );
  assert!(
    output.contains( path_str ),
    "dry-run must show the file path. Got:\n{output}"
  );
}

/// US-2: --file with print mode and message.
#[ test ]
fn us11_2_file_with_print_mode()
{
  let tmp = tempfile::NamedTempFile::new().expect( "create temp" );
  std::fs::write( tmp.path(), "data" ).expect( "write" );
  let path_str = tmp.path().to_str().unwrap();
  let output = run_dry( &[ "--file", path_str, "Extract the key points" ] );
  assert!(
    output.contains( "--print" ),
    "message must trigger --print. Got:\n{output}"
  );
  assert!(
    output.contains( "< " ),
    "--file must appear as stdin redirect. Got:\n{output}"
  );
}

/// US-3: --file with non-readable path → exit non-zero with path in error.
#[ test ]
fn us11_3_nonreadable_file_errors()
{
  let out = run_cli( &[ "--file", "/tmp/clr_us11_nonexistent_99999.txt", "test" ] );
  assert!(
    !out.status.success(),
    "--file with nonexistent path must exit non-zero. Got exit: {:?}",
    out.status.code()
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "/tmp/clr_us11_nonexistent_99999.txt" ),
    "error must include the file path. Got:\n{stderr}"
  );
}

/// US-4: --file path resolved relative to effective directory.
///
/// With --dir changing the working directory, --file path appears in the assembled command.
#[ test ]
fn us11_4_file_path_with_dir()
{
  let tmp = tempfile::NamedTempFile::new().expect( "create temp" );
  std::fs::write( tmp.path(), "data" ).expect( "write" );
  let path_str = tmp.path().to_str().unwrap();
  let output = run_dry( &[ "--dir", "/tmp", "--file", path_str, "Analyze" ] );
  assert!(
    output.contains( "cd /tmp" ),
    "--dir must produce cd prefix. Got:\n{output}"
  );
  assert!(
    output.contains( path_str ),
    "--file path must appear in output. Got:\n{output}"
  );
}

// ── US12: Code Block Extraction ─────────────────────────────────────────────
// Source: tests/docs/cli/user_story/12_code_block_extraction.md

/// US-1: --strip-fences appears in assembled command.
///
/// Stripping is a post-processing step; dry-run shows the flag acceptance.
#[ test ]
fn us12_1_strip_fences_in_command()
{
  let output = run_dry( &[ "--strip-fences", "Generate a hello world function" ] );
  // --strip-fences is a runner-level flag, not forwarded to claude subprocess.
  // Dry-run shows the claude command; strip-fences applies post-process.
  // We verify the flag is accepted (exit 0 via run_dry assertion).
  assert!(
    output.contains( "claude" ),
    "dry-run must show assembled command. Got:\n{output}"
  );
}

/// US-2: --strip-fences with no fence pair — passthrough (no-op).
///
/// Dry-run verifies the flag is accepted; runtime no-op tested in `execution_mode_test`.
#[ test ]
fn us12_2_no_fence_pair_passthrough()
{
  // --strip-fences accepted even for non-code queries; no-op at runtime.
  let out = run_cli( &[ "--dry-run", "--strip-fences", "What is 2+2?" ] );
  assert!(
    out.status.success(),
    "--strip-fences must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

/// US-3: --strip-fences with --file for code generation pipeline.
#[ test ]
fn us12_3_strip_fences_with_file()
{
  let tmp = tempfile::NamedTempFile::new().expect( "create temp" );
  std::fs::write( tmp.path(), "{}" ).expect( "write" );
  let path_str = tmp.path().to_str().unwrap();
  let output = run_dry( &[ "--file", path_str, "--strip-fences", "Generate code from this schema" ] );
  assert!(
    output.contains( "< " ),
    "--file must appear as stdin redirect. Got:\n{output}"
  );
  // --strip-fences accepted (exit 0) alongside --file
}

/// US-4: --strip-fences has no effect in --dry-run mode.
///
/// Stripping is post-processing on subprocess output; dry-run returns before
/// any subprocess runs, so stripping never fires.
#[ test ]
fn us12_4_strip_fences_ignored_in_dry_run()
{
  let out = run_cli( &[ "--dry-run", "--strip-fences", "Generate code" ] );
  assert!(
    out.status.success(),
    "--strip-fences with --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "claude" ),
    "dry-run must show command. Got:\n{stdout}"
  );
}

// ── US13: Structured JSON Pipeline ──────────────────────────────────────────
// Source: tests/docs/cli/user_story/13_structured_json_pipeline.md

/// US-1: --json-schema constrains output to given schema.
#[ test ]
fn us13_1_json_schema_constrained()
{
  let schema = r#"{"type":"object","properties":{"name":{"type":"string"}}}"#;
  let output = run_dry( &[ "--json-schema", schema, "Extract the author name" ] );
  assert!(
    output.contains( "--json-schema" ),
    "--json-schema must appear in assembled command. Got:\n{output}"
  );
  assert!(
    output.contains( r#""type":"object""# ),
    "schema value must be forwarded. Got:\n{output}"
  );
}

/// US-2: --json-schema with --strip-fences for bare JSON output.
#[ test ]
fn us13_2_json_schema_with_strip_fences()
{
  let output = run_dry( &[
    "--json-schema", r#"{"type":"object"}"#,
    "--strip-fences",
    "Extract data",
  ] );
  assert!(
    output.contains( "--json-schema" ),
    "--json-schema must appear. Got:\n{output}"
  );
  // --strip-fences accepted alongside --json-schema (exit 0 via run_dry)
}

/// US-3: file-driven JSON extraction with --file + --json-schema + --strip-fences.
#[ test ]
fn us13_3_file_driven_json_extraction()
{
  let tmp = tempfile::NamedTempFile::new().expect( "create temp" );
  std::fs::write( tmp.path(), "data to extract" ).expect( "write" );
  let path_str = tmp.path().to_str().unwrap();
  let output = run_dry( &[
    "--file", path_str,
    "--json-schema", r#"{"type":"array"}"#,
    "--strip-fences",
    "Extract entities",
  ] );
  assert!(
    output.contains( "--json-schema" ),
    "--json-schema must appear. Got:\n{output}"
  );
  assert!(
    output.contains( "< " ),
    "--file must appear as stdin redirect. Got:\n{output}"
  );
}

/// US-4: schema passed inline appears in assembled command.
#[ test ]
fn us13_4_schema_inline()
{
  let schema = r#"{"type":"object","properties":{"result":{"type":"string"}}}"#;
  let output = run_dry( &[ "--json-schema", schema, "Extract" ] );
  assert!(
    output.contains( "--json-schema" ),
    "--json-schema must appear. Got:\n{output}"
  );
  assert!(
    output.contains( "result" ),
    "schema property 'result' must appear in output. Got:\n{output}"
  );
}

// ── US14: Credential Refresh ────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/14_credential_refresh.md

/// US-1: refresh --creds refreshes OAuth token.
///
/// Verified via --trace output (refresh has no --dry-run support).
#[ test ]
fn us14_1_refresh_credentials()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out = run_cli( &[ "refresh", "--creds", path, "--trace" ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "# clr refresh" ),
    "refresh --trace must emit '# clr refresh'. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "# creds:" ),
    "refresh --trace must emit '# creds:'. Got:\n{stderr}"
  );
}

/// US-2: default timeout is 45 seconds for refresh.
#[ test ]
fn us14_2_default_timeout_45s()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out = run_cli( &[ "refresh", "--creds", path, "--trace" ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "# timeout: 45s" ),
    "refresh default timeout must be 45s. Got:\n{stderr}"
  );
}

/// US-3: non-existent credentials file → exit non-zero.
#[ test ]
fn us14_3_nonexistent_creds_errors()
{
  let out = run_cli( &[ "refresh", "--creds", "/tmp/clr_us14_nonexistent.json" ] );
  assert!(
    !out.status.success(),
    "refresh with nonexistent creds must exit non-zero. Got exit: {:?}",
    out.status.code()
  );
}

/// US-4: --trace shows `run_isolated` details on stderr.
#[ test ]
fn us14_4_trace_shows_details()
{
  let creds = make_creds_file( "{}" );
  let path  = creds.path().to_str().expect( "temp path is valid UTF-8" );
  let out = run_cli( &[ "refresh", "--creds", path, "--trace" ] );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "# clr refresh" ),
    "refresh --trace must emit '# clr refresh'. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "# timeout: 45s" ),
    "refresh --trace must emit timeout. Got:\n{stderr}"
  );
  let code = exit_code( &out );
  assert!( code == 0 || code == 1, "expected exit 0 or 1; got {code}" );
}

// ── US15: Ask Mode ──────────────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/15_ask_mode.md

/// US-1: ask applies conservative defaults — no -c, no skip-perms bypass, effort high, max 16384.
#[ test ]
fn us15_1_conservative_ask_defaults()
{
  let output = run_ask_dry( &[ "What does this function do?" ] );
  assert!(
    !output.contains( " -c" ),
    "ask must not include -c. Got:\n{output}"
  );
  assert!(
    !output.contains( "--dangerously-skip-permissions" ),
    "ask must not include --dangerously-skip-permissions. Got:\n{output}"
  );
  assert!(
    output.contains( "--effort high" ),
    "ask must use --effort high. Got:\n{output}"
  );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384" ),
    "ask must use max tokens 16384. Got:\n{output}"
  );
}

/// US-2: print mode always on for ask regardless of message presence.
#[ test ]
fn us15_2_print_mode_always_on()
{
  let output = run_ask_dry( &[ "Explain closures" ] );
  assert!(
    output.contains( "--print" ),
    "ask must always include --print. Got:\n{output}"
  );
}

/// US-3: ask defaults overridable via explicit flags.
#[ test ]
fn us15_3_override_ask_defaults()
{
  let output = run_ask_dry( &[ "--effort", "max", "--max-tokens", "200000", "Write a detailed analysis" ] );
  assert!(
    output.contains( "--effort max" ),
    "--effort max must override ask default. Got:\n{output}"
  );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000" ),
    "--max-tokens 200000 must override ask default. Got:\n{output}"
  );
}

/// US-4: --no-persist and --no-chrome default ON for ask.
#[ test ]
fn us15_4_no_persist_no_chrome_defaults()
{
  let output = run_ask_dry( &[ "Quick question" ] );
  assert!(
    !output.contains( "--chrome" ),
    "ask must suppress --chrome (--no-chrome default ON). Got:\n{output}"
  );
}

// ── US16: CLI Discoverability ────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/16_cli_discoverability.md

/// US-1: `clr help` prints usage to stdout and exits 0.
#[ test ]
fn us16_1_help_prints_usage()
{
  let out = run_cli( &[ "help" ] );
  assert!( out.status.success(), "clr help must exit 0. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "USAGE:" ),
    "`clr help` must print USAGE section. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "COMMANDS:" ),
    "`clr help` must print COMMANDS section. Got:\n{stdout}"
  );
}

/// US-2: `clr -h` and `clr --help` produce identical output to `clr help`.
#[ test ]
fn us16_2_flag_aliases_identical()
{
  let help_out  = run_cli( &[ "help" ] );
  let short_out = run_cli( &[ "-h" ] );
  let long_out  = run_cli( &[ "--help" ] );
  assert!( help_out.status.success(),  "`clr help` must exit 0" );
  assert!( short_out.status.success(), "`clr -h` must exit 0" );
  assert!( long_out.status.success(),  "`clr --help` must exit 0" );
  assert_eq!(
    help_out.stdout, short_out.stdout,
    "`clr -h` output must be identical to `clr help`"
  );
  assert_eq!(
    help_out.stdout, long_out.stdout,
    "`clr --help` output must be identical to `clr help`"
  );
}

/// US-3: help output lists all 5 subcommands and available flags.
///
/// All 5 named subcommands — run, ask, isolated, refresh, help — appear in COMMANDS.
/// `run` is both the default invocation form (shown in USAGE as `clr [OPTIONS] [MESSAGE]`)
/// and an explicit subcommand (`clr run [OPTIONS] [MESSAGE]`).
#[ test ]
fn us16_3_all_subcommands_listed()
{
  let out = run_cli( &[ "help" ] );
  assert!( out.status.success(), "clr help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "COMMANDS:" ),
    "clr help must print COMMANDS section. Got:\n{stdout}"
  );
  // Extract COMMANDS block to assert each subcommand appears there (not just anywhere in output).
  let after_cmds = stdout
    .split_once( "COMMANDS:\n" )
    .map_or( "", | ( _, rest ) | rest );
  let cmds_block = after_cmds
    .split_once( "\nARGUMENTS:" )
    .map_or( after_cmds, | ( block, _ ) | block );
  assert!(
    cmds_block.lines().any( | l | l.trim_start().starts_with( "run" ) ),
    "COMMANDS section must list 'run' subcommand. Got COMMANDS block:\n{cmds_block}"
  );
  assert!(
    stdout.contains( "ask" ),
    "clr help must list 'ask' subcommand. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "isolated" ),
    "clr help must list 'isolated' subcommand. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "refresh" ),
    "clr help must list 'refresh' subcommand. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "help" ),
    "clr help must list 'help' subcommand. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "[OPTIONS]" ),
    "clr help USAGE must show [OPTIONS]. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "--dry-run" ),
    "clr help must list --dry-run flag. Got:\n{stdout}"
  );
}

/// US-4: `clr help` launches no subprocess, reads no credentials, mutates no session state.
///
/// PATH=/nonexistent ensures any subprocess attempt would fail with binary-not-found.
/// `clr help` still exits 0, proving no claude invocation occurs.
#[ test ]
fn us16_4_no_side_effects()
{
  let out = run_cli_with_env( &[ "help" ], &[ ( "PATH", "/nonexistent" ) ] );
  assert!(
    out.status.success(),
    "clr help must exit 0 even with PATH=/nonexistent (no subprocess launched). Got: {:?}",
    out.status.code()
  );
  assert!(
    out.stderr.is_empty(),
    "clr help must produce no stderr. Got:\n{}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "USAGE:" ),
    "clr help output must be complete even without claude in PATH. Got:\n{stdout}"
  );
}

/// MRE: `clr help` COMMANDS section omits the `run` subcommand.
///
/// ## Root Cause
/// `print_help()` lists only 4 named subcommands in COMMANDS (ask, isolated, refresh, help).
/// `run` — the default mode, also invocable as `clr run [OPTIONS] [MESSAGE]` — is absent.
/// The feature doc AC (`docs/cli/user_story/016_cli_discoverability.md`) and test spec
/// (`tests/docs/cli/user_story/16_cli_discoverability.md` US-3) both require all 5 subcommands
/// (run, isolated, refresh, ask, help) to appear in the help output.
///
/// ## Why Not Caught
/// `us16_3_all_subcommands_listed` was written to work around the gap: it asserted `[OPTIONS]`
/// (which appears in USAGE for the default run form) as a proxy for `run`, rather than asserting
/// the string "run" directly in the COMMANDS section. The validation marked the task complete
/// without verifying that `run` appeared as a named subcommand per the AC.
///
/// ## Fix Applied
/// `print_help()` now includes `run` as a named subcommand in COMMANDS and in USAGE.
/// `run_cli()` strips a leading `run` token so `clr run [ARGS]` behaves identically to
/// `clr [ARGS]`. `guard_unknown_subcommand` adds `run` to KNOWN for prefix-match protection.
///
/// ## Prevention
/// US-3 acceptance criterion must be verified by extracting the COMMANDS block and asserting
/// each named subcommand, not by checking substrings that could match non-command contexts.
///
/// ## Pitfall
/// `stdout.contains("run")` is ambiguous: "--dry-run" in OPTIONS also matches. Always verify
/// `run` within the COMMANDS block specifically (between "COMMANDS:" and the next section).
///
/// Fix(BUG-212)
#[ test ]
fn bug_mre_212_run_subcommand_not_in_help_commands()
{
  let out = run_cli( &[ "help" ] );
  assert!( out.status.success(), "clr help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  // Extract only the COMMANDS block (after "COMMANDS:\n", before "\nARGUMENTS:").
  let after_cmds = stdout
    .split_once( "COMMANDS:\n" )
    .map_or( "", | ( _, rest ) | rest );
  let cmds_block = after_cmds
    .split_once( "\nARGUMENTS:" )
    .map_or( after_cmds, | ( block, _ ) | block );
  assert!(
    cmds_block.lines().any( | l | l.trim_start().starts_with( "run" ) ),
    "COMMANDS section must list 'run' as a named subcommand.\n\
     Fix(BUG-212): add 'run' to print_help() COMMANDS section.\n\
     Got COMMANDS block:\n{cmds_block}"
  );
}

/// IN-4: `clr run [ARGS]` (explicit bare-word) produces identical output to `clr [ARGS]` (default).
///
/// ## Root Cause
/// Before BUG-212, `run_cli()` had no dispatch branch for a leading `run` token.  `clr run msg`
/// passed `"run"` as a positional message argument to Claude instead of stripping it and routing
/// through the normal run path — silent wrong behavior with no error.
///
/// ## Why Not Caught
/// No test exercised `clr run --dry-run "…"` to compare output against `clr --dry-run "…"`.
/// The misbehavior (treating `"run"` as the message) was never observed until BUG-212 audit.
///
/// ## Fix Applied
/// `run_cli()` now strips a leading `"run"` token before passing remaining tokens to
/// `parse_args()`.  `clr run [ARGS]` is a transparent alias for `clr [ARGS]`.
///
/// ## Prevention
/// When any new explicit subcommand form is added, add a dry-run equivalence test immediately
/// to confirm the leading token is stripped and the remaining args parse identically.
///
/// ## Pitfall
/// Comparing raw stdout byte-for-byte works only for `--dry-run` (deterministic output).
/// Never compare live-invocation stdout: it contains subprocess-dependent, timing-sensitive data.
///
/// Fix(BUG-212)
#[ test ]
fn in4_run_subcommand_explicit_dispatch_identical_to_default()
{
  let with_run = run_cli( &[ "run", "--dry-run", "Fix bug" ] );
  let default  = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( with_run.status.success(), "clr run --dry-run must exit 0" );
  assert!( default.status.success(), "clr --dry-run must exit 0" );
  let stdout_run     = String::from_utf8_lossy( &with_run.stdout );
  let stdout_default = String::from_utf8_lossy( &default.stdout );
  assert_eq!(
    stdout_run,
    stdout_default,
    "`clr run --dry-run 'Fix bug'` stdout must be identical to `clr --dry-run 'Fix bug'` stdout"
  );
}

// ── US17: Model Selection ────────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/17_model_selection.md

/// US-1: --model flag appears in the assembled dry-run command.
#[ test ]
fn us17_1_model_flag_in_command()
{
  let output = run_dry( &[ "--model", "sonnet", "Fix bug" ] );
  assert!(
    output.contains( "--model sonnet" ),
    "--model sonnet must appear in dry-run output. Got:\n{output}"
  );
}

/// US-2: `CLR_MODEL` env var injects `--model` when CLI flag is absent.
#[ test ]
fn us17_2_env_var_sets_model()
{
  let out = run_cli_with_env( &[ "--dry-run", "Fix bug" ], &[ ( "CLR_MODEL", "haiku" ) ] );
  assert!( out.status.success(), "CLR_MODEL must be accepted. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--model haiku" ),
    "CLR_MODEL=haiku must inject --model haiku. Got:\n{stdout}"
  );
}

/// US-3: explicit CLI `--model` overrides `CLR_MODEL` env var.
#[ test ]
fn us17_3_cli_wins_over_env_var()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "--model", "opus", "Fix bug" ],
    &[ ( "CLR_MODEL", "haiku" ) ],
  );
  assert!( out.status.success(), "CLI --model with CLR_MODEL must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--model opus" ),
    "CLI --model opus must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--model haiku" ),
    "CLR_MODEL=haiku must be suppressed by CLI flag. Got:\n{stdout}"
  );
}

/// US-4: --model accepted in the ask command.
#[ test ]
fn us17_4_model_in_ask_command()
{
  let output = run_ask_dry( &[ "--model", "sonnet", "What is X?" ] );
  assert!(
    output.contains( "--model sonnet" ),
    "--model sonnet must appear in ask dry-run. Got:\n{output}"
  );
}

// ── US18: Env-var Configuration ──────────────────────────────────────────────
// Source: tests/docs/cli/user_story/18_env_var_configuration.md

/// US-1: `CLR_*` env var applies when the corresponding CLI flag is absent.
#[ test ]
fn us18_1_env_var_applies_when_cli_absent()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_MODEL", "haiku" ) ] );
  assert!( out.status.success(), "CLR_MODEL must be accepted" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--model haiku" ),
    "CLR_MODEL must inject --model when CLI flag absent. Got:\n{stdout}"
  );
}

/// US-2: explicit CLI flag always wins over `CLR_*` env var.
#[ test ]
fn us18_2_cli_wins_over_env_var()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "--model", "opus", "task" ],
    &[ ( "CLR_MODEL", "haiku" ) ],
  );
  assert!( out.status.success(), "CLI --model with CLR_MODEL must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--model opus" ),
    "CLI --model opus must win. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "haiku" ),
    "CLR_MODEL=haiku must be overridden by CLI flag. Got:\n{stdout}"
  );
}

/// US-3: bool `CLR_*` env var accepts "true" literal as truthy.
///
/// `CLR_NO_ULTRATHINK=true` sets `no_ultrathink=true`, suppressing the "ultrathink"
/// suffix. The word "ultrathink" must be absent from the assembled command.
#[ test ]
fn us18_3_bool_env_var_accepts_true_literal()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_NO_ULTRATHINK", "true" ) ] );
  assert!( out.status.success(), "CLR_NO_ULTRATHINK=true must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "ultrathink" ),
    "CLR_NO_ULTRATHINK=true must suppress ultrathink suffix. Got:\n{stdout}"
  );
}

/// US-4: bool `CLR_*` env var rejects "yes" — not a valid truthy value.
///
/// `CLR_NO_ULTRATHINK=yes` is silently rejected; the default behaviour (inject
/// "ultrathink") applies and the suffix must appear in the assembled command.
#[ test ]
fn us18_4_bool_env_var_rejects_yes()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_NO_ULTRATHINK", "yes" ) ] );
  assert!( out.status.success(), "CLR_NO_ULTRATHINK=yes must exit 0 (silently rejected)" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "ultrathink" ),
    "CLR_NO_ULTRATHINK=yes must not suppress ultrathink ('yes' is not truthy). Got:\n{stdout}"
  );
}

// ── US19: MCP Config Injection ───────────────────────────────────────────────
// Source: tests/docs/cli/user_story/19_mcp_config_injection.md

/// US-1: --mcp-config path appears in the assembled command.
#[ test ]
fn us19_1_mcp_config_in_command()
{
  let output = run_dry( &[ "--mcp-config", "/tmp/mcp.json", "Fix bug" ] );
  assert!(
    output.contains( "--mcp-config /tmp/mcp.json" ),
    "--mcp-config /tmp/mcp.json must appear in dry-run output. Got:\n{output}"
  );
}

/// US-2: multiple --mcp-config flags are each forwarded individually.
#[ test ]
fn us19_2_multiple_mcp_configs_forwarded()
{
  let output = run_dry( &[
    "--mcp-config", "/tmp/us19a.json",
    "--mcp-config", "/tmp/us19b.json",
    "Fix bug",
  ] );
  assert!(
    output.contains( "--mcp-config /tmp/us19a.json" ),
    "first --mcp-config must appear. Got:\n{output}"
  );
  assert!(
    output.contains( "--mcp-config /tmp/us19b.json" ),
    "second --mcp-config must appear. Got:\n{output}"
  );
}

/// US-3: `CLR_MCP_CONFIG` env var sets a single config when flag is absent.
#[ test ]
fn us19_3_env_var_sets_mcp_config()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_MCP_CONFIG", "/tmp/us19env.json" ) ],
  );
  assert!( out.status.success(), "CLR_MCP_CONFIG must be accepted. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config /tmp/us19env.json" ),
    "CLR_MCP_CONFIG must inject --mcp-config. Got:\n{stdout}"
  );
}

/// US-4: no --mcp-config flag appears by default (absent when not specified).
#[ test ]
fn us19_4_no_mcp_config_by_default()
{
  let output = run_dry( &[ "Fix bug" ] );
  assert!(
    !output.contains( "--mcp-config" ),
    "no --mcp-config must appear when flag is absent. Got:\n{output}"
  );
}

// ── US20: Suppress Effort Max ────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/20_suppress_effort_max.md

/// US-1: default assembled command includes --effort max.
#[ test ]
fn us20_1_default_has_effort_max()
{
  let output = run_dry( &[ "Fix bug" ] );
  assert!(
    output.contains( "--effort max" ),
    "default run must inject --effort max. Got:\n{output}"
  );
}

/// US-2: --no-effort-max suppresses all --effort injection.
#[ test ]
fn us20_2_no_effort_max_suppresses_injection()
{
  let output = run_dry( &[ "--no-effort-max", "Fix bug" ] );
  assert!(
    !output.contains( "--effort" ),
    "--no-effort-max must suppress all --effort flags. Got:\n{output}"
  );
}

/// US-3: --no-effort-max wins over an explicit --effort flag.
#[ test ]
fn us20_3_no_effort_max_wins_over_effort()
{
  let output = run_dry( &[ "--no-effort-max", "--effort", "medium", "Fix bug" ] );
  assert!(
    !output.contains( "--effort" ),
    "--no-effort-max must suppress --effort even when --effort is explicit. Got:\n{output}"
  );
}

/// US-4: `CLR_NO_EFFORT_MAX=1` suppresses effort injection via env var.
#[ test ]
fn us20_4_env_var_suppresses_effort()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_NO_EFFORT_MAX", "1" ) ],
  );
  assert!( out.status.success(), "CLR_NO_EFFORT_MAX=1 must exit 0. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--effort" ),
    "CLR_NO_EFFORT_MAX=1 must suppress --effort. Got:\n{stdout}"
  );
}

// ── US21: Keep ClaudeCode Context ────────────────────────────────────────────
// Source: tests/docs/cli/user_story/21_keep_claudecode_context.md

/// US-1: --keep-claudecode flag accepted; command assembles without error.
#[ test ]
fn us21_1_flag_accepted()
{
  let output = run_dry( &[ "--keep-claudecode", "Fix bug" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--keep-claudecode must assemble a valid command. Got:\n{output}"
  );
}

/// US-2: `CLR_KEEP_CLAUDECODE=1` env var accepted; command assembles.
#[ test ]
fn us21_2_env_var_1_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_KEEP_CLAUDECODE", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_KEEP_CLAUDECODE=1 must exit 0. Got: {:?}",
    out.status.code()
  );
}

/// US-3: `CLR_KEEP_CLAUDECODE=true` env var accepted; command assembles.
#[ test ]
fn us21_3_env_var_true_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_KEEP_CLAUDECODE", "true" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_KEEP_CLAUDECODE=true must exit 0. Got: {:?}",
    out.status.code()
  );
}

/// US-4: `CLR_KEEP_CLAUDECODE=yes` silently rejected; exit 0 (not a hard error).
#[ test ]
fn us21_4_env_var_yes_silently_rejected()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_KEEP_CLAUDECODE", "yes" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_KEEP_CLAUDECODE=yes must exit 0 (silently rejected). Got: {:?}",
    out.status.code()
  );
}

// ── US22: Session Isolation via Subdirectory ──────────────────────────────────
// Source: tests/docs/cli/user_story/22_session_isolation_subdir.md

/// US-1: `--subdir NAME` appends `/-NAME` to the effective working directory.
#[ test ]
fn us22_us1_subdir_name_appends_hyphen_name()
{
  let output = run_dry( &[ "--subdir", "build", "Fix bug" ] );
  assert!(
    output.contains( "/-build" ),
    "--subdir build must produce path ending in /-build. Got:\n{output}"
  );
}

/// US-2: `--subdir .` is identity — no `/-` suffix in effective dir.
#[ test ]
fn us22_us2_subdir_dot_identity()
{
  let output = run_dry( &[ "--subdir", ".", "Fix bug" ] );
  assert!(
    !output.contains( "/-" ),
    "--subdir . must not append any /-NAME suffix. Got:\n{output}"
  );
}

/// US-3: `CLR_SUBDIR=feature` env var accepted; effective dir ends in `/-feature`.
#[ test ]
fn us22_us3_clr_subdir_env_var()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_SUBDIR", "feature" ) ],
  );
  assert!( out.status.success(), "CLR_SUBDIR must exit 0. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/-feature" ),
    "CLR_SUBDIR=feature must produce path ending in /-feature. Got:\n{stdout}"
  );
}

/// US-4: `CLR_SUBDIR=.` env var identity — no `/-` suffix in effective dir.
#[ test ]
fn us22_us4_clr_subdir_dot_identity()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "CLR_SUBDIR", "." ) ],
  );
  assert!( out.status.success(), "CLR_SUBDIR=. must exit 0. Got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "/-" ),
    "CLR_SUBDIR=. must not append any /-NAME suffix. Got:\n{stdout}"
  );
}

/// US-5: CLI `--subdir cliname` wins over `CLR_SUBDIR=envname`.
#[ test ]
fn us22_us5_cli_wins_over_env_var()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "--subdir", "cliname", "Fix bug" ],
    &[ ( "CLR_SUBDIR", "envname" ) ],
  );
  assert!( out.status.success(), "CLI --subdir with CLR_SUBDIR must exit 0: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/-cliname" ),
    "CLI --subdir cliname must win over CLR_SUBDIR=envname. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "/-envname" ),
    "CLR_SUBDIR=envname must be suppressed by CLI --subdir. Got:\n{stdout}"
  );
}
