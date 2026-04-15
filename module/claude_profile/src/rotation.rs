//! Background credential auto-rotation daemon.
//!
//! Provides start / stop / status operations for a long-running background
//! process that monitors the 5-hour rate-limit utilization and calls
//! [`auto_rotate`](claude_profile_core::account::auto_rotate) when it reaches ≥ 90%.
//!
//! # Files on disk
//!
//! | Path | Purpose |
//! |------|---------|
//! | `~/.claude/.transient/.rotation.pid` | PID of the running daemon; removed on clean exit |
//! | `~/.claude/.transient/rotation.log`  | Append-only daemon log with `[unix_ts]`-prefixed rotation events |
//!
//! # Public surface
//!
//! - [`rotation_run`] — called by `lib.rs` when the binary is re-invoked with
//!   `--bg-rotation-daemon`; runs the daemon loop and never returns normally
//! - [`rotation_background_spawn`] / [`rotation_stop`] / [`rotation_status`] — called by
//!   the command handlers in `commands.rs`

use std::convert::Infallible;
use std::fs::{ self, OpenOptions };
use std::io::{ BufWriter, Write };
use std::path::{ Path, PathBuf };
use std::thread;
use std::time::Duration;
use claude_profile_core::account::auto_rotate;
use claude_runner_core::process;
use unilang::data::{ ErrorCode, ErrorData };

use crate::ClaudePaths;
use crate::commands::{ fetch_rate_limits, require_active_credentials, require_claude_paths };
use crate::output::format_duration_secs;

// ── PID file helpers ──────────────────────────────────────────────────────────

/// RAII guard that removes the PID file when the daemon exits for any reason.
struct PidGuard( PathBuf );

impl Drop for PidGuard
{
  fn drop( &mut self )
  {
    let _ = fs::remove_file( &self.0 );
  }
}

fn pid_file_create( pid_path : &Path ) -> Result< (), ErrorData >
{
  use std::io::Write as _;

  let mut file = OpenOptions::new()
    .write( true )
    .create_new( true )
    .open( pid_path )
    .map_err( |e|
    {
      if e.kind() == std::io::ErrorKind::AlreadyExists
      {
        ErrorData::new(
          ErrorCode::InternalError,
          format!( "PID file '{}' already exists. Is the auto-rotation routine already running?", pid_path.display() ),
        )
      }
      else
      {
        ErrorData::new(
          ErrorCode::InternalError,
          format!( "failed to create PID file '{}': {e}", pid_path.display() ),
        )
      }
    } )?;

  write!( file, "{}", process::current_pid() )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to write PID file '{}': {e}", pid_path.display() ),
    ) )
}

// ── Timestamp helper ──────────────────────────────────────────────────────────

fn unix_secs() -> u64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs()
}

// ── Core daemon operations ────────────────────────────────────────────────────

/// Daemon loop: poll rate limits every 5 minutes and rotate when ≥ 90% utilised.
///
/// Writes a PID file on entry and removes it on exit via [`PidGuard`].
/// Returns `Err` only on startup failures (cannot open log, cannot write PID file,
/// or cannot resolve active credentials). Never returns `Ok` — the loop is infinite.
pub(crate) fn rotation_run() -> Result< Infallible, ErrorData >
{
  let paths        = require_claude_paths()?;
  let transient    = paths.base().join( ".transient" );
  let pid_path     = transient.join( ".rotation.pid" );
  let log_path     = transient.join( "rotation.log" );
  let creds_path = require_active_credentials( &paths )?;

  fs::create_dir_all( &transient )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to create transient directory '{}': {e}", transient.display() ),
    ) )?;

  let file = OpenOptions::new()
    .create( true )
    .append( true )
    .open( &log_path )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to open log file '{}': {e}", log_path.display() ),
    ) )?;

  let mut writer = BufWriter::new( file );
  writeln!( writer, "Running credential auto-rotation routine started with flag..." ).ok();
  writer.flush().ok();

  pid_file_create( &pid_path )?;
  let _guard = PidGuard( pid_path );

  loop
  {
    let limits = fetch_rate_limits( &creds_path );
    writeln!( writer, "Fetched rate limits: {limits:?}" ).ok();
    writer.flush().ok();

    match limits
    {
      Ok( limits ) =>
      {
        if limits.utilization_5h >= 0.9
        {
          match auto_rotate()
          {
            Ok( _ )  => writeln!( writer, "[{}] Auto-rotation triggered due to high utilization: {limits:?}", unix_secs() ).ok(),
            Err( e ) => writeln!( writer, "[{}] auto_rotate failed (limits: {limits:?}): {e}", unix_secs() ).ok(),
          };
          writer.flush().ok();
        }
      }
      Err( e ) =>
      {
        writeln!( writer, "Error fetching rate limits: {e}" ).ok();
        writer.flush().ok();
      }
    }

    thread::sleep( Duration::from_secs( 5 * 60 ) );
  }
}

pub(crate) fn rotation_background_spawn() -> Result< (), ErrorData >
{
  let paths    = require_claude_paths()?;
  let pid_path = paths.base().join( ".transient" ).join( ".rotation.pid" );

  if pid_path.exists()
  {
    let raw = fs::read_to_string( &pid_path ).map_err( |e|
      ErrorData::new( ErrorCode::InternalError, format!( "failed to read PID file: {e}" ) )
    )?;

    let pid : u32 = raw.trim().parse().map_err( |_|
      ErrorData::new(
        ErrorCode::InternalError,
        format!( "PID file contains invalid value: '{}'", raw.trim() ),
      )
    )?;

    if process::process_is_alive( pid )
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "rotation daemon is already running (pid {pid})" ),
      ) );
    }

    // Stale file — process is gone, remove it and proceed.
    fs::remove_file( &pid_path ).map_err( |e|
      ErrorData::new( ErrorCode::InternalError, format!( "failed to remove stale PID file: {e}" ) )
    )?;
    eprintln!( "warning: removed stale PID file (pid {pid} is no longer running)" );
  }

  process::spawn_background_self( "--bg-rotation-daemon" ).map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to spawn background daemon: {e}" ),
  ) )?;

  println!( "Started in background" );
  Ok( () )
}

pub(crate) fn rotation_stop( paths : &ClaudePaths ) -> Result< String, ErrorData >
{
  let pid_path = paths.base().join( ".transient" ).join( ".rotation.pid" );

  if !pid_path.exists()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "PID file '{}' not found — is the auto-rotation daemon running?", pid_path.display() ),
    ) );
  }

  let raw = fs::read_to_string( &pid_path ).map_err( |e|
    ErrorData::new( ErrorCode::InternalError, format!( "failed to read PID file: {e}" ) )
  )?;

  let pid : u32 = raw.trim().parse().map_err( |_|
    ErrorData::new(
      ErrorCode::InternalError,
      format!( "PID file contains invalid value: '{}'", raw.trim() ),
    )
  )?;

  if !process::process_is_alive( pid )
  {
    let _ = fs::remove_file( &pid_path );
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "process {pid} is not running (stale PID file removed)" ),
    ) );
  }

  println!( "Stopping rotation daemon (pid {pid})..." );

  process::send_sigterm( pid ).map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to send SIGTERM to pid {pid}: {e}" ),
  ) )?;

  const POLL_ATTEMPTS : u32 = 10;
  const POLL_INTERVAL : Duration = Duration::from_millis( 200 );

  let terminated = ( 0..POLL_ATTEMPTS ).any( |_|
  {
    thread::sleep( POLL_INTERVAL );
    !process::process_is_alive( pid )
  } );

  if !terminated
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "process {pid} did not terminate within {}ms after SIGTERM; PID file left in place", POLL_ATTEMPTS * POLL_INTERVAL.as_millis() as u32 ),
    ) );
  }

  fs::remove_file( &pid_path ).map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to remove PID file '{}': {e}", pid_path.display() ),
  ) )?;

  Ok( format!( "Sent SIGTERM to rotation daemon (pid {pid}); PID file removed.\n" ) )
}

pub(crate) fn rotation_status( paths : &ClaudePaths ) -> Result< String, ErrorData >
{
  let transient = paths.base().join( ".transient" );
  let pid_path  = transient.join( ".rotation.pid" );
  let log_path  = transient.join( "rotation.log" );

  // ── Daemon liveness ───────────────────────────────────────────────────────
  let ( running, pid_display ) = if pid_path.exists()
  {
    let raw = fs::read_to_string( &pid_path ).unwrap_or_default();
    let pid : u32 = raw.trim().parse().unwrap_or( 0 );
    let alive = pid > 0 && process::process_is_alive( pid );
    ( alive, if pid > 0 { format!( " (pid {pid})" ) } else { String::new() } )
  }
  else
  {
    ( false, String::new() )
  };

  // ── Last rotation timestamp ───────────────────────────────────────────────
  // Scan the log for the last "[<unix_secs>] Auto-rotation triggered" line.
  let last_rotated_str = if log_path.exists()
  {
    let content = fs::read_to_string( &log_path ).unwrap_or_default();
    let ts = content
      .lines()
      .filter( | l | l.contains( "Auto-rotation triggered" ) )
      .last()
      .and_then( |line| line.strip_prefix( '[' ) )
      .and_then( |s| s.split_once( ']' ) )
      .and_then( |( ts_str, _ )| ts_str.trim().parse::< u64 >().ok() );

    match ts
    {
      Some( t ) =>
      {
        let age = unix_secs().saturating_sub( t );
        format!( "{} ago", format_duration_secs( age ) )
      }
      None => "never".to_string(),
    }
  }
  else
  {
    "never".to_string()
  };

  let status = if running
  {
    format!( "running{pid_display}" )
  }
  else
  {
    format!( "stopped{pid_display}" )
  };

  Ok( format!( "Daemon:       {status}\nLast rotated: {last_rotated_str}\n" ) )
}

