//! Error type for `daemon_kit`.

use core::fmt;

/// Result alias for every fallible operation in this crate.
pub type Result< T > = core::result::Result< T, Error >;

/// Errors produced by the lock, listener, IPC framing, and client layers.
#[ derive( Debug ) ]
#[ non_exhaustive ]
pub enum Error
{
  /// An I/O operation failed.
  Io( std::io::Error ),
  /// Another instance already holds the lock.
  ///
  /// Not a failure the caller should retry through — exactly one instance may
  /// run, and the correct response is to talk to the existing one.
  AlreadyRunning
  {
    /// Path of the contended lock file.
    lock_path : std::path::PathBuf,
  },
  /// The instance lock offered as evidence does not cover the socket being bound.
  ///
  /// Removing a stale socket is safe only because no other instance can be
  /// listening on it, and the instance lock is what establishes that. A lock
  /// held over a different directory establishes nothing about this one.
  LockMismatch
  {
    /// Path of the lock that was offered.
    lock_path : std::path::PathBuf,
    /// Path of the socket it was offered for.
    socket_path : std::path::PathBuf,
  },
  /// A protocol line exceeded [`crate::ipc::MAX_IPC_LINE_BYTES`].
  LineTooLong,
  /// A protocol line was not valid UTF-8.
  NonUtf8Line,
  /// A protocol line was not valid JSON, or not a known request shape.
  Malformed( String ),
  /// The daemon answered, and its answer was a failure.
  ///
  /// Carries the daemon's own message verbatim. Distinct from every other
  /// variant here, which describe something going wrong on *this* side of the
  /// socket — a `Remote` means the round trip worked and the request did not.
  Remote( String ),
}

impl fmt::Display for Error
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::Io( source ) => write!( f, "io error: {source}" ),
      Self::AlreadyRunning { lock_path } =>
        write!( f, "another instance already holds {}", lock_path.display() ),
      Self::LockMismatch { lock_path, socket_path } => write!
      (
        f,
        "instance lock {} does not cover socket {}",
        lock_path.display(),
        socket_path.display(),
      ),
      Self::LineTooLong =>
        write!( f, "protocol line exceeds {} bytes", crate::ipc::MAX_IPC_LINE_BYTES ),
      Self::NonUtf8Line => write!( f, "protocol line is not valid UTF-8" ),
      Self::Malformed( detail ) => write!( f, "malformed request: {detail}" ),
      Self::Remote( message ) => write!( f, "daemon reported: {message}" ),
    }
  }
}

impl std::error::Error for Error
{
  #[ inline ]
  fn source( &self ) -> Option< &( dyn std::error::Error + 'static ) >
  {
    match self
    {
      Self::Io( source ) => Some( source ),
      Self::AlreadyRunning { .. }
      | Self::LockMismatch { .. }
      | Self::LineTooLong
      | Self::NonUtf8Line
      | Self::Malformed( _ )
      | Self::Remote( _ ) => None,
    }
  }
}

impl From< std::io::Error > for Error
{
  #[ inline ]
  fn from( source : std::io::Error ) -> Self
  {
    Self::Io( source )
  }
}
