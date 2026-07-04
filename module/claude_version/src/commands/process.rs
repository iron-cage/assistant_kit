//! `.processes` and `.processes.kill` — list and terminate Claude Code processes.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_runner_core::process::{ ProcessInfo, find_claude_processes, send_sigkill, send_sigterm };

/// `.processes` — list all running Claude Code processes.
///
/// # Errors
///
/// Returns `Err` if `format::` has an unrecognised value.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn processes_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let procs = find_claude_processes();

  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      if procs.is_empty()
      {
        "{\"processes\":[]}\n".to_string()
      }
      else
      {
        let entries : Vec< String > = procs.iter().map( | p |
        {
          let cwd = json_escape( &p.cwd.to_string_lossy() );
          format!( "  {{\"pid\":{},\"cwd\":\"{cwd}\"}}", p.pid )
        } ).collect();
        format!( "{{\"processes\":[\n{}\n]}}\n", entries.join( ",\n" ) )
      }
    }
    OutputFormat::Text =>
    {
      if procs.is_empty()
      {
        String::new()
      }
      else
      {
        let lines : Vec< String > = procs.iter().map( | p |
          match opts.verbosity
          {
            0 => format!( "{} {}", p.pid, p.cwd.display() ),
            _ => format!( "PID: {}  CWD: {}", p.pid, p.cwd.display() ),
          }
        ).collect();
        format!( "{}\n", lines.join( "\n" ) )
      }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}

/// Deliver SIGTERM+SIGKILL (with 2 s wait) or bare SIGKILL to a process list.
///
/// Called only from `processes_kill_routine` (`.processes.kill` explicit user command).
/// MUST NOT be called from guard routines, install routines, or any scheduled path.
///
/// `force == true` → immediate SIGKILL; `force == false` → SIGTERM first, then
/// SIGKILL any survivors after a 2-second grace period.
fn send_kill_signals( procs : &[ ProcessInfo ], force : bool ) -> Result< (), ErrorData >
{
  if force
  {
    let mut failures = Vec::new();
    for p in procs
    {
      if let Err( e ) = send_sigkill( p.pid ) { failures.push( format!( "PID {}: {e}", p.pid ) ); }
    }
    if !failures.is_empty()
    {
      return Err( ErrorData::new( ErrorCode::InternalError, format!( "SIGKILL failed: {}", failures.join( ", " ) ) ) );
    }
  }
  else
  {
    let mut failures = Vec::new();
    for p in procs
    {
      if let Err( e ) = send_sigterm( p.pid ) { failures.push( format!( "PID {}: {e}", p.pid ) ); }
    }
    if !failures.is_empty()
    {
      return Err( ErrorData::new( ErrorCode::InternalError, format!( "SIGTERM failed: {}", failures.join( ", " ) ) ) );
    }
    std::thread::sleep( core::time::Duration::from_secs( 2 ) );
    let survivors = find_claude_processes();
    let mut kfailures = Vec::new();
    for p in &survivors
    {
      if let Err( e ) = send_sigkill( p.pid ) { kfailures.push( format!( "PID {}: {e}", p.pid ) ); }
    }
    if !kfailures.is_empty()
    {
      return Err( ErrorData::new( ErrorCode::InternalError, format!( "SIGKILL failed: {}", kfailures.join( ", " ) ) ) );
    }
  }
  Ok( () )
}

/// `.processes.kill` — terminate all Claude Code processes.
///
/// Handler for explicit user command `.processes.kill`. Never invoke from automatic paths.
///
/// # Errors
///
/// Returns `Err(InternalError)` if signal delivery fails or processes survive.
///
/// Fix(BUG-007): signal delivery results were discarded via the
///   `let _ = …` pattern on `send_sigterm`/`send_sigkill`, so EPERM and other
///   errors were silently swallowed — surviving processes only surfaced via the
///   trailing `remaining > 0` check; stale-list errors were completely invisible.
/// Root cause: discarding the `Result` from signal functions hides every error
///   that does not manifest as a surviving process in the follow-up scan.
/// Pitfall: ESRCH ("no such process") is a benign race — the process already
///   exited — so collect all signal errors but filter ESRCH from final report.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn processes_kill_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let procs = find_claude_processes();

  if procs.is_empty()
  {
    let content = match opts.format
    {
      OutputFormat::Json => "{\"killed\":0}\n".to_string(),
      // v::0 = bare count; v::1+ = labeled message.
      OutputFormat::Text =>
      {
        if opts.verbosity == 0 { "0\n".to_string() } else { "no active processes\n".to_string() }
      }
    };
    return Ok( OutputData::new( content, "text" ) );
  }

  if super::is_dry( &cmd )
  {
    let signal = if super::is_force( &cmd ) { "SIGKILL" } else { "SIGTERM" };
    let pids : Vec< String > = procs.iter().map( | p | p.pid.to_string() ).collect();
    let content = match opts.format
    {
      OutputFormat::Json =>
      {
        format!( "{{\"dry_run\":true,\"signal\":\"{signal}\",\"pids\":[{}]}}\n", pids.join( "," ) )
      }
      OutputFormat::Text =>
      {
        if opts.verbosity == 0
        {
          // v::0: bare PID list only.
          format!( "{}\n", pids.join( "\n" ) )
        }
        else
        {
          let lines : Vec< String > = procs.iter()
          .map( | p | format!( "[dry-run] would send {signal} to PID {}", p.pid ) )
          .collect();
          format!( "{}\n", lines.join( "\n" ) )
        }
      }
    };
    return Ok( OutputData::new( content, "text" ) );
  }

  let count = procs.len();

  send_kill_signals( &procs, super::is_force( &cmd ) )?;

  std::thread::sleep( core::time::Duration::from_millis( 500 ) );
  let remaining = find_claude_processes().len();
  if remaining > 0
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "killed {count} process(es) but {remaining} could not be terminated" ),
    ) );
  }

  let content = match opts.format
  {
    OutputFormat::Json => format!( "{{\"killed\":{count}}}\n" ),
    // v::0 = bare count; v::1+ = labeled message.
    OutputFormat::Text =>
    {
      if opts.verbosity == 0
      {
        format!( "{count}\n" )
      }
      else
      {
        format!( "killed {count} process(es)\n" )
      }
    }
  };
  Ok( OutputData::new( content, "text" ) )
}
