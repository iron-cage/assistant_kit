//! User Story Integration Tests — Core Workflows (US01–US09)
//!
//! ## Purpose
//!
//! End-to-end workflow tests implementing specs from `tests/docs/cli/user_story/`
//! for US01 through US09.  US10-US18 live in `user_story_creds_isolated_test.rs`;
//! US19-US25 live in `user_story_output_test.rs`.
//!
//! ## Strategy
//!
//! Most cases use `--dry-run` to inspect the assembled command without spawning a
//! Claude subprocess.  Error-path cases invoke real execution against absent or
//! invalid resources.
//!
//! ## Doc Comment Convention
//!
//! Clippy `doc_markdown` lint flags `SCREAMING_SNAKE_CASE` identifiers and
//! `IDENT=value` patterns in `///` doc comments that are not wrapped in backticks.
//! All `CLR_*` env var names (e.g. `` `CLR_MODEL` ``) and `IDENT=value` patterns
//! (e.g. `` `CLR_NO_ULTRATHINK=true` ``) must use backticks in doc comments.
//! `--flag` patterns do **not** trigger this lint and may appear bare.
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

#![ cfg( feature = "enabled" ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ make_creds_file, make_session_dir, run_cli, run_cli_with_env, run_dry, stderr_str };
#[ cfg( unix ) ]
use cli_binary_test_helpers::make_proc_dir;

// ── US01: Interactive REPL ──────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/01_interactive_repl.md

/// US-1: bare clr opens REPL — subprocess args include --dangerously-skip-permissions.
///
/// Validated via --dry-run (no message → REPL route). Print mode is NOT injected.
/// Note: -c is NOT asserted here — the test cwd has no prior Claude session so
/// `session_exists()` correctly returns `None`. Session continuation is tested
/// separately in `us01_2` (which uses --session-dir with a dummy session file).
#[ test ]
fn us01_1_bare_clr_repl_defaults()
{
  let ( _session, session_path ) = make_session_dir();
  let output = run_dry( &[ "--session-dir", &session_path ] );
  assert!(
    output.contains( "--dangerously-skip-permissions" ),
    "REPL mode must inject --dangerously-skip-permissions. Got:\n{output}"
  );
  assert!(
    !output.contains( "--print" ),
    "REPL mode (no message) must NOT inject --print. Got:\n{output}"
  );
}

/// US-2: session continuation flag -c present when a prior session exists.
///
/// Uses --session-dir pointing to a non-empty temp dir so `session_exists()` returns `Some(SessionId)`.
#[ test ]
fn us01_2_session_continuation_flag_present()
{
  let session_dir = tempfile::tempdir().expect( "create temp session dir" );
  std::fs::write( session_dir.path().join( "00000000-0000-0000-0000-000000000000.jsonl" ), b"{}" )
    .expect( "write dummy session file" );
  let session_dir_str = session_dir.path().to_str().expect( "session dir path is valid utf-8" );
  let output = run_dry( &[ "--session-dir", session_dir_str ] );
  assert!(
    output.contains( " -c" ),
    "non-empty --session-dir must inject -c. Got:\n{output}"
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
  // Note: -c is NOT asserted here — /tmp has no prior Claude session so session_exists()
  // correctly returns `None`. Session continuation is tested separately in us01_2 (default cwd).
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
///
/// Note: -c is NOT asserted — the test cwd has no prior Claude session so
/// `session_exists()` correctly returns `None`. See `us01_2` for the positive -c test.
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
    !output.contains( "--chrome" ),
    "print-mode dry-run must NOT show --chrome (BUG-304 mitigation). Got:\n{output}"
  );
  assert!(
    output.contains( "--effort max" ),
    "dry-run must show --effort max. Got:\n{output}"
  );
}

/// US-2: all unconditional injected defaults visible in dry-run output.
///
/// Note: -c is NOT asserted — it is conditional on prior session history.
/// See `us01_2` for the positive -c test.
#[ test ]
fn us04_2_all_defaults_visible()
{
  let output = run_dry( &[ "test" ] );
  assert!(
    output.contains( "--dangerously-skip-permissions" ),
    "must have --dangerously-skip-permissions. Got:\n{output}"
  );
  assert!( !output.contains( "--chrome" ), "print-mode must NOT have --chrome (BUG-304). Got:\n{output}" );
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
///
/// This is a real (non-dry-run) `run` invocation with a message, so it reaches
/// `run_built_command()`'s concurrency gate (`wait_for_session_slot()`) before the
/// OS-level spawn failure (nonexistent `current_dir`) ever fires — `dispatch_run()`
/// has no `--dir` existence pre-check (unlike `dispatch_isolated()`).  `CLR_PROC_DIR`
/// points at an empty proc-isolation dir so `find_claude_processes()` never scans the
/// real host `/proc` (BUG-326 defect class).
#[ cfg( unix ) ]
#[ test ]
fn us05_3_nonexistent_dir_errors()
{
  let proc     = make_proc_dir( &[] );
  let proc_dir = proc.path().to_str().expect( "proc dir UTF-8" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let out = run_cli_with_env(
    &[ "--dir", "/tmp/clr_nonexistent_project_us05_3", "fix it" ],
    &[ ( "CLR_PROC_DIR", proc_dir ), ( "CLR_GATE_DIR", gate_dir.path().to_str().expect( "gate dir UTF-8" ) ) ],
  );
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

// ── US06: Output Suppression ─────────────────────────────────────────────────
// Source: tests/docs/cli/user_story/06_verbose_debugging.md

/// US-1: --quiet with --dry-run still shows assembled command on stdout.
#[ test ]
fn us06_1_quiet_dry_run_shows_command()
{
  let output = run_dry( &[ "--quiet", "test" ] );
  assert!(
    output.contains( "claude" ),
    "--quiet --dry-run must show command on stdout. Got:\n{output}"
  );
}

/// US-2: Without --quiet, dry-run output is always visible (default behavior unchanged).
#[ test ]
fn us06_2_no_quiet_dry_run_visible()
{
  let output = run_dry( &[ "test" ] );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "dry-run without --quiet must show env+command. Got:\n{output}"
  );
}

/// US-3: --quiet suppresses the keep-claudecode warning.
#[ test ]
fn us06_3_quiet_suppresses_keep_claudecode_warning()
{
  let out = run_cli_with_env(
    &[ "--keep-claudecode", "--quiet", "--dry-run", "task" ],
    &[ ( "CLAUDECODE", "1" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "nested-agent" ),
    "--quiet must suppress keep-claudecode warning. Got:\n{stderr}"
  );
  assert!( out.status.success(), "must exit 0. stderr: {stderr}" );
}

/// US-4: --dry-run output always visible regardless of --quiet.
#[ test ]
fn us06_4_dry_run_independent_of_quiet()
{
  let output = run_dry( &[ "--quiet", "test" ] );
  assert!(
    output.contains( "claude" ),
    "--quiet --dry-run must show command on stdout. Got:\n{output}"
  );
  assert!(
    output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=" ),
    "--quiet --dry-run must show env on stdout. Got:\n{output}"
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
  assert!( !output.contains( "--chrome" ), "print-mode must NOT have --chrome (BUG-304). Got:\n{output}" );
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
///
/// This is a real (non-dry-run) `run` invocation with a message — `--trace` prints
/// to stderr AFTER the concurrency gate in `run_built_command()` (`wait_for_session_slot()`
/// runs first), so this still reaches the gate.  `CLR_PROC_DIR` points at an empty
/// proc-isolation dir so `find_claude_processes()` never scans the real host `/proc`
/// (BUG-326 defect class).
#[ cfg( unix ) ]
#[ test ]
fn us08_1_trace_prints_command_to_stderr()
{
  let proc     = make_proc_dir( &[] );
  let proc_dir = proc.path().to_str().expect( "proc dir UTF-8" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let out = run_cli_with_env(
    &[ "--trace", "test message" ],
    &[ ( "PATH", "/nonexistent" ), ( "CLR_PROC_DIR", proc_dir ), ( "CLR_GATE_DIR", gate_dir.path().to_str().expect( "gate dir UTF-8" ) ) ],
  );
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

/// US-3: --trace output independent of --quiet flag.
///
/// Trace is a separate channel from runner quiet flag.
/// PATH=/nonexistent forces exit 1 but trace fires first.
///
/// Real (non-dry-run) `run` invocation with a message; reaches the concurrency gate
/// in `run_built_command()` before `--trace` output fires (same ordering as US-1).
/// `CLR_PROC_DIR` isolates `find_claude_processes()` from the real host `/proc`.
#[ cfg( unix ) ]
#[ test ]
fn us08_3_trace_independent_of_quiet()
{
  let proc     = make_proc_dir( &[] );
  let proc_dir = proc.path().to_str().expect( "proc dir UTF-8" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let out = run_cli_with_env(
    &[ "--trace", "--quiet", "test" ],
    &[ ( "PATH", "/nonexistent" ), ( "CLR_PROC_DIR", proc_dir ), ( "CLR_GATE_DIR", gate_dir.path().to_str().expect( "gate dir UTF-8" ) ) ],
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "claude" ),
    "--trace must emit to stderr even with --quiet. Got:\n{stderr}"
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
