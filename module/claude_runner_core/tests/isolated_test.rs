//! Isolated subprocess runner tests: `IsolatedRunResult` and `RunnerError`.
//!
//! T01–T06 are offline (no `lim_it`) — struct construction and Display.
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
//! | T08 | `run_isolated()` with timeout 0 → `Err(Timeout)`      | `RunnerError::Timeout { secs: 0 }`   | yes   |

use claude_runner_core::{ IsolatedRunResult, RunnerError };

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

  let creds = r#"{"accessToken":"tok-test","refreshToken":"rtok-test","expiresAt":9999999999}"#;
  let result = run_isolated( creds, vec![ "--version".to_string() ], 30 );
  assert!( result.is_ok(), "run_isolated must return Ok for valid args, got: {:?}", result.err() );
  let out = result.unwrap();
  // exit_code of 0 is expected for --version; -1 is acceptable if binary returns none
  assert!( out.exit_code >= -1, "exit_code must be -1 or higher, got: {}", out.exit_code );
}

// ── T08 ───────────────────────────────────────────────────────────────────────

/// T08 (`lim_it`): `run_isolated()` with timeout 0 returns `Err(RunnerError::Timeout)`.
///
/// A 0-second timeout should expire before any subprocess can complete.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn t08_lim_it_run_isolated_timeout()
{
  use claude_runner_core::run_isolated;

  let creds = r#"{"accessToken":"tok-test","refreshToken":"rtok-test","expiresAt":9999999999}"#;
  let result = run_isolated( creds, vec![ "--version".to_string() ], 0 );
  match result
  {
    Err( RunnerError::Timeout { secs } ) =>
    {
      assert_eq!( secs, 0, "Timeout secs must match the given timeout, got: {secs}" );
    }
    other => panic!( "expected Err(Timeout), got: {other:?}" ),
  }
}
