//! Unix signal-aware exit code computation.
//!
//! Provides [`signal_exit_code`] which follows the POSIX `128 + signal_number`
//! convention: if a subprocess was killed by a signal, the canonical exit code
//! reported to the caller is `128 + N`, allowing signal kills to be distinguished
//! from normal exits (0–125).

use std::process::ExitStatus;

/// Compute the exit code to propagate for a subprocess [`ExitStatus`].
///
/// Follows the POSIX `128 + signal` convention on Unix platforms:
/// if the process was killed by signal `N`, returns `128 + N`.
/// Falls back to `status.code().unwrap_or(1)` on non-Unix or when
/// the status reports a numeric exit code (no signal).
///
/// # Fix(BUG-242)
///
/// Before this helper existed every `unwrap_or(-1)` / `unwrap_or(1)` at
/// exit-code call sites collapsed all signal kills to a single code.
/// Replace those call sites with `signal_exit_code(&status)`.
///
/// # Examples
///
/// ```
/// # #[ cfg( unix ) ]
/// # {
/// use std::process::Command;
/// use claude_runner_core::signal_exit_code;
///
/// // A process that exits 0 → 0
/// let status = Command::new( "true" ).status().unwrap();
/// assert_eq!( signal_exit_code( &status ), 0 );
///
/// // A process that exits 1 → 1
/// let status = Command::new( "false" ).status().unwrap();
/// assert_eq!( signal_exit_code( &status ), 1 );
/// # }
/// ```
#[ inline ]
#[ must_use ]
pub fn signal_exit_code( status : &ExitStatus ) -> i32
{
  // Fix(BUG-242): signal-killed subprocesses must follow 128+signal convention.
  // Root cause: status.code().unwrap_or(N) returns None for signal kills on Unix;
  //   any signal kill (SIGTERM=15, SIGKILL=9) produced a hardcoded 1 or -1,
  //   masking the actual termination reason from all callers.
  // Pitfall: always check code() first — if it's Some, the process exited normally
  //   via _exit() and there is no signal; the #[cfg(unix)] branch fires only when
  //   code() is None, which only happens on Unix when a signal terminated the process.
  if let Some( code ) = status.code()
  {
    return code;
  }
  #[ cfg( unix ) ]
  {
    use std::os::unix::process::ExitStatusExt;
    if let Some( signal ) = status.signal()
    {
      return 128 + signal;
    }
  }
  1
}
