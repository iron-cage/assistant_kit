#![ allow( clippy::doc_markdown ) ]
//! B6: each project directory accumulates one `.jsonl` per session invocation.
//!
//! Validates that real `~/.claude/` storage has at least one project with
//! multiple `.jsonl` files — confirming accumulation (not overwrite).

/// B6: at least one real project has accumulated many sessions without rotation.
///
/// Uses a higher threshold than B2 (>= 5 vs >= 2) to test that Claude Code
/// does NOT rotate or compact old session files. B2 tests per-session creation
/// (>= 2 files); B6 tests long-term accumulation (>= 5 files). If Claude Code
/// started rotating old sessions or compacting into a single file, this would fail.
#[ test ]
fn b6_sessions_accumulate_in_real_project()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  // Include ALL .jsonl files (including agent and zero-byte) to check long-term accumulation.
  // Threshold >= 5 distinguishes this from B2's creation check (>= 2).
  let has_many = projects.iter().any( | p | super::find_all_jsonl( p ).len() >= 5 );
  assert!(
    has_many,
    "B6 violated: no project has 5+ .jsonl files.\n\
     Claude Code may be rotating or compacting session files (threshold >= 5 tests non-rotation)."
  );
}
