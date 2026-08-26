//! Error type for `claude_session_core`.
//!
//! Hand-rolled, matching `claude_storage_core` and `claude_core`. `error_tools`
//! enters the workspace graph at Layer 1; this crate is Layer 0.

use core::fmt;

/// Result alias for every fallible operation in this crate.
pub type Result< T > = core::result::Result< T, Error >;

/// Errors produced while reading the live-session registry.
#[ derive( Debug ) ]
#[ non_exhaustive ]
pub enum Error
{
  /// The registry directory could not be read.
  ///
  /// A *missing* directory is not this error — it means no Claude Code process
  /// has ever registered on this machine, which [`crate::registry::scan`]
  /// reports as an empty result rather than a failure.
  ReadDir
  {
    /// The directory that could not be read.
    path : std::path::PathBuf,
    /// The underlying OS error.
    source : std::io::Error,
  },
}

impl fmt::Display for Error
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::ReadDir { path, source } =>
        write!( f, "cannot read session registry directory {}: {source}", path.display() ),
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
      Self::ReadDir { source, .. } => Some( source ),
    }
  }
}
