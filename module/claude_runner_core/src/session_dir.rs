//! Directory-based session isolation for Claude Code invocations.
//!
//! # Design
//!
//! Sessions are identified by topic name and isolated in `-{topic}/` directories
//! within a configured base directory.
//!
//! ## Session Strategy
//!
//! - `Resume`: Continue existing session or create new (default)
//! - `Fresh`: Discard existing session directory and start clean
//!
//! ## Session Location
//!
//! Sessions are created as subdirectories of the configured base:
//! - Location: `{sessions_base_dir}/-{session_name}/`
//! - Example: Base `/project/`, session `debug` → `/project/-debug/`
//! - Sessions prefixed with hyphen for git exclusion
//!
//! This ensures Claude Code executes with access to project files.
//!
//! ## Continuation Detection
//!
//! To check if a session has conversation history, use
//! `claude_storage_core::continuation::check_continuation()` with the
//! session directory path returned by `session_dir()` or `ensure_session()`.
//!
//! # Examples
//!
//! ```
//! use claude_runner_core::{ SessionManager, Strategy };
//! use std::path::PathBuf;
//!
//! let sessions_dir = PathBuf::from( "/tmp/sessions" );
//! let mgr = SessionManager::new( &sessions_dir );
//! let session_dir = mgr.ensure_session( "debug", Strategy::Resume )?;
//! assert!( session_dir.exists() );
//! # Ok::<(), std::io::Error>(())
//! ```

use std::path::{ Path, PathBuf };
use core::str::FromStr;

/// Session creation strategy.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum Strategy
{
  /// Resume existing session or create new.
  Resume,
  /// Discard existing session directory and create fresh.
  Fresh,
}

impl FromStr for Strategy
{
  type Err = String;

  #[ inline ]
  fn from_str( s : &str ) -> Result< Self, Self::Err >
  {
    match s
    {
      "resume" => Ok( Strategy::Resume ),
      "fresh" => Ok( Strategy::Fresh ),
      _ => Err( format!( "Invalid strategy: '{s}'. Use 'resume' or 'fresh'" ) ),
    }
  }
}

/// Session manager for Claude Code invocation directories.
///
/// Manages directory-based session isolation. Creates `-{name}/` directories
/// within a configured base directory. Does NOT touch `~/.claude/projects/` —
/// that is `claude_storage_core`'s domain.
#[ derive( Debug ) ]
pub struct SessionManager
{
  sessions_base_dir : PathBuf,
}

impl SessionManager
{
  /// Create session manager with sessions base directory.
  ///
  /// # Parameters
  ///
  /// - `sessions_base_dir`: Base directory for sessions (e.g., `{topic_dir}/sessions/`)
  ///
  /// # Examples
  ///
  /// ```
  /// use claude_runner_core::SessionManager;
  /// use std::path::PathBuf;
  ///
  /// let sessions_dir = PathBuf::from( "/tmp/wplan/abc123/sessions" );
  /// let mgr = SessionManager::new( &sessions_dir );
  /// ```
  #[ inline ]
  pub fn new( sessions_base_dir : impl AsRef< Path > ) -> Self
  {
    Self
    {
      sessions_base_dir : sessions_base_dir.as_ref().to_path_buf(),
    }
  }

  /// Get session directory path for a session name.
  ///
  /// Session directories are prefixed with hyphen for git exclusion:
  /// `{sessions_base_dir}/-{session_name}/`
  ///
  /// # Parameters
  ///
  /// - `session_name`: Name of the session (e.g., "debug", "default")
  ///
  /// # Returns
  ///
  /// Full path to session directory (may not exist yet)
  #[ inline ]
  #[ must_use ]
  pub fn session_dir( &self, session_name : &str ) -> PathBuf
  {
    self.sessions_base_dir
      .join( format!( "-{session_name}" ) )
  }

  /// Check if session exists using legacy storage-based detection.
  ///
  /// **DEPRECATED:** This method checks for `.claude_history` files which
  /// Claude Code v1.x created. Claude Code v2.0+ uses centralized storage
  /// at `~/.claude/projects/` instead.
  ///
  /// Use `claude_storage_core::continuation::check_continuation()` with the
  /// path returned by `session_dir()` for v2.0+ compatibility.
  ///
  /// # Legacy Behavior
  ///
  /// A session exists if:
  /// 1. Session directory exists: `-{session_name}/`
  /// 2. Claude history file exists: `-{session_name}/.claude_history`
  ///
  /// # Returns
  ///
  /// `true` if `.claude_history` file exists (Claude Code v1.x sessions only)
  #[ deprecated(
    since = "0.2.0",
    note = "Only detects Claude Code v1.x sessions. \
            Use `claude_storage_core::continuation::check_continuation()` for v2.0+ compatibility."
  ) ]
  #[ inline ]
  #[ must_use ]
  pub fn session_exists( &self, session_name : &str ) -> bool
  {
    let session_dir = self.session_dir( session_name );
    let history_file = session_dir.join( ".claude_history" );

    session_dir.exists() && history_file.exists()
  }

  /// Ensure session directory exists (idempotent).
  ///
  /// Creates or resumes session based on strategy:
  /// - `Resume`: Create if doesn't exist, keep existing if it does
  /// - `Fresh`: Delete existing session directory, create clean one
  ///
  /// # Parameters
  ///
  /// - `session_name`: Name of the session (e.g., "debug", "default")
  /// - `strategy`: Session creation strategy
  ///
  /// # Returns
  ///
  /// Path to session directory on success
  ///
  /// # Errors
  ///
  /// Returns error if filesystem operations fail
  ///
  /// # Examples
  ///
  /// ```
  /// use claude_runner_core::{ SessionManager, Strategy };
  /// use std::path::PathBuf;
  ///
  /// let sessions_dir = PathBuf::from( "/tmp/test/sessions" );
  /// let mgr = SessionManager::new( &sessions_dir );
  ///
  /// // Resume (create if needed)
  /// let dir = mgr.ensure_session( "default", Strategy::Resume ).unwrap();
  ///
  /// // Fresh (delete and recreate)
  /// let dir = mgr.ensure_session( "default", Strategy::Fresh ).unwrap();
  /// ```
  #[ inline ]
  pub fn ensure_session(
    &self,
    session_name : &str,
    strategy : Strategy,
  ) -> Result< PathBuf, std::io::Error >
  {
    let session_dir = self.session_dir( session_name );

    match strategy
    {
      Strategy::Fresh if session_dir.exists() =>
      {
        // Delete existing local session directory
        std::fs::remove_dir_all( &session_dir )?;
      }
      _ => {}
    }

    // Create directory (idempotent)
    std::fs::create_dir_all( &session_dir )?;

    Ok( session_dir )
  }

  /// Get base sessions directory.
  #[ inline ]
  #[ must_use ]
  pub fn sessions_base_dir( &self ) -> &Path
  {
    &self.sessions_base_dir
  }
}
