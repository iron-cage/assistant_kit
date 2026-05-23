#![ allow( clippy::doc_markdown ) ]
//! B10: Claude Code writes entries with `parentUuid` linking each to its predecessor.
//!
//! Validates that real `.jsonl` sessions in `~/.claude/` contain conversation entries
//! with `parentUuid` fields: null for the root entry, non-null for subsequent entries.
//!
//! Note: JSONL files may contain non-conversation lines (e.g. `queue-operation`).
//! Tests filter to only `"type":"user"` and `"type":"assistant"` entries.

/// Return only conversation-type lines (user or assistant) from a JSONL file.
fn conversation_lines( path : &std::path::Path ) -> Vec< String >
{
  std::fs::read_to_string( path )
    .unwrap_or_default()
    .lines()
    .filter( | l |
      l.contains( r#""type":"user""# )
      || l.contains( r#""type":"assistant""# )
      || l.contains( r#""type": "user""# )
      || l.contains( r#""type": "assistant""# )
    )
    .map( String::from )
    .collect()
}

/// B10a: the first conversation entry of a real session has `parentUuid: null`.
///
/// If Claude Code removed the `parentUuid` field or changed the root entry
/// convention, this test would fail.
#[ test ]
fn b10_first_entry_has_null_parent_uuid()
{
  let projects = super::find_projects();
  let session = projects.iter()
    .flat_map( | p | super::find_sessions( p ) )
    .find( | f |
    {
      let lines = conversation_lines( f );
      !lines.is_empty()
    });

  let Some( path ) = session else
  {
    eprintln!( "skip: no session with conversation entries found in ~/.claude/" );
    return;
  };

  let lines = conversation_lines( &path );
  let first = &lines[ 0 ];

  assert!(
    first.contains( "parentUuid" ),
    "B10 violated: first conversation entry has no parentUuid field at all.\n\
     File: {}\nFirst entry (truncated): {}",
    path.display(),
    &first[ ..first.len().min( 200 ) ]
  );
  assert!(
    first.contains( r#""parentUuid":null"# )
      || first.contains( r#""parentUuid": null"# ),
    "B10 violated: first conversation entry does not have parentUuid:null.\n\
     File: {}\nFirst entry (truncated): {}",
    path.display(),
    &first[ ..first.len().min( 200 ) ]
  );
}

/// B10b: conversation entries after the first have non-null `parentUuid`.
///
/// If Claude Code stopped linking entries via parentUuid, conversation
/// threading would break.
#[ test ]
fn b10_subsequent_entries_have_non_null_parent_uuid()
{
  let projects = super::find_projects();
  let session = projects.iter()
    .flat_map( | p | super::find_sessions( p ) )
    .find( | f |
    {
      let lines = conversation_lines( f );
      lines.len() >= 2
    });

  let Some( path ) = session else
  {
    eprintln!( "skip: no session with 2+ conversation entries found in ~/.claude/" );
    return;
  };

  let lines = conversation_lines( &path );
  let second = &lines[ 1 ];

  assert!(
    second.contains( "parentUuid" ),
    "B10 violated: second conversation entry has no parentUuid field.\n\
     File: {}\nSecond entry (truncated): {}",
    path.display(),
    &second[ ..second.len().min( 200 ) ]
  );

  // parentUuid should NOT be null on the second entry
  let is_null = second.contains( r#""parentUuid":null"# )
    || second.contains( r#""parentUuid": null"# );

  assert!(
    !is_null,
    "B10 violated: second conversation entry has null parentUuid (should reference first entry).\n\
     File: {}\nSecond entry (truncated): {}",
    path.display(),
    &second[ ..second.len().min( 200 ) ]
  );
}
