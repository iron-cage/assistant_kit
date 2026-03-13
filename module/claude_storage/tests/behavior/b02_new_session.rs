#![ allow( clippy::doc_markdown ) ]
//! B2: `--new-session` creates a new `.jsonl` file; does not append to existing.
//!
//! Validates that at least one real project has 2+ non-agent `.jsonl` session files,
//! confirming that Claude Code creates separate files rather than appending.

/// B2: at least one project has multiple session files (evidence of separate creation).
///
/// If Claude Code switched to a single-file-per-project model, no project would
/// have more than one `.jsonl` and this test would fail.
#[ test ]
fn b2_multiple_session_files_exist_in_real_project()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  let has_multiple = projects.iter().any( | p | super::find_sessions( p ).len() >= 2 );
  assert!(
    has_multiple,
    "B2 violated: no project in ~/.claude/projects/ has 2+ non-empty non-agent .jsonl files.\n\
     Claude Code may no longer create separate session files per invocation."
  );
}
