//! `.search` command — full-text search across session content.

use core::fmt::Write as FmtWrite;
use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::{ create_storage, validate_verbosity, load_project_for_param, find_session_mut };

/// Search session content for query string
///
/// Performs full-text search through session content with optional filtering.
///
/// # Errors
///
/// Returns error if query is missing, verbosity is out of range, entry type
/// is invalid, storage creation fails, project loading fails, or search fails.
#[allow(clippy::too_many_lines)]
// CLI routine handler processes multiple scope branches and verbosity levels —
// extraction would obscure the command's logic without reducing complexity.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn search_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let query = cmd.get_string( "query" )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "query is required".to_string() ) )?;

  let project_id = cmd.get_string( "project" );
  let session_id = cmd.get_string( "session" );
  let case_sensitive = cmd.get_boolean( "case_sensitive" ).unwrap_or( false );
  let entry_type = cmd.get_string( "entry_type" );
  let verbosity = cmd.get_integer( "verbosity" ).unwrap_or( 1 );

  // Fix(issue-010): Validate verbosity range
  //
  // Root cause: search_routine accepted any verbosity value without validation,
  // inconsistent with status_routine and show_routine which validate 0-5 range.
  //
  // Pitfall: Don't assume default values prevent invalid input. Parameters with
  // defaults still need validation since users can override with invalid values.
  validate_verbosity( verbosity )?;

  // Create storage instance
  let storage = create_storage()?;

  // Build search filter
  let mut filter = claude_storage_core::SearchFilter::new( query )
    .case_sensitive( case_sensitive );

  // Add entry type filter if specified
  //
  // Fix(issue-021): Handle "all" as a valid entry_type value
  //
  // Root cause: Only "user" and "assistant" were handled in the match; "all" fell
  // through to the error arm despite the YAML spec documenting it as valid
  // ("Filter by entry type (user, assistant, or all)").
  //
  // Pitfall: Enumerated parameter match arms must cover every value listed in the
  // YAML spec description. Check the YAML spec when adding match arms, not just
  // what you remember implementing.
  if let Some( et ) = entry_type
  {
    match et
    {
      "user" => filter = filter.match_entry_type( claude_storage_core::EntryType::User ),
      "assistant" => filter = filter.match_entry_type( claude_storage_core::EntryType::Assistant ),
      "all" => { /* no type filter — same as omitting entry_type */ }
      _ => return Err( ErrorData::new( ErrorCode::InternalError, format!( "Invalid entry_type: {et}. Valid values: user, assistant, all" ) ) ),
    }
  }

  // Determine search scope
  let mut all_matches = Vec::new();

  if let Some( sess_id ) = session_id
  {
    // Search specific session
    let project = if let Some( proj_id ) = project_id
    {
      // Fix(issue-012): Support path projects in .search command
      //
      // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
      // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
      // but not propagated.
      //
      // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
      // Bugs often exist in multiple locations sharing the same flawed assumption.
      load_project_for_param( &storage, proj_id )
    }
    else
    {
      storage.load_project_for_cwd()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load project: {e}" ) ) )
    }?;

    let mut sessions = project.all_sessions()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

    // Fix(issue-020): Use prefix matching for partial UUID, consistent with show_routine
    // and export_routine (issue-011 fix).
    //
    // Root cause: search_routine used exact equality only, so ".search session::79f86582"
    // failed even though ".show session_id::79f86582" succeeds via starts_with.
    //
    // Pitfall: Partial-UUID support must be applied uniformly. Any session find()
    // predicate that uses only == will silently reject valid prefix IDs.
    let session = find_session_mut( &mut sessions, sess_id )?;

    let matches = session.search( &filter )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Search failed: {e}" ) ) )?;

    for m in matches
    {
      all_matches.push( ( project.id().clone(), sess_id.to_string(), m ) );
    }
  }
  else if let Some( proj_id ) = project_id
  {
    // Search specific project
    // Fix(issue-012): Support path projects in .search command
    //
    // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
    // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
    // but not propagated.
    //
    // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
    // Bugs often exist in multiple locations sharing the same flawed assumption.
    let project = load_project_for_param( &storage, proj_id )?;

    let mut sessions = project.sessions()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

    for session in &mut sessions
    {
      let matches = match session.search( &filter )
      {
        Ok( m )  => m,
        Err( e ) => { eprintln!( "warning: search skipped session {}: {e}", session.id() ); continue; }
      };

      for m in matches
      {
        all_matches.push( ( project.id().clone(), session.id().to_string(), m ) );
      }
    }
  }
  else
  {
    // Search all projects and sessions (current working directory project only)
    let project = storage.load_project_for_cwd()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load project: {e}" ) ) )?;

    let mut sessions = project.sessions()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

    for session in &mut sessions
    {
      let matches = match session.search( &filter )
      {
        Ok( m )  => m,
        Err( e ) => { eprintln!( "warning: search skipped session {}: {e}", session.id() ); continue; }
      };

      for m in matches
      {
        all_matches.push( ( project.id().clone(), session.id().to_string(), m ) );
      }
    }
  }

  // Format output
  let mut output = String::new();

  if verbosity >= 1
  {
    let noun = if all_matches.len() == 1 { "match" } else { "matches" };
    writeln!( output, "Found {} {noun}:\n", all_matches.len() ).unwrap();
  }

  for ( proj_id, sess_id, m ) in &all_matches
  {
    match verbosity
    {
      0 =>
      {
        // Minimal: just excerpt
        writeln!( output, "{}", m.excerpt() ).unwrap();
      }
      1 =>
      {
        // Standard: session + excerpt
        writeln!
        (
          output,
          "[{}] [{:?}] {}",
          sess_id,
          m.entry_type(),
          m.excerpt()
        ).unwrap();
      }
      _ =>
      {
        // Detailed: full metadata
        write!
        (
          output,
          "Project: {:?}\nSession: {}\nEntry: {} ({})\nLine: {}\nExcerpt: {}\nFull Line: {}\n\n",
          proj_id,
          sess_id,
          m.entry_index(),
          match m.entry_type()
          {
            claude_storage_core::EntryType::User => "user",
            claude_storage_core::EntryType::Assistant => "assistant",
          },
          m.line_number(),
          m.excerpt(),
          m.full_line()
        ).unwrap();
      }
    }
  }

  if all_matches.is_empty()
  {
    output.push_str( "No matches found.\n" );
  }

  Ok( OutputData::new( output, "text" ) )
}
