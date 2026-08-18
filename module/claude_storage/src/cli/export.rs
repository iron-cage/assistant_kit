//! `.export` command — export a session to a file.

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::{ create_storage, load_project_for_param, find_session_mut };
use super::scope::{ validate_scope, resolve_scoped_projects };

/// Export session to file
///
/// Exports a session to the specified format (markdown, JSON, or text).
///
/// # Errors
///
/// Returns error if `session_id` or output are missing, format is invalid,
/// storage creation fails, project or session loading fails, or export fails.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn export_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let session_id = cmd.get_string( "session_id" )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "session_id is required".to_string() ) )?;

  let output_path_str = cmd.get_string( "output" )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "output is required".to_string() ) )?;

  let format_str = cmd.get_string( "format" ).unwrap_or( "markdown" );
  let project_id = cmd.get_string( "project" );

  // Parse export format
  let format = claude_storage_core::ExportFormat::from_str( format_str )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Invalid format: {e}" ) ) )?;

  // Create storage instance
  let storage = create_storage()?;
  let output_path = std::path::Path::new( output_path_str );

  // Fix(issue-012): Support path projects in .export command
  //
  // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
  // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
  // but not propagated.
  //
  // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
  // Bugs often exist in multiple locations sharing the same flawed assumption.
  if let Some( proj_id ) = project_id
  {
    // scope::/path:: only apply to the "project:: absent" branch below — an
    // explicit project:: is already fully scoped.
    let project = load_project_for_param( &storage, proj_id )?;
    export_from_project( &project, session_id, format, output_path )
  }
  else
  {
    let scope = validate_scope( cmd.get_string( "scope" ), "local" )?;
    let scoped_projects = resolve_scoped_projects( &storage, &scope, cmd.get_string( "path" ) )?;

    for project in &scoped_projects
    {
      if let Ok( output ) = export_from_project( project, session_id, format, output_path )
      {
        return Ok( output );
      }
    }

    Err( ErrorData::new( ErrorCode::InternalError, format!( "Session '{session_id}' not found in scope-resolved projects" ) ) )
  }
}

/// Helper: find `session_id` in `project`, export it to `output_path`, and
/// build the confirmation message.
///
/// # Errors
///
/// Returns error if the session cannot be found in `project`, sessions
/// cannot be listed, or the export write itself fails.
fn export_from_project(
  project     : &claude_storage_core::Project,
  session_id  : &str,
  format      : claude_storage_core::ExportFormat,
  output_path : &std::path::Path,
) -> core::result::Result< OutputData, ErrorData >
{
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

  claude_storage_core::export_session_to_file( session, format, output_path )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Export failed: {e}" ) ) )?;

  let output = format!( "Exported session '{session_id}' to {} (format: {format:?})", output_path.display() );
  Ok( OutputData::new( output, "text" ) )
}
