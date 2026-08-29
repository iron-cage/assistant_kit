//! Error type for `claude_context_report_core`.
//!
//! Hand-rolled, matching `claude_storage_core`, `claude_core`, and
//! `claude_session_core`. `error_tools` enters the workspace graph at Layer 1;
//! this crate is Layer 0.

use core::fmt;
use std::path::PathBuf;

/// Result alias for every fallible operation in this crate.
pub type Result< T > = core::result::Result< T, Error >;

/// Errors produced while building a context report.
#[ derive( Debug ) ]
#[ non_exhaustive ]
pub enum Error
{
  /// The transcript file does not exist.
  ///
  /// Distinct from [`Error::Read`] on purpose: a session spawned moments ago
  /// legitimately has not written a transcript yet, and a caller polling for one
  /// retries on this while treating [`Error::Read`] as a fault. The CLI surfaces
  /// the same split as exit code 1 versus 2.
  NoTranscript
  {
    /// The transcript path that does not exist.
    path : PathBuf,
  },

  /// The transcript exists but could not be read or folded.
  Read
  {
    /// The transcript that could not be read.
    path : PathBuf,
    /// What the underlying storage layer reported.
    source : claude_storage_core::Error,
  },
}

impl fmt::Display for Error
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::NoTranscript { path } =>
        write!( f, "session has not written a transcript yet: {}", path.display() ),
      Self::Read { path, source } =>
        write!( f, "cannot read session transcript {}: {source}", path.display() ),
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
      Self::NoTranscript { .. } => None,
      Self::Read { source, .. } => Some( source ),
    }
  }
}
