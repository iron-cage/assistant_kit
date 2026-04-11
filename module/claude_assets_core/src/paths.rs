//! Source and target path resolution for Claude Code assets.
//!
//! `AssetPaths` resolves `$PRO_CLAUDE` (with `$PRO/genai/claude/` fallback)
//! as the source root and the current working directory as the target root.

use std::path::{ Path, PathBuf };
use crate::artifact::ArtifactKind;

/// Typed error returned when `$PRO_CLAUDE` cannot be resolved.
#[ derive( Debug ) ]
pub enum AssetPathsError
{
  /// Neither `$PRO_CLAUDE` nor `$PRO` is set in the environment.
  EnvVarNotSet,
}

impl core::fmt::Display for AssetPathsError
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      Self::EnvVarNotSet => write!(
        f,
        "environment variable $PRO_CLAUDE is not set \
         (fallback: set $PRO and ensure $PRO/genai/claude/ exists) \
         — run: export PRO_CLAUDE=/path/to/your/claude-assets"
      ),
    }
  }
}

impl core::error::Error for AssetPathsError {}

/// Resolved source and target roots for asset installation.
///
/// - `source_root` — absolute path to `$PRO_CLAUDE` (or `$PRO/genai/claude/`)
/// - `target_root` — directory into which `.claude/<kind>/` symlinks are placed
#[ derive( Debug, Clone ) ]
pub struct AssetPaths
{
  source_root : PathBuf,
  target_root : PathBuf,
}

impl AssetPaths
{
  /// Resolve paths from the current environment.
  ///
  /// Lookup order:
  /// 1. `$PRO_CLAUDE` — used directly if set
  /// 2. `$PRO/genai/claude/` — used if `$PRO` is set and `$PRO_CLAUDE` is not
  /// 3. Error: `AssetPathsError::EnvVarNotSet`
  ///
  /// The target root is the current working directory.
  ///
  /// # Errors
  ///
  /// Returns `AssetPathsError::EnvVarNotSet` if neither env var is set.
  #[ inline ]
  pub fn from_env() -> Result< Self, AssetPathsError >
  {
    let source_root = if let Ok( v ) = std::env::var( "PRO_CLAUDE" )
    {
      PathBuf::from( v )
    }
    else if let Ok( pro ) = std::env::var( "PRO" )
    {
      PathBuf::from( pro ).join( "genai" ).join( "claude" )
    }
    else
    {
      return Err( AssetPathsError::EnvVarNotSet );
    };

    let target_root = std::env::current_dir().unwrap_or_else( |_| PathBuf::from( "." ) );
    Ok( Self { source_root, target_root } )
  }

  /// Construct directly from explicit paths (useful for tests).
  #[ must_use ]
  #[ inline ]
  pub fn new( source_root : PathBuf, target_root : PathBuf ) -> Self
  {
    Self { source_root, target_root }
  }

  /// Absolute path to the source subdirectory for `kind`.
  ///
  /// e.g., `$PRO_CLAUDE/rules/` for `ArtifactKind::Rule`.
  #[ must_use ]
  #[ inline ]
  pub fn source_dir( &self, kind : ArtifactKind ) -> PathBuf
  {
    self.source_root.join( kind.source_subdir() )
  }

  /// Absolute path to the target subdirectory for `kind` (inside `.claude/`).
  ///
  /// e.g., `<cwd>/.claude/rules/` for `ArtifactKind::Rule`.
  #[ must_use ]
  #[ inline ]
  pub fn target_dir( &self, kind : ArtifactKind ) -> PathBuf
  {
    self.target_root.join( ".claude" ).join( kind.target_subdir() )
  }

  /// The source root path.
  #[ must_use ]
  #[ inline ]
  pub fn source_root( &self ) -> &Path
  {
    &self.source_root
  }
}
