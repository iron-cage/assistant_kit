//! `.status` command — storage statistics overview.

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use claude_storage_core::Storage;
use super::storage::{ create_storage, resolve_path_parameter };

/// Show storage status and statistics
///
/// Displays comprehensive information about Claude Code storage including
/// project counts, session counts, token usage, and storage location.
///
/// # Errors
///
/// Returns error if path resolution fails, storage creation fails, or
/// statistics retrieval fails.
#[ allow( clippy::needless_pass_by_value ) ]
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
pub fn status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let show_tokens = cmd.get_boolean( "show_tokens" ).unwrap_or( false );
  let custom_path = cmd.get_string( "path" );

  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );
  if !( 0..=5 ).contains( &verbosity )
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "Invalid verbosity: {verbosity}. Valid range: 0-5" ),
    ) );
  }

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

  // Exit code 2 when the storage root path does not exist (not found = usage error, not command error)
  if let Some( ref path ) = resolved_path
  {
    if !std::path::Path::new( path ).exists()
    {
      eprintln!( "Storage root does not exist: {path}" );
      std::process::exit( 2 );
    }
  }

  // Create storage instance
  let storage = if let Some( path ) = resolved_path
  {
    Storage::with_root( &path )
  }
  else
  {
    create_storage()?
  };

  // Fix(issue-015): Use fast stats (filesystem-only) by default; full
  // JSONL-parsing stats only when show_tokens::1 is explicitly requested.
  //
  // Root cause: global_stats() parsed all session JSONL files to count entries and
  // tokens. With 1903 projects / 2449 sessions / 7 GB of JSONL this took >2 minutes,
  // making .status completely unusable at the default invocation.
  //
  // Pitfall: Never call global_stats() for a command that only needs project/session
  // counts — the entry/token parsing is O(total JSONL bytes), not O(project count).

  // verbosity::0 — compact machine-readable key-value format (no decorations)
  if verbosity == 0
  {
    let stats = storage.global_stats_fast()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get statistics: {e}" ) ) )?;
    let output = format!( "projects: {}\nsessions: {}\n", stats.total_projects, stats.total_sessions );
    return Ok( OutputData::new( output, "text" ) );
  }

  let output = if show_tokens
  {
    // Full path: parses all JSONL files for entry counts and token usage.
    // Slow for large storage (minutes for thousands of sessions) but gives complete stats.
    let stats = storage.global_stats()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get statistics: {e}" ) ) )?;

    // Detailed: include entry breakdown and token usage (show_tokens::1)
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
  }
  else
  {
    // Fast path: filesystem listing only — no JSONL parsing, completes in < 1 second
    let stats = storage.global_stats_fast()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get statistics: {e}" ) ) )?;

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
  };

  // verbosity::2+ — append per-project breakdown with user/assistant entry counts
  if verbosity >= 2
  {
    use core::fmt::Write as FmtWrite;
    let mut detail = output;
    detail.push_str( "\n\nPer-project sessions:\n" );
    let projects = storage.list_projects()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;
    for project in projects
    {
      if let Ok( stats ) = project.project_stats()
      {
        writeln!(
          detail,
          "  {:?}: {} sessions, User: {}, Assistant: {}",
          project.id(),
          stats.session_count,
          stats.total_user_entries,
          stats.total_assistant_entries,
        ).ok();
      }
      else
      {
        let count = project.count_sessions().unwrap_or( 0 );
        writeln!( detail, "  {:?}: {count} sessions", project.id() ).ok();
      }
    }
    return Ok( OutputData::new( detail, "text" ) );
  }

  Ok( OutputData::new( output, "text" ) )
}
