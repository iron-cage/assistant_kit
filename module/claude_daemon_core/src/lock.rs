//! Single-instance enforcement via an advisory `flock`.
//!
//! Exactly one daemon may run: it owns every hosted PTY, and a second instance
//! would bind the same socket path and split the session table in two.
//!
//! # Why `flock` and not a PID file
//!
//! A PID file records an intention; the kernel enforces nothing, and a daemon
//! killed with `SIGKILL` leaves a file naming a PID that may since have been
//! recycled. An advisory `flock` is released by the kernel when the holder's last
//! descriptor closes — including on crash — so a stale lock cannot outlive its
//! owner. This is the same idiom as
//! `claude_profile_core::account::store::lock_store`.
//!
//! Advisory only: nothing stops a process that never calls this. It guards
//! against a second *daemon*, not against arbitrary interference.

use std::path::{ Path, PathBuf };

use crate::error::{ Error, Result };

/// A held single-instance lock.
///
/// The lock lives as long as this value: dropping it closes the descriptor,
/// which releases the lock.
#[ derive( Debug ) ]
pub struct InstanceLock
{
  _file : std::fs::File,
  path : PathBuf,
}

impl InstanceLock
{
  /// Path of the lock file being held.
  #[ inline ]
  #[ must_use ]
  pub fn path( &self ) -> &Path
  {
    &self.path
  }
}

/// Take the single-instance lock at `lock_path`, failing if another daemon holds it.
///
/// Non-blocking: a contended lock returns [`Error::AlreadyRunning`] immediately
/// rather than waiting, because the caller's correct response is to talk to the
/// running daemon, not to queue behind it.
///
/// # Errors
///
/// - [`Error::AlreadyRunning`] — another process holds the lock.
/// - [`Error::Io`] — the parent directory or lock file could not be created.
// `extern "C"` decl and unsafe call are scoped to this one function — the same
// idiom as claude_profile_core's lock_store (the workspace denies unsafe
// globally; std's own File::try_lock would need MSRV 1.89 vs the declared 1.75).
#[ inline ]
#[ allow( unsafe_code ) ]
pub fn acquire( lock_path : &Path ) -> Result< InstanceLock >
{
  if let Some( parent ) = lock_path.parent()
  {
    std::fs::create_dir_all( parent )?;
  }
  let file = std::fs::OpenOptions::new()
    .create( true )
    .write( true )
    .truncate( false ) // a lock file's content is irrelevant — never disturb a held lock's file
    .open( lock_path )?;

  #[ cfg( unix ) ]
  {
    use core::ffi::c_int;
    use std::os::unix::io::AsRawFd as _;
    extern "C"
    {
      fn flock( fd : c_int, operation : c_int ) -> c_int;
    }
    /// Exclusive lock.
    const LOCK_EX : c_int = 2;
    /// Fail instead of blocking when the lock is contended.
    const LOCK_NB : c_int = 4;

    // SAFETY: flock takes only an owned open fd and an integer op flag; no
    // pointers cross the boundary.
    let rc = unsafe { flock( file.as_raw_fd(), LOCK_EX | LOCK_NB ) };
    if rc != 0
    {
      return Err( Error::AlreadyRunning { lock_path : lock_path.to_path_buf() } );
    }
  }

  Ok( InstanceLock { _file : file, path : lock_path.to_path_buf() } )
}
