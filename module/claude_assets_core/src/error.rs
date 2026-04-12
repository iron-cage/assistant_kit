//! Domain-level error type for asset operations.
//!
//! Layer 2 (`claude_assets`) adapts `AssetError` to unilang's `ErrorData`
//! at call sites via `.map_err(|e| ErrorData::new(code, e.to_string()))`.

/// Error type for all `claude_assets_core` operations.
#[ derive( Debug ) ]
pub enum AssetError
{
  /// The named artifact was not found in the source directory.
  SourceNotFound
  {
    /// Artifact kind label (e.g., `"rule"`).
    kind : String,
    /// Artifact name (e.g., `"rust"`).
    name : String,
  },
  /// The target path exists but is not a symlink — uninstall refused.
  NotASymlink
  {
    /// Artifact kind label.
    kind : String,
    /// Artifact name.
    name : String,
  },
  /// Underlying I/O error from filesystem operations.
  Io( std::io::Error ),
}

impl core::fmt::Display for AssetError
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      Self::SourceNotFound { kind, name } =>
        write!( f, "{kind} '{name}' not found in $PRO_CLAUDE/{kind}s/" ),
      Self::NotASymlink { kind, name } =>
        write!( f, "{kind} '{name}' target is not a symlink — refusing to remove (data-loss guard)" ),
      Self::Io( e ) =>
        write!( f, "io error: {e}" ),
    }
  }
}

impl core::error::Error for AssetError
{
  #[ inline ]
  fn source( &self ) -> Option< &( dyn core::error::Error + 'static ) >
  {
    match self
    {
      Self::Io( e ) => Some( e ),
      _             => None,
    }
  }
}

impl From< std::io::Error > for AssetError
{
  #[ inline ]
  fn from( e : std::io::Error ) -> Self
  {
    Self::Io( e )
  }
}
