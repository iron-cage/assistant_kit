//! `.tail` command — display the last N entries of a session.
// BUG-002 — real implementation replacing the hardcoded-output stub

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::{ create_storage, resolve_path_parameter, find_session_mut, most_recent_session_mut };
use super::format::format_entry_content;

/// Display the last N entries of a session (most-recent context refresh).
///
/// Smart behavior based on parameters (see `docs/cli/command/12_tail.md`):
/// - No parameters → current directory's project, most recently modified
///   non-agent session, last 4 entries
/// - `tail::N` → last N entries (`tail::0` = all entries, uncapped)
/// - `topic::NAME` → session `-NAME` explicitly, instead of the recency fallback
/// - `path::DIR` → resolve the project from `DIR` instead of the current directory
///
/// # Errors
///
/// Returns error (exit 1) if `tail` is negative, or if the session cannot be located.
///
/// # Exit Codes
///
/// Exits directly with code 2 (bypassing the standard exit-1 error path) when no
/// project exists for the resolved directory — matches the `.status` command's
/// "not found = usage error" convention (see `status.rs`).
///
/// # Panics
///
/// Does not panic — the `tail_count` conversion below is only reached after the
/// negative-value branch already returned, so the value is always non-negative.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn tail_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  // Validate `tail` before any storage access — rejection happens before entries
  // (or even the project) are loaded, per docs/cli/command/12_tail.md INT-8.
  let tail_count = cmd.get_integer( "tail" ).unwrap_or( 4 );
  if tail_count < 0
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "tail must be non-negative".to_string() ) );
  }
  let tail_count = usize::try_from( tail_count ).expect( "tail < 0 rejected above" );

  let topic = cmd.get_string( "topic" );

  let storage = create_storage()?;
  let path_param = cmd.get_string( "path" );

  let project = if let Some( raw_path ) = path_param
  {
    let resolved = resolve_path_parameter( raw_path )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to resolve path '{raw_path}': {e}" ) ) )?;
    if let Ok( project ) = storage.load_project_for_path( &resolved )
    {
      project
    }
    else
    {
      eprintln!( "No project found for path: {resolved}" );
      std::process::exit( 2 );
    }
  }
  else if let Ok( project ) = storage.load_project_for_cwd()
  {
    project
  }
  else
  {
    eprintln!( "No project found for current directory" );
    std::process::exit( 2 );
  };

  let mut sessions = project.all_sessions()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list sessions: {e}" ) ) )?;

  // Fix(BUG-488)
  // Root cause: an explicit topic:: always resolved to session `-{topic}` via
  //   exact/substring ID match, but with no topic:: given, the code guessed a
  //   fixed `-default_topic` ID even though real Claude Code sessions are
  //   UUID-named — the guess could never match, so the common case (bare
  //   `.tail` in a project with only native sessions) always failed.
  // Pitfall: a "default value" is not always the right fallback strategy —
  //   when the default collides with a naming scheme that never occurs in
  //   production data, prefer resolving by an orthogonal signal (recency)
  //   over guessing an ID that happens to match the parameter's own default.
  let session = if let Some( topic ) = topic
  {
    let session_id = format!( "-{topic}" );
    // Report the error against `topic` (what the user typed), not
    // `session_id` (the internally-prepended `-{topic}` form `find_session_mut`
    // matches against) — a caller-facing message must not leak an internal
    // naming convention the user never entered.
    find_session_mut( &mut sessions, &session_id )
      .map_err( | _ | ErrorData::new( ErrorCode::InternalError, format!( "Session not found for topic: {topic}" ) ) )?
  }
  else
  {
    most_recent_session_mut( &mut sessions )?
  };

  let entries = session.entries()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load entries: {e}" ) ) )?;

  // Entries are stored oldest-first (append-only JSONL); a tail slice therefore
  // needs no reordering — the suffix is already oldest-first.
  let sliced = if tail_count == 0 || tail_count >= entries.len()
  {
    &entries[ .. ]
  }
  else
  {
    &entries[ entries.len() - tail_count.. ]
  };

  let mut output = String::new();
  for entry in sliced
  {
    output.push_str( &format_entry_content( entry, None ) );
    output.push_str( "\n\n" );
  }

  Ok( OutputData::new( output, "text" ) )
}
