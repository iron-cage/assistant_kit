//! Persistent user storage paths for `claude_profile`.
//!
//! Resolves `$PRO/persistent/claude_profile/` from environment variables,
//! falling back to `$HOME/persistent/claude_profile/` when `$PRO` is unset,
//! non-existent, or points to a file rather than a directory. See `docs/feature/010_persistent_storage.md` (FR-15).
//!
//! # Known Pitfalls
//!
//! ## P1 — `exists()` vs `is_dir()` for `$PRO` validation (issue-001)
//!
//! `path.exists()` returns `true` for both files and directories. Using
//! `exists()` to guard `$PRO` allows a file path to silently pass as a
//! valid storage root, producing a nonsensical base like
//! `<file>/persistent/claude_profile/` that causes `ensure_exists()` to
//! fail with `ENOTDIR` at runtime — not at the validation call site.
//!
//! **Always use `is_dir()`** when validating environment variables that
//! must resolve to a directory root. Use `exists()` only when the
//! distinction between file and directory does not matter.
//!
//! Reproducer: `persist_test.rs::p14_pro_set_to_existing_file_falls_back_to_home`.

use std::path::{ Path, PathBuf };

/// Persistent user storage paths for `claude_profile`.
///
/// Resolves the storage root from environment variables: `$PRO` (if set and
/// is an existing directory) → `$HOME` / `$USERPROFILE`. The resolved base is
/// `{root}/persistent/claude_profile/`.
///
/// # Examples
///
/// ```no_run
/// use claude_profile::PersistPaths;
///
/// let paths = PersistPaths::new().expect( "failed to resolve persistent storage root" );
/// println!( "storage at: {}", paths.base().display() );
/// ```
#[ derive( Debug ) ]
pub struct PersistPaths
{
  base : PathBuf,
}

impl PersistPaths
{
  /// Resolve the persistent storage root.
  ///
  /// Tries `$PRO` first (if set and is an existing directory), then falls back
  /// to `$HOME` (or `$USERPROFILE` on Windows). Returns `Err` if none resolve.
  ///
  /// # Errors
  ///
  /// Returns `std::io::Error` (kind `NotFound`) if both `$PRO` and `$HOME`
  /// are unset or `$HOME` points to a non-existent path.
  #[ inline ]
  pub fn new() -> Result< Self, std::io::Error >
  {
    let root = Self::resolve_root()?;
    Ok( Self { base : root.join( "persistent" ).join( "claude_profile" ) } )
  }

  /// The resolved base directory: `{root}/persistent/claude_profile/`.
  #[ must_use ]
  #[ inline ]
  pub fn base( &self ) -> &Path
  {
    &self.base
  }

  /// Create the base directory if it does not exist.
  ///
  /// # Errors
  ///
  /// Returns `std::io::Error` if directory creation fails.
  #[ inline ]
  pub fn ensure_exists( &self ) -> Result< (), std::io::Error >
  {
    std::fs::create_dir_all( &self.base )
  }

  fn resolve_root() -> Result< PathBuf, std::io::Error >
  {
    // Fix(issue-001):
    // Root cause: path.exists() returns true for files — is_dir() is the correct guard;
    //   a file path produces a nonsensical base like `<file>/persistent/claude_profile`
    //   and makes ensure_exists() fail with ENOTDIR at call time, not at validation.
    // Pitfall: do not replace is_dir() with exists() — files silently break ensure_exists()
    if let Some( pro ) = std::env::var_os( "PRO" )
    {
      let path = PathBuf::from( pro );
      if path.is_dir()
      {
        return Ok( path );
      }
    }
    // fall back to $HOME (Unix) or $USERPROFILE (Windows)
    std::env::var_os( "HOME" )
    .or_else( || std::env::var_os( "USERPROFILE" ) )
    .map( PathBuf::from )
    .ok_or_else( || std::io::Error::new( std::io::ErrorKind::NotFound, "neither $PRO nor $HOME is set" ) )
  }
}
