//! Isolated subprocess runner for credential-safe Claude execution.
//!
//! Provides `run_isolated()` (under `enabled` feature) which spawns the Claude binary
//! with an isolated `HOME` directory containing only the supplied credentials. The caller
//! receives an `IsolatedRunResult` indicating what the subprocess produced and whether
//! credentials were refreshed.
//!
//! ## Feature Gate Summary
//!
//! | Item               | Requires `enabled` |
//! |--------------------|--------------------|
//! | `IsolatedRunResult`| No                 |
//! | `RunnerError`      | No                 |
//! | `run_isolated()`   | Yes                |

use core::fmt;

// ── Public types ─────────────────────────────────────────────────────────────

/// Result of an isolated Claude subprocess invocation.
///
/// All four fields are `pub` to support direct struct construction in tests.
#[ derive( Debug ) ]
pub struct IsolatedRunResult
{
  /// Process exit code; `-1` if the process was terminated without an exit code.
  pub exit_code   : i32,
  /// Captured standard output from the subprocess.
  pub stdout      : String,
  /// Captured standard error from the subprocess.
  pub stderr      : String,
  /// Updated credentials JSON if the subprocess changed the credentials file;
  /// `None` if the file was byte-identical to the input or could not be read.
  pub credentials : Option< String >,
}

/// Errors that `run_isolated()` can return.
#[ derive( Debug ) ]
pub enum RunnerError
{
  /// The `claude` binary was not found in `PATH`.
  ClaudeNotFound,
  /// Creating the isolated temp directory failed.
  TempDirFailed( String ),
  /// The subprocess did not complete within `secs` seconds.
  Timeout
  {
    /// The timeout limit that was exceeded.
    secs : u64,
  },
  /// An I/O error occurred (file write, read, or cleanup).
  Io( String ),
}

impl fmt::Display for RunnerError
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      RunnerError::ClaudeNotFound => write!( f, "claude binary not found in PATH" ),
      RunnerError::TempDirFailed( reason ) =>
        write!( f, "failed to create temp dir: {reason}" ),
      RunnerError::Timeout { secs } =>
        write!( f, "claude timed out after {secs} seconds" ),
      RunnerError::Io( reason ) =>
        write!( f, "{reason}" ),
    }
  }
}

impl core::error::Error for RunnerError {}

// ── run_isolated ─────────────────────────────────────────────────────────────

/// Spawn Claude in an isolated `HOME` and return the result.
///
/// Creates a temporary directory containing only `credentials_json` written to
/// `<temp>/.claude/.credentials.json`, then invokes the Claude binary via the
/// existing `ClaudeCommand` infrastructure with `HOME=<temp>`. A background
/// thread drives the subprocess; the caller blocks for at most `timeout_secs`
/// seconds.
///
/// The temp directory is removed unconditionally after execution or timeout.
///
/// # Errors
///
/// - `RunnerError::TempDirFailed` if the temp directory cannot be created.
/// - `RunnerError::Io` if writing credentials or cleanup fails critically.
/// - `RunnerError::ClaudeNotFound` if the `claude` binary is absent from `PATH`.
/// - `RunnerError::Timeout { secs }` if the subprocess exceeds `timeout_secs`.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
pub fn run_isolated
(
  credentials_json : &str,
  args             : Vec< String >,
  timeout_secs     : u64,
) -> Result< IsolatedRunResult, RunnerError >
{
  use std::sync::mpsc;
  use core::time::Duration;

  // Step 1: Create isolated temp HOME containing only .claude/
  let temp_dir  = std::env::temp_dir()
    .join( format!( "claude_isolated_{}", std::process::id() ) );
  let claude_dir = temp_dir.join( ".claude" );
  std::fs::create_dir_all( &claude_dir )
    .map_err( |e| RunnerError::TempDirFailed( e.to_string() ) )?;

  // Step 2: Write caller-supplied credentials to the path claude reads
  let creds_path = claude_dir.join( ".credentials.json" );
  std::fs::write( &creds_path, credentials_json )
    .map_err( |e| RunnerError::Io( e.to_string() ) )?;

  // Step 3: Build command — single execution point via ClaudeCommand::execute()
  let cmd = crate::ClaudeCommand::new()
    .with_home( &temp_dir )
    .with_args( args );

  // Step 4: Spawn thread; subprocess result arrives via channel
  let ( tx, rx ) = mpsc::channel();
  std::thread::spawn( move ||
  {
    let _ = tx.send( cmd.execute() );
  } );

  // Step 5: Wait up to timeout_secs for the subprocess to finish
  let exec_result = rx.recv_timeout( Duration::from_secs( timeout_secs ) );

  // Step 6: Read credentials unconditionally (before cleanup — order matters)
  let credentials = std::fs::read_to_string( &creds_path )
    .ok()
    .and_then( |new|
    {
      if new.as_bytes() == credentials_json.as_bytes() { None } else { Some( new ) }
    } );

  // Step 7: Unconditional cleanup — no early return may appear before this line
  let _ = std::fs::remove_dir_all( &temp_dir );

  // Step 8: Translate execution result into IsolatedRunResult or RunnerError
  match exec_result
  {
    Err( _ ) => Err( RunnerError::Timeout { secs : timeout_secs } ),

    Ok( Err( e ) ) =>
    {
      let msg = e.to_string();
      if msg.contains( "not found" ) || msg.contains( "No such file" ) || msg.contains( "cannot find" )
      {
        Err( RunnerError::ClaudeNotFound )
      }
      else
      {
        Err( RunnerError::Io( msg ) )
      }
    }

    Ok( Ok( output ) ) => Ok( IsolatedRunResult
    {
      exit_code   : output.exit_code,
      stdout      : output.stdout,
      stderr      : output.stderr,
      credentials,
    } ),
  }
}
