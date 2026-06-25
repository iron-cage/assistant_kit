//! Continuous live-monitor mode for the quota table.
//!
//! `execute_live_mode` loops indefinitely until SIGINT (Ctrl-C), fetching
//! quota data and rendering a countdown footer between cycles.

use unilang::data::{ ErrorData, OutputData };
use super::types::UsageParams;
#[ cfg( unix ) ]
use super::fetch::fetch_all_quota;
#[ cfg( unix ) ]
use super::render::render_text;

// ── SIGINT plumbing ───────────────────────────────────────────────────────────

/// Shared quit flag — set to `true` by `on_sigint` on SIGINT; polled each second.
#[ cfg( unix ) ]
static STOP_FLAG : core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new( false );

/// SIGINT handler: sets `STOP_FLAG` so the countdown loop exits cleanly.
#[ cfg( unix ) ]
extern "C" fn on_sigint( _ : std::os::raw::c_int )
{
  STOP_FLAG.store( true, core::sync::atomic::Ordering::Relaxed );
}

// ── UTC clock helper ──────────────────────────────────────────────────────────

/// Format a Unix timestamp as `HH:MM:SS` in UTC (no external dep).
#[ cfg_attr( not( unix ), allow( dead_code ) ) ]
fn secs_to_hms_utc( unix_secs : u64 ) -> String
{
  let sod = unix_secs % 86400;
  let h   = sod / 3600;
  let m   = ( sod % 3600 ) / 60;
  let s   = sod % 60;
  format!( "{h:02}:{m:02}:{s:02}" )
}

// ── Live loop ─────────────────────────────────────────────────────────────────

/// Continuous quota monitor loop.
///
/// Clears the screen, fetches all accounts with per-account stagger delays,
/// renders the table, displays a countdown footer rewritten in-place each second,
/// and repeats until Ctrl-C (SIGINT) sets `STOP_FLAG`.
///
/// Timing is governed by `params.interval` (minimum seconds between cycles, ≥ 30)
/// and `params.jitter` (maximum random seconds added per cycle, 0 = none).
/// When `params.trace` is `true`, per-account timestamped diagnostic lines are emitted to stderr.
// Fix(BUG-317): Root cause: execute_live_mode called sigemptyset/sigaddset/sigprocmask
// unconditionally — POSIX symbols absent on Windows → LNK2019 at link time.
// Pitfall: `extern "C"` declarations for POSIX signal functions compile on all
// platforms but fail to link on Windows; the entire function must be #[cfg(unix)].
#[ cfg( unix ) ]
#[ allow( unsafe_code ) ]
pub( crate ) fn execute_live_mode(
  credential_store : &std::path::Path,
  live_creds_file  : &std::path::Path,
  params           : &UsageParams,
) -> Result< OutputData, ErrorData >
{
  use std::os::raw::{ c_int, c_void };
  use core::sync::atomic::Ordering;
  use std::time::{ SystemTime, UNIX_EPOCH };
  use std::io::Write;

  type SignalFn = extern "C" fn( c_int );
  extern "C"
  {
    fn signal     ( signum : c_int, handler : SignalFn ) -> usize;
    fn sigprocmask( how : c_int, set : *const c_void, oldset : *mut c_void ) -> c_int;
    fn sigemptyset( set : *mut c_void ) -> c_int;
    fn sigaddset  ( set : *mut c_void, signum : c_int ) -> c_int;
  }

  // Reset STOP_FLAG before registering the handler (safe across sequential test runs).
  STOP_FLAG.store( false, Ordering::Relaxed );
  // Unblock SIGINT: test runners (nextest) block SIGINT in their own mask; child processes
  // inherit this blocked mask.  A blocked signal is never delivered even with a registered
  // handler, so the STOP_FLAG is never set and the monitor loops forever.
  // Fix: explicitly unblock SIGINT before registering the handler.
  // sigset_t on Linux = 128 bytes, represented as [u64; 16].
  let mut sigset = [ 0u64; 16 ];
  // SAFETY: `on_sigint` is a valid C-compatible function pointer.
  //         `sigset` is zero-initialised and large enough for sigset_t on Linux.
  unsafe
  {
    sigemptyset( sigset.as_mut_ptr().cast::< c_void >() );
    sigaddset  ( sigset.as_mut_ptr().cast::< c_void >(), 2 );  // 2 = SIGINT
    sigprocmask( 1, sigset.as_ptr().cast::< c_void >(), core::ptr::null_mut() ); // 1 = SIG_UNBLOCK
    signal( 2, on_sigint );
  }

  loop
  {
    if STOP_FLAG.load( Ordering::Relaxed ) { break; }

    // Clear terminal and move cursor to top-left on each cycle.
    print!( "\x1B[2J\x1B[H" );
    let _ = std::io::stdout().flush();

    // Fetch with per-account stagger delays (thunder-herd mitigation).
    let accounts = fetch_all_quota( credential_store, live_creds_file, true, params.trace, params.solo )?;

    // live mode: no session context for footer; no credential_store path for sessions table.
    let text = render_text( &accounts, params.sort, params.desc, params.prefer, &params.cols, None, None, None, None, false );
    print!( "{text}" );

    // Compute next-refresh wall-clock time.
    let now_secs = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().as_secs();
    let jitter_extra = if params.jitter > 0
    {
      let nanos = u64::from( SystemTime::now().duration_since( UNIX_EPOCH ).unwrap_or_default().subsec_nanos() );
      nanos % ( params.jitter + 1 ) // 0..=jitter seconds
    }
    else
    {
      0
    };
    let wait_secs = params.interval + jitter_extra;
    let next_at   = now_secs + wait_secs;

    // Countdown footer — rewritten in-place each second via \r.
    let mut remaining = wait_secs;
    loop
    {
      if STOP_FLAG.load( Ordering::Relaxed ) { break; }
      let next_hms = secs_to_hms_utc( next_at );
      let m        = remaining / 60;
      let s        = remaining % 60;
      let line     = format!( "  Next update in {m}:{s:02} (at {next_hms} UTC)  [Ctrl-C to exit]" );
      // Right-pad to 80+ chars to erase leftover characters from a previous longer line.
      print!( "\r{line:<80}" );
      let _ = std::io::stdout().flush();
      if remaining == 0 { break; }
      remaining -= 1;
      std::thread::sleep( core::time::Duration::from_secs( 1 ) );
    }
    println!();

    if STOP_FLAG.load( Ordering::Relaxed ) { break; }
  }

  println!( "\nMonitor stopped." );
  Ok( OutputData::new( String::new(), "text" ) )
}

/// Live mode is not available on non-Unix platforms (requires POSIX signal handling).
#[ cfg( not( unix ) ) ]
pub( crate ) fn execute_live_mode(
  _credential_store : &std::path::Path,
  _live_creds_file  : &std::path::Path,
  _params           : &UsageParams,
) -> Result< OutputData, ErrorData >
{
  Err( ErrorData::new(
    ErrorCode::InternalError,
    "Live mode requires POSIX signals and is not supported on this platform".to_string(),
  ) )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::secs_to_hms_utc;

  /// C15 — Zero seconds → "00:00:00".
  #[ test ]
  fn test_secs_to_hms_utc_zero()
  {
    assert_eq!( secs_to_hms_utc( 0 ), "00:00:00" );
  }

  /// C16 — End of day → "23:59:59".
  #[ test ]
  fn test_secs_to_hms_utc_end_of_day()
  {
    assert_eq!( secs_to_hms_utc( 86399 ), "23:59:59" );
  }

  /// C17 — Exactly one day wraps to "00:00:00".
  #[ test ]
  fn test_secs_to_hms_utc_day_wrap()
  {
    assert_eq!( secs_to_hms_utc( 86400 ), "00:00:00" );
  }

  /// C18 — Mid-day timestamp.
  #[ test ]
  fn test_secs_to_hms_utc_midday()
  {
    assert_eq!( secs_to_hms_utc( 45045 ), "12:30:45" );
  }
}
