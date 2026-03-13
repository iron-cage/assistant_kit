//! Bug Reproducer (issue-016 + issue-018): `count_entries()` accuracy vs performance
//!
//! ## Issue-016: `count_entries()` counted ALL JSONL lines (accuracy bug)
//!
//! ### Root Cause
//!
//! `Session::count_entries()` used `content.lines().count()` — counting every non-empty
//! JSONL line including internal Claude Code metadata entries (type: "queue-operation",
//! "system", "summary", etc.). This produced counts ~5% higher than
//! `session.stats().total_entries`, creating a visible inconsistency:
//!
//! - `.count target::entries`: 2135 (all lines)
//! - "Total Entries" in `.show`:  2034 (user+assistant only)
//!
//! ### Why Not Caught
//!
//! Tests used simple synthetic sessions with only user and assistant entries.
//! Real Claude Code sessions contain metadata lines (queue-operation records for
//! wplan integration, summary entries, etc.) that are invisible to existing tests.
//!
//! ### Fix Applied (then replaced by issue-018 fix)
//!
//! Changed `count_entries()` to parse each line's JSON `"type"` field. Correct,
//! but full `parse_json()` per line proved too slow for session filtering at scale.
//!
//! ## Issue-018: Full JSON parse in `count_entries()` made `.list min_entries::N` hang
//!
//! ### Root Cause
//!
//! The issue-016 fix replaced `lines().count()` with `parse_json()` per line.
//! `session.matches_filter()` calls `count_entries()` for every session across all
//! projects. With 1903 projects / 2429 sessions / ~7 GB of JSONL, full JSON parsing
//! per line caused `.list min_entries::N` to SIGTERM in nextest (>26 s timeout).
//!
//! ### Fix Applied
//!
//! Replaced `parse_json()` with a byte-level string search for `"type":"user"` and
//! `"type":"assistant"`. JSON escaping guarantees these literal patterns are unique
//! to top-level type fields — nested content is always escaped (`\"type\":\"user\"`).
//! The string-search approach is O(bytes) but ~10x faster than full JSON parsing.
//!
//! ## Prevention
//!
//! Never count JSONL lines as a proxy for conversation depth — filter by type.
//! Never use a full JSON parse in a function called O(session count) times; prefer
//! targeted string search exploiting JSON escaping invariants.
//!
//! ## Pitfall
//!
//! `content.lines().count()` and `stats().total_entries` measure different things.
//! The former is file-line count; the latter is conversation-entry count. And
//! a correct-but-slow O(JSONL-bytes) fix can silently break downstream performance
//! when the fixed function is called in a tight loop over many sessions.

use std::fs;
use tempfile::TempDir;

/// Helper: create a project directory in `projects_dir`
fn create_project( projects_dir : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let p = projects_dir.join( name );
  fs::create_dir_all( &p ).expect( "create project dir" );
  p
}

/// Helper: write a JSONL session file with a mix of entry types
fn create_mixed_session( project_dir : &std::path::Path, session_id : &str ) -> std::path::PathBuf
{
  // 2 user entries + 2 assistant entries + 3 metadata entries (queue-operation, system, summary)
  // count_entries (fixed) should return 4; line count would return 7
  let content = concat!(
    r#"{"type":"queue-operation","operation":"enqueue","sessionId":"s1","content":"test"}"#, "\n",
    r#"{"type":"user","message":{"role":"user","content":"hello"},"timestamp":"2026-01-01T00:00:00Z"}"#, "\n",
    r#"{"type":"assistant","message":{"role":"assistant","content":"hi"},"timestamp":"2026-01-01T00:00:01Z"}"#, "\n",
    r#"{"type":"system","content":"context info"}"#, "\n",
    r#"{"type":"user","message":{"role":"user","content":"bye"},"timestamp":"2026-01-01T00:00:02Z"}"#, "\n",
    r#"{"type":"summary","summary":"short session"}"#, "\n",
    r#"{"type":"assistant","message":{"role":"assistant","content":"goodbye"},"timestamp":"2026-01-01T00:00:03Z"}"#, "\n",
  );

  let path = project_dir.join( format!( "{session_id}.jsonl" ) );
  fs::write( &path, content ).expect( "write session file" );
  path
}

/// Test `count_entries()` counts only user+assistant entries, not all JSONL lines.
///
/// ## Coverage
///
/// Verifies the fix: a session with 4 conversation entries and 3 metadata entries
/// should return 4, not 7 (the total line count before the fix).
#[test]
fn count_entries_excludes_metadata_lines()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-test-project" );
  let session_path = create_mixed_session( &p_dir, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" );

  let session = claude_storage_core::Session::load( &session_path ).expect( "load session" );

  // Before fix: would return 7 (all non-empty lines)
  // After fix:  returns 4 (user + assistant only)
  let count = session.count_entries().expect( "count_entries" );
  assert_eq!( count, 4, "should count only user+assistant entries (4), not all lines (7)" );
}

/// Test `count_entries()` matches `stats().total_entries` for mixed sessions.
///
/// ## Coverage
///
/// Verifies that `count_entries()` and `stats().total_entries` return the same value
/// for sessions containing metadata entries. Before the fix they diverged.
#[test]
fn count_entries_matches_stats_total_entries()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-test-project-2" );
  let session_path = create_mixed_session( &p_dir, "11111111-2222-3333-4444-555555555555" );

  let session = claude_storage_core::Session::load( &session_path ).expect( "load session" );

  // Immutable count via fixed count_entries()
  let count = session.count_entries().expect( "count_entries" );

  // Mutable stats via stats()
  let mut session_mut = claude_storage_core::Session::load( &session_path ).expect( "load session" );
  let stats = session_mut.stats().expect( "stats" );

  assert_eq!(
    count,
    stats.total_entries,
    "count_entries() must match stats().total_entries for sessions with metadata lines"
  );
}

/// Test `count_entries()` on session with no metadata (pure conversation) still works.
#[test]
fn count_entries_pure_conversation_session()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-pure-project" );

  let content = concat!(
    r#"{"type":"user","message":{"role":"user","content":"Q1"},"timestamp":"2026-01-01T00:00:00Z"}"#, "\n",
    r#"{"type":"assistant","message":{"role":"assistant","content":"A1"},"timestamp":"2026-01-01T00:00:01Z"}"#, "\n",
    r#"{"type":"user","message":{"role":"user","content":"Q2"},"timestamp":"2026-01-01T00:00:02Z"}"#, "\n",
    r#"{"type":"assistant","message":{"role":"assistant","content":"A2"},"timestamp":"2026-01-01T00:00:03Z"}"#, "\n",
  );
  let session_path = p_dir.join( "pure-session.jsonl" );
  fs::write( &session_path, content ).expect( "write" );

  let session = claude_storage_core::Session::load( &session_path ).expect( "load" );
  let count = session.count_entries().expect( "count" );
  assert_eq!( count, 4 );
}

/// Test `count_entries()` on an empty session file returns 0.
///
/// ## Coverage
///
/// An empty JSONL file (zero bytes or only whitespace/blank lines) has no entries.
/// `count_entries()` must return 0 without panicking or returning an error.
/// This is the base case for new/empty sessions.
#[test]
fn count_entries_empty_session()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-empty-session" );

  // Completely empty file
  let session_path = p_dir.join( "empty-session.jsonl" );
  fs::write( &session_path, "" ).expect( "write" );

  let session = claude_storage_core::Session::load( &session_path ).expect( "load" );
  let count = session.count_entries().expect( "count" );
  assert_eq!( count, 0, "empty session should have 0 entries" );
}

/// Test `count_entries()` on a session containing only blank lines returns 0.
///
/// ## Coverage
///
/// Blank lines must be skipped by the iterator. Neither the old implementation
/// (which counted non-empty lines) nor the new one should count blank lines as entries.
#[test]
fn count_entries_blank_lines_only_session()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-blank-session" );

  // File with only blank lines
  let session_path = p_dir.join( "blank-session.jsonl" );
  fs::write( &session_path, "\n\n   \n\n" ).expect( "write" );

  let session = claude_storage_core::Session::load( &session_path ).expect( "load" );
  let count = session.count_entries().expect( "count" );
  assert_eq!( count, 0, "blank-lines-only session should have 0 entries" );
}

/// Test `count_entries()` on a session with only metadata entries returns 0.
///
/// ## Coverage
///
/// Metadata entries (queue-operation, system, summary) must NOT be counted.
/// This is the pure metadata case — no user/assistant entries present at all.
/// Before the fix, this would have returned 3 (the line count).
#[test]
fn count_entries_metadata_only_session()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-metadata-only" );

  let content = concat!(
    r#"{"type":"queue-operation","operation":"enqueue","sessionId":"s1","content":"test"}"#, "\n",
    r#"{"type":"system","content":"context info"}"#, "\n",
    r#"{"type":"summary","summary":"no actual conversation"}"#, "\n",
  );
  let session_path = p_dir.join( "metadata-only.jsonl" );
  fs::write( &session_path, content ).expect( "write" );

  let session = claude_storage_core::Session::load( &session_path ).expect( "load" );
  let count = session.count_entries().expect( "count" );
  // Before fix: returned 3 (all lines); after fix: returns 0 (no user/assistant)
  assert_eq!( count, 0, "metadata-only session should have 0 user/assistant entries" );
}
