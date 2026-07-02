//! Session path resolution — `ClaudeScope` and `scope_for()`.
//!
//! Computes all 6 `CLAUDE_*` path variables for a target directory in a
//! single call, honouring `CLAUDE_HOME` and `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE`
//! environment variable overrides.
//!
//! # `CLAUDE_HOME` Override Semantics
//!
//! When `CLAUDE_HOME` is set, its value is used **directly** as `claude_home` —
//! `.claude` is NOT appended.  Only the `$HOME` fallback appends `.claude`.
//! Appending `.claude` to an already-resolved `CLAUDE_HOME` is the double-`.claude`
//! defect; this module explicitly avoids it.

use std::path::{ Path, PathBuf };
use crate::{ encode_path, most_recent_session_in_dir };

/// All 6 `CLAUDE_*` path variables for a target directory.
///
/// Produced by [`scope_for`].  All fields are fully-resolved `PathBuf` values;
/// `claude_session_file` is `None` when no qualifying `.jsonl` exists in storage.
#[ derive( Debug, Clone ) ]
pub struct ClaudeScope
{
  /// `${CLAUDE_HOME}` or `$HOME/.claude` — Claude Code's home directory.
  pub claude_home         : PathBuf,
  /// `claude_home/projects` — root of all project-scoped storage.
  pub claude_projects_dir : PathBuf,
  /// `claude_projects_dir/Df(dir)` — session storage for the target directory.
  pub claude_session_dir  : PathBuf,
  /// Memory directory anchored to git root, or `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE`.
  pub claude_memory_dir   : PathBuf,
  /// `claude_memory_dir/MEMORY.md` — canonical memory file.
  pub claude_memory_file  : PathBuf,
  /// Most-recent `.jsonl` session in `claude_session_dir`; `None` when absent.
  pub claude_session_file : Option< PathBuf >,
}

/// Walk up from `dir` looking for a `.git` entry (file or directory).
///
/// Returns the first ancestor directory that contains `.git`, or `dir` itself
/// when no `.git` is found (non-git directory or filesystem root reached).
///
/// The `.git` entry may be a directory (standard clone) or a file (git worktree).
#[ inline ]
#[ must_use ]
pub fn git_root_for( dir : &Path ) -> PathBuf
{
  let mut current = dir.to_path_buf();
  loop
  {
    if current.join( ".git" ).exists()
    {
      return current;
    }
    match current.parent()
    {
      Some( parent ) => current = parent.to_path_buf(),
      None           => return dir.to_path_buf(),
    }
  }
}

/// Compute all 6 `CLAUDE_*` path variables for `dir`.
///
/// # Resolution order
///
/// | Variable | Source |
/// |----------|--------|
/// | `claude_home` | `$CLAUDE_HOME` (direct) or `$HOME` + `/.claude` |
/// | `claude_projects_dir` | `claude_home/projects` |
/// | `claude_session_dir` | `claude_projects_dir/Df(dir)` |
/// | `claude_memory_dir` | `$CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` or `claude_projects_dir/Df(git_root_for(dir))/memory` |
/// | `claude_memory_file` | `claude_memory_dir/MEMORY.md` |
/// | `claude_session_file` | most-recent `.jsonl` in `claude_session_dir`; `None` if absent |
///
/// # `CLAUDE_HOME` semantics
///
/// When `CLAUDE_HOME` is set, it is the entire `claude_home` — `.claude` is not
/// appended.  This is the documented override contract; appending `.claude` would
/// produce a double-suffix defect.
#[ inline ]
#[ must_use ]
pub fn scope_for( dir : &Path ) -> ClaudeScope
{
  // CLAUDE_HOME: when set, use the value directly — do NOT append ".claude".
  // Fallback: $HOME + "/.claude" (standard Claude Code layout).
  let claude_home = std::env::var( "CLAUDE_HOME" )
    .map_or_else(
      |_|
      {
        let home = std::env::var( "HOME" ).unwrap_or_else( |_| ".".to_string() );
        PathBuf::from( home ).join( ".claude" )
      },
      PathBuf::from,
    );

  let claude_projects_dir = claude_home.join( "projects" );

  // Session dir: projects/<Df(dir)>
  let session_encoded = encode_path( dir ).unwrap_or_else( |_| "-unknown".to_string() );
  let claude_session_dir = claude_projects_dir.join( &session_encoded );

  // Memory dir: CLAUDE_COWORK_MEMORY_PATH_OVERRIDE takes priority.
  // Fallback: projects/<Df(git_root_for(dir))>/memory — anchored to git root.
  let claude_memory_dir = std::env::var( "CLAUDE_COWORK_MEMORY_PATH_OVERRIDE" )
    .map_or_else(
      |_|
      {
        let git_root       = git_root_for( dir );
        let memory_encoded = encode_path( &git_root ).unwrap_or_else( |_| "-unknown".to_string() );
        claude_projects_dir.join( &memory_encoded ).join( "memory" )
      },
      PathBuf::from,
    );

  let claude_memory_file = claude_memory_dir.join( "MEMORY.md" );

  // Session file: convert SessionId stem to full .jsonl path.
  let claude_session_file = most_recent_session_in_dir( &claude_session_dir )
    .map( |id| claude_session_dir.join( format!( "{}.jsonl", id.as_str() ) ) );

  ClaudeScope
  {
    claude_home,
    claude_projects_dir,
    claude_session_dir,
    claude_memory_dir,
    claude_memory_file,
    claude_session_file,
  }
}
