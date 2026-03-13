//! Continuation detection for Claude Code sessions.
//!
//! # Design
//!
//! Claude Code v2.0+ tracks conversations by EXECUTION DIRECTORY.
//! When Claude runs from `/path/to/-session/`, it stores conversation files in
//! `~/.claude/projects/{encoded}/` where `{encoded}` is the v1 lossy encoding
//! of the execution directory path (see [`encode_path`][crate::encode_path]).
//!
//! # Examples
//!
//! ```no_run
//! use claude_storage_core::continuation;
//! use std::path::PathBuf;
//!
//! let session_dir = PathBuf::from( "/tmp/my-session" );
//! if continuation::check_continuation( &session_dir )
//! {
//!   println!( "Session has conversation history — resuming" );
//! }
//! else
//! {
//!   println!( "No history — starting fresh" );
//! }
//! ```

use std::path::{ Path, PathBuf };
use crate::encode_path;

/// Compute the Claude storage path for a given execution directory.
///
/// Claude Code stores conversations at `~/.claude/projects/{encoded}/`
/// where `{encoded}` is the v1 lossy encoding of the execution directory path.
///
/// # Parameters
///
/// - `session_dir`: Execution directory to compute storage path for
///
/// # Returns
///
/// `Some(PathBuf)` with the `~/.claude/projects/{encoded}/` path,
/// or `None` if `HOME` is not set or the path cannot be encoded
/// (empty path, invalid UTF-8).
///
/// # Examples
///
/// ```no_run
/// use claude_storage_core::continuation;
/// use std::path::PathBuf;
///
/// let session_dir = PathBuf::from( "/home/user/project/-debug" );
/// if let Some( storage ) = continuation::to_storage_path_for( &session_dir )
/// {
///   println!( "Claude stores sessions at: {}", storage.display() );
/// }
/// ```
#[ inline ]
#[ must_use ]
pub fn to_storage_path_for( session_dir : &Path ) -> Option< PathBuf >
{
  let home_dir = std::env::var( "HOME" ).ok()?;
  let encoded = encode_path( session_dir ).ok()?;
  Some
  (
    PathBuf::from( home_dir )
      .join( ".claude" )
      .join( "projects" )
      .join( encoded )
  )
}

/// Check if Claude storage contains conversation history for an execution directory.
///
/// Returns `true` if `~/.claude/projects/{encoded}/` exists and contains at
/// least one non-empty, non-agent conversation file.
///
/// # Conversation File Detection
///
/// A file counts as conversation history if it:
/// - Has a `.jsonl` extension (case-insensitive)
/// - Is named `conversation.json`
/// - Starts with `.claude`
///
/// Excluded from detection:
/// - `agent-*.jsonl` — agent definition files, not user conversations
/// - Empty files (0 bytes) — created by Claude Code during initialization or crash
///
/// # Parameters
///
/// - `session_dir`: Execution directory to check for conversation history
///
/// # Returns
///
/// `true` if at least one valid conversation file exists
///
/// # Examples
///
/// ```no_run
/// use claude_storage_core::continuation;
/// use std::path::PathBuf;
///
/// let session_dir = PathBuf::from( "/home/user/project/-debug" );
/// if continuation::check_continuation( &session_dir )
/// {
///   println!( "Has conversation history" );
/// }
/// ```
#[ inline ]
#[ must_use ]
pub fn check_continuation( session_dir : &Path ) -> bool
{
  let Some( storage_path ) = to_storage_path_for( session_dir ) else
  {
    return false;
  };

  if !storage_path.exists()
  {
    return false;
  }

  if let Ok( entries ) = std::fs::read_dir( &storage_path )
  {
    for entry in entries.flatten()
    {
      if let Some( filename ) = entry.file_name().to_str()
      {
        // Skip agent definition files (not user conversations)
        if filename.starts_with( "agent-" )
        {
          continue;
        }

        // Fix(issue-wplan-empty-session-file): Skip empty files in continuation detection
        //
        // Root cause: Claude Code creates 0-byte .jsonl files during initialization.
        // When Claude crashes before writing content, these 0-byte files remain and
        // would otherwise produce false-positive continuation detection.
        //
        // Pitfall: File existence alone does not guarantee valid session state.
        // Always check file size — empty files must be skipped.
        if let Ok( metadata ) = entry.metadata()
        {
          if metadata.len() == 0
          {
            continue;
          }
        }

        if Path::new( filename )
            .extension()
            .is_some_and( | ext | ext.eq_ignore_ascii_case( "jsonl" ) )
          || filename == "conversation.json"
          || filename.starts_with( ".claude" )
        {
          return true;
        }
      }
    }
  }

  false
}
