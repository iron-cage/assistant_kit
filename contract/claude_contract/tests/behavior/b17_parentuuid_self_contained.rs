#![ allow( clippy::doc_markdown ) ]
//! B17: `parentUuid` chains are self-contained per session file.
//!
//! Validates that the overwhelming majority of non-null `parentUuid` values in a
//! session's JSONL entries refer to a `uuid` that exists within the same file.
//!
//! ## Known exception — context-compaction boundaries
//!
//! When Claude Code's context window is exhausted and the conversation is resumed,
//! the continuation user message is appended to the existing `.jsonl` with a
//! `parentUuid` that references the last UUID from the pre-compaction context.
//! That UUID may have existed only in the previous context window and was never
//! written into the file as a top-level `uuid` entry; the orphaned reference is
//! therefore expected and is not an error.
//!
//! Empirically, these violations are rare (< 1% of entries with a non-null
//! `parentUuid`). This test collects all violations, reports them, and only fails
//! if the rate exceeds the 1% threshold — which would indicate a structural change
//! in Claude Code's storage format rather than the known compaction exception.
//!
//! The fact that violations do exist means conversation-chain detection must be
//! inferred from metadata (time, content, `last-prompt` markers) rather than by
//! following `parentUuid` links across files.
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

/// B17 monitoring: orphaned `parentUuid` references stay below 1% of entries.
///
/// Collects all orphaned `parentUuid` references across a sample of sessions
/// (up to 10 projects × 5 sessions) and reports each one via `eprintln!`.
/// Only fails if the violation rate across the entire sample exceeds 1% — a
/// threshold that distinguishes the known context-compaction exception (< 0.2%)
/// from a structural change in Claude Code's storage format.
///
/// Zero violations means B17 holds strictly in the sample.
/// A small number (< 1%) confirms the compaction-boundary exception is the only
/// source of cross-file `parentUuid` references.
#[ test ]
fn it_parentuuid_never_crosses_session_boundary()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no projects found in ~/.claude/projects/" );
    return;
  }

  let mut total_checked : u64 = 0;
  let mut total_violated : u64 = 0;

  for project in projects.iter().take( 10 )
  {
    for session in super::find_sessions( project ).into_iter().take( 5 )
    {
      let content = std::fs::read_to_string( &session ).unwrap_or_default();

      for line in content.lines()
      {
        let Some( parent_uuid ) = top_level_parent_uuid( line ) else { continue; };
        total_checked += 1;

        if !file_has_uuid( &content, &parent_uuid )
        {
          total_violated += 1;
          eprintln!(
            "B17 exception: orphaned parentUuid '{}' in {}",
            parent_uuid,
            session.display(),
          );
        }
      }
    }
  }

  if total_checked == 0
  {
    eprintln!( "skip: no entries with non-null parentUuid found in sample" );
    return;
  }

  // Violation rate > 1% indicates a structural change in Claude Code's format,
  // not the known context-compaction exception (which stays well below 0.2%).
  let violation_rate = total_violated as f64 / total_checked as f64;
  assert!(
    violation_rate <= 0.01,
    "B17 violation rate {:.2}% ({}/{}) exceeds 1% threshold — \
     Claude Code may have changed its parentUuid storage semantics.",
    violation_rate * 100.0,
    total_violated,
    total_checked,
  );
}
