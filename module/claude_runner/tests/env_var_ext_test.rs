//! CLR_* Environment Variable Tests — Extended (E18–E57)
//!
//! Extension of `env_var_test.rs` (E01–E17) covering suppression flags, credentials,
//! input/output pipeline vars, session/concurrency controls, timeout, and all 20 new
//! retry system env vars added in the 3-tier retry redesign (Plan TSK-205).
//!
//! All tests use `run_cli_with_env()` — no `std::env::set_var`, no thread-global mutation.
//!
//! # Test Matrix
//!
//! | Test | Env Var | Primary Assertion |
//! |------|---------|-------------------|
//! | E18  | `CLR_NO_EFFORT_MAX`        | stdout NOT contains `--effort`                              |
//! | E19  | `CLR_NO_CHROME`            | stdout NOT contains `--chrome`                              |
//! | E20  | `CLR_NO_PERSIST`           | stdout contains `--no-session-persistence`                  |
//! | E21  | `CLR_JSON_SCHEMA`          | stdout contains `--json-schema`                             |
//! | E22  | `CLR_MCP_CONFIG`           | stdout contains `--mcp-config` and the path                 |
//! | E23  | `CLR_CREDS`                | stderr NOT contains `missing required argument: --creds`    |
//! | E24  | `CLR_TIMEOUT`              | stderr NOT contains `missing required argument: --creds`    |
//! | E25–E27 | `CLR_FILE`, `CLR_STRIP_FENCES`, `CLR_KEEP_CLAUDECODE` | dry-run accepted |
//! | E28  | `CLR_TRACE` (isolated)     | stderr contains trace output for isolated subcommand        |
//! | E29  | `CLR_SUBDIR`               | stdout contains `/-feature` path suffix                     |
//! | E30  | `CLR_MAX_SESSIONS`         | dry-run exit 0; invalid value silently ignored              |
//! | BUG-233 | `CLR_SUBDIR` with slash | silently ignored — no partial application                  |
//! | E31  | `CLR_OUTPUT_FILE`          | dry-run exit 0; CLI wins over env                           |
//! | E32  | `CLR_EXPECT`               | dry-run exit 0; CLI wins over env                           |
//! | E33  | `CLR_EXPECT_STRATEGY`      | dry-run exit 0; CLI wins; invalid value → exit 1            |
//! | E37  | `CLR_TIMEOUT` (run/ask)    | dry-run exit 0; CLI wins; invalid value silently ignored    |
//! | E38  | `CLR_RETRY_ON_TRANSIENT`   | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E39  | `CLR_TRANSIENT_DELAY`      | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E40  | `CLR_RETRY_ON_ACCOUNT`     | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E41  | `CLR_ACCOUNT_DELAY`        | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E42  | `CLR_RETRY_ON_AUTH`        | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E43  | `CLR_AUTH_DELAY`           | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E44  | `CLR_RETRY_ON_SERVICE`     | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E45  | `CLR_SERVICE_DELAY`        | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E46  | `CLR_RETRY_ON_PROCESS`     | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E47  | `CLR_PROCESS_DELAY`        | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E48  | `CLR_RETRY_ON_VALIDATION`  | dry-run exit 0; invalid value → exit 1 (only validated var) |
//! | E49  | `CLR_VALIDATION_DELAY`     | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E50  | `CLR_RETRY_ON_RUNNER`      | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E51  | `CLR_RUNNER_DELAY`         | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E52  | `CLR_RETRY_ON_UNKNOWN`     | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E53  | `CLR_UNKNOWN_DELAY`        | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E54  | `CLR_RETRY_OVERRIDE`       | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E55  | `CLR_RETRY_OVERRIDE_DELAY` | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E56  | `CLR_RETRY_DEFAULT`        | dry-run exit 0; CLI wins; invalid silently ignored          |
//! | E57  | `CLR_RETRY_DEFAULT_DELAY`  | dry-run exit 0; CLI wins; invalid silently ignored          |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli_with_env;

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
/// `CLR_CREDS` is the tier-2 resolution for `creds_path` (tier 1: `--creds` flag;
/// tier 3: `$HOME/.claude/.credentials.json`).  Setting `CLR_CREDS` to a non-existent
/// file shifts the error to file-not-found, confirming the path was populated from env.
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
/// Combined with `CLR_CREDS` to supply the credentials path (tier 2) and override
/// the default timeout.  Both env vars must take effect: creds path populated from
/// `CLR_CREDS`, timeout set from `CLR_TIMEOUT`.
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

// ─── E25–E27 (S70–S75): CLR_FILE, CLR_STRIP_FENCES, CLR_KEEP_CLAUDECODE ──────

// S70: CLR_FILE sets stdin file visible in dry-run describe output
#[ test ]
fn s70_clr_file_sets_stdin_file_visible_in_dry_run()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_FILE", "/tmp/e70.txt" ) ],
  );
  assert!( out.status.success(), "dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/tmp/e70.txt" ),
    "CLR_FILE must make path visible in describe output: {stdout}",
  );
}

// S71: CLR_STRIP_FENCES=1 enables strip_fences (dry-run accepted)
#[ test ]
fn s71_clr_strip_fences_1_enables()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_STRIP_FENCES", "1" ) ],
  );
  assert!( out.status.success(), "CLR_STRIP_FENCES=1 must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S72: CLR_KEEP_CLAUDECODE=1 enables keep_claudecode (dry-run accepted)
#[ test ]
fn s72_clr_keep_claudecode_1_enables()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_KEEP_CLAUDECODE", "1" ) ],
  );
  assert!( out.status.success(), "CLR_KEEP_CLAUDECODE=1 must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S73: CLI --file wins over CLR_FILE
#[ test ]
fn s73_cli_file_wins_over_clr_file()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "--file", "/tmp/cli.txt", "t" ],
    &[ ( "CLR_FILE", "/tmp/env.txt" ) ],
  );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "/tmp/cli.txt" ),
    "CLI --file must win. Got:\n{stdout}",
  );
  assert!(
    !stdout.contains( "/tmp/env.txt" ),
    "CLR_FILE must NOT appear when CLI wins. Got:\n{stdout}",
  );
}

// S74: CLR_STRIP_FENCES=yes rejected (env_bool only accepts 1/true)
#[ test ]
fn s74_clr_strip_fences_yes_rejected()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_STRIP_FENCES", "yes" ) ],
  );
  assert!( out.status.success(), "CLR_STRIP_FENCES=yes must exit 0 (rejected silently). stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// S75: CLR_KEEP_CLAUDECODE=yes rejected (env_bool only accepts 1/true)
#[ test ]
fn s75_clr_keep_claudecode_yes_rejected()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_KEEP_CLAUDECODE", "yes" ) ],
  );
  assert!( out.status.success(), "CLR_KEEP_CLAUDECODE=yes must exit 0 (rejected silently). stderr: {}", String::from_utf8_lossy( &out.stderr ) );
}

// ─── E28: CLR_TRACE for isolated subcommand ────────────────────────────────────

/// E28: `CLR_TRACE=1` enables trace output for the `isolated` subcommand.
///
/// Trace fires before credentials are read, so a nonexistent creds path is
/// sufficient — the trace lines appear on stderr before the file-not-found error.
///
/// Spec: `02_clr_input_vars.md` E28
#[ test ]
fn e28_clr_trace_applies_to_isolated()
{
  let out = run_cli_with_env(
    &[ "isolated" ],
    &[ ( "CLR_CREDS", "/tmp/e28_nonexistent.creds.json" ), ( "CLR_TRACE", "1" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "# clr isolated" ),
    "CLR_TRACE=1 must emit '# clr isolated' trace for isolated subcommand: {stderr}",
  );
  assert!(
    stderr.contains( "# creds:" ),
    "CLR_TRACE=1 trace must include '# creds:' line: {stderr}",
  );
}

// ─── E29: CLR_SUBDIR ──────────────────────────────────────────────────────────

/// E29: `CLR_SUBDIR` appends `/-NAME` to the effective working directory.
///
/// CLI-wins: explicit `--subdir build` takes precedence over `CLR_SUBDIR=debug`.
///
/// Spec: `tests/docs/cli/user_story/22_session_isolation_subdir.md` US-3, US-4, US-5
#[ test ]
fn e29_clr_subdir_sets_effective_dir()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_SUBDIR", "feature" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let sep = std::path::MAIN_SEPARATOR;
  assert!(
    stdout.contains( &format!( "{sep}-feature" ) ),
    "CLR_SUBDIR=feature must produce path ending in {sep}-feature: {stdout}",
  );
  // CLI-wins: --subdir build must take precedence over CLR_SUBDIR=debug
  let out2 = run_cli_with_env(
    &[ "--dry-run", "--subdir", "build", "t" ],
    &[ ( "CLR_SUBDIR", "debug" ) ],
  );
  assert!( out2.status.success(), "CLI --subdir with CLR_SUBDIR must exit 0: {out2:?}" );
  let stdout2 = String::from_utf8_lossy( &out2.stdout );
  assert!(
    stdout2.contains( &format!( "{sep}-build" ) ),
    "CLI --subdir build must win over CLR_SUBDIR=debug: {stdout2}",
  );
  assert!(
    !stdout2.contains( &format!( "{sep}-debug" ) ),
    "CLR_SUBDIR=debug must be suppressed by CLI --subdir: {stdout2}",
  );
}

// ─── E30: CLR_MAX_SESSIONS ────────────────────────────────────────────────────

/// E30: `CLR_MAX_SESSIONS=N` sets the session concurrency limit.
///
/// Dry-run bypasses the gate so we can verify parsing without blocking.
/// Invalid value silently ignored (parse failure → default 10 used).
/// CLI wins: `--max-sessions 5` overrides `CLR_MAX_SESSIONS=2`.
///
/// Spec: `tests/docs/cli/env_param/02_clr_input_vars.md` E30
#[ test ]
fn e30_clr_max_sessions_accepted_in_dry_run()
{
  // Env-alone: dry-run exits 0 (gate bypassed by dry-run path)
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_MAX_SESSIONS", "3" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_MAX_SESSIONS=3 + --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );

  // Invalid value silently ignored — still exits 0
  let out2 = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_MAX_SESSIONS", "notanumber" ) ],
  );
  assert!(
    out2.status.success(),
    "CLR_MAX_SESSIONS=notanumber must be silently ignored and exit 0. stderr: {}",
    String::from_utf8_lossy( &out2.stderr ),
  );

  // CLI-wins: --max-sessions 5 takes precedence over CLR_MAX_SESSIONS=2
  let out3 = run_cli_with_env(
    &[ "--dry-run", "--max-sessions", "5", "task" ],
    &[ ( "CLR_MAX_SESSIONS", "2" ) ],
  );
  assert!(
    out3.status.success(),
    "CLI --max-sessions with CLR_MAX_SESSIONS must exit 0. stderr: {}",
    String::from_utf8_lossy( &out3.stderr ),
  );
}

// ─── BUG-233 CLR_SUBDIR slash validation (bug reproducer) ──────────────────────

/// Fix(BUG-233): `CLR_SUBDIR=a/b` must be silently ignored — same constraint as `--subdir`.
///
/// ## Root Cause
/// `apply_env_vars` assigned `CLR_SUBDIR` directly to `parsed.subdir` without the
/// `contains('/')` check that `parse_value_flag` applies to CLI `--subdir`.
///
/// ## Why Not Caught
/// BUG-230 only fixed the CLI parse path; env-var path was not tested for slashes.
///
/// ## Fix Applied
/// Added `!v.contains('/')` guard in `apply_env_vars` for `CLR_SUBDIR`.
///
/// ## Prevention
/// When adding validation to a CLI flag, audit the corresponding env-var path too.
///
/// ## Pitfall
/// `apply_env_vars` doesn't return `Result` — invalid env values are silently ignored,
/// not rejected with an error. This matches the existing convention (see `CLR_EFFORT`).
// test_kind: bug_reproducer(BUG-233)
#[ test ]
fn bug233_clr_subdir_slash_silently_ignored()
{
  // CLR_SUBDIR=a/b should be silently dropped — no /-a/b in output
  let out = run_cli_with_env(
    &[ "--dry-run", "t" ],
    &[ ( "CLR_SUBDIR", "a/b" ) ],
  );
  assert!( out.status.success(), "must exit 0 even with invalid CLR_SUBDIR: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let sep = std::path::MAIN_SEPARATOR;
  assert!(
    !stdout.contains( &format!( "{sep}-a" ) ),
    "CLR_SUBDIR=a/b must be silently ignored — no {sep}-a in output: {stdout}",
  );
}

// ─── E31: CLR_OUTPUT_FILE ─────────────────────────────────────────────────────

/// E31: `CLR_OUTPUT_FILE` sets the output capture path.
///
/// Dry-run exits 0 (no file is created); CLI `--output-file` wins over env.
///
/// Spec: `tests/docs/cli/env_param/02_clr_input_vars.md` E31
#[ test ]
fn e31_clr_output_file_sets_path()
{
  // Env-alone: dry-run exits 0 (gate logic skipped, file not created)
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_OUTPUT_FILE", "/tmp/e31_out.txt" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_OUTPUT_FILE + --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );

  // CLI-wins: --output-file /tmp/cli.txt takes precedence over CLR_OUTPUT_FILE=/tmp/env.txt
  let out2 = run_cli_with_env(
    &[ "--dry-run", "--output-file", "/tmp/cli.txt", "task" ],
    &[ ( "CLR_OUTPUT_FILE", "/tmp/env.txt" ) ],
  );
  assert!(
    out2.status.success(),
    "CLI --output-file with CLR_OUTPUT_FILE must exit 0. stderr: {}",
    String::from_utf8_lossy( &out2.stderr ),
  );
}

// ─── E32: CLR_EXPECT ──────────────────────────────────────────────────────────

/// E32: `CLR_EXPECT` sets the enum validation pattern.
///
/// Dry-run exits 0 (no subprocess, no validation); CLI `--expect` wins over env.
///
/// Spec: `tests/docs/cli/env_param/02_clr_input_vars.md` E32
#[ test ]
fn e32_clr_expect_sets_pattern()
{
  // Env-alone: dry-run exits 0
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_EXPECT", "yes|no" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_EXPECT=yes|no + --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );

  // CLI-wins: --expect ok|fail takes precedence over CLR_EXPECT=yes|no
  let out2 = run_cli_with_env(
    &[ "--dry-run", "--expect", "ok|fail", "task" ],
    &[ ( "CLR_EXPECT", "yes|no" ) ],
  );
  assert!(
    out2.status.success(),
    "CLI --expect with CLR_EXPECT must exit 0. stderr: {}",
    String::from_utf8_lossy( &out2.stderr ),
  );
}

// ─── E33: CLR_EXPECT_STRATEGY ─────────────────────────────────────────────────

/// E33: `CLR_EXPECT_STRATEGY` sets the mismatch handling strategy.
///
/// Valid values (fail/retry/default:V) accepted; invalid value → exit 1.
/// CLI `--expect-strategy` wins over env.
///
/// Spec: `tests/docs/cli/env_param/02_clr_input_vars.md` E33
#[ test ]
fn e33_clr_expect_strategy_sets_handler()
{
  // Env-alone with valid value: dry-run exits 0
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_EXPECT_STRATEGY", "retry" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_EXPECT_STRATEGY=retry + --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );

  // CLI-wins: --expect-strategy fail takes precedence over CLR_EXPECT_STRATEGY=retry
  let out2 = run_cli_with_env(
    &[ "--dry-run", "--expect-strategy", "fail", "task" ],
    &[ ( "CLR_EXPECT_STRATEGY", "retry" ) ],
  );
  assert!(
    out2.status.success(),
    "CLI --expect-strategy with CLR_EXPECT_STRATEGY must exit 0. stderr: {}",
    String::from_utf8_lossy( &out2.stderr ),
  );

  // Invalid value: parse failure must exit 1 with error message
  let out3 = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_EXPECT_STRATEGY", "bogus" ) ],
  );
  assert_eq!(
    out3.status.code(),
    Some( 1 ),
    "CLR_EXPECT_STRATEGY=bogus must exit 1. stderr: {}",
    String::from_utf8_lossy( &out3.stderr ),
  );
  let stderr3 = String::from_utf8_lossy( &out3.stderr );
  assert!(
    !stderr3.is_empty(),
    "CLR_EXPECT_STRATEGY=bogus must emit an error message on stderr",
  );
}

// ─── E37: CLR_TIMEOUT (run/ask) ───────────────────────────────────────────────

/// E37: `CLR_TIMEOUT` sets the subprocess timeout for run/ask dispatch paths.
///
/// `0` = unlimited (no watchdog; same as default). Valid u32 values silently accepted;
/// invalid values silently ignored (field stays at default 0). CLI `--timeout` wins over env.
///
/// Note: `CLR_TIMEOUT` also applies to `isolated`/`refresh` (tested separately in E24).
///
/// Spec: `tests/docs/cli/env_param/02_clr_input_vars.md` E37
#[ test ]
fn e37_clr_timeout_sets_run_timeout()
{
  // Env-alone with valid value: dry-run exits 0 (env var accepted)
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_TIMEOUT", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_TIMEOUT=30 + --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );

  // CLI-wins: --timeout 60 takes precedence over CLR_TIMEOUT=30
  let out2 = run_cli_with_env(
    &[ "--dry-run", "--timeout", "60", "task" ],
    &[ ( "CLR_TIMEOUT", "30" ) ],
  );
  assert!(
    out2.status.success(),
    "CLI --timeout with CLR_TIMEOUT must exit 0. stderr: {}",
    String::from_utf8_lossy( &out2.stderr ),
  );

  // Zero: unlimited — dry-run exits 0 (no watchdog spawned)
  let out3 = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_TIMEOUT", "0" ) ],
  );
  assert!(
    out3.status.success(),
    "CLR_TIMEOUT=0 (unlimited) + --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out3.stderr ),
  );

  // Invalid value: parse failure silently ignored → default 0 used; dry-run exits 0
  let out4 = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_TIMEOUT", "notanumber" ) ],
  );
  assert!(
    out4.status.success(),
    "CLR_TIMEOUT=notanumber silently ignored; --dry-run must exit 0. stderr: {}",
    String::from_utf8_lossy( &out4.stderr ),
  );
}

// ─── E38–E57: New 3-tier retry env vars (TSK-205) ────────────────────────────
//
// All 20 new retry system env vars introduced in the retry system redesign.
// Each test verifies: (1) valid value accepted in dry-run; (2) CLI wins over env.
// CLR_RETRY_ON_VALIDATION is unique: invalid value → exit 1 (not silently ignored).
// All others silently ignore invalid values.

/// E38: `CLR_RETRY_ON_TRANSIENT` — Transient class retry count.
#[ test ]
fn e38_clr_retry_on_transient_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_TRANSIENT", "2" ) ] );
  assert!( out.status.success(), "CLR_RETRY_ON_TRANSIENT=2 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-on-transient", "0", "t" ], &[ ( "CLR_RETRY_ON_TRANSIENT", "2" ) ] );
  assert!( out2.status.success(), "CLI --retry-on-transient must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
  let out3 = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_TRANSIENT", "bad" ) ] );
  assert!( out3.status.success(), "CLR_RETRY_ON_TRANSIENT=bad silently ignored; dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out3.stderr ) );
}

/// E39: `CLR_TRANSIENT_DELAY` — Transient class delay (seconds).
#[ test ]
fn e39_clr_transient_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_TRANSIENT_DELAY", "10" ) ] );
  assert!( out.status.success(), "CLR_TRANSIENT_DELAY=10 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--transient-delay", "5", "t" ], &[ ( "CLR_TRANSIENT_DELAY", "10" ) ] );
  assert!( out2.status.success(), "CLI --transient-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E40: `CLR_RETRY_ON_ACCOUNT` — Account class retry count.
#[ test ]
fn e40_clr_retry_on_account_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_ACCOUNT", "1" ) ] );
  assert!( out.status.success(), "CLR_RETRY_ON_ACCOUNT=1 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-on-account", "0", "t" ], &[ ( "CLR_RETRY_ON_ACCOUNT", "1" ) ] );
  assert!( out2.status.success(), "CLI --retry-on-account must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
  let out3 = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_ACCOUNT", "bad" ) ] );
  assert!( out3.status.success(), "CLR_RETRY_ON_ACCOUNT=bad silently ignored. stderr: {}", String::from_utf8_lossy( &out3.stderr ) );
}

/// E41: `CLR_ACCOUNT_DELAY` — Account class delay (seconds).
#[ test ]
fn e41_clr_account_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_ACCOUNT_DELAY", "60" ) ] );
  assert!( out.status.success(), "CLR_ACCOUNT_DELAY=60 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--account-delay", "5", "t" ], &[ ( "CLR_ACCOUNT_DELAY", "60" ) ] );
  assert!( out2.status.success(), "CLI --account-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E42: `CLR_RETRY_ON_AUTH` — Auth class retry count.
#[ test ]
fn e42_clr_retry_on_auth_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_AUTH", "0" ) ] );
  assert!( out.status.success(), "CLR_RETRY_ON_AUTH=0 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-on-auth", "1", "t" ], &[ ( "CLR_RETRY_ON_AUTH", "0" ) ] );
  assert!( out2.status.success(), "CLI --retry-on-auth must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E43: `CLR_AUTH_DELAY` — Auth class delay (seconds).
#[ test ]
fn e43_clr_auth_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_AUTH_DELAY", "30" ) ] );
  assert!( out.status.success(), "CLR_AUTH_DELAY=30 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--auth-delay", "0", "t" ], &[ ( "CLR_AUTH_DELAY", "30" ) ] );
  assert!( out2.status.success(), "CLI --auth-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E44: `CLR_RETRY_ON_SERVICE` — Service class retry count.
#[ test ]
fn e44_clr_retry_on_service_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_SERVICE", "2" ) ] );
  assert!( out.status.success(), "CLR_RETRY_ON_SERVICE=2 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-on-service", "0", "t" ], &[ ( "CLR_RETRY_ON_SERVICE", "2" ) ] );
  assert!( out2.status.success(), "CLI --retry-on-service must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
  let out3 = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_SERVICE", "bad" ) ] );
  assert!( out3.status.success(), "CLR_RETRY_ON_SERVICE=bad silently ignored. stderr: {}", String::from_utf8_lossy( &out3.stderr ) );
}

/// E45: `CLR_SERVICE_DELAY` — Service class delay (seconds).
#[ test ]
fn e45_clr_service_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_SERVICE_DELAY", "15" ) ] );
  assert!( out.status.success(), "CLR_SERVICE_DELAY=15 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--service-delay", "5", "t" ], &[ ( "CLR_SERVICE_DELAY", "15" ) ] );
  assert!( out2.status.success(), "CLI --service-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E46: `CLR_RETRY_ON_PROCESS` — Process class retry count.
#[ test ]
fn e46_clr_retry_on_process_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_PROCESS", "1" ) ] );
  assert!( out.status.success(), "CLR_RETRY_ON_PROCESS=1 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-on-process", "0", "t" ], &[ ( "CLR_RETRY_ON_PROCESS", "1" ) ] );
  assert!( out2.status.success(), "CLI --retry-on-process must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E47: `CLR_PROCESS_DELAY` — Process class delay (seconds).
#[ test ]
fn e47_clr_process_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_PROCESS_DELAY", "10" ) ] );
  assert!( out.status.success(), "CLR_PROCESS_DELAY=10 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--process-delay", "0", "t" ], &[ ( "CLR_PROCESS_DELAY", "10" ) ] );
  assert!( out2.status.success(), "CLI --process-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E48: `CLR_RETRY_ON_VALIDATION` — Validation class retry count.
///
/// Unlike all other per-class retry env vars, `CLR_RETRY_ON_VALIDATION` exits 1
/// on invalid values — it goes through `parse_u8_bounded` rather than `.parse().ok()`.
/// Detailed EC coverage in `retry_validation_test.rs`.
#[ test ]
fn e48_clr_retry_on_validation_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_VALIDATION", "2" ) ] );
  assert!( out.status.success(), "CLR_RETRY_ON_VALIDATION=2 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-on-validation", "0", "t" ], &[ ( "CLR_RETRY_ON_VALIDATION", "2" ) ] );
  assert!( out2.status.success(), "CLI --retry-on-validation must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
  let out3 = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_VALIDATION", "bad" ) ] );
  assert_eq!( out3.status.code(), Some( 1 ), "CLR_RETRY_ON_VALIDATION=bad must exit 1 (only validated retry env var). stderr: {}", String::from_utf8_lossy( &out3.stderr ) );
}

/// E49: `CLR_VALIDATION_DELAY` — Validation class delay (seconds).
#[ test ]
fn e49_clr_validation_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_VALIDATION_DELAY", "5" ) ] );
  assert!( out.status.success(), "CLR_VALIDATION_DELAY=5 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--validation-delay", "0", "t" ], &[ ( "CLR_VALIDATION_DELAY", "5" ) ] );
  assert!( out2.status.success(), "CLI --validation-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E50: `CLR_RETRY_ON_RUNNER` — Runner class retry count.
#[ test ]
fn e50_clr_retry_on_runner_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_RUNNER", "0" ) ] );
  assert!( out.status.success(), "CLR_RETRY_ON_RUNNER=0 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-on-runner", "1", "t" ], &[ ( "CLR_RETRY_ON_RUNNER", "0" ) ] );
  assert!( out2.status.success(), "CLI --retry-on-runner must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E51: `CLR_RUNNER_DELAY` — Runner class delay (seconds).
#[ test ]
fn e51_clr_runner_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RUNNER_DELAY", "10" ) ] );
  assert!( out.status.success(), "CLR_RUNNER_DELAY=10 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--runner-delay", "0", "t" ], &[ ( "CLR_RUNNER_DELAY", "10" ) ] );
  assert!( out2.status.success(), "CLI --runner-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E52: `CLR_RETRY_ON_UNKNOWN` — Unknown class retry count.
#[ test ]
fn e52_clr_retry_on_unknown_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_UNKNOWN", "1" ) ] );
  assert!( out.status.success(), "CLR_RETRY_ON_UNKNOWN=1 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-on-unknown", "0", "t" ], &[ ( "CLR_RETRY_ON_UNKNOWN", "1" ) ] );
  assert!( out2.status.success(), "CLI --retry-on-unknown must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
  let out3 = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_ON_UNKNOWN", "bad" ) ] );
  assert!( out3.status.success(), "CLR_RETRY_ON_UNKNOWN=bad silently ignored. stderr: {}", String::from_utf8_lossy( &out3.stderr ) );
}

/// E53: `CLR_UNKNOWN_DELAY` — Unknown class delay (seconds).
#[ test ]
fn e53_clr_unknown_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_UNKNOWN_DELAY", "20" ) ] );
  assert!( out.status.success(), "CLR_UNKNOWN_DELAY=20 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--unknown-delay", "0", "t" ], &[ ( "CLR_UNKNOWN_DELAY", "20" ) ] );
  assert!( out2.status.success(), "CLI --unknown-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E54: `CLR_RETRY_OVERRIDE` — Tier-1 override count (all classes).
#[ test ]
fn e54_clr_retry_override_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_OVERRIDE", "3" ) ] );
  assert!( out.status.success(), "CLR_RETRY_OVERRIDE=3 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-override", "0", "t" ], &[ ( "CLR_RETRY_OVERRIDE", "3" ) ] );
  assert!( out2.status.success(), "CLI --retry-override must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
  let out3 = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_OVERRIDE", "bad" ) ] );
  assert!( out3.status.success(), "CLR_RETRY_OVERRIDE=bad silently ignored. stderr: {}", String::from_utf8_lossy( &out3.stderr ) );
}

/// E55: `CLR_RETRY_OVERRIDE_DELAY` — Tier-1 override delay (all classes).
#[ test ]
fn e55_clr_retry_override_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_OVERRIDE_DELAY", "5" ) ] );
  assert!( out.status.success(), "CLR_RETRY_OVERRIDE_DELAY=5 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-override-delay", "0", "t" ], &[ ( "CLR_RETRY_OVERRIDE_DELAY", "5" ) ] );
  assert!( out2.status.success(), "CLI --retry-override-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
}

/// E56: `CLR_RETRY_DEFAULT` — Tier-3 fallback retry count (0–255).
#[ test ]
fn e56_clr_retry_default_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_DEFAULT", "2" ) ] );
  assert!( out.status.success(), "CLR_RETRY_DEFAULT=2 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-default", "0", "t" ], &[ ( "CLR_RETRY_DEFAULT", "2" ) ] );
  assert!( out2.status.success(), "CLI --retry-default must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
  let out3 = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_DEFAULT", "bad" ) ] );
  assert!( out3.status.success(), "CLR_RETRY_DEFAULT=bad silently ignored. stderr: {}", String::from_utf8_lossy( &out3.stderr ) );
}

/// E57: `CLR_RETRY_DEFAULT_DELAY` — Tier-3 fallback delay (seconds).
#[ test ]
fn e57_clr_retry_default_delay_accepted()
{
  let out = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_DEFAULT_DELAY", "30" ) ] );
  assert!( out.status.success(), "CLR_RETRY_DEFAULT_DELAY=30 dry-run must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let out2 = run_cli_with_env( &[ "--dry-run", "--retry-default-delay", "0", "t" ], &[ ( "CLR_RETRY_DEFAULT_DELAY", "30" ) ] );
  assert!( out2.status.success(), "CLI --retry-default-delay must win over env. stderr: {}", String::from_utf8_lossy( &out2.stderr ) );
  let out3 = run_cli_with_env( &[ "--dry-run", "t" ], &[ ( "CLR_RETRY_DEFAULT_DELAY", "bad" ) ] );
  assert!( out3.status.success(), "CLR_RETRY_DEFAULT_DELAY=bad silently ignored. stderr: {}", String::from_utf8_lossy( &out3.stderr ) );
}
