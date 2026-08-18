//! `.search` command — full-text search across session content.

use core::fmt::Write as FmtWrite;
use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::{ create_storage, load_project_for_param };
use super::scope::{ validate_scope, resolve_scoped_projects };

/// One search match, tagged with the project and session it was found in.
type SearchHit = ( claude_storage_core::ProjectId, String, claude_storage_core::SearchMatch );

/// Search session content for query string
///
/// Performs full-text search through session content with optional filtering.
///
/// # Errors
///
/// Returns error if query is missing, entry type is invalid, storage creation
/// fails, project loading fails, or search fails.
#[ allow( clippy::too_many_lines ) ]
// CLI routine handler processes multiple scope branches —
// extraction would obscure the command's logic without reducing complexity.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn search_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let query_raw = cmd.get_string( "query" )
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, "query is required".to_string() ) )?;

  // Fix(issue-030): Reject whitespace-only query values.
  //
  // Root cause: cli_main.rs quotes argv values containing spaces before joining into the
  // REPL command line, so `query::   ` (spaces only) becomes `query::"   "`. The REPL
  // parser preserves the 3-space string (non-empty), so `ok_or_else` alone no longer
  // catches whitespace-only input.
  //
  // Pitfall: Always trim-validate string parameters with a "must be non-empty" constraint.
  // `is_some()` and `!is_empty()` are insufficient — `"   ".is_empty()` is false.
  let query = query_raw.trim();
  if query.is_empty()
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "query must be non-empty".to_string() ) );
  }

  let project_id = cmd.get_string( "project" );
  let session_id = cmd.get_string( "session" );
  let case_sensitive = cmd.get_boolean( "case_sensitive" ).unwrap_or( false );
  let entry_type = cmd.get_string( "entry_type" );
  // scope::/path:: only apply to the two "no project::" branches below — an
  // explicit project:: is already fully scoped.
  let scope_raw = cmd.get_string( "scope" );
  let path_raw = cmd.get_string( "path" );

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
    if let Some( proj_id ) = project_id
    {
      // Fix(issue-012): Support path projects in .search command
      //
      // Root cause: Hardcoded ProjectId::uuid() prevented path projects from working.
      // Commands .count/.search/.export shared this bug which was fixed for .show (Finding #008)
      // but not propagated.
      //
      // Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.
      // Bugs often exist in multiple locations sharing the same flawed assumption.
      let project = load_project_for_param( &storage, proj_id )?;
      all_matches.extend( search_session_in_project( &project, sess_id, &filter )? );
    }
    else
    {
      // Fix(BUG-scope-014): session:: given without project:: now searches
      // scope::-resolved projects (default global) instead of the single cwd
      // project — see docs/cli/param/12_scope.md's flat global default row.
      let scope = validate_scope( scope_raw, "global" )?;
      let scoped_projects = resolve_scoped_projects( &storage, &scope, path_raw )?;

      // Collect every project the session id resolves in, instead of stopping at the
      // first success — a global-scope search over multiple projects can hit a session
      // id (or, via try_search_session_in_project's prefix matching, a shared prefix)
      // in more than one project. Silently taking the first candidate would return the
      // wrong project's content with no indication to the caller. "Not found in this
      // candidate" is the expected, common outcome of scanning N projects for 1 session
      // and stays silent (Ok(None)); only genuine per-candidate errors (I/O, corrupted
      // session) are logged, matching the eprintln! convention already used by the
      // "no project, no session" branch below.
      let mut hits : Vec< ( claude_storage_core::ProjectId, Vec< SearchHit > ) > = Vec::new();
      for project in &scoped_projects
      {
        match try_search_session_in_project( project, sess_id, &filter )
        {
          Ok( Some( matches ) ) => hits.push( ( project.id().clone(), matches ) ),
          Ok( None ) => {}
          Err( e ) => eprintln!( "warning: search skipped project {:?} while resolving session {sess_id}: {e}", project.id() ),
        }
      }

      match hits.len()
      {
        0 =>
        {
          return Err( ErrorData::new( ErrorCode::InternalError, format!( "Session not found: {sess_id}" ) ) );
        }
        1 =>
        {
          for ( _, matches ) in hits
          {
            all_matches.extend( matches );
          }
        }
        _ =>
        {
          let ids : Vec< String > = hits.iter().map( | ( id, _ ) | format!( "{id:?}" ) ).collect();
          return Err( ErrorData::new(
            ErrorCode::InternalError,
            format!(
              "Ambiguous session:: '{sess_id}' matches sessions in {} projects ({}); narrow with project:: or a longer session id",
              hits.len(), ids.join( ", " )
            ),
          ) );
        }
      }
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
    // No project or session specified: search per scope:: (default global —
    // identical project set to the prior unconditional storage.list_projects()).
    let scope = validate_scope( scope_raw, "global" )?;
    let projects = resolve_scoped_projects( &storage, &scope, path_raw )?;

    for project in &projects
    {
      let mut sessions = project.sessions()
        .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions for {:?}: {e}", project.id() ) ) )?;

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
  }

  // Format output
  let mut output = String::new();

  let noun = if all_matches.len() == 1 { "match" } else { "matches" };
  writeln!( output, "Found {} {noun}:\n", all_matches.len() ).unwrap();

  for ( _proj_id, sess_id, m ) in &all_matches
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

  if all_matches.is_empty()
  {
    output.push_str( "No matches found.\n" );
  }

  Ok( OutputData::new( output, "text" ) )
}

/// Search `sess_id` within a single `project`, returning `(project_id, session_id, match)` triples.
///
/// Thin wrapper over `try_search_session_in_project` that turns "not found" into
/// an error — correct here because this caller (the `project::`+`session::` branch)
/// already knows `sess_id` is expected to live in exactly this `project`.
///
/// Fix(issue-020): Use prefix matching for partial UUID, consistent with `show_routine`
/// and `export_routine` (issue-011 fix).
///
/// Root cause: `search_routine` used exact equality only, so ".search `session::79f86582`"
/// failed even though ".show `session_id::79f86582`" succeeds via `starts_with`.
///
/// Pitfall: Partial-UUID support must be applied uniformly. Any session `find()`
/// predicate that uses only == will silently reject valid prefix IDs.
///
/// # Errors
///
/// Returns error if `sess_id` is not found in `project`, sessions cannot be
/// listed, or the search itself fails.
fn search_session_in_project(
  project : &claude_storage_core::Project,
  sess_id : &str,
  filter  : &claude_storage_core::SearchFilter,
) -> core::result::Result< Vec< SearchHit >, ErrorData >
{
  try_search_session_in_project( project, sess_id, filter )?
    .ok_or_else( || ErrorData::new( ErrorCode::InternalError, format!( "Session not found: {sess_id}" ) ) )
}

/// Search `sess_id` within a single `project`, distinguishing "not present here"
/// from a genuine failure.
///
/// Returns `Ok(None)` when `sess_id` does not resolve in `project` — the expected,
/// non-exceptional outcome when scanning multiple candidate projects for one
/// session id (see the multi-project loop above), so callers there must not log
/// it as a warning. Returns `Err` only for genuine failures: sessions unlistable,
/// or the search itself fails.
///
/// # Errors
///
/// Returns error if sessions cannot be listed in `project`, or the search itself
/// fails — never for "`sess_id` not found here".
fn try_search_session_in_project(
  project : &claude_storage_core::Project,
  sess_id : &str,
  filter  : &claude_storage_core::SearchFilter,
) -> core::result::Result< Option< Vec< SearchHit > >, ErrorData >
{
  let mut sessions = project.all_sessions()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

  let Some( session ) = sessions.iter_mut().find( | s | s.id() == sess_id || s.id().starts_with( sess_id ) )
  else
  {
    return Ok( None );
  };

  let matches = session.search( filter )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Search failed: {e}" ) ) )?;

  Ok( Some( matches.into_iter().map( | m | ( project.id().clone(), sess_id.to_string(), m ) ).collect() ) )
}
