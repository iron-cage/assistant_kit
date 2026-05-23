#![ allow( clippy::doc_markdown ) ]
//! B6: each project directory accumulates one `.jsonl` per session invocation.
//!
//! Validates that real `~/.claude/` storage has at least one project with
//! multiple `.jsonl` files — confirming accumulation (not overwrite).

/// B6: at least one real project has accumulated multiple sessions.
///
/// Identical observable to B2 but framed as accumulation (files persist
/// and are not cleaned up or rotated). If Claude Code started rotating
/// old sessions or compacting into a single file, this would fail.
#[ test ]
fn b6_sessions_accumulate_in_real_project()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  // Include ALL .jsonl files (including agent and zero-byte) to check accumulation
  let has_multiple = projects.iter().any( | p | super::find_all_jsonl( p ).len() >= 2 );
  assert!(
    has_multiple,
    "B6 violated: no project has 2+ .jsonl files.\n\
     Claude Code may no longer accumulate session files."
  );
}
