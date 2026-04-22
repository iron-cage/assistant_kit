#![ allow( clippy::doc_markdown ) ]
//! B17: `parentUuid` chains are self-contained per session file.
//!
//! Validates that every non-null `parentUuid` value in a session's JSONL entries
//! refers to a `uuid` that exists within the same file. Cross-file `parentUuid`
//! references would violate B17 and break conversation chain detection —
//! which must therefore be inferred from metadata, not storage links.
//!
//! Tests inspect real `~/.claude/` storage; skip gracefully when absent.
//! Samples up to 10 projects × 5 sessions for fast execution.
//!
//! ## Implementation notes
//!
//! Claude Code serializes `parentUuid` as the first field in every JSONL entry.
//! The `uuid` field may appear AFTER nested `data`/`message` fields, so a
//! first-match extraction would pick up the wrong value. Instead, this test uses
//! a whole-file `contains` search to verify each `parentUuid` is present anywhere
//! in the file as a `uuid` value.

/// Extract the top-level `parentUuid` if the entry has a non-null string value.
///
/// Claude Code serializes `parentUuid` as the first field in every JSONL entry.
/// Only extracts when the line starts with `{"parentUuid":"` (non-null string).
/// Lines starting with `{"parentUuid":null` or other patterns return `None`.
fn top_level_parent_uuid( line : &str ) -> Option< String >
{
  for prefix in [ r#"{"parentUuid":""#, r#"{"parentUuid": ""# ]
  {
    if let Some( rest ) = line.strip_prefix( prefix )
    {
      let end = rest.find( '"' )?;
      return Some( rest[ ..end ].to_string() );
    }
  }
  None
}

/// Return true if the given uuid appears anywhere in the file content as a `uuid` value.
///
/// Searches for both compact (`"uuid":"<val>"`) and spaced (`"uuid": "<val>"`) forms.
/// This whole-file approach correctly handles entries where `uuid` appears after
/// nested `data` fields in the serialized JSON.
fn file_has_uuid( content : &str, uuid : &str ) -> bool
{
  content.contains( &format!( "\"uuid\":\"{uuid}\"" ) )
    || content.contains( &format!( "\"uuid\": \"{uuid}\"" ) )
}

/// B17: every non-null `parentUuid` in a session references a `uuid` in the same file.
///
/// Checks up to 10 projects × 5 sessions from real `~/.claude/` storage.
/// Skips cleanly when no projects are found.
///
/// If this test fails, cross-session `parentUuid` links exist in storage —
/// invalidating B17 and the assumption that conversation chains must be inferred
/// from external metadata (time, content) rather than explicit storage links.
#[ test ]
fn it_parentuuid_never_crosses_session_boundary()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no projects found in ~/.claude/projects/" );
    return;
  }

  for project in projects.iter().take( 10 )
  {
    for session in super::find_sessions( project ).into_iter().take( 5 )
    {
      let content = std::fs::read_to_string( &session ).unwrap_or_default();

      for line in content.lines()
      {
        let Some( parent_uuid ) = top_level_parent_uuid( line ) else { continue; };

        assert!(
          file_has_uuid( &content, &parent_uuid ),
          "B17 violated: parentUuid '{}' not found in same session file.\nFile: {}",
          parent_uuid,
          session.display(),
        );
      }
    }
  }
}
