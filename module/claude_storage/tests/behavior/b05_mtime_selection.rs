#![ allow( clippy::doc_markdown ) ]
//! B5: the "current" session is the most recently modified `.jsonl` file (mtime).
//!
//! Validates that real sessions within a project have distinct, orderable mtimes —
//! the precondition for mtime-based session selection.

/// B5: sessions in a real project have distinct mtimes.
///
/// If Claude Code wrote all sessions with identical timestamps (or used a
/// different selection mechanism), mtimes would not be reliably orderable.
#[ test ]
fn b5_real_sessions_have_distinct_mtimes()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  // Find a project with 2+ sessions
  let multi = projects.iter().find( | p | super::find_sessions( p ).len() >= 2 );
  let Some( project ) = multi else
  {
    eprintln!( "skip: no project with 2+ sessions found" );
    return;
  };

  let sessions = super::find_sessions( project );
  let mut mtimes : Vec<_> = sessions.iter()
    .filter_map( | p | std::fs::metadata( p ).ok()?.modified().ok() )
    .collect();
  mtimes.sort();
  mtimes.dedup();

  assert_eq!(
    mtimes.len(),
    sessions.len(),
    "B5 violated: {} sessions but only {} distinct mtimes in {:?}.\n\
     Mtime-based session selection would be ambiguous.",
    sessions.len(),
    mtimes.len(),
    project
  );
}
