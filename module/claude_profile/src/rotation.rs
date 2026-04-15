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
//! | `~/.claude/.rotation.pid` | PID of the running daemon; removed on clean exit |
//! | `~/.claude/rotation.log`  | Append-only daemon log with `[unix_ts]`-prefixed rotation events |
//!
//! # Public surface
//!
//! - [`rotation_run`] — called by `lib.rs` when the binary is re-invoked with
//!   `--bg-rotation-daemon`; runs the daemon loop and never returns normally
//! - [`credentials_enable_auto_rotation_routine`] — `.credentials.rotation.start` handler
//! - [`credentials_disable_auto_rotation_routine`] — `.credentials.rotation.stop` handler
//! - [`credentials_rotation_status_routine`] — `.credentials.rotation.status` handler

use std::convert::Infallible;
use std::fs::{ self, OpenOptions };
use std::io::{ BufWriter, Write };
use std::path::{ Path, PathBuf };
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use claude_profile_core::account::auto_rotate;
use error_tools::Context;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

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
  if pid_path.exists()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "PID file '{}' already exists. Is the auto-rotation routine already running?", pid_path.display() ),
    ) );
  }

  fs::write( pid_path, std::process::id().to_string() )
    .context( format!( "failed to write PID file at {}", pid_path.display() ) )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to write PID file at '{}': {e}", pid_path.display() ),
    ) )
}

/// Returns `true` if `kill -0 <pid>` succeeds — process exists and is signal-able.
fn pid_is_alive( pid : u32 ) -> bool
{
  std::process::Command::new( "kill" )
    .args( [ "-0", &pid.to_string() ] )
    .stdin( Stdio::null() )
    .stdout( Stdio::null() )
    .stderr( Stdio::null() )
    .status()
    .map( | s | s.success() )
    .unwrap_or( false )
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
  let paths      = require_claude_paths()?;
  let pid_path   = paths.rotation_pid_file();
  let log_path   = paths.rotation_log_file();
  let creds_path = require_active_credentials( &paths )?;

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

fn rotation_background_spawn() -> Result< (), ErrorData >
{
  let paths    = require_claude_paths()?;
  let pid_path = paths.rotation_pid_file();

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

    if pid_is_alive( pid )
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

  std::process::Command::new(
    std::env::current_exe().map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "Failed to get current executable path: {e}" ),
    ) )?
  )
    .arg( "--bg-rotation-daemon" )
    .stdin( Stdio::null() )
    .stdout( Stdio::null() )
    .stderr( Stdio::null() )
    .spawn()
    .context( "Failed to spawn background process" )
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "Failed to spawn background process: {e}" ),
    ) )?;

  println!( "Started in background" );
  Ok( () )
}

fn rotation_stop( paths : &ClaudePaths ) -> Result< String, ErrorData >
{
  let pid_path = paths.rotation_pid_file();

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

  // Verify the process is alive via /proc/{pid} before signalling.
  if !Path::new( &format!( "/proc/{pid}" ) ).exists()
  {
    let _ = fs::remove_file( &pid_path );
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "process {pid} is not running (stale PID file removed)" ),
    ) );
  }

  println!( "Stopping rotation daemon (pid {pid})..." );

  let status = std::process::Command::new( "kill" )
    .args( [ "-TERM", &pid.to_string() ] )
    .status()
    .map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to invoke kill(1): {e}" ),
    ) )?;

  if !status.success()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "kill -TERM {pid} exited with status {status}" ),
    ) );
  }

  fs::remove_file( &pid_path ).map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "SIGTERM sent but failed to remove PID file '{}': {e}", pid_path.display() ),
  ) )?;

  Ok( format!( "Sent SIGTERM to rotation daemon (pid {pid}); PID file removed.\n" ) )
}

fn rotation_status( paths : &ClaudePaths ) -> Result< String, ErrorData >
{
  let pid_path = paths.rotation_pid_file();
  let log_path = paths.rotation_log_file();

  // ── Daemon liveness ───────────────────────────────────────────────────────
  let ( running, pid_display ) = if pid_path.exists()
  {
    let raw = fs::read_to_string( &pid_path ).unwrap_or_default();
    let pid : u32 = raw.trim().parse().unwrap_or( 0 );
    let alive = pid > 0 && Path::new( &format!( "/proc/{pid}" ) ).exists();
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

// ── Command handlers ──────────────────────────────────────────────────────────

/// `.credentials.rotation.start` — spawn the background auto-rotation daemon.
///
/// Refuses if a live daemon is already running. Removes a stale PID file and
/// re-spawns if the recorded process is no longer alive.
///
/// # Errors
///
/// Returns `ErrorData` if the daemon is already running or if spawning fails.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn credentials_enable_auto_rotation_routine(
  _cmd : VerifiedCommand,
  _ctx : ExecutionContext,
) -> Result< OutputData, ErrorData >
{
  rotation_background_spawn()?;
  Ok( OutputData::new( "Credentials auto-rotation enabled.\n", "text" ) )
}

/// `.credentials.rotation.stop` — send SIGTERM to the daemon identified by the PID file.
///
/// Verifies the process is alive before signalling. Removes a stale PID file and
/// returns an error if the recorded process no longer exists.
///
/// # Errors
///
/// Returns `ErrorData` if the PID file is missing, unreadable, contains an invalid
/// value, the process is not alive, or `kill(2)` fails.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn credentials_disable_auto_rotation_routine(
  _cmd : VerifiedCommand,
  _ctx : ExecutionContext,
) -> Result< OutputData, ErrorData >
{
  let paths = require_claude_paths()?;
  let msg   = rotation_stop( &paths )?;
  Ok( OutputData::new( msg, "text" ) )
}

/// `.credentials.rotation.status` — show whether the daemon is running and when it last rotated.
///
/// Liveness is determined by checking `/proc/{pid}` against the PID in
/// `~/.claude/.rotation.pid`. Last-rotation time is parsed from the most recent
/// `[unix_secs] Auto-rotation triggered` line in `~/.claude/rotation.log`.
///
/// # Errors
///
/// Only returns `ErrorData` on internal I/O failures; a stopped daemon or a missing log
/// are reported as human-readable text, not errors.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn credentials_rotation_status_routine(
  _cmd : VerifiedCommand,
  _ctx : ExecutionContext,
) -> Result< OutputData, ErrorData >
{
  let paths = require_claude_paths()?;
  let msg   = rotation_status( &paths )?;
  Ok( OutputData::new( msg, "text" ) )
}
