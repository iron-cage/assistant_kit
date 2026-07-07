//! Canonical clv-known filesystem paths, composed over [`claude_core::ClaudePaths`].

use std::path::PathBuf;
use claude_core::ClaudePaths;

/// Canonical paths clv reads from or writes to, beyond the base `ClaudePaths` set.
#[ derive( Debug, Clone ) ]
pub struct ClaudeVersionPaths
{
  core : ClaudePaths,
}

impl ClaudeVersionPaths
{
  /// Create a `ClaudeVersionPaths` from the `HOME` environment variable.
  ///
  /// Returns `None` if `HOME` is unset or empty.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use claude_version_core::paths::ClaudeVersionPaths;
  ///
  /// let p = ClaudeVersionPaths::new().expect( "HOME must be set" );
  /// ```
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Option< Self >
  {
    std::env::var( "HOME" ).ok().filter( | h | !h.is_empty() )?;
    Some( Self { core : ClaudePaths::new()? } )
  }

  /// `~/.claude/settings.json` — delegates to `ClaudePaths::settings_file()`.
  #[ inline ] #[ must_use ]
  pub fn settings_file( &self ) -> PathBuf { self.core.settings_file() }

  /// `~/.local/share/claude/versions` — delegates to `version::versions_dir_path()`.
  #[ inline ] #[ must_use ]
  pub fn versions_dir( &self ) -> PathBuf { PathBuf::from( crate::version::versions_dir_path() ) }

  /// `~/.local/bin/claude` — delegates to `version::binary_symlink_path()`.
  #[ inline ] #[ must_use ]
  pub fn binary_symlink( &self ) -> PathBuf { PathBuf::from( crate::version::binary_symlink_path() ) }

  /// `~/.claude/.transient/version_history_cache.json` — delegates to `version::version_history_cache_path()`.
  #[ inline ] #[ must_use ]
  pub fn version_history_cache_file( &self ) -> PathBuf { PathBuf::from( crate::version::version_history_cache_path() ) }

  /// Nearest ancestor `.claude/settings.json` from `cwd`. `None` if not found before
  /// the git-repository boundary or filesystem root.
  #[ inline ] #[ must_use ]
  pub fn project_settings_file( &self, cwd : &std::path::Path ) -> Option< PathBuf >
  {
    crate::config_resolve::find_project_config_file( cwd )
  }
}
