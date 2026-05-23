#![ allow( clippy::doc_markdown ) ]
//! B18: the first conversation entry of every session has `parentUuid: null`.
//!
//! Validates that sessions always start clean — no entry in a new session file
//! references a `parentUuid` from a previous session. A non-null `parentUuid` on
//! the first entry would imply cross-session continuation metadata, which B18 says
//! never exists in real Claude Code storage.
//!
//! Tests inspect real `~/.claude/` storage; skip gracefully when absent.
//! Samples up to 10 projects × 5 sessions for fast execution.

/// Return the first conversation-type line (user or assistant) from a JSONL file.
///
/// Skips non-conversation entries (`queue-operation`, `summary`, etc.) consistent
/// with the B10 entry threading tests.
fn first_conversation_line( path : &std::path::Path ) -> Option< String >
{
  std::fs::read_to_string( path )
    .unwrap_or_default()
    .lines()
    .find( | l |
      l.contains( r#""type":"user""# )
        || l.contains( r#""type":"assistant""# )
        || l.contains( r#""type": "user""# )
        || l.contains( r#""type": "assistant""# )
    )
    .map( String::from )
}

/// B18: the first conversation entry of every session has `parentUuid: null`.
///
/// Checks up to 10 projects × 5 sessions from real `~/.claude/` storage.
/// Skips cleanly when no sessions with conversation entries are found.
///
/// If this test fails, a session file was found whose first user/assistant entry
/// carries a non-null `parentUuid` — indicating an unexpected cross-session link
/// that would invalidate B18.
#[ test ]
fn it_first_entry_parentuuid_is_null()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no projects found in ~/.claude/projects/" );
    return;
  }

  let mut checked = 0usize;
  for project in projects.iter().take( 10 )
  {
    for session in super::find_sessions( project ).into_iter().take( 5 )
    {
      let Some( first ) = first_conversation_line( &session ) else { continue; };

      // parentUuid must be null (or absent) — never a non-null UUID.
      let is_null_or_absent = first.contains( r#""parentUuid":null"# )
        || first.contains( r#""parentUuid": null"# )
        || !first.contains( "parentUuid" );

      assert!(
        is_null_or_absent,
        "B18 violated: first conversation entry has a non-null parentUuid.\n\
         File: {}\nFirst entry (truncated): {}",
        session.display(),
        &first[ ..first.len().min( 200 ) ]
      );

      checked += 1;
    }
  }

  if checked == 0
  {
    eprintln!( "skip: no session with conversation entries found in ~/.claude/" );
  }
}
