//! User Story Integration Tests — Credentials, Files, Discoverability
//!
//! Covers US10-US18: credential-isolated execution, file input, code block extraction,
//! structured JSON pipeline, credential refresh, ask mode, CLI discoverability, model
//! selection, and env-var configuration.
//!
//! Source: `tests/docs/cli/user_story/`

#![ cfg( feature = "enabled" ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, make_creds_file, run_ask_dry, run_cli, run_cli_with_env, run_dry, stderr_str };
#[ cfg( unix ) ]
use cli_binary_test_helpers::make_proc_dir;
use std::process::Command;

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
///
/// This is a real (non-dry-run) `run` invocation with a message, so it reaches
/// `run_built_command()`'s concurrency gate (`wait_for_session_slot()`) before the
/// `--file` existence check fires inside `run_print_mode()`.  `CLR_PROC_DIR` points
/// at an empty proc-isolation dir so `find_claude_processes()` never scans the real
/// host `/proc` (BUG-326 defect class — ambient `/proc` racing concurrent nextest runs).
#[ cfg( unix ) ]
#[ test ]
fn us11_3_nonreadable_file_errors()
{
  let proc     = make_proc_dir( &[] );
  let proc_dir = proc.path().to_str().expect( "proc dir UTF-8" );
  let out = run_cli_with_env(
    &[ "--file", "/tmp/clr_us11_nonexistent_99999.txt", "test" ],
    &[ ( "CLR_PROC_DIR", proc_dir ) ],
  );
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

/// US-1: ask is a pure semantic alias for run — identical dry-run output.
#[ test ]
fn us15_1_ask_is_pure_alias_for_run()
{
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let ask_out = Command::new( bin )
    .args( [ "ask", "--dry-run", "What does this function do?" ] )
    .output()
    .expect( "failed to invoke clr ask" );
  let run_out = Command::new( bin )
    .args( [ "run", "--dry-run", "What does this function do?" ] )
    .output()
    .expect( "failed to invoke clr run" );
  assert!(
    ask_out.status.success() && run_out.status.success(),
    "both ask and run --dry-run must exit 0"
  );
  let ask_stdout = String::from_utf8_lossy( &ask_out.stdout );
  let run_stdout = String::from_utf8_lossy( &run_out.stdout );
  assert_eq!(
    ask_stdout, run_stdout,
    "ask and run must produce identical dry-run output.\nask:\n{ask_stdout}\nrun:\n{run_stdout}"
  );
}

/// US-2: ask with message triggers print mode (same as run).
#[ test ]
fn us15_2_ask_with_message_triggers_print_mode()
{
  let output = run_ask_dry( &[ "Explain closures" ] );
  assert!(
    output.contains( "--print" ),
    "ask with message must include --print. Got:\n{output}"
  );
}

/// US-3: ask uses run defaults — effort max, 200000 max tokens.
#[ test ]
fn us15_3_ask_uses_run_defaults()
{
  let output = run_ask_dry( &[ "Write a detailed analysis" ] );
  assert!(
    output.contains( "--effort max" ),
    "ask must use --effort max (run default). Got:\n{output}"
  );
  assert!(
    !output.contains( "CLAUDE_CODE_MAX_OUTPUT_TOKENS=16384" ),
    "ask must NOT use old ask default of 16384. Got:\n{output}"
  );
}

/// US-4: explicit flags are accepted by ask (same as run).
#[ test ]
fn us15_4_ask_explicit_flags_accepted()
{
  let output = run_ask_dry( &[ "--effort", "low", "--no-effort-max", "Quick question" ] );
  assert!(
    !output.contains( "--effort" ) || output.contains( "--effort low" ),
    "explicit --effort low must be respected by ask. Got:\n{output}"
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
    stdout.contains( "RUNNER OPTIONS:" ),
    "`clr help` must print RUNNER OPTIONS section. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "Commands:" ),
    "`clr help` must print Commands section. Got:\n{stdout}"
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

/// US-3: help output lists all 8 subcommands and available flags.
///
/// All 8 named subcommands — run, ask, isolated, refresh, ps, kill, tools, help — appear in COMMANDS.
#[ test ]
fn us16_3_all_subcommands_listed()
{
  let out = run_cli( &[ "help" ] );
  assert!( out.status.success(), "clr help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Commands:" ),
    "clr help must print Commands section. Got:\n{stdout}"
  );
  // Extract COMMANDS block to assert each subcommand appears there (not just anywhere in output).
  let after_cmds = stdout
    .split_once( "Commands:\n" )
    .map_or( "", | ( _, rest ) | rest );
  let cmds_block = after_cmds
    .split_once( "\nRUNNER OPTIONS:" )
    .map_or( after_cmds, | ( block, _ ) | block );
  assert!(
    cmds_block.lines().any( | l | l.trim_start().starts_with( "run" ) ),
    "Commands section must list 'run' subcommand. Got Commands block:\n{cmds_block}"
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
    cmds_block.lines().any( | l | l.trim_start().starts_with( "ps" ) ),
    "Commands section must list 'ps' subcommand. Got Commands block:\n{cmds_block}"
  );
  assert!(
    cmds_block.lines().any( | l | l.trim_start().starts_with( "kill" ) ),
    "Commands section must list 'kill' subcommand. Got Commands block:\n{cmds_block}"
  );
  assert!(
    cmds_block.lines().any( | l | l.trim_start().starts_with( "tools" ) ),
    "Commands section must list 'tools' subcommand. Got Commands block:\n{cmds_block}"
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
    stdout.contains( "RUNNER OPTIONS:" ),
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
    .split_once( "Commands:\n" )
    .map_or( "", | ( _, rest ) | rest );
  let cmds_block = after_cmds
    .split_once( "\nRUNNER OPTIONS:" )
    .map_or( after_cmds, | ( block, _ ) | block );
  assert!(
    cmds_block.lines().any( | l | l.trim_start().starts_with( "run" ) ),
    "Commands section must list 'run' as a named subcommand.\n\
     Fix(BUG-212): add 'run' to print_help() Commands section.\n\
     Got Commands block:\n{cmds_block}"
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
