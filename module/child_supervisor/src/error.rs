//! Error type for `child_supervisor`.

use core::fmt;

/// Result alias for every fallible operation in this crate.
pub type Result< T > = core::result::Result< T, Error >;

/// Errors produced by the output pump and the session table.
#[ derive( Debug ) ]
#[ non_exhaustive ]
pub enum Error
{
  /// A session's reader was already taken, so its output cannot be drained.
  ///
  /// A pty master that nobody reads stalls its child as soon as the kernel's
  /// buffer fills, and a stalled child is indistinguishable from a thinking one.
  /// Refusing to host the session is better than hosting one that will silently
  /// wedge.
  ReaderTaken,
  /// A PTY-layer operation failed.
  Pty( claude_pty_core::Error ),
  /// No hosted session carries the requested id.
  UnknownSession( String ),
}

impl fmt::Display for Error
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::ReaderTaken => write!( f, "session output is already being read elsewhere" ),
      Self::Pty( source ) => write!( f, "pty error: {source}" ),
      Self::UnknownSession( id ) => write!( f, "no such session: {id}" ),
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
      Self::Pty( source ) => Some( source ),
      Self::ReaderTaken | Self::UnknownSession( _ ) => None,
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
