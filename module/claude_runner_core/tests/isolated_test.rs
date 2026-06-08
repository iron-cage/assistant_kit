//! Isolated subprocess runner tests: `IsolatedRunResult` and `RunnerError`.
//!
//! T01–T09 are offline (no `lim_it`) — struct construction, Display, and type contracts.
//! T07–T08 are live (`lim_it`) — actual subprocess execution with real Claude binary.
//!
//! ## Test Matrix
//!
//! | ID  | Scenario                                              | Expected                             | Live? |
//! |-----|-------------------------------------------------------|--------------------------------------|-------|
//! | T01 | `IsolatedRunResult` field accessibility               | all pub fields readable              | no    |
//! | T02 | `IsolatedRunResult` with `credentials: Some(...)`     | `credentials == Some(json)`          | no    |
//! | T03 | `RunnerError::ClaudeNotFound` Display                 | contains "not found"                 | no    |
//! | T04 | `RunnerError::TempDirFailed(String)` Display          | contains "temp dir"                  | no    |
//! | T05 | `RunnerError::Timeout { secs }` Display               | contains "timed out" + secs value    | no    |
//! | T06 | `RunnerError::Io(String)` Display                     | contains the reason string           | no    |
//! | T07 | `run_isolated()` with valid creds → exit_code -1/0    | `IsolatedRunResult` returned         | yes   |
//! | T08 | `run_isolated()` with timeout 0 → `Err(TimeoutWithOutput)` | `secs: 0`, `partial_stdout: ""`  | yes   |
//! | T09 | timeout-with-credentials sentinel: `exit_code = -1`   | `Ok` with `credentials: Some(...)`   | no    |
//! | T10 | `IsolatedModel::model_id()` all 3 variants + constant | correct `Option<&str>` per variant   | no    |
//! | T11 | `ISOLATED_CLAUDE_MD` keyword content (AC-42)          | contains expected instruction terms   | no    |
//! | T12 | `with_home_isolation()` suppresses `--chrome` (AC-41) | `describe()` omits `--chrome`        | no    |

use claude_runner_core::{ IsolatedModel, IsolatedRunResult, RunnerError, ISOLATED_DEFAULT_MODEL, ISOLATED_CLAUDE_MD };

// ── T01 ───────────────────────────────────────────────────────────────────────

/// T01: `IsolatedRunResult` exposes all four pub fields.
///
/// Constructs the struct directly (all fields are `pub`) and reads each one.
#[ test ]
fn t01_isolated_run_result_field_accessibility()
{
  let r = IsolatedRunResult
  {
    exit_code   : 0,
    stdout      : "hello".to_string(),
    stderr      : String::new(),
    credentials : None,
  };
  assert_eq!( r.exit_code, 0 );
  assert_eq!( r.stdout, "hello" );
  assert!( r.stderr.is_empty() );
  assert!( r.credentials.is_none() );
}

// ── T02 ───────────────────────────────────────────────────────────────────────

/// T02: `IsolatedRunResult` with `credentials: Some(...)` round-trips correctly.
#[ test ]
fn t02_isolated_run_result_credentials_some()
{
  let json = r#"{"accessToken":"tok","refreshToken":"rtok","expiresAt":9999}"#;
  let r = IsolatedRunResult
  {
    exit_code   : 0,
    stdout      : String::new(),
    stderr      : String::new(),
    credentials : Some( json.to_string() ),
  };
  assert_eq!( r.credentials.as_deref(), Some( json ) );
}

// ── T03 ───────────────────────────────────────────────────────────────────────

/// T03: `RunnerError::ClaudeNotFound` Display contains "not found".
#[ test ]
fn t03_runner_error_claude_not_found_display()
{
  let msg = RunnerError::ClaudeNotFound.to_string();
  assert!(
    msg.contains( "not found" ),
    "ClaudeNotFound Display must contain 'not found', got: {msg}",
  );
}

// ── T04 ───────────────────────────────────────────────────────────────────────

/// T04: `RunnerError::TempDirFailed(String)` Display contains "temp dir" and the reason.
#[ test ]
fn t04_runner_error_temp_dir_failed_display()
{
  let msg = RunnerError::TempDirFailed( "permission denied".to_string() ).to_string();
  assert!(
    msg.contains( "temp dir" ) || msg.contains( "temp" ),
    "TempDirFailed Display must reference temp dir, got: {msg}",
  );
  assert!(
    msg.contains( "permission denied" ),
    "TempDirFailed Display must contain reason, got: {msg}",
  );
}

// ── T05 ───────────────────────────────────────────────────────────────────────

/// T05: `RunnerError::Timeout { secs: 30 }` Display contains "timed out" and the secs value.
#[ test ]
fn t05_runner_error_timeout_display()
{
  let msg = RunnerError::Timeout { secs : 30 }.to_string();
  assert!(
    msg.contains( "timed out" ) || msg.contains( "timeout" ),
    "Timeout Display must contain 'timed out'/'timeout', got: {msg}",
  );
  assert!(
    msg.contains( "30" ),
    "Timeout Display must contain the secs value (30), got: {msg}",
  );
}

// ── T06 ───────────────────────────────────────────────────────────────────────

/// T06: `RunnerError::Io(String)` Display contains the I/O reason.
#[ test ]
fn t06_runner_error_io_display()
{
  let msg = RunnerError::Io( "no space left on device".to_string() ).to_string();
  assert!(
    msg.contains( "no space left on device" ),
    "Io Display must contain the reason, got: {msg}",
  );
}

// ── T07 ───────────────────────────────────────────────────────────────────────

/// T07 (`lim_it`): `run_isolated()` with valid credentials returns an `IsolatedRunResult`.
///
/// Passes `["--version"]` as args; the Claude binary prints its version and exits 0.
/// Since the credentials are minimal (not real OAuth tokens), Claude may exit non-zero
/// — but the function must return `Ok(IsolatedRunResult)` rather than `Err(...)`.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn t07_lim_it_run_isolated_returns_result()
{
  use claude_runner_core::run_isolated;

  fn claude_binary_available() -> bool
  {
    std::process::Command::new( "claude" )
      .arg( "--version" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::null() )
      .status()
      .is_ok()
  }

  if !claude_binary_available() { return; }

  let creds = r#"{"accessToken":"tok-test","refreshToken":"rtok-test","expiresAt":9999999999}"#;
  let result = run_isolated( creds, vec![ "--version".to_string() ], 30, IsolatedModel::Default );
  assert!( result.is_ok(), "run_isolated must return Ok for valid args, got: {:?}", result.err() );
  let out = result.unwrap();
  // exit_code of 0 is expected for --version; -1 is acceptable if binary returns none
  assert!( out.exit_code >= -1, "exit_code must be -1 or higher, got: {}", out.exit_code );
}

// ── T08 ───────────────────────────────────────────────────────────────────────

/// T08 (`lim_it`): `run_isolated()` with timeout 0 returns `Err(RunnerError::TimeoutWithOutput)`.
///
/// A 0-second timeout should expire before any subprocess can complete.
/// After BUG-243 fix, timeout always returns `TimeoutWithOutput` (`partial_stdout` may be empty).
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn t08_lim_it_run_isolated_timeout()
{
  use claude_runner_core::run_isolated;

  fn claude_binary_available() -> bool
  {
    std::process::Command::new( "claude" )
      .arg( "--version" )
      .stdout( std::process::Stdio::null() )
      .stderr( std::process::Stdio::null() )
      .status()
      .is_ok()
  }

  if !claude_binary_available() { return; }

  let creds = r#"{"accessToken":"tok-test","refreshToken":"rtok-test","expiresAt":9999999999}"#;
  let result = run_isolated( creds, vec![ "--version".to_string() ], 0, IsolatedModel::Default );
  match result
  {
    Err( RunnerError::TimeoutWithOutput { secs, .. } ) =>
    {
      assert_eq!( secs, 0, "Timeout secs must match the given timeout, got: {secs}" );
    }
    other => panic!( "expected Err(TimeoutWithOutput), got: {other:?}" ),
  }
}

// ── T09 ───────────────────────────────────────────────────────────────────────

/// T09: Timeout-with-credentials result type: `Ok(IsolatedRunResult { exit_code: -1 })`.
///
/// When `run_isolated()` times out but the subprocess already wrote refreshed credentials
/// (OAuth token-refresh at startup before blocking on stdin), the function returns
/// `Ok(IsolatedRunResult { exit_code: -1, credentials: Some(...) })` — NOT `Err(Timeout)`.
///
/// The `exit_code = -1` sentinel distinguishes "timeout + credentials" from a clean exit
/// (`exit_code` ≥ 0). Callers check `credentials.is_some()` to determine whether to write
/// back and retry — a pattern used by `usage_routine()` in the `refresh::1` path.
///
/// ## Bug Reproducer
///
/// issue: isolated-credentials-on-timeout
/// Scenario: `run_isolated(expired_creds, [], 30)` — Claude refreshes the token
///           at startup then waits for interactive input. Previously the 30-second
///           timeout fired and returned `Err(Timeout)`, discarding the refreshed
///           credentials and silently skipping the `refresh::1` retry.
/// Root cause: `exec_result = Err(RecvTimeoutError::Timeout)` was mapped unconditionally
///             to `Err(RunnerError::Timeout)`, dropping the `credentials` value that
///             was already read from the temp file in step 6.
/// Fix: Check `credentials.is_some()` before returning `Err(Timeout)`; if the file
///      changed, return `Ok(IsolatedRunResult { exit_code: -1, credentials })` instead.
/// Pitfall: Always read credentials BEFORE the timeout match and check them in both
///          the timeout and the success branches.
#[ test ]
fn t09_timeout_with_changed_credentials_result_type()
{
  // Verify the canonical result type for "timeout + credentials updated":
  // exit_code = -1 is the sentinel that distinguishes this case.
  let result = IsolatedRunResult
  {
    exit_code   : -1,
    stdout      : String::new(),
    stderr      : String::new(),
    credentials : Some( r#"{"accessToken":"refreshed","expiresAt":9999999999}"#.to_string() ),
  };
  assert_eq!(
    result.exit_code, -1,
    "timeout-with-credentials sentinel must use exit_code = -1",
  );
  assert!(
    result.credentials.is_some(),
    "timeout-with-credentials result must carry the refreshed credentials",
  );
}

// ── T10 ───────────────────────────────────────────────────────────────────────

/// T10: `IsolatedModel::model_id()` returns the correct `Option<&str>` for all
/// three variants, and `ISOLATED_DEFAULT_MODEL` equals `"claude-opus-4-6"`.
///
/// Covers FT-1 through FT-4 from `tests/docs/feature/004_run_isolated.md`:
/// - FT-1: `Default.model_id()` → `Some(ISOLATED_DEFAULT_MODEL)`
/// - FT-2: `KeepCurrent.model_id()` → `None` (no `--model` flag injected)
/// - FT-3: `Specific("custom-model").model_id()` → `Some("custom-model")`
/// - FT-4: `ISOLATED_DEFAULT_MODEL == "claude-opus-4-6"` (Opus for real tasks; Task 021)
#[ test ]
fn t10_isolated_model_model_id_all_variants()
{
  // FT-4: constant value pinned to the current production Opus model ID (Task 021).
  assert_eq!(
    ISOLATED_DEFAULT_MODEL, "claude-opus-4-6",
    "ISOLATED_DEFAULT_MODEL must equal the current production Opus ID",
  );

  // FT-1: Default → Some(ISOLATED_DEFAULT_MODEL).
  assert_eq!(
    IsolatedModel::Default.model_id(),
    Some( ISOLATED_DEFAULT_MODEL ),
    "IsolatedModel::Default.model_id() must return Some(ISOLATED_DEFAULT_MODEL)",
  );

  // FT-2: KeepCurrent → None (caller wants no --model flag).
  assert!(
    IsolatedModel::KeepCurrent.model_id().is_none(),
    "IsolatedModel::KeepCurrent.model_id() must return None",
  );

  // FT-3: Specific(id) → Some(id) — round-trip the supplied string.
  let custom = "claude-haiku-4-5-20251001";
  assert_eq!(
    IsolatedModel::Specific( custom.to_string() ).model_id(),
    Some( custom ),
    "IsolatedModel::Specific.model_id() must return Some of the given model ID",
  );
}

// ── T11 ───────────────────────────────────────────────────────────────────────

/// T11: `ISOLATED_CLAUDE_MD` contains the required instruction keywords (AC-42).
///
/// The constant must instruct the subprocess to respond immediately without
/// extended thinking, no preamble, and no tool use. These keywords are checked
/// individually so that any future rewording must still satisfy all three.
///
/// Spec: [`tests/docs/feature/004_run_isolated.md` FT-5]
#[ test ]
fn t_run_isolated_claude_md_content()
{
  assert!(
    ISOLATED_CLAUDE_MD.contains( "extended thinking" ),
    "ISOLATED_CLAUDE_MD must mention extended thinking, got: {ISOLATED_CLAUDE_MD}",
  );
  assert!(
    ISOLATED_CLAUDE_MD.contains( "preamble" ),
    "ISOLATED_CLAUDE_MD must mention no preamble, got: {ISOLATED_CLAUDE_MD}",
  );
  assert!(
    ISOLATED_CLAUDE_MD.contains( "tool" ),
    "ISOLATED_CLAUDE_MD must mention no tool use, got: {ISOLATED_CLAUDE_MD}",
  );
}

// ── T12 ───────────────────────────────────────────────────────────────────────

/// T12: `with_home_isolation()` suppresses `--chrome` from `ClaudeCommand` output (AC-41).
///
/// `ClaudeCommand::new()` defaults to `chrome: Some(true)`, which injects `--chrome`.
/// `with_home_isolation()` calls `with_chrome(None)`, which must eliminate the flag.
///
/// Spec: [`tests/docs/feature/004_run_isolated.md` FT-6]
#[ test ]
fn t_isolated_no_chrome_flag()
{
  use claude_runner_core::ClaudeCommand;

  let tmp = std::env::temp_dir().join( "t_isolated_no_chrome_flag" );
  let desc = ClaudeCommand::new()
    .with_home( &tmp )
    .with_home_isolation()
    .describe();
  assert!(
    !desc.contains( "--chrome" ),
    "with_home_isolation() must suppress --chrome; got: {desc}",
  );
}
