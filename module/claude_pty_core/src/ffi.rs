//! Every `extern "C"` declaration and `unsafe` block in this crate.
//!
//! # Containment
//!
//! The workspace sets `unsafe-code = "deny"` globally. This module is the single
//! documented exception, following the same idiom as
//! `claude_profile_core::account::store::lock_store` — an `extern "C"` block and
//! its `unsafe` call sites confined to one scope, each with a `SAFETY:`
//! justification. No other module in this crate contains the token `unsafe`;
//! `tests/unsafe_containment_test.rs` enforces that mechanically.
//!
//! Every function here returns a safe Rust type, so callers never handle a raw
//! fd or a nullable pointer.
//!
//! # Platform
//!
//! The `TIOC*` request numbers below are Linux values. The workspace is already
//! Linux-only in practice (`claude_session_core`'s liveness reads `/proc`), so
//! this crate does not attempt to abstract them.
#![ allow( unsafe_code ) ]

use core::ffi::{ c_char, c_int, c_ulong, c_ushort };
use std::os::unix::io::{ FromRawFd, OwnedFd, RawFd };
use std::process::Command;

use crate::error::{ Error, Result };

// ──────────────────────────────── constants ────────────────────────────────

/// `O_RDWR` — open the pty master for both reading and writing.
const O_RDWR : c_int = 2;
/// `O_NOCTTY` — do not make the master our controlling terminal.
///
/// The *child* claims the slave as its controlling terminal; the parent must not
/// claim the master, or a `Ctrl-C` in the operator's own shell would be delivered
/// to the wrong process group.
const O_NOCTTY : c_int = 0o400;
/// `O_CLOEXEC` — do not let the master survive into an `exec`ed child.
///
/// Not a hygiene nicety. A descriptor opened without it is inherited by every
/// child spawned afterwards, so the pty's own child ends up holding a copy of the
/// master to its own terminal. Closing every descriptor the parent holds then
/// never produces `EOF` on the slave — the child is keeping its own terminal
/// alive — so a child blocked reading stdin never exits and `shutdown` waits
/// forever. `try_clone` already uses `F_DUPFD_CLOEXEC`, so only this original
/// needs the flag; the slave descriptors are meant to reach the child and get
/// there through `dup2`, which clears it by design.
const O_CLOEXEC : c_int = 0o2_000_000;
/// `TIOCSWINSZ` — set the terminal window size (Linux).
const TIOCSWINSZ : c_ulong = 0x5414;
/// `TIOCSCTTY` — claim this terminal as the controlling terminal (Linux).
const TIOCSCTTY : c_ulong = 0x540E;
/// `TIOCSCTTY`'s only argument: whether to steal the terminal from its current
/// owning session. Always zero here — a freshly allocated pty has no owner, and
/// stealing would be a bug rather than a fallback. Typed as `c_int` because a
/// variadic call carries no signature to coerce an untyped literal against.
const CTTY_STEAL_NO : c_int = 0;
/// Upper bound for a `ptsname_r` result — `/dev/pts/<n>` is far shorter.
const PTSNAME_MAX : usize = 128;

extern "C"
{
  fn posix_openpt( flags : c_int ) -> c_int;
  fn grantpt( fd : c_int ) -> c_int;
  fn unlockpt( fd : c_int ) -> c_int;
  fn ptsname_r( fd : c_int, buf : *mut c_char, buflen : usize ) -> c_int;
  fn ioctl( fd : c_int, request : c_ulong, ... ) -> c_int;
  fn setsid() -> c_int;
}

// ─────────────────────────────── raw structs ───────────────────────────────

/// Kernel `struct winsize`, as consumed by `TIOCSWINSZ`.
///
/// Field names mirror `<termios.h>` verbatim, `ws_` prefix included. Renaming
/// them would not change the layout, but it would break the one property that
/// makes an FFI mirror struct auditable: that a reader can diff it against the
/// kernel header line for line.
#[ repr( C ) ]
#[ derive( Debug, Clone, Copy ) ]
#[ allow( clippy::struct_field_names ) ]
struct WinSizeRaw
{
  ws_row : c_ushort,
  ws_col : c_ushort,
  ws_xpixel : c_ushort,
  ws_ypixel : c_ushort,
}

// ──────────────────────────────── operations ───────────────────────────────

/// Allocate a pty master and return it as an owned file descriptor.
///
/// Performs the full POSIX unlock sequence: `posix_openpt`, `grantpt`,
/// `unlockpt`. The returned [`OwnedFd`] closes the master on drop.
///
/// # Errors
///
/// Returns [`Error::Os`] naming whichever step of the sequence failed.
#[ inline ]
pub fn open_master() -> Result< OwnedFd >
{
  // SAFETY: posix_openpt takes only an integer flag set and returns a new fd or
  // -1; no pointers cross the boundary.
  let raw = unsafe { posix_openpt( O_RDWR | O_NOCTTY | O_CLOEXEC ) };
  if raw < 0
  {
    return Err( Error::last_os( "posix_openpt" ) );
  }

  // SAFETY: `raw` is a fresh fd returned by posix_openpt above and is not owned
  // anywhere else, so this transfer of ownership is unique. Constructed before
  // the grantpt/unlockpt checks so an early return still closes it.
  let owned = unsafe { OwnedFd::from_raw_fd( raw ) };

  // SAFETY: grantpt takes only the owned master fd; no pointers cross.
  if unsafe { grantpt( raw ) } != 0
  {
    return Err( Error::last_os( "grantpt" ) );
  }
  // SAFETY: unlockpt takes only the owned master fd; no pointers cross.
  if unsafe { unlockpt( raw ) } != 0
  {
    return Err( Error::last_os( "unlockpt" ) );
  }
  Ok( owned )
}

/// Return the filesystem path of the slave device paired with `master`.
///
/// # Errors
///
/// Returns [`Error::Os`] if `ptsname_r` fails, or [`Error::NonUtf8SlavePath`] if
/// the kernel reports a path that is not valid UTF-8.
#[ inline ]
pub fn slave_path( master : RawFd ) -> Result< String >
{
  let mut buf = [ 0_u8; PTSNAME_MAX ];
  // SAFETY: `buf` is a live, uniquely-borrowed stack array of exactly
  // PTSNAME_MAX bytes, and that same length is passed as `buflen`, so
  // ptsname_r cannot write out of bounds. It NUL-terminates within that bound.
  let rc = unsafe { ptsname_r( master, buf.as_mut_ptr().cast::< c_char >(), PTSNAME_MAX ) };
  if rc != 0
  {
    return Err( Error::last_os( "ptsname_r" ) );
  }
  let end = buf.iter().position( | b | *b == 0 ).unwrap_or( PTSNAME_MAX );
  core::str::from_utf8( &buf[ ..end ] )
    .map( str::to_string )
    .map_err( | _ | Error::NonUtf8SlavePath )
}

/// Set the window size of the terminal behind `fd`.
///
/// Pixel dimensions are reported as zero — the kernel accepts that, and no
/// consumer of this crate renders in pixel units.
///
/// # Errors
///
/// Returns [`Error::Os`] if the `TIOCSWINSZ` request fails.
#[ inline ]
pub fn set_win_size( fd : RawFd, rows : u16, cols : u16 ) -> Result< () >
{
  let size = WinSizeRaw { ws_row : rows, ws_col : cols, ws_xpixel : 0, ws_ypixel : 0 };
  // SAFETY: TIOCSWINSZ reads exactly one `struct winsize` through the pointer.
  // `size` is a live, correctly-typed `#[repr(C)]` local that outlives the call,
  // and the pointer is not retained by the kernel past it.
  let rc = unsafe { ioctl( fd, TIOCSWINSZ, core::ptr::addr_of!( size ) ) };
  if rc != 0
  {
    return Err( Error::last_os( "ioctl(TIOCSWINSZ)" ) );
  }
  Ok( () )
}

/// Arrange for the spawned child to lead a new session whose controlling
/// terminal is `slave`.
///
/// Registers a `pre_exec` hook that runs in the forked child between `fork` and
/// `exec`. Both calls it makes are async-signal-safe, which is the binding
/// constraint on anything executed in that window.
///
/// Without this, the child has no controlling terminal: job control, `Ctrl-C`
/// delivery, and `SIGWINCH` on resize all fail silently.
///
/// The terminal is claimed through descriptor 0 rather than the parent's own
/// slave descriptor, because `std`'s child-side setup performs its stdio `dup2`
/// calls *before* running any `pre_exec` closure — by the time this hook runs,
/// descriptor 0 already refers to the slave. Using it avoids depending on the
/// parent's copy still being open at that point.
#[ inline ]
pub fn attach_controlling_terminal( cmd : &mut Command )
{
  /// The child's stdin, already redirected to the pty slave when `pre_exec` runs.
  const CHILD_STDIN_FD : RawFd = 0;

  use std::os::unix::process::CommandExt as _;

  // SAFETY: `pre_exec` requires the closure to be async-signal-safe, because it
  // runs after fork(2) in a process that may hold locks from arbitrary parent
  // threads. `setsid` and `ioctl` are both on the POSIX async-signal-safe list;
  // the closure allocates nothing, takes no locks, and touches no shared state.
  unsafe
  {
    cmd.pre_exec( move ||
    {
      if setsid() < 0
      {
        return Err( std::io::Error::last_os_error() );
      }
      if ioctl( CHILD_STDIN_FD, TIOCSCTTY, CTTY_STEAL_NO ) != 0
      {
        return Err( std::io::Error::last_os_error() );
      }
      Ok( () )
    });
  }
}
