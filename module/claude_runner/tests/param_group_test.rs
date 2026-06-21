//! Parameter Group Interaction Tests
//!
//! Covers CC-N interaction cases for all five parameter groups.
//! Source: `tests/docs/cli/param_group/`
//!
//! - Group 1 (Claude-Native Flags): G1CC1–G1CC6 (`01_claude_native_flags.md`)
//! - Group 2 (Runner Control):      G2CC1–G2CC6 (`02_runner_control.md`)
//! - Group 3 (System Prompt):       G3CC1–G3CC4 (`03_system_prompt.md`)
//! - Group 4 (Credential Ops):      G4CC6       (`04_credential_operations.md`; CC-1–CC-5 are `lim_it`)
//! - Group 5 (Session Listing):     G5CC1–G5CC5 (`05_session_listing.md`)

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;

// ─── Group 1: Claude-Native Flags ─────────────────────────────────────────────
// Source: tests/docs/cli/param_group/01_claude_native_flags.md

/// G1CC1: All seven claude-native flags forwarded together without conflict.
///
/// `--print`, `--model sonnet`, `--verbose`, `--effort high`, `--no-persist`,
/// `--json-schema`, and `--mcp-config` all appear in the assembled command; exit 0.
///
/// Spec: `01_claude_native_flags.md` CC-1
#[ test ]
fn g1cc1_all_claude_native_flags_forwarded_together()
{
  let out = run_cli( &[
    "--dry-run",
    "--print",
    "--model", "sonnet",
    "--verbose",
    "--effort", "high",
    "--no-persist",
    "--json-schema", r#"{"type":"string"}"#,
    "--mcp-config", "/tmp/mcp.json",
    "Fix bug",
  ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--print" ),                 "output must contain --print: {stdout}" );
  assert!( stdout.contains( "--model" ),                 "output must contain --model: {stdout}" );
  assert!( stdout.contains( "sonnet" ),                  "output must contain model value: {stdout}" );
  assert!( stdout.contains( "--verbose" ),               "output must contain --verbose: {stdout}" );
  assert!( stdout.contains( "--effort" ),                "output must contain --effort: {stdout}" );
  assert!( stdout.contains( "high" ),                    "output must contain effort value: {stdout}" );
  assert!( stdout.contains( "--no-session-persistence" ), "output must contain --no-session-persistence: {stdout}" );
  assert!( stdout.contains( "--json-schema" ),           "output must contain --json-schema: {stdout}" );
  assert!( stdout.contains( "--mcp-config" ),            "output must contain --mcp-config: {stdout}" );
}

/// G1CC2: `--model` and `--verbose` coexist without conflict.
///
/// Both flags appear in the assembled command; exit 0.
///
/// Spec: `01_claude_native_flags.md` CC-2
#[ test ]
fn g1cc2_model_and_verbose_coexist()
{
  let out = run_cli( &[ "--dry-run", "--model", "opus", "--verbose", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--model" ),   "output must contain --model: {stdout}" );
  assert!( stdout.contains( "opus" ),      "output must contain model value: {stdout}" );
  assert!( stdout.contains( "--verbose" ), "output must contain --verbose: {stdout}" );
}

/// G1CC3: `--verbose` and `--effort max` both present in assembled command.
///
/// Spec: `01_claude_native_flags.md` CC-3
#[ test ]
fn g1cc3_verbose_and_effort_max_both_present()
{
  let out = run_cli( &[ "--dry-run", "--verbose", "--effort", "max", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--verbose" ), "output must contain --verbose: {stdout}" );
  assert!( stdout.contains( "--effort" ),  "output must contain --effort: {stdout}" );
  assert!( stdout.contains( "max" ),       "output must contain effort value: {stdout}" );
}

/// G1CC4: No group flags → only defaults injected; no user-supplied group flags appear.
///
/// Default `--effort max` and `--print` are present; `--verbose` and `--model` are absent.
///
/// Spec: `01_claude_native_flags.md` CC-4
#[ test ]
fn g1cc4_no_group_flags_only_defaults_injected()
{
  let out = run_cli( &[ "--dry-run", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--effort" ),  "default --effort must be present: {stdout}" );
  assert!( stdout.contains( "max" ),       "default effort value must be present: {stdout}" );
  assert!( stdout.contains( "--print" ),   "default --print must be present: {stdout}" );
  assert!( !stdout.contains( "--verbose" ), "no --verbose without explicit flag: {stdout}" );
  assert!( !stdout.contains( "--model" ),   "no --model without explicit flag: {stdout}" );
}

/// G1CC5: `--no-persist` + `--json-schema` + `--mcp-config` → all three new members forwarded.
///
/// `--no-session-persistence`, `--json-schema`, and `--mcp-config` all appear in the
/// assembled command without conflict; exit 0.
///
/// Spec: `01_claude_native_flags.md` CC-5
#[ test ]
fn g1cc5_new_claude_native_flags_forwarded_together()
{
  let out = run_cli( &[
    "--dry-run",
    "--no-persist",
    "--json-schema", r#"{"type":"object"}"#,
    "--mcp-config", "/tmp/servers.json",
    "Fix bug",
  ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "output must contain --no-session-persistence: {stdout}",
  );
  assert!(
    stdout.contains( "--json-schema" ),
    "output must contain --json-schema: {stdout}",
  );
  assert!(
    stdout.contains( "/tmp/servers.json" ),
    "output must contain mcp-config path: {stdout}",
  );
}

/// G1CC6: All 7 new passthrough params forwarded together without conflict.
///
/// `--output-format json`, `--max-turns 5`, `--allowed-tools Read,Edit`,
/// `--disallowed-tools Bash`, `--max-budget-usd 5.00`, `--add-dir /tmp`,
/// and `--fallback-model sonnet` all appear in the assembled command; exit 0.
///
/// Spec: `01_claude_native_flags.md` CC-6
#[ test ]
fn g1cc6_all_new_passthrough_params_forwarded_together()
{
  let out = run_cli( &[
    "--dry-run",
    "--output-format", "json",
    "--max-turns", "5",
    "--allowed-tools", "Read,Edit",
    "--disallowed-tools", "Bash",
    "--max-budget-usd", "5.00",
    "--add-dir", "/tmp",
    "--fallback-model", "sonnet",
    "Fix bug",
  ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--output-format" ),    "must contain --output-format: {stdout}" );
  assert!( stdout.contains( "json" ),              "must contain json: {stdout}" );
  assert!( stdout.contains( "--max-turns" ),       "must contain --max-turns: {stdout}" );
  assert!( stdout.contains( "--allowed-tools" ),   "must contain --allowed-tools: {stdout}" );
  assert!( stdout.contains( "Read,Edit" ),         "must contain Read,Edit: {stdout}" );
  assert!( stdout.contains( "--disallowed-tools" ), "must contain --disallowed-tools: {stdout}" );
  assert!( stdout.contains( "Bash" ),              "must contain Bash: {stdout}" );
  assert!( stdout.contains( "--max-budget-usd" ),  "must contain --max-budget-usd: {stdout}" );
  assert!( stdout.contains( "5.00" ),              "must contain 5.00: {stdout}" );
  assert!( stdout.contains( "--add-dir" ),         "must contain --add-dir: {stdout}" );
  assert!( stdout.contains( "--fallback-model" ),  "must contain --fallback-model: {stdout}" );
  assert!( stdout.contains( "sonnet" ),            "must contain sonnet: {stdout}" );
}

// ─── Group 2: Runner Control ───────────────────────────────────────────────────
// Source: tests/docs/cli/param_group/02_runner_control.md

/// G2CC1: `--dry-run` + `--no-ultrathink` → preview shows no ultrathink suffix.
///
/// `--dry-run` prevents execution; `--no-ultrathink` suppresses the default suffix.
///
/// Spec: `02_runner_control.md` CC-1
#[ test ]
fn g2cc1_dry_run_and_no_ultrathink_preview_suppressed()
{
  let out = run_cli( &[ "--dry-run", "--no-ultrathink", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "ultrathink" ),
    "ultrathink must not appear in preview when --no-ultrathink given: {stdout}",
  );
}

/// G2CC2: `--new-session` + `--session-dir` → both accepted; `-c` default suppressed.
///
/// `--session-dir` path appears in assembled command; `--new-session` suppresses `-c`.
///
/// Spec: `02_runner_control.md` CC-2
#[ test ]
fn g2cc2_new_session_and_session_dir_both_accepted()
{
  let out = run_cli( &[
    "--dry-run", "--new-session", "--session-dir", "/tmp/sessions", "Fix bug",
  ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  // --session-dir is converted to CLAUDE_CODE_SESSION_DIR env var in dry-run output
  assert!(
    stdout.contains( "CLAUDE_CODE_SESSION_DIR=/tmp/sessions" ),
    "output must contain CLAUDE_CODE_SESSION_DIR env var: {stdout}",
  );
  assert!(
    !stdout.contains( " -c" ),
    "no -c flag when --new-session given: {stdout}",
  );
}

/// G2CC3: `--no-skip-permissions` + `--no-effort-max` → both defaults suppressed.
///
/// Neither `--dangerously-skip-permissions` nor `--effort` appear in assembled command.
///
/// Spec: `02_runner_control.md` CC-3
#[ test ]
fn g2cc3_no_skip_permissions_and_no_effort_max_both_suppressed()
{
  let out = run_cli( &[ "--dry-run", "--no-skip-permissions", "--no-effort-max", "Fix bug" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--dangerously-skip-permissions" ),
    "no --dangerously-skip-permissions when --no-skip-permissions given: {stdout}",
  );
  assert!(
    !stdout.contains( "--effort" ),
    "no --effort when --no-effort-max given: {stdout}",
  );
}

/// G2CC4: All 42 runner control flags together → exit 0; no unknown-flag error.
///
/// Every runner control flag accepted without conflict. `--dry-run` wins over `--trace`,
/// so stderr is empty. `--no-chrome` suppresses the default `--chrome` injection.
/// `--subdir work` produces an effective dir containing `/-work`.
/// All 20 new retry params (3-tier: override, 8 class-specific pairs, fallback) plus
/// `--output-file`, `--expect`, `--expect-strategy`, `--retry-on-validation`, `--max-sessions`,
/// and `--timeout` are all parsed and accepted; `--dry-run` short-circuits before execution.
///
/// `CLAUDECODE` is removed from the subprocess environment to implement the spec's
/// "clean environment" precondition (CC-4 Given). Without removal, the BUG-248 fix
/// emits a `--keep-claudecode` warning on stderr when `CLAUDECODE` is inherited from
/// the host Claude Code session, failing the `stderr.is_empty()` assertion.
///
/// Spec: `02_runner_control.md` CC-4
#[ test ]
fn g2cc4_all_runner_control_flags_no_conflict()
{
  let tmp = tempfile::NamedTempFile::new().expect( "tmp" );
  std::io::Write::write_all( &mut tmp.as_file(), b"input" ).expect( "write" );
  let file_path = tmp.path().to_str().expect( "path" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [
      "--dry-run",
      "--no-skip-permissions",
      "--interactive",
      "--new-session",
      "--dir", "/tmp/test",
      "--subdir", "work",
      "--max-tokens", "100000",
      "--session-dir", "/tmp/sessions",
      "--verbosity", "2",
      "--trace",
      "--no-ultrathink",
      "--no-effort-max",
      "--no-chrome",
      "--no-persist",
      "--file", file_path,
      "--strip-fences",
      "--keep-claudecode",
      "--output-file", "/tmp/rc_out.txt",
      "--expect", "yes|no",
      "--expect-strategy", "fail",
      "--retry-on-validation", "2",
      "--validation-delay", "0",
      "--max-sessions", "5",
      "--timeout", "60",
      "--retry-on-transient", "3",
      "--transient-delay", "10",
      "--retry-on-account", "1",
      "--account-delay", "0",
      "--retry-on-auth", "0",
      "--auth-delay", "0",
      "--retry-on-service", "1",
      "--service-delay", "0",
      "--retry-on-process", "1",
      "--process-delay", "0",
      "--retry-on-runner", "0",
      "--runner-delay", "0",
      "--retry-on-unknown", "1",
      "--unknown-delay", "0",
      "--retry-override", "0",
      "--retry-override-delay", "0",
      "--retry-default", "2",
      "--retry-default-delay", "30",
      "--output-style", "summary",
      "Fix bug",
    ] )
    // Spec CC-4 requires "clean environment" — unset CLAUDECODE so the BUG-248 warning
    // (fires when --keep-claudecode + CLAUDECODE in env + verbosity >= 2) does not appear.
    // Root cause of fragility: host Claude Code sessions inject CLAUDECODE=1 into the
    // environment; container runs (CLAUDECODE absent) pass without this guard.
    .env_remove( "CLAUDECODE" )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "all 43 runner control flags must be accepted without conflict: {out:?}",
  );
  assert!(
    out.stderr.is_empty(),
    "stderr must be empty (dry-run wins over trace): {:?}",
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "--no-chrome must suppress --chrome injection: {stdout}",
  );
  assert!(
    stdout.contains( "/tmp/test/-work" ),
    "effective dir must contain /tmp/test/-work: {stdout}",
  );
}

/// G2CC5: `--file` + `--strip-fences` + `--keep-claudecode` → all three accepted.
///
/// All three new runner-control flags coexist without conflict; exit 0.
///
/// Spec: `02_runner_control.md` CC-5
#[ test ]
fn g2cc5_file_strip_fences_keep_claudecode_accepted()
{
  let tmp = tempfile::NamedTempFile::new().expect( "tmp" );
  std::io::Write::write_all( &mut tmp.as_file(), b"task input" ).expect( "write" );
  let path = tmp.path().to_str().expect( "path" );

  let out = run_cli( &[
    "--dry-run",
    "--file", path,
    "--strip-fences",
    "--keep-claudecode",
    "task",
  ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( path ), "output must reference file path: {stdout}" );
}

/// G2CC6: `--dir PATH` + `--subdir NAME` → effective dir is `PATH/-NAME`.
///
/// Spec: `02_runner_control.md` CC-6
#[ test ]
fn g2cc6_dir_plus_subdir_effective_dir()
{
  let out = run_cli( &[ "--dry-run", "--dir", "/tmp", "--subdir", "build", "task" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/tmp/-build" ),
    "effective dir must be /tmp/-build (not /tmp alone, not /tmp/build): {stdout}",
  );
}

// ─── Group 3: System Prompt ────────────────────────────────────────────────────
// Source: tests/docs/cli/param_group/03_system_prompt.md

/// G3CC1: `--system-prompt` alone → forwarded; `--append-system-prompt` absent.
///
/// Spec: `03_system_prompt.md` CC-1
#[ test ]
fn g3cc1_system_prompt_alone_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--system-prompt", "Be concise.", "test" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--system-prompt" ),
    "output must contain --system-prompt: {stdout}",
  );
  assert!(
    !stdout.contains( "--append-system-prompt" ),
    "no --append-system-prompt when not given: {stdout}",
  );
}

/// G3CC2: `--append-system-prompt` alone → forwarded; `--system-prompt` absent.
///
/// Spec: `03_system_prompt.md` CC-2
#[ test ]
fn g3cc2_append_system_prompt_alone_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--append-system-prompt", "Always JSON.", "test" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--append-system-prompt" ),
    "output must contain --append-system-prompt: {stdout}",
  );
  // `--append-system-prompt` does not contain `--system-prompt` as a substring
  // (the `--` prefix only appears once, at the start of each flag), so this check
  // correctly rejects any injected standalone `--system-prompt`.
  assert!(
    !stdout.contains( "--system-prompt " ),
    "no bare --system-prompt when not given: {stdout}",
  );
}

/// G3CC3: Both `--system-prompt` and `--append-system-prompt` → both forwarded.
///
/// Spec: `03_system_prompt.md` CC-3
#[ test ]
fn g3cc3_both_system_prompt_flags_forwarded()
{
  let out = run_cli( &[
    "--dry-run",
    "--system-prompt", "Base.",
    "--append-system-prompt", "Extra.",
    "test",
  ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--system-prompt" ),
    "output must contain --system-prompt: {stdout}",
  );
  assert!(
    stdout.contains( "--append-system-prompt" ),
    "output must contain --append-system-prompt: {stdout}",
  );
}

/// G3CC4: Neither system-prompt flag → no injection by default.
///
/// Neither `--system-prompt` nor `--append-system-prompt` in assembled command.
///
/// Spec: `03_system_prompt.md` CC-4
#[ test ]
fn g3cc4_neither_system_prompt_no_injection()
{
  let out = run_cli( &[ "--dry-run", "test" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--system-prompt" ),
    "no --system-prompt injected by default: {stdout}",
  );
  assert!(
    !stdout.contains( "--append-system-prompt" ),
    "no --append-system-prompt injected by default: {stdout}",
  );
}

// ─── Group 4: Credential Operations ──────────────────────────────────────────
// Source: tests/docs/cli/param_group/04_credential_operations.md
//
// CC-1 through CC-5 require live credentials (lim_it) and are covered by
// `isolated_test.rs`. CC-6 is testable without live creds.

/// G4CC6: `--trace` on credential ops → call details printed to stderr.
///
/// `# clr isolated`, `# creds:`, `# timeout: 30s` appear on stderr before any
/// subprocess attempt; does not require live credentials (trace fires first).
///
/// Spec: `04_credential_operations.md` CC-6
#[ test ]
fn g4cc6_trace_on_credential_ops()
{
  use cli_binary_test_helpers::run_cli_with_env;

  let mut tmp = tempfile::NamedTempFile::new().expect( "tmp" );
  std::io::Write::write_all( &mut tmp, b"{}" ).expect( "write" );
  let path = tmp.path().to_str().expect( "path" );

  let out = run_cli_with_env(
    &[ "isolated", "--creds", path, "--trace" ],
    &[ ( "PATH", "/nonexistent" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "# clr isolated" ), "stderr must contain '# clr isolated': {stderr}" );
  assert!( stderr.contains( "# creds:" ),        "stderr must contain '# creds:': {stderr}" );
  assert!( stderr.contains( "# timeout: 30s" ),  "stderr must contain '# timeout: 30s': {stderr}" );
}

// ─── Group 5: Session Listing ──────────────────────────────────────────────
// Source: tests/docs/cli/param_group/05_session_listing.md

/// G5CC1: `clr ps --mode all --columns pid,task` succeeds; no subprocess spawned.
///
/// `clr ps` is a read-only inspection command — it never spawns a `claude` subprocess.
/// With ≥1 fake process running, exit 0; no subprocess stderr pollution.
///
/// Spec: `05_session_listing.md` G5-CC1
#[ cfg( unix ) ]
#[ test ]
fn g5cc1_ps_consumes_params_no_subprocess()
{
  use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude };

  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "all", "--columns", "pid,task" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --mode all --columns pid,task" );

  let _ = bg.kill();
  let _ = bg.wait();

  assert!(
    out.status.success(),
    "G5CC1: exit 0 expected, got {:?}",
    out.status.code()
  );
}

/// G5CC2: All 3 params accepted by `clr ps` without error.
///
/// `--mode all`, `--columns pid,task`, `--wide` are all accepted; `--columns` wins
/// over `--wide` so only PID and Task are shown.
///
/// Spec: `05_session_listing.md` G5-CC2
#[ cfg( unix ) ]
#[ test ]
fn g5cc2_all_three_params_accepted()
{
  use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude };

  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "all", "--columns", "pid,task", "--wide" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --mode all --columns pid,task --wide" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    out.status.success(),
    "G5CC2: exit 0 expected, got {:?}; stderr: {stderr}",
    out.status.code()
  );
  assert!(
    stderr.is_empty(),
    "G5CC2: no error on stderr about unknown flags. Got: {stderr}"
  );
}

/// G5CC3: `--mode print --columns pid,task --wide` → only print-mode shown; PID + Task only.
///
/// `--columns` wins over `--wide`; `--mode print` filters to print-mode rows only.
///
/// Spec: `05_session_listing.md` G5-CC3
#[ cfg( unix ) ]
#[ test ]
fn g5cc3_columns_wins_mode_filter_applied()
{
  use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude, spawn_print_claude };

  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_interactive = spawn_fake_claude( &path_val );
  let mut bg_print       = spawn_print_claude( &path_val );
  let     pid_print      = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "print", "--columns", "pid,task", "--wide" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --mode print --columns pid,task --wide" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( out.status.success(), "G5CC3: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "G5CC3: print PID {pid_print} must appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "PID" ) && stdout.contains( "Task" ),
    "G5CC3: only PID and Task columns must be visible. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "Mode" ) && !stdout.contains( "Command" ) && !stdout.contains( "Binary" ),
    "G5CC3: Mode/Command/Binary must NOT appear (--columns wins over --wide). Got:\n{stdout}"
  );
}

/// G5CC4: Session Listing params do not appear in `clr run --help` output.
///
/// `--mode`, `--columns`, `--wide` are ps-only and must not be listed
/// in the `run` command's help.
///
/// Note: uses `"--mode "` (with trailing space) to avoid matching the
/// `--model` option which shares `--mode` as a prefix.
///
/// Spec: `05_session_listing.md` G5-CC4
#[ test ]
fn g5cc4_session_listing_params_not_in_run_help()
{
  let out    = run_cli( &[ "run", "--help" ] );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( out.status.success(), "G5CC4: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.contains( "--mode " ),
    "G5CC4: --mode must NOT appear in clr run --help (note: --model is allowed). Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--columns" ),
    "G5CC4: --columns must NOT appear in clr run --help. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--wide" ),
    "G5CC4: --wide must NOT appear in clr run --help. Got:\n{stdout}"
  );
}

/// G5CC5: `CLR_PS_MODE=interactive` + `CLR_PS_COLUMNS=pid,elapsed` env vars respected.
///
/// Only interactive sessions shown; only PID and Elapsed columns visible.
///
/// Spec: `05_session_listing.md` G5-CC5
#[ cfg( unix ) ]
#[ test ]
fn g5cc5_env_vars_respected()
{
  use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude, spawn_print_claude };

  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_interactive  = spawn_fake_claude( &path_val );
  let     pid_interactive = bg_interactive.id();
  let mut bg_print        = spawn_print_claude( &path_val );
  let     pid_print       = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_MODE", "interactive" )
    .env( "CLR_PS_COLUMNS", "pid,elapsed" )
    .output()
    .expect( "run clr ps with CLR_PS_MODE=interactive CLR_PS_COLUMNS=pid,elapsed" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( out.status.success(), "G5CC5: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_interactive.to_string() ),
    "G5CC5: interactive PID {pid_interactive} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_print.to_string() ),
    "G5CC5: print PID {pid_print} must NOT appear. Got:\n{stdout}"
  );
  assert!( stdout.contains( "PID" ),     "G5CC5: PID column must appear. Got:\n{stdout}" );
  assert!( stdout.contains( "Elapsed" ), "G5CC5: Elapsed column must appear. Got:\n{stdout}" );
  assert!( !stdout.contains( "CPU%" ),   "G5CC5: CPU% must NOT appear. Got:\n{stdout}" );
}

/// G5CC6: `CLR_PS_PID=<PID-A>` env var restricts active table to session A only.
///
/// With two fake sessions (A and B), only A's PID appears in the table.
///
/// Spec: `05_session_listing.md` G5-CC6
#[ cfg( unix ) ]
#[ test ]
fn g5cc6_clr_ps_pid_env_var_filters_active_table()
{
  use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude };

  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_a = spawn_fake_claude( &path_val );
  let pid_a    = bg_a.id();
  let mut bg_b = spawn_fake_claude( &path_val );
  let pid_b    = bg_b.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_PID", pid_a.to_string() )
    .output()
    .expect( "run clr ps with CLR_PS_PID=A" );

  let _ = bg_a.kill(); let _ = bg_a.wait();
  let _ = bg_b.kill(); let _ = bg_b.wait();

  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( out.status.success(), "G5CC6: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_a.to_string() ),
    "G5CC6: PID A {pid_a} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_b.to_string() ),
    "G5CC6: PID B {pid_b} must NOT appear. Got:\n{stdout}"
  );
}

/// G5CC7: `--inspect` switches to key:value format; `--columns` and `--wide` are ignored.
///
/// With `--inspect --columns pid --wide`, all 12 attribute keys must appear in the output
/// and no table header row should be present.
///
/// Spec: `05_session_listing.md` G5-CC7
#[ cfg( unix ) ]
#[ test ]
fn g5cc7_inspect_switches_format_ignores_columns_wide()
{
  use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude };

  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--inspect", "--columns", "pid", "--wide" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --inspect --columns pid --wide" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( out.status.success(), "G5CC7: exit 0 expected, got {:?}", out.status.code() );

  // All 12 inspect attribute keys must appear despite --columns pid being present.
  for key in &[ "pid:", "mode:", "elapsed:", "cpu:", "ram:", "state:",
                "path:", "task:", "binary:", "cmd:", "cmdline:", "started:" ]
  {
    assert!(
      stdout.contains( key ),
      "G5CC7: inspect attribute '{key}' must appear (--columns/--wide ignored). Got:\n{stdout}"
    );
  }

  // No table header row.
  assert!(
    !stdout.contains( "Active Sessions" ),
    "G5CC7: table header 'Active Sessions' must NOT appear in inspect mode. Got:\n{stdout}"
  );
}
