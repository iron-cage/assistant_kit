//! Parameter Group Interaction Tests
//!
//! Covers CC-N interaction cases for all four parameter groups.
//! Source: `tests/docs/cli/param_group/`
//!
//! - Group 1 (Claude-Native Flags): G1CC1–G1CC5 (`01_claude_native_flags.md`)
//! - Group 2 (Runner Control):      G2CC1–G2CC6 (`02_runner_control.md`)
//! - Group 3 (System Prompt):       G3CC1–G3CC4 (`03_system_prompt.md`)
//! - Group 4 (Credential Ops):      G4CC6       (`04_credential_operations.md`; CC-1–CC-5 are `lim_it`)

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

/// G2CC4: All 20 runner control flags together → exit 0; no unknown-flag error.
///
/// Every runner control flag accepted without conflict. `--dry-run` wins over `--trace`,
/// so stderr is empty. `--no-chrome` suppresses the default `--chrome` injection.
/// `--subdir work` produces an effective dir containing `/-work`.
/// `--retry-on-rate-limit`, `--retry-delay`, and `--timeout` are parsed and accepted;
/// no retry or watchdog fires because `--dry-run` short-circuits before subprocess spawn.
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
      "--retry-on-rate-limit", "3",
      "--retry-delay", "30",
      "--timeout", "60",
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
    "all 20 runner control flags must be accepted without conflict: {out:?}",
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
