//! Filesystem locations the daemon owns.
//!
//! Runtime state lives under a hyphen-prefixed directory so it is git-ignored by
//! the workspace's global `-*` rule — these files are machine-local and must
//! never be committed.

use std::path::{ Path, PathBuf };

use claude_core::ClaudePaths;

/// Directory holding the daemon's runtime state, relative to a base directory.
pub const RUNTIME_DIR_NAME : &str = "-daemon";

/// Filename of the single-instance lock.
pub const LOCK_FILE_NAME : &str = "instance.lock";

/// Filename of the daemon's listening socket.
pub const SOCKET_FILE_NAME : &str = "daemon.sock";

/// Filename the daemon's own output is appended to once it detaches.
pub const LOG_FILE_NAME : &str = "daemon.log";

/// Resolved locations for one daemon instance.
#[ derive( Debug, Clone ) ]
pub struct DaemonPaths
{
  runtime_dir : PathBuf,
  sessions_dir : PathBuf,
}

impl DaemonPaths
{
  /// Resolve from the real `~/.claude` location.
  ///
  /// Returns `None` when `HOME` is not set, matching [`ClaudePaths::new`] —
  /// which reads `HOME` and nothing else.
  #[ inline ]
  #[ must_use ]
  pub fn new() -> Option< Self >
  {
    ClaudePaths::new().map( | p | Self::from_claude_paths( &p ) )
  }

  /// Resolve against an explicit Claude home — the form tests use.
  #[ inline ]
  #[ must_use ]
  pub fn with_home( home : &Path ) -> Self
  {
    Self::from_claude_paths( &ClaudePaths::with_home( home ) )
  }

  fn from_claude_paths( paths : &ClaudePaths ) -> Self
  {
    Self
    {
      runtime_dir : paths.base().join( RUNTIME_DIR_NAME ),
      sessions_dir : paths.sessions_dir(),
    }
  }

  /// Directory holding this daemon's runtime state.
  #[ inline ]
  #[ must_use ]
  pub fn runtime_dir( &self ) -> &Path
  {
    &self.runtime_dir
  }

  /// Path of the single-instance lock file.
  #[ inline ]
  #[ must_use ]
  pub fn lock_file( &self ) -> PathBuf
  {
    self.runtime_dir.join( LOCK_FILE_NAME )
  }

  /// Path of the daemon's listening socket.
  #[ inline ]
  #[ must_use ]
  pub fn socket_file( &self ) -> PathBuf
  {
    self.runtime_dir.join( SOCKET_FILE_NAME )
  }

  /// Path of the daemon's log file.
  ///
  /// A detached daemon has no terminal to complain to. Whoever starts it points
  /// its stdout and stderr here instead, appending rather than truncating — the
  /// interesting case is a daemon that keeps dying at startup, and truncating
  /// would erase the evidence on every restart.
  #[ inline ]
  #[ must_use ]
  pub fn log_file( &self ) -> PathBuf
  {
    self.runtime_dir.join( LOG_FILE_NAME )
  }

  /// Claude Code's live-session registry directory.
  ///
  /// Passed to `claude_session_core`, which takes the directory as a parameter
  /// rather than resolving it itself.
  #[ inline ]
  #[ must_use ]
  pub fn sessions_dir( &self ) -> &Path
  {
    &self.sessions_dir
  }
}
