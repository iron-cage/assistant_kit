//! `.show` command — display session or project details.

use core::fmt::Write as FmtWrite;
use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::{ create_storage, validate_verbosity, parse_project_parameter, find_session_mut };
use super::format::format_entry_content;

/// Show session or project details (location-aware)
///
/// Smart behavior based on parameters:
/// - No parameters → Show current directory project (all sessions)
/// - `session_id` only → Show that session in current project
/// - project only → Show that project (all sessions)
/// - Both parameters → Show that session in that project
///
/// # Errors
///
/// Returns error if verbosity is out of range, parameter combinations are
/// invalid, storage creation fails, or project/session loading fails.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn show_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let session_id = cmd.get_string( "session_id" );
  let project_param = cmd.get_string( "project" );
  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );
  let show_entries = cmd.get_boolean( "entries" ).unwrap_or( false );
  let metadata_only = cmd.get_boolean( "metadata" ).unwrap_or( false );

  validate_verbosity( verbosity )?;

  // Fix(issue-001): Validate entries parameter requires session_id
  //
  // Root cause: entries parameter was accepted and parsed but silently ignored
  // when displaying projects (cases 1 and 3). This created a "garbage parameter"
  // that users could pass but had no effect, wasting debugging time.
  //
  // Pitfall: Always validate parameter compatibility. If parameter P only works
  // with parameter Q, reject the combination where P is set but Q is not.
  // Silent ignoring of valid-syntax parameters creates user frustration.
  if show_entries && session_id.is_none()
  {
    return Err
    (
      ErrorData::new
      (
        ErrorCode::InternalError,
        "Parameter 'entries' requires 'session_id'. \
         Use '.show session_id::<id> entries::1' to display session entries."
          .to_string()
      )
    );
  }

  // Fix(issue-022): Accept entries::1 in content mode as a no-op
  //
  // Root cause: A prior "fix" (issue-008) added an error when entries::1 was used
  // in content mode (verbosity >= 1 && !metadata_only), intending to prevent a
  // "garbage parameter" scenario. However, the YAML spec explicitly lists
  // `.show session_id::abc123 entries::1` as a valid example (example 6), and
  // content mode already shows all entries by default — entries::1 is a valid
  // no-op in this context.
  //
  // Pitfall: Don't add errors for parameters whose spec examples show them working
  // standalone. A no-op is preferable to an error when the parameter has no
  // additional effect in the current mode. Errors should be reserved for truly
  // incompatible combinations, not for parameters that are simply redundant.

  // Smart parameter detection (4 cases)
  match ( session_id, project_param )
  {
    // Case 1: No parameters → Show current directory project
    ( None, None ) =>
    {
      show_project_for_cwd_impl( verbosity )
    }

    // Case 2: session_id only → Show session in current project
    ( Some( sid ), None ) =>
    {
      show_session_in_cwd_impl( sid, verbosity, show_entries, metadata_only )
    }

    // Case 3: project only → Show that project
    ( None, Some( proj ) ) =>
    {
      show_project_impl( proj, verbosity )
    }

    // Case 4: Both parameters → Show session in that project
    ( Some( sid ), Some( proj ) ) =>
    {
      show_session_in_project_impl( sid, proj, verbosity, show_entries, metadata_only )
    }
  }
}

/// Helper: Show session in current directory project
fn show_session_in_cwd_impl(
  session_id : &str,
  verbosity : i64,
  show_entries : bool,
  metadata_only : bool
) -> core::result::Result< OutputData, ErrorData >
{
  // Fix(issue-036)
  // Root cause: load_project_for_cwd() only matches the exact encoded base path, so sessions
  //   stored under topic project dirs ({base}--commit, {base}--default-topic) were invisible
  //   when running .show from the corresponding working directory.
  // Pitfall: Use double-hyphen ({eb}--) not single ({eb}-) for the topic prefix predicate;
  //   single-hyphen would falsely match sibling directories sharing a common prefix.
  let storage = create_storage()?;

  let cwd = std::env::current_dir()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get current directory: {e}" ) ) )?;

  let eb = claude_storage_core::encode_path( &cwd )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to encode current directory: {e}" ) ) )?;

  let topic_prefix = format!( "{eb}--" );

  let all_projects = storage.list_projects()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;

  for project in &all_projects
  {
    let dir_name = project.storage_dir()
      .file_name()
      .and_then( | n | n.to_str() )
      .unwrap_or( "" );

    if dir_name != eb && !dir_name.starts_with( &topic_prefix ) { continue; }

    if let Ok( output ) = format_session_output( project, session_id, verbosity, show_entries, metadata_only )
    {
      return Ok( output );
    }
  }

  Err( ErrorData::new( ErrorCode::InternalError, format!( "Session '{session_id}' not found in current directory projects" ) ) )
}

/// Helper: Show session in specific project
fn show_session_in_project_impl(
  session_id : &str,
  project_param : &str,
  verbosity : i64,
  show_entries : bool,
  metadata_only : bool
) -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  // Parse project parameter
  let proj_id = parse_project_parameter( project_param )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Invalid project parameter: {e}" )
    ))?;

  let project = storage.load_project( &proj_id )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to load project {proj_id:?}: {e}" )
    ))?;

  format_session_output( &project, session_id, verbosity, show_entries, metadata_only )
}

/// Helper: Show project for current directory
fn show_project_for_cwd_impl( verbosity : i64 )
  -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  let project = storage.load_project_for_cwd()
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to load project from current directory: {e}" )
    ))?;

  format_project_output( &project, verbosity )
}

/// Helper: Show specific project
fn show_project_impl( project_param : &str, verbosity : i64 )
  -> core::result::Result< OutputData, ErrorData >
{
  let storage = create_storage()?;

  // Parse project parameter
  let proj_id = parse_project_parameter( project_param )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Invalid project parameter: {e}" )
    ))?;

  let project = storage.load_project( &proj_id )
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to load project {proj_id:?}: {e}" )
    ))?;

  format_project_output( &project, verbosity )
}

/// Helper: Format session output (extracted logic)
///
/// REQ-011: Content-First Display
///
/// By default (`verbosity::1`), shows conversation content in readable chat-log format.
/// Use `metadata::1` or `verbosity::0` for old metadata-only behavior.
fn format_session_output(
  project : &claude_storage_core::Project,
  session_id : &str,
  verbosity : i64,
  show_entries : bool,
  metadata_only : bool
) -> core::result::Result< OutputData, ErrorData >
{
  // Find session
  let mut sessions = project.all_sessions()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

  // Fix(issue-011): Support partial UUID matching (first 8 chars)
  //
  // Root cause: Session lookup only did exact string matching without checking
  // if provided ID is a prefix of existing session IDs. Users expect Git-style
  // prefix matching for UUIDs (e.g., "79f86582" matches "79f86582-1435-442c-935a-13f8d874918a").
  //
  // Pitfall: ID lookups should always support prefix matching for UUIDs. Test with
  // both exact and partial IDs to ensure both work. Use production-format test data
  // (actual UUIDs) not test-friendly strings like "test-session-123".
  let session = find_session_mut( &mut sessions, session_id )?;

  // Get session stats
  let stats = session.stats()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to get session stats: {e}" ) ) )?;

  // Format output
  let mut output = String::new();

  // REQ-011: Content-first paradigm
  // - verbosity::0 or metadata::1 → Metadata only (old behavior)
  // - verbosity::1+ (default) → Conversation content (NEW behavior)
  let show_content = verbosity >= 1 && !metadata_only;

  // Always show basic session header
  // Fix(issue-028): derive "entry"/"entries" from count; same pattern as issue-025/027.
  // Root cause: hardcoded plural "entries" produced "Session: abc (1 entries)".
  // Pitfall: "entry" is irregular — singular differs from plural root.
  let entry_noun = if stats.total_entries == 1 { "entry" } else { "entries" };
  writeln!( output, "Session: {} ({} {entry_noun})", session_id, stats.total_entries ).unwrap();

  // Metadata-only mode (old behavior)
  if metadata_only || verbosity == 0
  {
    writeln!( output, "Path: {}", session.storage_path().display() ).unwrap();
    writeln!( output, "Agent Session: {}", stats.is_agent_session ).unwrap();
    writeln!( output, "Total Entries: {}", stats.total_entries ).unwrap();
    writeln!( output, "User Entries: {}", stats.user_entries ).unwrap();
    writeln!( output, "Assistant Entries: {}", stats.assistant_entries ).unwrap();

    if let Some( first ) = &stats.first_timestamp
    {
      writeln!( output, "First Entry: {first}" ).unwrap();
    }

    if let Some( last ) = &stats.last_timestamp
    {
      writeln!( output, "Last Entry: {last}" ).unwrap();
    }

    if verbosity >= 2
    {
      output.push_str( "\nToken Usage:\n" );
      writeln!( output, "- Input: {}", stats.total_input_tokens ).unwrap();
      writeln!( output, "- Output: {}", stats.total_output_tokens ).unwrap();
      writeln!( output, "- Cache Read: {}", stats.total_cache_read_tokens ).unwrap();
      writeln!( output, "- Cache Creation: {}", stats.total_cache_creation_tokens ).unwrap();
    }

    // Old entries::1 behavior (UUID list) for backward compat
    if show_entries
    {
      let entries = session.entries()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load entries: {e}" ) ) )?;

      output.push_str( "\nEntries:\n" );

      for ( idx, entry ) in entries.iter().enumerate()
      {
        writeln!
        (
          output,
          "{}. [{:?}] {} ({})",
          idx + 1,
          entry.entry_type,
          entry.uuid,
          entry.timestamp
        ).unwrap();
      }
    }
  }
  // Content-first mode (NEW default behavior)
  else if show_content
  {
    let entries = session.entries()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load entries: {e}" ) ) )?;

    // Add separator for readability
    output.push_str( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n" );
    output.push( '\n' );

    // Format each entry as conversation
    for entry in entries
    {
      let formatted = format_entry_content( entry, None );
      output.push_str( &formatted );
      output.push_str( "\n\n" );
    }

    output.push_str( "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n" );

    // Add metadata footer for verbosity::2+
    if verbosity >= 2
    {
      output.push( '\n' );
      output.push_str( "Session Metadata:\n" );
      writeln!( output, "- Path: {}", session.storage_path().display() ).unwrap();
      writeln!( output, "- Total Entries: {}", stats.total_entries ).unwrap();
      writeln!( output, "- User/Assistant: {}/{}", stats.user_entries, stats.assistant_entries ).unwrap();

      if verbosity >= 3
      {
        output.push_str( "\nToken Usage:\n" );
        writeln!( output, "- Input: {}", stats.total_input_tokens ).unwrap();
        writeln!( output, "- Output: {}", stats.total_output_tokens ).unwrap();
        writeln!( output, "- Cache Read: {}", stats.total_cache_read_tokens ).unwrap();
        writeln!( output, "- Cache Creation: {}", stats.total_cache_creation_tokens ).unwrap();
      }
    }
  }

  Ok( OutputData::new( output, "text" ) )
}

/// Helper: Format project output (extracted logic)
fn format_project_output(
  project : &claude_storage_core::Project,
  verbosity : i64
) -> core::result::Result< OutputData, ErrorData >
{
  // Get project statistics
  let stats = project.project_stats()
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to get project stats: {e}" )
    ))?;

  // Get all sessions
  let mut sessions = project.sessions()
    .map_err( | e | ErrorData::new
    (
      ErrorCode::InternalError,
      format!( "Failed to list sessions: {e}" )
    ))?;

  // Format output
  let mut output = String::new();

  // Project header
  writeln!( output, "Project: {:?}", project.id() ).unwrap();
  writeln!( output, "Storage: {}", project.storage_dir().display() ).unwrap();
  output.push( '\n' );

  // Statistics
  writeln!( output, "Sessions: {} (Main: {}, Agent: {})",
    stats.session_count,
    stats.main_session_count,
    stats.agent_session_count
  ).unwrap();

  writeln!( output, "Total Entries: {}", stats.total_entries ).unwrap();

  if verbosity >= 2
  {
    output.push_str( "Tokens:\n" );
    writeln!( output, "  Input: {}", stats.total_input_tokens ).unwrap();
    writeln!( output, "  Output: {}", stats.total_output_tokens ).unwrap();
  }

  output.push( '\n' );

  // Sessions list
  if sessions.is_empty()
  {
    output.push_str( "No sessions found in this project.\n" );
  }
  else
  {
    output.push_str( "Sessions:\n" );

    for session in &mut sessions
    {
      let session_stats = session.stats()
        .map_err( | e | ErrorData::new
        (
          ErrorCode::InternalError,
          format!( "Failed to get session stats: {e}" )
        ))?;

      if verbosity == 0
      {
        // Minimal: just IDs
        writeln!( output, "  - {}", session.id() ).unwrap();
      }
      else if verbosity == 1
      {
        // Standard: ID + entry count + last timestamp
        let last = session_stats.last_timestamp
          .unwrap_or_else( || "unknown".to_string() );

        // Fix(issue-028): derive "entry"/"entries" from count — sibling of session_count fix.
        // Root cause: hardcoded "entries" produced "(1 entries, last: ...)".
        // Pitfall: "entry" is irregular — singular differs from plural root.
        let e_noun = if session_stats.total_entries == 1 { "entry" } else { "entries" };
        writeln!( output, "  - {} ({} {e_noun}, last: {})",
          session.id(),
          session_stats.total_entries,
          last
        ).unwrap();
      }
      else
      {
        // Detailed: full stats
        writeln!( output, "  - {}", session.id() ).unwrap();
        writeln!( output, "      Entries: {} (User: {}, Assistant: {})",
          session_stats.total_entries,
          session_stats.user_entries,
          session_stats.assistant_entries
        ).unwrap();

        if let Some( first ) = &session_stats.first_timestamp
        {
          writeln!( output, "      First: {first}" ).unwrap();
        }

        if let Some( last ) = &session_stats.last_timestamp
        {
          writeln!( output, "      Last: {last}" ).unwrap();
        }
      }
    }
  }

  Ok( OutputData::new( output, "text" ) )
}
