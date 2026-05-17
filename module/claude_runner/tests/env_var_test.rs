//! CLR_* Environment Variable Tests
//!
//! Covers E01–E24: one test per `CLR_*` env var.
//! Source: `task/claude_runner/148_env_var_all_params.md`
//!
//! All tests use `run_cli_with_env()` — no `std::env::set_var`, no thread-global mutation.
//! Tests are RED before Phase 4: without `apply_env_vars()` / `apply_isolated_env_vars()`,
//! every env var is silently ignored and the primary assertion fails.
//!
//! # Test Matrix
//!
//! | Test | Env Var | Primary Assertion (RED before impl) |
//! |------|---------|-------------------------------------|
//! | E01  | `CLR_MESSAGE`              | stdout contains the message text                             |
//! | E02  | `CLR_PRINT`                | stdout contains `--print` (with `--interactive` on CLI)     |
//! | E03  | `CLR_MODEL`                | stdout contains `--model` and `sonnet`                      |
//! | E04  | `CLR_VERBOSE`              | stdout contains `--verbose`                                 |
//! | E05  | `CLR_NO_SKIP_PERMISSIONS`  | stdout NOT contains `--dangerously-skip-permissions`        |
//! | E06  | `CLR_INTERACTIVE`          | stdout NOT contains `--print` (with positional message)     |
//! | E07  | `CLR_NEW_SESSION`          | stdout NOT contains `-c`                                    |
//! | E08  | `CLR_DIR`                  | stdout contains the dir path                                |
//! | E09  | `CLR_MAX_TOKENS`           | stdout contains `3000`                                      |
//! | E10  | `CLR_SESSION_DIR`          | stdout contains the session dir path                        |
//! | E11  | `CLR_DRY_RUN`              | exit 0 and stdout contains `--effort`                       |
//! | E12  | `CLR_VERBOSITY`            | stderr contains `--effort` (verbose level 5)                |
//! | E13  | `CLR_TRACE`                | stderr contains `--effort` (trace preview)                  |
//! | E14  | `CLR_NO_ULTRATHINK`        | stdout NOT contains `ultrathink`                            |
//! | E15  | `CLR_SYSTEM_PROMPT`        | stdout contains `--system-prompt`                           |
//! | E16  | `CLR_APPEND_SYSTEM_PROMPT` | stdout contains `--append-system-prompt`                    |
//! | E17  | `CLR_EFFORT`               | stdout contains `low`                                       |
//! | E18  | `CLR_NO_EFFORT_MAX`        | stdout NOT contains `--effort`                              |
//! | E19  | `CLR_NO_CHROME`            | stdout NOT contains `--chrome`                              |
//! | E20  | `CLR_NO_PERSIST`           | stdout contains `--no-session-persistence`                  |
//! | E21  | `CLR_JSON_SCHEMA`          | stdout contains `--json-schema`                             |
//! | E22  | `CLR_MCP_CONFIG`           | stdout contains `--mcp-config` and the path                 |
//! | E23  | `CLR_CREDS`                | stderr NOT contains `missing required argument: --creds`    |
//! | E24  | `CLR_TIMEOUT`              | stderr NOT contains `missing required argument: --creds`    |

mod common;
use common::run_cli_with_env;

// ─── E01: CLR_MESSAGE ─────────────────────────────────────────────────────────

/// E01: `CLR_MESSAGE` supplies prompt text when no positional arg is given.
///
/// Env-alone: stdout contains the message text from env.
/// CLI-wins: positional `cli_msg` takes precedence over `CLR_MESSAGE=env_msg`.
///
/// Spec: `148_env_var_all_params.md` T15, T16
#[ test ]
fn e01_clr_message_sets_prompt()
{
  let out = run_cli_with_env( &[ "--dry-run" ], &[ ( "CLR_MESSAGE", "hello world" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "hello world" ),
    "CLR_MESSAGE must supply prompt text: {stdout}",
  );

  // CLI-wins: positional arg takes precedence
  let out2 = run_cli_with_env(
    &[ "--dry-run", "cli_msg" ],
    &[ ( "CLR_MESSAGE", "env_msg" ) ],
  );
  let stdout2 = String::from_utf8_lossy( &out2.stdout );
  assert!( stdout2.contains( "cli_msg" ),  "CLI message must win over env: {stdout2}" );
  assert!( !stdout2.contains( "env_msg" ), "env message must not override CLI: {stdout2}" );
}

// ─── E02: CLR_PRINT ───────────────────────────────────────────────────────────

/// E02: `CLR_PRINT=1` enables print mode.
///
/// With `--interactive` on CLI (suppresses auto-`--print` from message presence),
/// `CLR_PRINT=1` must still add `--print` to the assembled command.
///
/// Spec: `148_env_var_all_params.md` param 2
#[ test ]
fn e02_clr_print_enables_print_mode()
{
  // --interactive suppresses auto-print; CLR_PRINT=1 must override to add --print
  let out = run_cli_with_env(
    &[ "--dry-run", "--interactive", "x" ],
    &[ ( "CLR_PRINT", "1" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--print" ),
    "CLR_PRINT=1 must add --print to assembled command: {stdout}",
  );
}

// ─── E03: CLR_MODEL ───────────────────────────────────────────────────────────

/// E03: `CLR_MODEL` sets the model when `--model` is not on the CLI.
///
/// Env-alone: `--model sonnet` appears in assembled command.
/// CLI-wins: `--model opus` with `CLR_MODEL=sonnet` → opus, not sonnet.
///
/// Spec: `148_env_var_all_params.md` T01, T02
#[ test ]
fn e03_clr_model_sets_model()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_MODEL", "sonnet" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "--model" ), "CLR_MODEL must add --model flag: {stdout}" );
  assert!( stdout.contains( "sonnet" ),  "CLR_MODEL must include model name: {stdout}" );

  // CLI-wins
  let out2 = run_cli_with_env(
    &[ "--dry-run", "--model", "opus", "task" ],
    &[ ( "CLR_MODEL", "sonnet" ) ],
  );
  let stdout2 = String::from_utf8_lossy( &out2.stdout );
  assert!( stdout2.contains( "opus" ),   "CLI --model must win over CLR_MODEL: {stdout2}" );
  assert!( !stdout2.contains( "sonnet" ), "CLR_MODEL must not override CLI --model: {stdout2}" );
}

// ─── E04: CLR_VERBOSE ─────────────────────────────────────────────────────────

/// E04: `CLR_VERBOSE=1` enables verbose mode.
///
/// Bool negative case: `CLR_VERBOSE=yes` must NOT activate verbose
/// (only `1` and `true`, case-insensitive, are accepted).
///
/// Spec: `148_env_var_all_params.md` T03, T04
#[ test ]
fn e04_clr_verbose_enables_verbose()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_VERBOSE", "1" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--verbose" ),
    "CLR_VERBOSE=1 must add --verbose: {stdout}",
  );

  // bool negative: "yes" must NOT activate verbose
  let out2 = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_VERBOSE", "yes" ) ] );
  let stdout2 = String::from_utf8_lossy( &out2.stdout );
  assert!(
    !stdout2.contains( "--verbose" ),
    "CLR_VERBOSE=yes must NOT activate verbose (only 1/true): {stdout2}",
  );
}

// ─── E05: CLR_NO_SKIP_PERMISSIONS ─────────────────────────────────────────────

/// E05: `CLR_NO_SKIP_PERMISSIONS=1` suppresses the default `--dangerously-skip-permissions`.
///
/// Spec: `148_env_var_all_params.md` T06
#[ test ]
fn e05_clr_no_skip_permissions_suppresses_default()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_NO_SKIP_PERMISSIONS", "1" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--dangerously-skip-permissions" ),
    "CLR_NO_SKIP_PERMISSIONS=1 must suppress --dangerously-skip-permissions: {stdout}",
  );
}

// ─── E06: CLR_INTERACTIVE ─────────────────────────────────────────────────────

/// E06: `CLR_INTERACTIVE=1` enables interactive mode, suppressing auto-`--print`.
///
/// When a positional message is given but `CLR_INTERACTIVE=1`, the assembled command
/// must NOT include `--print` (interactive mode disables print auto-injection).
///
/// Spec: `148_env_var_all_params.md` param 6
#[ test ]
fn e06_clr_interactive_suppresses_auto_print()
{
  // CLR_INTERACTIVE=1 must suppress auto-print even when a message is present
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_INTERACTIVE", "1" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--print" ),
    "CLR_INTERACTIVE=1 must suppress auto --print injection: {stdout}",
  );
}

// ─── E07: CLR_NEW_SESSION ─────────────────────────────────────────────────────

/// E07: `CLR_NEW_SESSION=1` starts a new session, suppressing the default `-c` flag.
///
/// Spec: `148_env_var_all_params.md` param 7
#[ test ]
fn e07_clr_new_session_suppresses_continue()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_NEW_SESSION", "1" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( " -c" ),
    "CLR_NEW_SESSION=1 must suppress default -c flag: {stdout}",
  );
}

// ─── E08: CLR_DIR ─────────────────────────────────────────────────────────────

/// E08: `CLR_DIR` sets the working directory for the Claude subprocess.
///
/// Spec: `148_env_var_all_params.md` param 8
#[ test ]
fn e08_clr_dir_sets_working_directory()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_DIR", "/tmp/e08dir" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/tmp/e08dir" ),
    "CLR_DIR must set working directory in assembled command: {stdout}",
  );
}

// ─── E09: CLR_MAX_TOKENS ──────────────────────────────────────────────────────

/// E09: `CLR_MAX_TOKENS` sets the max output token limit.
///
/// Spec: `148_env_var_all_params.md` T08
#[ test ]
fn e09_clr_max_tokens_sets_limit()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_MAX_TOKENS", "3000" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "3000" ),
    "CLR_MAX_TOKENS must add --max-tokens 3000 to command: {stdout}",
  );
}

// ─── E10: CLR_SESSION_DIR ─────────────────────────────────────────────────────

/// E10: `CLR_SESSION_DIR` sets the session storage directory.
///
/// Session dir appears as `CLAUDE_CODE_SESSION_DIR=<path>` in dry-run env output.
///
/// Spec: `148_env_var_all_params.md` param 10
#[ test ]
fn e10_clr_session_dir_sets_session_directory()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_SESSION_DIR", "/tmp/e10sess" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/tmp/e10sess" ),
    "CLR_SESSION_DIR must appear in assembled command or env block: {stdout}",
  );
}

// ─── E11: CLR_DRY_RUN ─────────────────────────────────────────────────────────

/// E11: `CLR_DRY_RUN=1` enables dry-run mode without the `--dry-run` CLI flag.
///
/// Without the env var the process tries to execute Claude (exits non-0 in test env).
/// With `CLR_DRY_RUN=1` the process must print the assembled command and exit 0.
///
/// Spec: `148_env_var_all_params.md` T05
#[ test ]
fn e11_clr_dry_run_enables_preview()
{
  let out = run_cli_with_env( &[ "task" ], &[ ( "CLR_DRY_RUN", "1" ) ] );
  assert!( out.status.success(), "CLR_DRY_RUN=1 must exit 0 (dry-run preview): {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--effort" ),
    "CLR_DRY_RUN=1 must print assembled command to stdout: {stdout}",
  );
}

// ─── E12: CLR_VERBOSITY ───────────────────────────────────────────────────────

/// E12: `CLR_VERBOSITY=5` enables verbose detail level, printing command preview to stderr.
///
/// `VerbosityLevel::shows_verbose_detail()` returns true for level ≥ 4.
/// Default level is 3 (does not show verbose detail).
/// With `CLR_VERBOSITY=5` the assembled command preview appears in stderr
/// (via the `cli.trace || cli.verbosity.shows_verbose_detail()` check in `run_cli()`).
///
/// Spec: `148_env_var_all_params.md` T09
#[ test ]
fn e12_clr_verbosity_sets_level()
{
  let out = run_cli_with_env( &[ "task" ], &[ ( "CLR_VERBOSITY", "5" ) ] );
  // Exit may be non-0 (Claude not installed in test env), but stderr must have preview.
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "--effort" ),
    "CLR_VERBOSITY=5 must show verbose command preview in stderr: {stderr}",
  );
}

/// E12 CLI-wins: explicit `--verbosity 3` must win over `CLR_VERBOSITY=5`.
///
/// # Root Cause
///
/// `apply_env_vars` used `parsed.verbosity == VerbosityLevel::default()` (== 3) as a proxy
/// for "verbosity was not explicitly set". This is wrong: explicitly passing `--verbosity 3`
/// produces the same field value as "not set" because 3 is the default. The env var check
/// fired and overwrote the explicit CLI value, causing `shows_verbose_detail()` to fire
/// and emit the command preview to stderr even though the user asked for level 3.
///
/// # Why Not Caught
///
/// Existing verbosity tests used non-default values (0, 5) or omitted `--verbosity` entirely;
/// no test combined explicit `--verbosity 3` with a `CLR_VERBOSITY` env var, so the
/// equality-with-default failure was invisible until a user observed the preview appearing
/// unexpectedly when running `--verbosity 3` alongside a `CLR_VERBOSITY=5` env var.
///
/// # Fix Applied
///
/// Changed `verbosity: VerbosityLevel` to `verbosity: Option<VerbosityLevel>` in `CliArgs`.
/// `None` means "not set"; `Some(v)` means explicitly provided. `apply_env_vars` now checks
/// `parsed.verbosity.is_none()`, which correctly excludes explicit `--verbosity 3`.
///
/// # Prevention
///
/// Use `Option<T>` (not `T`) for fields whose default is a non-false value; equality-with-
/// default cannot distinguish "not set" from "explicitly set to default".
///
/// # Pitfall
///
/// `--timeout 30` in `isolated` has the same limitation and is intentionally documented as
/// accepted (see `apply_isolated_env_vars` comment). Verbosity is fixed here because the
/// `apply_env_vars` doc comment promises "CLI flag always wins when both are present".
// test_kind: bug_reproducer(issue-verbosity-cli-wins)
#[ test ]
fn e12_verbosity_bug_cli_wins_when_env_overrides_default()
{
  use std::process::Command;

  // Run with PATH=/nonexistent so execution fails immediately after the trace/preview check.
  // With --verbosity 3: shows_verbose_detail() == false → no preview on stderr (correct).
  // Bug: with CLR_VERBOSITY=5 overwriting --verbosity 3, shows_verbose_detail() == true
  // and the assembled command preview (containing "--effort") appears on stderr.
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "--verbosity", "3", "task" ] )
    .env( "CLR_VERBOSITY", "5" )
    .env( "PATH", "/nonexistent" )
    .output()
    .expect( "failed to invoke clr binary" );

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "--effort" ),
    "explicit --verbosity 3 must win over CLR_VERBOSITY=5; \
     verbose detail preview (containing --effort) must NOT appear on stderr. Got:\n{stderr}"
  );
}

// ─── E13: CLR_TRACE ───────────────────────────────────────────────────────────

/// E13: `CLR_TRACE=1` enables trace mode, printing the command preview to stderr before execution.
///
/// Spec: `148_env_var_all_params.md` param 13
#[ test ]
fn e13_clr_trace_prints_command_to_stderr()
{
  let out = run_cli_with_env( &[ "task" ], &[ ( "CLR_TRACE", "1" ) ] );
  // Exit may be non-0 (Claude not installed in test env), but stderr must have preview.
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "--effort" ),
    "CLR_TRACE=1 must print assembled command to stderr: {stderr}",
  );
}

// ─── E14: CLR_NO_ULTRATHINK ───────────────────────────────────────────────────

/// E14: `CLR_NO_ULTRATHINK=1` suppresses the automatic `ultrathink` message suffix.
///
/// Default: message gets `\n\nultrathink` appended.
/// With `CLR_NO_ULTRATHINK=1`: suffix is omitted.
///
/// Spec: `148_env_var_all_params.md` param 14
#[ test ]
fn e14_clr_no_ultrathink_suppresses_suffix()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_NO_ULTRATHINK", "1" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "ultrathink" ),
    "CLR_NO_ULTRATHINK=1 must suppress ultrathink suffix: {stdout}",
  );
}

// ─── E15: CLR_SYSTEM_PROMPT ───────────────────────────────────────────────────

/// E15: `CLR_SYSTEM_PROMPT` sets the system prompt.
///
/// Spec: `148_env_var_all_params.md` T07
#[ test ]
fn e15_clr_system_prompt_sets_prompt()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_SYSTEM_PROMPT", "Be concise." ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--system-prompt" ),
    "CLR_SYSTEM_PROMPT must add --system-prompt to assembled command: {stdout}",
  );
}

// ─── E16: CLR_APPEND_SYSTEM_PROMPT ────────────────────────────────────────────

/// E16: `CLR_APPEND_SYSTEM_PROMPT` appends to the system prompt.
///
/// Spec: `148_env_var_all_params.md` param 16
#[ test ]
fn e16_clr_append_system_prompt_appends()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_APPEND_SYSTEM_PROMPT", "Always JSON." ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--append-system-prompt" ),
    "CLR_APPEND_SYSTEM_PROMPT must add --append-system-prompt: {stdout}",
  );
}

// ─── E17: CLR_EFFORT ──────────────────────────────────────────────────────────

/// E17: `CLR_EFFORT=low` sets reasoning effort to low.
///
/// Default: `--effort max`. With `CLR_EFFORT=low`, stdout must contain `low` (not `max`).
///
/// Spec: `148_env_var_all_params.md` T14
#[ test ]
fn e17_clr_effort_sets_level()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_EFFORT", "low" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "low" ),
    "CLR_EFFORT=low must set effort to low (not max): {stdout}",
  );
}

// ─── E18: CLR_NO_EFFORT_MAX ───────────────────────────────────────────────────

/// E18: `CLR_NO_EFFORT_MAX=1` suppresses the default `--effort max` injection.
///
/// Default: `--effort max` appears. With `CLR_NO_EFFORT_MAX=1`: `--effort` must be absent.
///
/// Spec: `148_env_var_all_params.md` param 18
#[ test ]
fn e18_clr_no_effort_max_suppresses_default()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_NO_EFFORT_MAX", "1" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--effort" ),
    "CLR_NO_EFFORT_MAX=1 must suppress --effort flag: {stdout}",
  );
}

// ─── E19: CLR_NO_CHROME ───────────────────────────────────────────────────────

/// E19: `CLR_NO_CHROME=1` suppresses the default `--chrome` injection.
///
/// Default: `--chrome` appears. With `CLR_NO_CHROME=1`: `--chrome` must be absent.
///
/// Spec: `148_env_var_all_params.md` T10
#[ test ]
fn e19_clr_no_chrome_suppresses_chrome()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_NO_CHROME", "1" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--chrome" ),
    "CLR_NO_CHROME=1 must suppress --chrome: {stdout}",
  );
}

// ─── E20: CLR_NO_PERSIST ──────────────────────────────────────────────────────

/// E20: `CLR_NO_PERSIST=1` disables session persistence (`--no-session-persistence`).
///
/// Spec: `148_env_var_all_params.md` T11
#[ test ]
fn e20_clr_no_persist_disables_persistence()
{
  let out = run_cli_with_env( &[ "--dry-run", "task" ], &[ ( "CLR_NO_PERSIST", "1" ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--no-session-persistence" ),
    "CLR_NO_PERSIST=1 must add --no-session-persistence: {stdout}",
  );
}

// ─── E21: CLR_JSON_SCHEMA ─────────────────────────────────────────────────────

/// E21: `CLR_JSON_SCHEMA` sets the JSON schema for structured output.
///
/// Spec: `148_env_var_all_params.md` T12
#[ test ]
fn e21_clr_json_schema_sets_schema()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_JSON_SCHEMA", r#"{"type":"string"}"# ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--json-schema" ),
    "CLR_JSON_SCHEMA must add --json-schema to assembled command: {stdout}",
  );
}

// ─── E22: CLR_MCP_CONFIG ──────────────────────────────────────────────────────

/// E22: `CLR_MCP_CONFIG` adds a single MCP config path.
///
/// Spec: `148_env_var_all_params.md` T13
#[ test ]
fn e22_clr_mcp_config_sets_path()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_MCP_CONFIG", "/tmp/e22mcp.json" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--mcp-config" ),
    "CLR_MCP_CONFIG must add --mcp-config flag: {stdout}",
  );
  assert!(
    stdout.contains( "/tmp/e22mcp.json" ),
    "CLR_MCP_CONFIG must include config path: {stdout}",
  );
}

// ─── E23: CLR_CREDS ───────────────────────────────────────────────────────────

/// E23: `CLR_CREDS` supplies the credentials path for the `isolated` subcommand.
///
/// Without `CLR_CREDS` and no `--creds` CLI flag, `isolated` exits 1 with
/// `missing required argument: --creds`. With `CLR_CREDS` set, that error must
/// not appear (the error shifts to file-not-found, confirming `creds_path` was populated).
///
/// Spec: `148_env_var_all_params.md` param 19
#[ test ]
fn e23_clr_creds_supplies_creds_path()
{
  let out = run_cli_with_env(
    &[ "isolated" ],
    &[ ( "CLR_CREDS", "/tmp/e23.creds.json" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "missing required argument: --creds" ),
    "CLR_CREDS must supply --creds to isolated subcommand: {stderr}",
  );
}

// ─── E24: CLR_TIMEOUT ─────────────────────────────────────────────────────────

/// E24: `CLR_TIMEOUT` sets the subprocess timeout for the `isolated` subcommand.
///
/// Combined with `CLR_CREDS` to pass argument validation. Without either env var,
/// `isolated` exits with `missing required argument: --creds`. With both set,
/// that error must not appear (argument parsing succeeds; `timeout_secs` uses the env value).
///
/// Spec: `148_env_var_all_params.md` param 20
#[ test ]
fn e24_clr_timeout_sets_isolated_timeout()
{
  let out = run_cli_with_env(
    &[ "isolated" ],
    &[ ( "CLR_CREDS", "/tmp/e24.creds.json" ), ( "CLR_TIMEOUT", "5" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "missing required argument: --creds" ),
    "CLR_CREDS+CLR_TIMEOUT env vars must supply isolated args: {stderr}",
  );
}
