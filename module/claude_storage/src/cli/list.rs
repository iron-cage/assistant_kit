//! `.list` command — list projects and their sessions.

use core::fmt::Write as FmtWrite;
use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::{ create_storage, validate_verbosity, resolve_path_parameter, load_project_for_param };
use super::projects::{ build_families, group_into_conversations };

/// List projects or sessions
///
/// Lists projects in Claude Code storage, with optional filtering by type.
///
/// Smart session display:
/// - Providing session filters (`session::`, `agent::`, `min_entries::`) auto-enables session display
/// - `sessions::1` always enables session display
/// - `sessions::0` has no effect when session filters are also active — filters win
/// - No filters → Projects only (default behavior)
///
/// Examples:
/// ```bash
/// # Projects only (no sessions)
/// .list
///
/// # Auto-enable sessions (filter provided)
/// .list session::commit
///
/// # sessions::0 ignored when filters active — sessions still shown
/// .list sessions::0 session::commit
/// ```
///
/// # Errors
///
/// Returns error if `min_entries` is negative, path resolution fails,
/// project type is invalid, storage creation fails, or listing projects fails.
#[allow(clippy::too_many_lines)]
// CLI routine handler processes many parameters and verbosity branches — extraction
// would obscure the command's logic without reducing complexity.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn list_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let project_type = cmd.get_string( "type" ).unwrap_or( "all" );

  // Early dispatch: conversation listing requires project:: and is handled separately.
  if project_type == "conversation"
  {
    let proj_id = cmd.get_string( "project" )
      .ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "project parameter required for listing conversations".to_string(),
      ) )?;
    let storage = create_storage()?;
    let project = load_project_for_param( &storage, proj_id )?;
    let sessions = project.all_sessions()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load sessions: {e}" ) ) )?;
    let families = build_families( sessions );
    let conversations = group_into_conversations( families );
    let count_mode = cmd.get_boolean( "count" ).unwrap_or( false );
    if count_mode
    {
      return Ok( OutputData::new( format!( "{}", conversations.len() ), "text" ) );
    }
    let mut out = String::new();
    for conv in &conversations
    {
      if let Some( s ) = conv.root_session()
      {
        writeln!( out, "{}", s.id() ).unwrap();
      }
    }
    return Ok( OutputData::new( out, "text" ) );
  }

  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );

  // Fix(issue-015): Validate verbosity range
  //
  // Root cause: list_routine retrieved verbosity without range validation, unlike
  // status_routine and show_routine which include explicit 0-5 checks. Values like
  // -1 or 10 were silently accepted and used, producing undefined output behavior.
  //
  // Pitfall: get_integer().unwrap_or(default) only substitutes the default when the
  // parameter is absent. An explicit out-of-range value is returned as-is. Range
  // validation is always the caller's responsibility, even when a default is set.
  validate_verbosity( verbosity )?;

  // Parse filter parameters
  let path_filter = cmd.get_string( "path" );
  let agent_filter = cmd.get_boolean( "agent" );

  // Validate and parse min_entries (must be non-negative)
  let min_entries_filter = if let Some( n ) = cmd.get_integer( "min_entries" )
  {
    if n < 0
    {
      return Err
      (
        ErrorData::new
        (
          ErrorCode::InternalError,
          format!( "Invalid min_entries: {n}. Must be non-negative" )
        )
      );
    }
    // min_entries validated as non-negative above, so cast is safe
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Some( n as usize )
  }
  else
  {
    None
  };

  let session_id_filter = cmd.get_string( "session" );

  // Fix(issue-002): Smart path resolution for path:: parameter
  //
  // Root cause: The path:: parameter used literal substring matching only.
  // When users provided path::., it searched for paths containing a literal "."
  // character instead of resolving "." to the current working directory.
  // This violated user expectations from shell semantics where . means "current
  // directory", .. means "parent directory", and ~ means "home directory".
  //
  // Pitfall: When implementing filters that accept both patterns and paths,
  // clearly define detection logic. Ambiguous cases (like .) should prioritize
  // user expectations over literal interpretation. Support shell semantics for
  // special characters (., .., ~) in all filesystem path parameters.

  // Resolve path parameter with smart detection
  let path_filter = if let Some( param ) = path_filter
  {
    match resolve_path_parameter( param )
    {
      Ok( resolved ) => Some( resolved ),
      Err( e ) =>
      {
        return Err
        (
          ErrorData::new
          (
            ErrorCode::InternalError,
            format!( "Failed to resolve path parameter '{}': {}", &param, e )
          )
        );
      }
    }
  }
  else
  {
    None
  };

  // Fix(issue-001): Smart session display - auto-enable when filters provided
  //
  // Root cause: `show_sessions` defaulted to false, blocking filter usage even when
  // session filters were provided. This made session::, agent::, and min_entries::
  // parameters "garbage" - accepted by parser but silently ignored by implementation.
  //
  // Pitfall: Garbage parameters create silent failures that waste user time. Users try
  // different parameter values but see no effect because the filter is built but never
  // used. ALWAYS trace parameter flow: parser → filter build → filter usage. If usage
  // is conditional on default-false flag, parameter is garbage.

  // Smart parameter detection: Auto-enable session display when filters provided
  let has_session_filters = session_id_filter.is_some()
    || agent_filter.is_some()
    || min_entries_filter.is_some();

  let show_sessions = has_session_filters || cmd.get_boolean( "sessions" ).unwrap_or( false );

  // Create storage instance
  let storage = create_storage()?;

  // Fix(issue-list-hang): min_entries:: must not be placed in ProjectFilter
  //
  // Root cause: Placing min_entries in ProjectFilter caused project.matches_filter()
  // to call project_stats() for EVERY project. project_stats() reads ALL session JSONL
  // files to count entries, scanning gigabytes of data when the user has many projects.
  // This caused the binary to hang indefinitely.
  //
  // min_entries:: is semantically a SESSION filter (show sessions with ≥N entries),
  // not a project filter (show projects whose total entries ≥ N). The auto-enable
  // behavior (show_sessions=true) is handled separately at line 512-516.
  //
  // Pitfall: When a parameter auto-enables a feature, don't also apply it as a
  // project-level filter unless that filtering is the stated purpose. Trace the
  // computational cost: project_stats() = O(projects × sessions × entries).

  // Build project filter (min_entries is a session filter, not a project filter)
  let project_filter = claude_storage_core::ProjectFilter
  {
    path_substring : path_filter,
    min_entries : None,
    min_sessions : None,
  };

  // Build session filter
  let session_filter = claude_storage_core::SessionFilter
  {
    agent_only : agent_filter,
    min_entries : min_entries_filter,
    session_id_substring : session_id_filter.map( std::string::ToString::to_string ),
  };

  // Get projects based on type filter
  let mut projects = match project_type
  {
    "uuid" => storage.list_uuid_projects(),
    "path" => storage.list_path_projects(),
    "all" => storage.list_projects(),
    _ => return Err
    (
      ErrorData::new
      (
        ErrorCode::InternalError,
        format!( "Invalid type: {project_type}. Valid values: uuid, path, all" )
      )
    ),
  }
  .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;

  // Apply project-level filtering
  if !project_filter.is_default()
  {
    projects.retain( | project |
    {
      project.matches_filter( &project_filter ).unwrap_or( false )
    });
  }

  // Format output
  let mut output = String::new();

  if verbosity >= 1
  {
    let noun = if projects.len() == 1 { "project" } else { "projects" };
    writeln!( output, "Found {} {noun}:\n", projects.len() ).unwrap();
  }

  for mut project in projects
  {
    // Handle projects that may have been deleted (gracefully skip them)
    match verbosity
    {
      0 =>
      {
        // Just ID
        writeln!( output, "{:?}", project.id() ).unwrap();
      }
      1 =>
      {
        // ID + conversation count (skip if project was deleted)
        let Ok( session_count ) = project.count_sessions() else { continue };  // Skip projects that can't be read

        // Fix(issue-027): Use singular "conversation" when count == 1; plural otherwise.
        // Root cause: hardcoded "sessions" regardless of count produced "(1 sessions)".
        // Pitfall: same pattern as issue-025 — always derive noun from count, never hardcode.
        let noun = if session_count == 1 { "conversation" } else { "conversations" };
        writeln!( output, "{:?} ({session_count} {noun})", project.id() ).unwrap();
      }
      _ =>
      {
        // Full details (skip if project was deleted)
        let Ok( project_stats ) = project.project_stats() else { continue };  // Skip projects that can't be read

        write!
        (
          output,
          "{:?}\n  Sessions: {} (Main: {}, Agent: {})\n  Entries: {}\n  Tokens: {} in, {} out\n\n",
          project.id(),
          project_stats.session_count,
          project_stats.main_session_count,
          project_stats.agent_session_count,
          project_stats.total_entries,
          project_stats.total_input_tokens,
          project_stats.total_output_tokens
        ).unwrap();
      }
    }

    // Show sessions if requested (skip if project was deleted)
    if show_sessions
    {
      let sessions = if session_filter.is_default()
      {
        match project.sessions()
        {
          Ok( s ) => s,
          Err( _ ) => continue,  // Skip if can't read sessions
        }
      }
      else
      {
        match project.sessions_filtered( &session_filter )
        {
          Ok( s ) => s,
          Err( _ ) => continue,  // Skip if can't read sessions
        }
      };

      for session in sessions
      {
        writeln!( output, "  - {}", session.id() ).unwrap();
      }
    }
  }

  Ok( OutputData::new( output, "text" ) )
}
