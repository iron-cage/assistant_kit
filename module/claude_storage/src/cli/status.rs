//! `.status` command — storage statistics overview.

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use claude_storage_core::Storage;
use super::storage::{ create_storage, validate_verbosity, resolve_path_parameter };

/// Show storage status and statistics
///
/// Displays comprehensive information about Claude Code storage including
/// project counts, session counts, token usage, and storage location.
///
/// # Errors
///
/// Returns error if verbosity is out of range, path resolution fails,
/// storage creation fails, or statistics retrieval fails.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );
  validate_verbosity( verbosity )?;

  let custom_path = cmd.get_string( "path" );

  // Fix(issue-014): Resolve path parameter before using
  //
  // Root cause: status_routine passed path directly to Storage::with_root() without
  // resolving special markers (".", "..", "~"), unlike list_routine which uses
  // resolve_path_parameter().
  //
  // Pitfall: Inconsistent parameter handling across commands leads to confusing UX
  // where the same parameter format works in one command but not another.
  let resolved_path = custom_path
    .map( | path | resolve_path_parameter( path )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to resolve path '{path}': {e}" ) ) )
    )
    .transpose()?;

  // Create storage instance
  let storage = if let Some( path ) = resolved_path
  {
    Storage::with_root( &path )
  }
  else
  {
    create_storage()?
  };

  // Fix(issue-015): Use fast stats (filesystem-only) for verbosity 0-1; full
  // JSONL-parsing stats only for verbosity 2+ which the user explicitly requests.
  //
  // Root cause: global_stats() parsed all session JSONL files to count entries and
  // tokens. With 1903 projects / 2449 sessions / 7 GB of JSONL this took >2 minutes,
  // making .status completely unusable at the default verbosity level.
  //
  // Pitfall: Never call global_stats() for a command that only needs project/session
  // counts — the entry/token parsing is O(total JSONL bytes), not O(project count).

  let output = if verbosity <= 1
  {
    // Fast path: filesystem listing only — no JSONL parsing, completes in < 1 second
    let stats = storage.global_stats_fast()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get statistics: {e}" ) ) )?;

    match verbosity
    {
      0 =>
      {
        // Minimal: just project count
        format!( "Projects: {}", stats.total_projects )
      }
      _ =>
      {
        // Standard: storage + project + session overview (no entry count — fast path)
        format!
        (
          "Storage: {}\nProjects: {} (UUID: {}, Path: {})\nSessions: {} (Main: {}, Agent: {})",
          storage.root().display(),
          stats.total_projects,
          stats.uuid_projects,
          stats.path_projects,
          stats.total_sessions,
          stats.main_sessions,
          stats.agent_sessions,
        )
      }
    }
  }
  else
  {
    // Full path: parses all JSONL files for entry counts and token usage.
    // Slow for large storage (minutes for thousands of sessions) but gives complete stats.
    let stats = storage.global_stats()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get statistics: {e}" ) ) )?;

    // Detailed: include entry breakdown and token usage
    format!
    (
      "Storage: {}\n\
      Projects: {} (UUID: {}, Path: {})\n\
      Sessions: {} (Main: {}, Agent: {})\n\
      Entries: {} (User: {}, Assistant: {})\n\
      Tokens:\n\
      - Input: {}\n\
      - Output: {}\n\
      - Cache Read: {}\n\
      - Cache Creation: {}",
      storage.root().display(),
      stats.total_projects,
      stats.uuid_projects,
      stats.path_projects,
      stats.total_sessions,
      stats.main_sessions,
      stats.agent_sessions,
      stats.total_entries,
      stats.total_user_entries,
      stats.total_assistant_entries,
      stats.total_input_tokens,
      stats.total_output_tokens,
      stats.total_cache_read_tokens,
      stats.total_cache_creation_tokens
    )
  };

  Ok( OutputData::new( output, "text" ) )
}
