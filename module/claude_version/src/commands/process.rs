//! `.ps` and `.ps.kill` — list and terminate Claude Code processes.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use crate::output::{ OutputFormat, OutputOptions, json_escape, trim_trailing_whitespace };
use claude_runner_core::process::{ ProcessInfo, find_claude_processes, send_sigkill, send_sigterm };
use claude_runner_core::ps_table::render_ps_table;
use claude_runner_core::OutputFormat as CoreOutputFormat;

/// `.ps` — list all running Claude Code processes.
///
/// # Errors
///
/// Returns `Err` if `format::` has an unrecognised value, or if `v::` is
/// outside the valid `0..=2` range.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn ps_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts = OutputOptions::from_cmd( &cmd )?;
  if opts.verbosity > 2
  {
    return Err( ErrorData::new( ErrorCode::ValidationRuleFailed,
      format!( "v:: must be 0, 1, or 2, got {}", opts.verbosity ) ) );
  }
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
      // Task 463: table rendering delegated to claude_runner_core::ps_table,
      // which knows nothing about CLI-specific trailing-newline conventions —
      // normalize exactly one trailing newline here, at the CLI output layer.
      let rendered = render_ps_table( &procs, CoreOutputFormat::Text, opts.verbosity );
      let mut content = trim_trailing_whitespace( &rendered );
      if !content.ends_with( '\n' ) { content.push( '\n' ); }
      content
    }
  };

  Ok( OutputData::new( content, "text" ) )
}

/// Deliver SIGTERM+SIGKILL (with 2 s wait) or bare SIGKILL to a process list.
///
/// Called only from `ps_kill_routine` (`.ps.kill` explicit user command).
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

/// Parse and validate the optional `pid::` argument for `.ps.kill`.
///
/// Returns `Ok(None)` when `pid::` is absent (bulk-kill mode targeting every
/// running Claude Code process). Returns `Err(ValidationRuleFailed)` when
/// `pid::` is present but not a positive integer that fits in a process id.
fn parse_pid_filter( cmd : &VerifiedCommand ) -> Result< Option< u32 >, ErrorData >
{
  match cmd.arguments.get( "pid" )
  {
    Some( Value::Integer( n ) ) =>
    {
      if *n <= 0
      {
        return Err( ErrorData::new( ErrorCode::ValidationRuleFailed,
          format!( "pid:: must be a positive integer, got {n}" ) ) );
      }
      u32::try_from( *n )
      .map( Some )
      .map_err( | _ | ErrorData::new( ErrorCode::ValidationRuleFailed,
        format!( "pid:: value {n} is out of range for a process id" ) ) )
    }
    _ => Ok( None ),
  }
}

/// `.ps.kill` — terminate all Claude Code processes, or a single process when
/// `pid::` is given.
///
/// Handler for explicit user command `.ps.kill`. Never invoke from automatic paths.
///
/// # Errors
///
/// Returns `Err(ValidationRuleFailed)` if `pid::` is not a positive integer or
/// does not match any running Claude Code process.
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
pub fn ps_kill_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts       = OutputOptions::from_cmd( &cmd )?;
  let pid_filter = parse_pid_filter( &cmd )?;
  let all_procs  = find_claude_processes();

  let procs = match pid_filter
  {
    Some( target ) =>
    {
      let matched : Vec< ProcessInfo > = all_procs.into_iter().filter( | p | p.pid == target ).collect();
      if matched.is_empty()
      {
        return Err( ErrorData::new( ErrorCode::ValidationRuleFailed,
          format!( "no running Claude Code process with pid {target}" ) ) );
      }
      matched
    }
    None => all_procs,
  };

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
  // Targeted kill: only the requested pid must be gone — unrelated Claude
  // processes left running are expected, not a failure. Bulk kill: every
  // process must be gone.
  let remaining = match pid_filter
  {
    Some( target ) => usize::from( find_claude_processes().iter().any( | p | p.pid == target ) ),
    None            => find_claude_processes().len(),
  };
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
