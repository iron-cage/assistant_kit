//! Error type for `claude_daemon_core`.

use core::fmt;

/// Result alias for every fallible operation in this crate.
pub type Result< T > = core::result::Result< T, Error >;

/// Errors produced by the daemon, its lock, and its IPC layer.
#[ derive( Debug ) ]
#[ non_exhaustive ]
pub enum Error
{
  /// An I/O operation failed.
  Io( std::io::Error ),
  /// Another daemon instance already holds the lock.
  ///
  /// Not a failure the caller should retry through — exactly one daemon may run,
  /// and the correct response is to talk to the existing one.
  AlreadyRunning
  {
    /// Path of the contended lock file.
    lock_path : std::path::PathBuf,
  },
  /// A protocol line exceeded [`crate::ipc::MAX_IPC_LINE_BYTES`].
  LineTooLong,
  /// A protocol line was not valid UTF-8.
  NonUtf8Line,
  /// A protocol line was not valid JSON, or not a known request shape.
  Malformed( String ),
  /// No hosted session carries the requested conversation id.
  UnknownSession( String ),
  /// A PTY-layer operation failed.
  Pty( claude_pty_core::Error ),
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
        write!( f, "another daemon already holds {}", lock_path.display() ),
      Self::LineTooLong =>
        write!( f, "protocol line exceeds {} bytes", crate::ipc::MAX_IPC_LINE_BYTES ),
      Self::NonUtf8Line => write!( f, "protocol line is not valid UTF-8" ),
      Self::Malformed( detail ) => write!( f, "malformed request: {detail}" ),
      Self::UnknownSession( id ) => write!( f, "no such session: {id}" ),
      Self::Pty( source ) => write!( f, "pty error: {source}" ),
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
      Self::Pty( source ) => Some( source ),
      Self::AlreadyRunning { .. }
      | Self::LineTooLong
      | Self::NonUtf8Line
      | Self::Malformed( _ )
      | Self::UnknownSession( _ ) => None,
    }
  }
}

impl From< claude_pty_core::Error > for Error
{
  #[ inline ]
  fn from( source : claude_pty_core::Error ) -> Self
  {
    Self::Pty( source )
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
