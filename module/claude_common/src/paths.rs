//! Canonical paths for all `~/.claude/` filesystem locations.
//!
//! # Design
//!
//! All `~/.claude/` paths are computed from the `HOME` environment variable.
//! [`ClaudePaths::new()`] returns `None` if `HOME` is not set (CI environments
//! without a home directory, containers, etc.).
//!
//! This module is the single authoritative source for `~/.claude/` path
//! constants across the workspace. All modules that need a `~/.claude/` path
//! must obtain it from `ClaudePaths`, not by constructing it independently.
//!
//! # Examples
//!
//! ```no_run
//! use claude_common::ClaudePaths;
//!
//! let p = ClaudePaths::new().expect( "HOME must be set" );
//! println!( "credentials: {}", p.credentials_file().display() );
//! println!( "accounts:    {}", p.accounts_dir().display() );
//! println!( "projects:    {}", p.projects_dir().display() );
//! ```

use std::path::{ Path, PathBuf };

/// Canonical paths for all `~/.claude/` filesystem locations.
///
/// Computed from the `HOME` environment variable. Returns `None` from
/// [`ClaudePaths::new()`] if `HOME` is not set.
///
/// # Examples
///
/// ```no_run
/// use claude_common::ClaudePaths;
///
/// if let Some( p ) = ClaudePaths::new()
/// {
///   println!( "base: {}", p.base().display() );
/// }
/// ```
#[ derive( Debug, Clone ) ]
pub struct ClaudePaths
{
  base : PathBuf,
}

impl ClaudePaths
{
  /// Create a `ClaudePaths` from the `HOME` environment variable.
  ///
  /// Returns `None` if `HOME` is not set.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use claude_common::ClaudePaths;
  ///
  /// let p = ClaudePaths::new().expect( "HOME must be set" );
  /// ```
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Option< Self >
  {
    let home = std::env::var( "HOME" ).ok()?;
    Some( Self { base : PathBuf::from( home ).join( ".claude" ) } )
  }

  /// Base `~/.claude/` directory.
  #[ inline ]
  #[ must_use ]
  pub fn base( &self ) -> &Path
  {
    &self.base
  }

  /// Path to `~/.claude/.credentials.json` — active OAuth token.
  #[ inline ]
  #[ must_use ]
  pub fn credentials_file( &self ) -> PathBuf
  {
    self.base.join( ".credentials.json" )
  }

  /// Path to `~/.claude/accounts/` — named credential snapshots.
  #[ inline ]
  #[ must_use ]
  pub fn accounts_dir( &self ) -> PathBuf
  {
    self.base.join( "accounts" )
  }

  /// Path to `~/.claude/projects/` — conversation history root.
  #[ inline ]
  #[ must_use ]
  pub fn projects_dir( &self ) -> PathBuf
  {
    self.base.join( "projects" )
  }

  /// Path to `~/.claude/stats-cache.json` — usage statistics.
  #[ inline ]
  #[ must_use ]
  pub fn stats_file( &self ) -> PathBuf
  {
    self.base.join( "stats-cache.json" )
  }

  /// Path to `~/.claude/settings.json` — user settings.
  #[ inline ]
  #[ must_use ]
  pub fn settings_file( &self ) -> PathBuf
  {
    self.base.join( "settings.json" )
  }

  /// Path to `~/.claude/session-env/` — per-session environment records.
  #[ inline ]
  #[ must_use ]
  pub fn session_env_dir( &self ) -> PathBuf
  {
    self.base.join( "session-env" )
  }

  /// Path to `~/.claude/sessions/` — session records.
  #[ inline ]
  #[ must_use ]
  pub fn sessions_dir( &self ) -> PathBuf
  {
    self.base.join( "sessions" )
  }
}
