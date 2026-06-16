//! Process scanner: enumerate running Claude Code instances via `/proc`.
//!
//! Reads `/proc/{pid}/cmdline` for every numeric entry in `/proc`, selects
//! entries whose basename is exactly `"claude"`, and excludes the current
//! process.  All I/O errors are silently ignored (`.ok()`) to handle TOCTOU
//! races gracefully.
//!
//! On Linux, also provides [`ProcessMetrics`] and [`read_process_metrics`] for
//! per-process resource snapshots (CPU%, RAM, state, start time) from `/proc/{pid}/stat`
//! and `/proc/{pid}/status`.

use std::io;
use std::path::PathBuf;

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
///
/// The `CLR_PROC_DIR` environment variable overrides the proc root (default
/// `/proc`). Set it to an empty directory in tests to simulate zero sessions
/// without depending on the ambient host process table.
#[ inline ]
#[ must_use ]
pub fn find_claude_processes() -> Vec< ProcessInfo >
{
  let self_pid  = std::process::id();
  let mut result = vec![];

  let proc_root = std::env::var( "CLR_PROC_DIR" )
    .unwrap_or_else( |_| "/proc".to_string() );

  let Ok( proc_dir ) = std::fs::read_dir( &proc_root ) else { return result; };

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
    let cmdline_path = format!( "{proc_root}/{pid}/cmdline" );
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
    let cwd_path = format!( "{proc_root}/{pid}/cwd" );
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

/// Resource snapshot for a running process, read from `/proc/{pid}/stat` and
/// `/proc/{pid}/status`.
///
/// All fields are Linux-only because the data sources are entries in the Linux
/// `/proc` virtual filesystem that do not exist on other platforms.
#[ cfg( target_os = "linux" ) ]
#[ derive( Debug ) ]
pub struct ProcessMetrics
{
  /// Single-character process state from `/proc/{pid}/stat` field 3
  /// (e.g. `'R'` = running, `'S'` = sleeping, `'D'` = uninterruptible, `'Z'` = zombie).
  pub state      : char,
  /// Resident set size in kilobytes from the `VmRSS` line in `/proc/{pid}/status`.
  pub ram_kb     : u64,
  /// Lifetime-average CPU percentage: `(utime + stime) / clock_ticks_per_sec / uptime_secs * 100`.
  ///
  /// This is NOT a point-in-time utilisation — it is the fraction of one CPU core consumed
  /// over the process lifetime.  A long-running sleeping process will trend toward 0%.
  pub cpu_pct    : f32,
  /// Approximate Unix timestamp (seconds since epoch) when the process started.
  ///
  /// Computed as `boot_epoch + starttime_jiffies / 100`, where `boot_epoch` is derived
  /// from `std::time::SystemTime::now()` minus `/proc/uptime`.
  pub started_at : u64,
}

/// Read per-process resource metrics for `pid` from `/proc/{pid}/stat` and
/// `/proc/{pid}/status`.
///
/// Returns `None` if the process does not exist or any required `/proc` entry is
/// unreadable — the most common case being a process that exited between the
/// [`find_claude_processes`] scan and this call (TOCTOU race).
///
/// The `cpu_pct` field is a lifetime average, not a point-in-time sample.
/// Clock ticks per second is assumed to be 100 (correct for all mainstream
/// `x86`/`x86_64` Linux kernels; `CONFIG_HZ=100` is the stable default).
#[ cfg( target_os = "linux" ) ]
#[ inline ]
#[ must_use ]
#[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
pub fn read_process_metrics( pid : u32 ) -> Option< ProcessMetrics >
{
  // --- /proc/{pid}/stat -------------------------------------------------------
  // Format: `pid (comm) state ppid pgrp session tty_nr tpgid flags
  //          minflt cminflt majflt cmajflt utime stime cutime cstime
  //          priority nice num_threads itrealvalue starttime ...`
  // The `comm` field (field 2) is enclosed in parentheses and may contain spaces.
  // We find the closing ')' to locate field 3 and beyond unambiguously.
  let stat        = std::fs::read_to_string( format!( "/proc/{pid}/stat" ) ).ok()?;
  let after_comm  = stat.find( ") " )?;
  let rest        = stat[ after_comm + 2 .. ].trim_start();
  let mut fields  = rest.split_whitespace();

  let state     = fields.next()?.chars().next().unwrap_or( '?' );
  // Skip fields 4–13 (ppid, pgrp, session, tty_nr, tpgid, flags,
  //                    minflt, cminflt, majflt, cmajflt) — 10 fields.
  for _ in 0..10 { fields.next()?; }
  let utime     : u64 = fields.next()?.parse().ok()?; // field 14
  let stime     : u64 = fields.next()?.parse().ok()?; // field 15
  // Skip fields 16–21 (cutime, cstime, priority, nice, num_threads, itrealvalue) — 6 fields.
  for _ in 0..6 { fields.next()?; }
  let starttime : u64 = fields.next()?.parse().ok()?; // field 22

  // --- /proc/uptime -----------------------------------------------------------
  let uptime_raw  = std::fs::read_to_string( "/proc/uptime" ).ok()?;
  let uptime_secs : f64 = uptime_raw.split_whitespace().next()?.parse().ok()?;

  // --- /proc/{pid}/status -----------------------------------------------------
  let status = std::fs::read_to_string( format!( "/proc/{pid}/status" ) ).ok()?;
  let mut ram_kb = 0_u64;
  for line in status.lines()
  {
    if let Some( rest ) = line.strip_prefix( "VmRSS:" )
    {
      ram_kb = rest.split_whitespace().next().and_then( | v | v.parse().ok() ).unwrap_or( 0 );
      break;
    }
  }

  // --- Derived fields ---------------------------------------------------------
  // Clock ticks per second on x86/x86_64 Linux (CONFIG_HZ=100, stable for 15+ years).
  // We cannot call libc::sysconf without `unsafe`, which the workspace forbids.
  let hz : f64        = 100.0;
  let cpu_total_secs  = ( utime + stime ) as f64 / hz;
  let cpu_pct         = if uptime_secs > 0.0
  {
    ( cpu_total_secs / uptime_secs * 100.0 ) as f32
  }
  else
  {
    0.0_f32
  };

  let current_unix : u64 = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, | d | d.as_secs() );
  let boot_epoch = current_unix.saturating_sub( uptime_secs as u64 );
  let started_at = boot_epoch.saturating_add( starttime / hz as u64 );

  Some( ProcessMetrics { state, ram_kb, cpu_pct, started_at } )
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
