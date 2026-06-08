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
//! | `IsolatedRunResult`     | No                 |
//! | `RunnerError`           | No                 |
//! | `IsolatedModel`         | No                 |
//! | `ISOLATED_DEFAULT_MODEL`| No                 |
//! | `run_isolated()`        | Yes                |

use core::fmt;

// ── Public types ─────────────────────────────────────────────────────────────

/// Default model ID used by [`IsolatedModel::Default`] for real user tasks.
pub const ISOLATED_DEFAULT_MODEL : &str = "claude-opus-4-6";

/// Default model ID for OAuth credential-refresh pings (trivial `"."` prompt).
pub const REFRESH_DEFAULT_MODEL : &str = "claude-sonnet-4-6";

/// Minimal instructions written to `<temp>/.claude/CLAUDE.md` by `run_isolated()`.
///
/// Directs the subprocess to execute immediately without interactive behavior:
/// no clarifying questions, no confirmation, no narration, no preamble (AC-42).
pub const ISOLATED_CLAUDE_MD : &str = "# Isolated subprocess\n\n\
    Execute the given task immediately and exit.\n\n\
    - Do not ask clarifying questions \u{2014} act on the message as given.\n\
    - Do not request human confirmation for any operation.\n\
    - Do not explain your reasoning or narrate your steps.\n\
    - Output only the direct result of the task; no preamble, no summary.\n\
    - If the input is a single character or whitespace only, reply with a single period.\n";

/// Claude model selection for isolated subprocess invocations.
///
/// Controls whether `--model <id>` is prepended to the subprocess argument list.
/// The `Default` variant targets the current production Opus (highest capability)
/// for real user tasks; callers that want the Claude binary to use whatever model
/// it would normally select should pass `KeepCurrent`.
#[ derive( Debug, Clone ) ]
pub enum IsolatedModel
{
  /// Prepend `--model claude-opus-4-6` to subprocess args.
  Default,
  /// Pass no `--model` flag; the Claude binary chooses the model.
  KeepCurrent,
  /// Prepend `--model <id>` to subprocess args.
  Specific( String ),
}

impl IsolatedModel
{
  /// Returns the model ID to inject via `--model`, or `None` for `KeepCurrent`.
  #[ inline ]
  #[ must_use ]
  pub fn model_id( &self ) -> Option< &str >
  {
    match self
    {
      IsolatedModel::Default        => Some( ISOLATED_DEFAULT_MODEL ),
      IsolatedModel::KeepCurrent    => None,
      IsolatedModel::Specific( id ) => Some( id.as_str() ),
    }
  }
}

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
  /// The subprocess did not complete within `secs` seconds (no stdout buffered).
  Timeout
  {
    /// The timeout limit that was exceeded.
    secs : u64,
  },
  /// The subprocess did not complete within `secs` seconds; partial stdout captured.
  ///
  /// Fix(BUG-243): the old `Timeout` variant discarded all buffered subprocess output.
  /// Root cause: the thread/channel approach lost the `Child` handle on timeout, making
  ///   `wait_with_output()` unreachable; all partial output was silently dropped.
  /// Pitfall: always use `spawn_piped()` + polling so the `Child` handle stays in scope
  ///   through the timeout; then `child.kill()` + `child.wait_with_output()` recovers data.
  TimeoutWithOutput
  {
    /// The timeout limit that was exceeded.
    secs           : u64,
    /// Partial stdout emitted by the subprocess before it was killed.
    partial_stdout : String,
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
      RunnerError::TimeoutWithOutput { secs, partial_stdout } =>
      {
        if partial_stdout.is_empty()
        {
          write!( f, "claude timed out after {secs} seconds (no output captured)" )
        }
        else
        {
          write!( f, "claude timed out after {secs} seconds; partial output:\n{partial_stdout}" )
        }
      }
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
/// If `model` is not `IsolatedModel::KeepCurrent`, `--model <id>` is prepended
/// to `args` before the subprocess is spawned.
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
#[ allow( clippy::too_many_lines ) ]
pub fn run_isolated
(
  credentials_json : &str,
  args             : Vec< String >,
  timeout_secs     : u64,
  model            : IsolatedModel,
) -> Result< IsolatedRunResult, RunnerError >
{
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

  // Step 2a: Write CLAUDE.md to isolated HOME before spawn.
  //
  // Without user-level behavioral instructions the subprocess may ask clarifying
  // questions, request confirmation, or produce verbose narration — all of which
  // block the subprocess permanently in non-interactive print mode.
  std::fs::write( claude_dir.join( "CLAUDE.md" ), ISOLATED_CLAUDE_MD )
    .map_err( |e| RunnerError::Io( e.to_string() ) )?;

  // Step 3: Build command — prepend --model flag then user args
  let mut full_args = Vec::with_capacity( args.len() + 2 );
  if let Some( id ) = model.model_id()
  {
    full_args.push( "--model".to_string() );
    full_args.push( id.to_string() );
  }
  full_args.extend( args );
  let cmd = crate::ClaudeCommand::new()
    .with_home( &temp_dir )
    .with_home_isolation()
    .with_args( full_args );

  // Step 4: Spawn subprocess with piped I/O so we keep the Child handle.
  //
  // Fix(BUG-243): use spawn_piped() + try_wait polling instead of the old
  //   thread/channel approach. The thread approach buried the Child inside the
  //   spawned thread; on recv_timeout the subprocess kept running as an orphan
  //   and all accumulated stdout was irrecoverably discarded.
  // Root cause: cmd.execute() (called inside the thread) calls cmd.output() which
  //   blocks until EOF; the main thread's recv_timeout fired before that, leaving
  //   the thread running with no way to kill or read the child.
  // Pitfall: always keep the Child handle in scope through the timeout so that
  //   child.kill() + child.wait_with_output() can recover buffered data.
  let mut child = cmd.spawn_piped().map_err( |e|
  {
    if e.kind() == std::io::ErrorKind::NotFound
    {
      RunnerError::ClaudeNotFound
    }
    else
    {
      RunnerError::Io( e.to_string() )
    }
  } )?;

  // Step 5: Poll for completion with a 50 ms tick up to the deadline.
  //
  // Fix(I2): when timeout_secs == 0, skip the deadline entirely (no watchdog),
  //   matching run/ask semantics where 0 means unlimited.  Previously 0 computed
  //   a deadline of Instant::now() + 0s, which fired on the very first poll tick
  //   and killed the subprocess immediately — making unlimited timeout impossible.
  let deadline : Option< std::time::Instant > = if timeout_secs > 0
  {
    Some( std::time::Instant::now() + Duration::from_secs( timeout_secs ) )
  }
  else
  {
    None
  };
  let mut timed_out = false;
  let subprocess_output : Result< std::process::Output, RunnerError > = loop
  {
    match child.try_wait()
    {
      Ok( Some( _ ) ) =>
      {
        // Subprocess exited — collect full stdout/stderr.
        break child.wait_with_output()
          .map_err( |e| RunnerError::Io( e.to_string() ) );
      }
      Ok( None ) =>
      {
        if deadline.is_some_and( |d| std::time::Instant::now() >= d )
        {
          // Timeout: kill the subprocess and collect whatever was buffered.
          timed_out = true;
          let _ = child.kill();
          break child.wait_with_output()
            .map_err( |e| RunnerError::Io( e.to_string() ) );
        }
        std::thread::sleep( Duration::from_millis( 50 ) );
      }
      Err( e ) => break Err( RunnerError::Io( e.to_string() ) ),
    }
  };

  // Step 6: Read credentials unconditionally (before cleanup — order matters).
  //
  // Fix(issue-isolated-credentials-on-timeout): when the subprocess times out but
  // already refreshed credentials (e.g. OAuth token refresh at startup before
  // waiting for interactive input), return Ok so callers can access the new creds.
  // Root cause: Claude refreshes the OAuth token at startup then waits for input;
  //             previously the timeout fired and discarded the refreshed credentials,
  //             breaking the refresh::1 retry path in usage_routine().
  // Pitfall: check credentials BEFORE deciding to return Err(Timeout) — the token
  //          may have been written before the subprocess started blocking.
  let credentials = std::fs::read_to_string( &creds_path )
    .ok()
    .and_then( |new|
    {
      if new.as_bytes() == credentials_json.as_bytes() { None } else { Some( new ) }
    } );

  // Step 7: Unconditional cleanup — no early return may appear before this line.
  let _ = std::fs::remove_dir_all( &temp_dir );

  // Step 8: Translate execution result into IsolatedRunResult or RunnerError.
  let output = subprocess_output?;
  let stdout = String::from_utf8_lossy( &output.stdout ).to_string();
  let stderr = String::from_utf8_lossy( &output.stderr ).to_string();

  if timed_out
  {
    // If credentials were updated during the timeout window, preserve them.
    if credentials.is_some()
    {
      return Ok( IsolatedRunResult
      {
        exit_code   : -1,
        stdout      : String::new(),
        stderr      : String::new(),
        credentials,
      } );
    }
    return Err( RunnerError::TimeoutWithOutput
    {
      secs           : timeout_secs,
      partial_stdout : stdout,
    } );
  }

  let exit_code = crate::signal_exit_code( &output.status );
  Ok( IsolatedRunResult { exit_code, stdout, stderr, credentials } )
}
