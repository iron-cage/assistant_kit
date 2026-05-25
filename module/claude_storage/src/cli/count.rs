//! `.count` command — count entries, sessions, or projects.

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use claude_storage_core::Storage;
use super::storage::{ create_storage, load_project_for_param, resolve_path_parameter };
use super::projects::{ build_families, group_into_conversations };

/// Count entries, sessions, or projects
///
/// Fast counting without loading all data into memory.
///
/// Context-aware: When called without parameters, counts entries in the current project
/// (detected from CWD), matching the behavior of `.show` for UX consistency.
///
/// # Errors
///
/// Returns error if storage creation fails, target is invalid, required parameters
/// (project or session) are missing, or counting operations fail.
#[ allow( clippy::needless_pass_by_value ) ]
#[ allow( clippy::too_many_lines ) ]
// CLI routine handles multiple target branches (projects/sessions/entries) and path parameter —
// extraction would increase indirection without reducing actual complexity.
#[ inline ]
pub fn count_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  // Fix(issue-003a): Make .count context-aware like .show
  //
  // Root cause: .count defaulted to counting all projects globally when called without
  // parameters, while .show showed current project stats. Users expected .count to
  // count what .show shows (entries in current project).
  //
  // Pitfall: Related commands should have consistent default behaviors. If .show is
  // context-aware (uses CWD), .count should be too. Don't make one global and one local.
  let target = cmd.get_string( "target" );
  let project_id = cmd.get_string( "project" );
  let session_id = cmd.get_string( "session" );
  let custom_path = cmd.get_string( "path" );

  // Create storage instance, respecting explicit path:: parameter
  let storage = if let Some( path ) = custom_path
  {
    let resolved = resolve_path_parameter( path )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to resolve path '{path}': {e}" ) ) )?;
    Storage::with_root( std::path::Path::new( &resolved ) )
  }
  else
  {
    create_storage()?
  };

  // Context-aware default: If no target and no project specified, try to count entries in CWD project
  // If CWD is not a project directory, fall back to counting all projects globally
  if target.is_none() && project_id.is_none()
  {
    if let Ok( project ) = storage.load_project_for_cwd()
    {
      // Count all entries across all sessions in the project
      let sessions = project.all_sessions()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

      let mut total_entries = 0usize;
      for session in &sessions
      {
        // Fix(issue-017): Skip corrupted sessions with warning, matching project_stats() behavior.
        //
        // Root cause: `?` propagated parse errors from individual corrupted sessions to the
        // entire `.count` command. A single corrupted JSONL line in any session would fail
        // the whole command, even though other sessions were valid. project_stats() already
        // handled this correctly by using `match` + `eprintln!` to skip corrupted sessions.
        //
        // Pitfall: Using `?` in a loop over user data is too strict. Real Claude Code sessions
        // can have truncated JSONL lines (from crashes mid-write). Always handle per-item
        // errors gracefully when iterating over a collection of user files.
        match session.count_entries()
        {
          Ok( n ) => total_entries += n,
          Err( e ) =>
          {
            eprintln!( "Warning: Skipping corrupted session {}: {e}", session.storage_path().display() );
          }
        }
      }

      let output = format!( "{total_entries}" );
      return Ok( OutputData::new( output, "text" ) );
    }
    // If load_project_for_cwd() fails, fall through to default behavior (count all projects)
  }

  // Explicit target specified, or project without target (counts sessions in project)
  let target : &str = target.unwrap_or( "projects" );
  let count = match target
  {
    "projects" =>
    {
      storage.count_projects()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to count projects: {e}" ) ) )?
    }
    "sessions" =>
    {
      if let Some( proj_id ) = project_id
      {
        // Fix(issue-012): Support path projects in .count command
        //
        // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
        // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
        // but not propagated.
        //
        // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
        // Bugs often exist in multiple locations sharing the same flawed assumption.
        let project = load_project_for_param( &storage, proj_id )?;

        project.count_sessions()
          .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to count sessions: {e}" ) ) )?
      }
      else
      {
        // No project specified — count sessions across all projects
        let projects = storage.list_projects()
          .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;

        let mut total = 0usize;
        for p in &projects
        {
          match p.count_sessions()
          {
            Ok( n )  => total += n,
            Err( e ) => eprintln!( "Warning: failed to count sessions in {:?}: {e}", p.id() ),
          }
        }
        total
      }
    }
    "entries" =>
    {
      // Requires project context
      let proj_id = project_id
        .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "project parameter required for counting entries".to_string() ) )?;

      // Fix(issue-012): Support path projects in .count command
      //
      // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
      // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
      // but not propagated.
      //
      // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
      // Bugs often exist in multiple locations sharing the same flawed assumption.
      let project = load_project_for_param( &storage, proj_id )?;

      let sessions = project.all_sessions()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

      if let Some( sess_id ) = session_id
      {
        // Fix(issue-019): Use prefix matching for partial UUID, consistent with show_routine
        // and export_routine (both use starts_with from the issue-011 fix).
        //
        // Root cause: count_routine used exact equality only, so "79f86582" failed even
        // though ".show session_id::79f86582" succeeds via prefix matching.
        //
        // Pitfall: When fixing partial-UUID support in one session lookup, grep for every
        // other `sessions.iter*().find(|s| s.id() == ...)` and apply the same change.
        let session = sessions.iter()
          .find( | s | s.id() == sess_id || s.id().contains( sess_id ) )
          .ok_or_else( || ErrorData::new( ErrorCode::InternalError, format!( "Session not found: {sess_id}" ) ) )?;

        session.count_entries()
          .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to count entries: {e}" ) ) )?
      }
      else
      {
        // No session specified — sum all entries across all sessions in project
        let mut total = 0usize;
        for session in &sessions
        {
          match session.count_entries()
          {
            Ok( n )  => total += n,
            Err( e ) => eprintln!( "Warning: skipping corrupted session {}: {e}", session.storage_path().display() ),
          }
        }
        total
      }
    }
    "conversations" =>
    {
      let proj_id = project_id
        .ok_or_else( || ErrorData::new(
          ErrorCode::InternalError,
          "project parameter required for counting conversations".to_string(),
        ) )?;
      let project = load_project_for_param( &storage, proj_id )?;
      let sessions = project.all_sessions()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load sessions: {e}" ) ) )?;
      let families = build_families( sessions );
      let conversations = group_into_conversations( families );
      conversations.len()
    }
    // Fix(issue-009): Validate target parameter against allowed values
    //
    // Root cause: target parameter accepted any string without validation,
    // causing confusing errors when invalid values provided.
    //
    // Pitfall: Don't assume unilang validates enum constraints. Always
    // validate enumerated parameters explicitly against allowed values.
    _ =>
    {
      return Err( ErrorData::new( ErrorCode::InternalError, format!( "Invalid target: {target}" ) ) );
    }
  };

  let output = format!( "{count}" );
  Ok( OutputData::new( output, "text" ) )
}
