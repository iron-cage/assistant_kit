//! The daemon's listening socket, and the two ways it goes wrong.
//!
//! # Removing a socket that is already there
//!
//! `bind` fails with `AddrInUse` when the path exists, whether or not anything
//! is listening on it — and a daemon killed with `SIGKILL` leaves one behind. So
//! a stale socket has to be unlinked before binding.
//!
//! Unlinking unconditionally is only safe if no other daemon is listening on it,
//! and the thing that establishes that is the instance lock. [`Listener::bind`]
//! therefore takes the lock as evidence rather than as documentation, and checks
//! it actually covers the socket being bound: a lock held over some *other*
//! directory proves nothing about this one.
//!
//! The lock is evidence about daemons, not about files. So the unlink is narrowed
//! to what it actually covers — a path that is a socket. Anything else there is
//! refused rather than deleted, because nothing about holding the lock makes some
//! other file at that path expendable.
//!
//! # Removing it afterwards
//!
//! Nothing else does. A socket file outlives the process that bound it, and the
//! next client to come along connects to it and gets `ECONNREFUSED` — which
//! reads as "the daemon is broken", not "the daemon is not running". [`Listener`]
//! removes its own path on drop, so the ordinary end of a daemon leaves nothing
//! behind for the next one to explain away.

use std::os::unix::fs::{ FileTypeExt, PermissionsExt };
use std::os::unix::net::{ UnixListener, UnixStream };
use std::path::{ Path, PathBuf };

use crate::error::{ Error, Result };
use crate::lock::InstanceLock;

/// Permission bits set on the socket after binding: owner read/write only.
const SOCKET_MODE : u32 = 0o600;

/// The daemon's listening socket.
#[ derive( Debug ) ]
pub struct Listener
{
  listener : UnixListener,
  socket_path : PathBuf,
}

impl Listener
{
  /// Bind `socket_path`, removing any stale socket already there.
  ///
  /// `lock` is the single-instance lock, and is what makes that removal safe —
  /// see the module docs. It must live in the same directory as `socket_path`,
  /// or it is evidence about a different daemon.
  ///
  /// **Caller obligation:** the parent directory must exist. In practice it
  /// always does — the lock has to live in it, and [`crate::lock::acquire`]
  /// creates it on the way to taking it. `bind` creates nothing itself.
  ///
  /// The socket is left readable and writable by its owner only. There is a
  /// window between binding and applying that in which the umask governs
  /// instead; the runtime directory is the real boundary, and this narrows what
  /// gets through it rather than replacing it.
  ///
  /// # Errors
  ///
  /// - [`Error::LockMismatch`] — `lock` does not live beside `socket_path`.
  /// - [`Error::Io`] — something that is not a socket occupies the path, or it
  ///   could not be unlinked or bound.
  #[ inline ]
  pub fn bind( socket_path : &Path, lock : &InstanceLock ) -> Result< Self >
  {
    if lock.path().parent() != socket_path.parent()
    {
      return Err( Error::LockMismatch
      {
        lock_path : lock.path().to_path_buf(),
        socket_path : socket_path.to_path_buf(),
      } );
    }

    // Nothing there is the expected case. A socket there is the crashed-daemon
    // case, and the lock says it is nobody's. Anything else is neither, and the
    // lock says nothing about it — so it stays.
    //
    // `symlink_metadata`, not `metadata`: a symlink to a socket somewhere else
    // is not this daemon's socket, whatever it resolves to.
    match std::fs::symlink_metadata( socket_path )
    {
      Ok( metadata ) if metadata.file_type().is_socket() =>
      {
        std::fs::remove_file( socket_path ).map_err( Error::Io )?;
      },
      Ok( _ ) => return Err( Error::Io( std::io::Error::new
      (
        std::io::ErrorKind::AlreadyExists,
        format!( "{} exists and is not a socket", socket_path.display() ),
      ) ) ),
      Err( source ) if source.kind() == std::io::ErrorKind::NotFound => {},
      Err( source ) => return Err( Error::Io( source ) ),
    }

    let listener = UnixListener::bind( socket_path ).map_err( Error::Io )?;
    std::fs::set_permissions( socket_path, std::fs::Permissions::from_mode( SOCKET_MODE ) )
      .map_err( Error::Io )?;

    Ok( Self { listener, socket_path : socket_path.to_path_buf() } )
  }

  /// The path this listener is bound to.
  #[ inline ]
  #[ must_use ]
  pub fn path( &self ) -> &Path
  {
    &self.socket_path
  }

  /// Wait for the next client and return its connection.
  ///
  /// Blocks. One connection carries exactly one request, so a client can never
  /// hold the daemon longer than the request it sent — which is what keeps a
  /// single-threaded accept loop from being a single-client accept loop.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Io`] if accepting fails.
  #[ inline ]
  pub fn accept( &self ) -> Result< UnixStream >
  {
    self.listener.accept().map( | ( stream, _addr ) | stream ).map_err( Error::Io )
  }
}

impl Drop for Listener
{
  #[ inline ]
  fn drop( &mut self )
  {
    // Best effort: if this fails the next `bind` unlinks it anyway. The point is
    // that the window in which a client can connect to nothing is short, not
    // that it is provably zero.
    drop( std::fs::remove_file( &self.socket_path ) );
  }
}
