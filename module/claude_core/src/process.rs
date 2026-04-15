//! Process scanner: enumerate running Claude Code instances via `/proc`.
//!
//! Reads `/proc/{pid}/cmdline` for every numeric entry in `/proc`, selects
//! entries whose basename is exactly `"claude"`, and excludes the current
//! process.  All I/O errors are silently ignored (`.ok()`) to handle TOCTOU
//! races gracefully.

use std::io;
use std::path::PathBuf;
use std::process::Stdio;

/// Information about a running Claude Code process.
#[ derive( Debug ) ]
pub struct ProcessInfo
{
  /// The process identifier.
  pub pid     : u32,
  /// Full cmdline string (NUL bytes replaced with spaces).
  pub cmdline : String,
  /// Working directory of the process (empty on error or deleted CWD).
  pub cwd     : PathBuf,
}

/// Scan `/proc` for Claude Code processes, returning one `ProcessInfo` per match.
///
/// Entries whose cmdline basename is not exactly `"claude"` are skipped.
/// The current process is always excluded.
/// All I/O errors are silently ignored.
#[ inline ]
#[ must_use ]
pub fn find_claude_processes() -> Vec< ProcessInfo >
{
  let self_pid = std::process::id();
  let mut result = vec![];

  let Ok( proc_dir ) = std::fs::read_dir( "/proc" ) else { return result; };

  for entry in proc_dir
  {
    let Ok( entry ) = entry else { continue; };
    let name     = entry.file_name();
    let name_str = name.to_string_lossy();

    // Only numeric entries (PIDs).
    let Ok( pid ) : Result< u32, _ > = name_str.parse() else { continue; };

    // Exclude self.
    if pid == self_pid { continue; }

    // Read cmdline (NUL-delimited).
    let cmdline_path = format!( "/proc/{pid}/cmdline" );
    let Ok( cmdline_raw ) = std::fs::read( &cmdline_path ) else { continue; };

    // First NUL-delimited field is the executable path.
    let first_field = cmdline_raw.split( | &b | b == 0 ).next().unwrap_or( &[] );
    let binary_path = core::str::from_utf8( first_field ).unwrap_or( "" );
    let binary_name = std::path::Path::new( binary_path )
    .file_name()
    .and_then( | s | s.to_str() )
    .unwrap_or( "" );

    if binary_name != "claude" { continue; }

    // Read CWD (may fail if deleted or unreadable).
    let cwd_path = format!( "/proc/{pid}/cwd" );
    let cwd = std::fs::read_link( &cwd_path ).unwrap_or_default();

    // Build human-readable cmdline (NUL → space).
    let cmdline = cmdline_raw
    .iter()
    .map( | &b | if b == 0 { b' ' } else { b } )
    .collect::< Vec< u8 > >();
    let cmdline = String::from_utf8_lossy( &cmdline ).trim_end().to_string();

    result.push( ProcessInfo { pid, cmdline, cwd } );
  }

  result
}

/// Send `SIGTERM` to the process with the given PID.
///
/// Invokes `kill -TERM {pid}` as a subprocess.
///
/// # Errors
///
/// Returns `Err` if `kill` could not be executed or if it exits non-zero.
#[ inline ]
pub fn send_sigterm( pid : u32 ) -> Result< (), io::Error >
{
  run_kill( &[ "-TERM", &pid.to_string() ] )
}

/// Send `SIGKILL` to the process with the given PID.
///
/// Invokes `kill -KILL {pid}` as a subprocess.
///
/// # Errors
///
/// Returns `Err` if `kill` could not be executed or if it exits non-zero.
#[ inline ]
pub fn send_sigkill( pid : u32 ) -> Result< (), io::Error >
{
  run_kill( &[ "-KILL", &pid.to_string() ] )
}

/// Returns the PID of the current process.
#[ inline ]
#[ must_use ]
pub fn current_pid() -> u32
{
  std::process::id()
}

/// Returns `true` if the process with the given PID exists and is signal-able
/// (equivalent to `kill -0 <pid>`).
///
/// Uses `kill -0` as a subprocess. Returns `false` on any error, including
/// permission errors and process-not-found.
#[ inline ]
#[ must_use ]
pub fn process_is_alive( pid : u32 ) -> bool
{
  run_kill( &[ "-0", &pid.to_string() ] ).is_ok()
}

/// Spawns the current executable as a detached background process with one argument.
///
/// All stdio streams are redirected to null so the child is fully detached.
/// Returns immediately after the OS accepts the spawn; does not wait for exit.
///
/// # Errors
///
/// Returns `Err` if [`std::env::current_exe`] fails or if spawning fails.
#[ inline ]
pub fn spawn_background_self( arg : &str ) -> Result< (), io::Error >
{
  let exe = std::env::current_exe()
    .map_err( | e | io::Error::other( e.to_string() ) )?;

  std::process::Command::new( exe )
    .arg( arg )
    .stdin( Stdio::null() )
    .stdout( Stdio::null() )
    .stderr( Stdio::null() )
    .spawn()?;

  Ok( () )
}

// `io::Error::other()` is required here; `io::Error::new(ErrorKind::Other, …)` is
// rejected by the `io_other_error` clippy lint (Rust 1.74+, -D warnings).
fn run_kill( args : &[ &str ] ) -> Result< (), io::Error >
{
  let status = std::process::Command::new( "kill" )
  .args( args )
  .status()
  .map_err( | e | io::Error::other( e.to_string() ) )?;

  if status.success()
  {
    Ok( () )
  }
  else
  {
    Err( io::Error::other(
      format!( "kill {} exited with: {status}", args.join( " " ) ),
    ) )
  }
}
