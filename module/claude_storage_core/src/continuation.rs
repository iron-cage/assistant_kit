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
use crate::{ encode_path, SessionId };

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

/// Return the `SessionId` of the most-recently-modified qualifying `.jsonl` session
/// file in a Claude storage directory.
///
/// A file qualifies when it has a `.jsonl` extension, does not start with `agent-`,
/// and is non-empty (> 0 bytes).  Returns the UUID stem (filename without `.jsonl`)
/// as a [`SessionId`].
///
/// Returns `None` if the directory does not exist, cannot be read, or contains no
/// qualifying files.
///
/// This is a low-level primitive for callers that already have the encoded storage
/// path.  Most callers should use [`most_recent_session_id`], which handles the
/// CWD-encoding step.
///
/// # Parameters
///
/// - `storage_path`: The `~/.claude/projects/{encoded}/` directory to scan.
///
/// # Examples
///
/// ```no_run
/// use claude_storage_core::continuation;
/// use std::path::PathBuf;
///
/// let storage = PathBuf::from( "/home/user/.claude/projects/-home-user-project" );
/// if let Some( id ) = continuation::most_recent_session_in_dir( &storage )
/// {
///   println!( "Most recent session UUID: {}", id.as_str() );
/// }
/// ```
#[ inline ]
#[ must_use ]
pub fn most_recent_session_in_dir( storage_path : &Path ) -> Option< SessionId >
{
  let entries = std::fs::read_dir( storage_path ).ok()?;
  let mut best : Option< ( std::time::SystemTime, String ) > = None;

  for entry in entries.flatten()
  {
    let name = entry.file_name();
    let Some( filename ) = name.to_str() else { continue };

    if filename.starts_with( "agent-" ) { continue; }

    if !Path::new( filename )
      .extension()
      .is_some_and( | ext | ext.eq_ignore_ascii_case( "jsonl" ) )
    {
      continue;
    }

    let Ok( meta ) = entry.metadata() else { continue };
    if meta.len() == 0 { continue; }

    let Some( stem ) = Path::new( filename )
      .file_stem()
      .and_then( | s | s.to_str() )
      .map( str::to_owned )
    else { continue };

    let mtime = meta.modified().ok();
    match ( &best, mtime )
    {
      ( None, Some( t ) ) => best = Some( ( t, stem ) ),
      ( Some( ( prev_t, _ ) ), Some( t ) ) if t > *prev_t => best = Some( ( t, stem ) ),
      _ => {}
    }
  }

  best.map( | ( _, stem ) | SessionId::new( stem ) )
}

/// Return the `SessionId` of the most-recently-modified qualifying session file for
/// an execution directory.
///
/// Encodes `session_dir` to its Claude storage path via [`to_storage_path_for`],
/// then delegates to [`most_recent_session_in_dir`].
///
/// Returns `None` if `HOME` is unset, if the path cannot be encoded, if the storage
/// directory does not exist, or if no qualifying `.jsonl` session files are found.
///
/// # Parameters
///
/// - `session_dir`: Execution directory whose Claude storage should be scanned.
///
/// # Examples
///
/// ```no_run
/// use claude_storage_core::continuation;
/// use std::path::PathBuf;
///
/// let session_dir = PathBuf::from( "/home/user/project/-debug" );
/// if let Some( id ) = continuation::most_recent_session_id( &session_dir )
/// {
///   println!( "Expected session UUID: {}", id.as_str() );
/// }
/// ```
#[ inline ]
#[ must_use ]
pub fn most_recent_session_id( session_dir : &Path ) -> Option< SessionId >
{
  to_storage_path_for( session_dir ).and_then( | p | most_recent_session_in_dir( &p ) )
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
// Fix(BUG-320): For callers that need the session UUID (not just a bool), use
// `most_recent_session_id(session_dir)` — the typed companion API that returns
// `Option<SessionId>` for `.jsonl`-based session files.  This function is kept
// for backward compatibility and also detects legacy `conversation.json` and
// `.claude*` formats that `most_recent_session_id` intentionally excludes.
//
// Root cause: `check_continuation` returns only `bool`, making the UUID inaccessible
//   to callers that need to verify which session will be resumed by `claude -c`.
// Pitfall: do not replace this function with `most_recent_session_id().is_some()` —
//   that would silently drop detection of legacy file formats and break callers
//   relying on `conversation.json` / `.claude*` continuation detection.
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

        // Fix(issue-consumer_runner-empty-session-file): Skip empty files in continuation detection
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
