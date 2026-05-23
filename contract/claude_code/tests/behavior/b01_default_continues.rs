#![ allow( clippy::doc_markdown ) ]
//! B1: the `claude` binary defaults to a NEW session on each invocation; the `clr` wrapper
//! inverts this by passing `-c` by default, making continuation appear as the default.
//!
//! Validates that real `~/.claude/` storage contains at least one project with a non-empty,
//! non-agent `.jsonl` session — the precondition for the wrapper's default continuation to work.
//! If Claude Code changed its session storage format, this test would fail and continuation
//! would break at the storage level regardless of what flags are passed.

/// B1: at least one project has a resumable (non-empty, non-agent) session.
///
/// If Claude Code stopped creating `.jsonl` files for sessions, or changed
/// to a different storage format, this test would fail.
#[ test ]
fn b1_resumable_session_exists_in_real_storage()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  let has_resumable = projects.iter().any( | p | !super::find_sessions( p ).is_empty() );
  assert!(
    has_resumable,
    "B1 violated: no project in ~/.claude/projects/ contains a non-empty non-agent .jsonl session.\n\
     Claude Code may have changed its session storage format."
  );
}
