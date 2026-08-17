//! Bug Reproducer (BUG-506): `Project::sessions_filtered()` discarded an entire project's
//! filtered session list when just one session's JSONL file was corrupted/unreadable
//!
//! ## Root Cause
//!
//! `sessions_filtered()`'s per-session loop called `session.matches_filter( filter )?`,
//! hard-propagating any error from `Session::matches_filter()`'s `min_entries` branch
//! (which calls `Session::count_entries()`, itself able to fail on `fs::read_to_string`
//! I/O or invalid-UTF-8 errors — e.g. a crash-truncated JSONL file, the exact corruption
//! class this codebase's own `Fix(issue-017)` comment documents as real). One corrupted
//! session anywhere in a project discarded every already-collected valid session in
//! `filtered`, not just the corrupted one.
//!
//! ## Why Not Caught
//!
//! Every existing `sessions_filtered()` test (`filtering.rs`) filters real or synthetic
//! all-valid sessions; none construct a session file that fails `count_entries()`.
//!
//! ## Fix Applied
//!
//! Changed the loop to `match session.matches_filter( filter ) { Ok(true) => ..., Ok(false)
//! => {}, Err(e) => eprintln!("Warning: ...") }`, mirroring the graceful per-session skip
//! already used by `Project::sessions()`, `Project::all_sessions()`, and
//! `Project::project_stats()` in the same file.
//!
//! ## Prevention
//!
//! This test locks in that one corrupted session's `matches_filter()` failure must never
//! discard other, valid sessions already collected in the same project's filtered result.
//!
//! ## Pitfall
//!
//! A per-item error-handling convention established in one loop (catch + warn + continue)
//! doesn't automatically apply to a sibling loop over the same collection in the same file
//! — each loop must be checked individually; a bare `?` in just one of several near-identical
//! loops is easy to miss in review.

use core::fmt::Write as _;
use std::fs;
use tempfile::TempDir;
use claude_storage_core::{ Project, ProjectId, Session, SessionFilter };

/// Helper: create a project directory in `projects_dir`
fn create_project( projects_dir : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let p = projects_dir.join( name );
  fs::create_dir_all( &p ).expect( "create project dir" );
  p
}

/// Helper: write a valid session file with `n` conversation entries
fn write_valid_session( project_dir : &std::path::Path, session_id : &str, n : usize )
{
  let mut content = String::new();
  for i in 0..n
  {
    writeln!
    (
      content,
      r#"{{"type":"user","message":{{"role":"user","content":"msg {i}"}},"timestamp":"2026-01-01T00:00:{i:02}Z"}}"#
    )
    .expect( "write to in-memory String cannot fail" );
  }
  let path = project_dir.join( format!( "{session_id}.jsonl" ) );
  fs::write( &path, content ).expect( "write valid session file" );
}

/// Helper: write a session file containing invalid UTF-8 bytes, causing
/// `Session::count_entries()`'s `fs::read_to_string` to fail.
fn write_corrupted_session( project_dir : &std::path::Path, session_id : &str )
{
  let path = project_dir.join( format!( "{session_id}.jsonl" ) );
  // 0xFF is never valid UTF-8 in any position — guarantees fs::read_to_string errors.
  fs::write( &path, [ 0xFFu8, 0xFE, 0x00, 0xFF ] ).expect( "write corrupted session file" );
}

/// Test `sessions_filtered()` skips a corrupted session instead of discarding the whole project.
///
/// ## Coverage
///
/// A project with 2 valid sessions (3 entries each) and 1 corrupted session (invalid UTF-8).
/// Filtering by `min_entries: Some(1)` must return the 2 valid sessions, not `Err` — before
/// the fix, the corrupted session's `count_entries()` failure aborted the whole loop via `?`,
/// discarding the 2 valid sessions too.
#[test]
fn sessions_filtered_skips_corrupted_session_keeps_valid_ones()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-sf3-project" );

  write_valid_session( &p_dir, "aaaaaaaa-0000-0000-0000-000000000001", 3 );
  write_corrupted_session( &p_dir, "bbbbbbbb-0000-0000-0000-000000000002" );
  write_valid_session( &p_dir, "cccccccc-0000-0000-0000-000000000003", 3 );

  let mut project = Project::new( ProjectId::Uuid( "sf3-project".to_string() ), p_dir );

  let filter = SessionFilter
  {
    agent_only : None,
    min_entries : Some( 1 ),
    session_id_substring : None,
  };

  // Before fix: Err( ... ) from the corrupted session's count_entries() failure,
  // discarding both valid sessions.
  // After fix: Ok( vec![ 2 valid sessions ] ), corrupted session skipped with a warning.
  let filtered = project.sessions_filtered( &filter )
    .expect( "BUG-506: one corrupted session must not abort filtering of the whole project" );

  assert_eq!( filtered.len(), 2, "should keep both valid sessions, skipping only the corrupted one" );
  let ids : Vec< &str > = filtered.iter().map( Session::id ).collect();
  assert!( ids.contains( &"aaaaaaaa-0000-0000-0000-000000000001" ) );
  assert!( ids.contains( &"cccccccc-0000-0000-0000-000000000003" ) );
  assert!( !ids.contains( &"bbbbbbbb-0000-0000-0000-000000000002" ), "corrupted session must be excluded, not included" );
}

/// Test `sessions_filtered()` on a project with ONLY a corrupted session returns an empty
/// `Ok` result, not an error.
///
/// ## Coverage
///
/// The all-corrupted edge case, mirroring BUG-489's `stats_all_malformed_lines_returns_empty_stats`
/// precedent: `sessions_filtered()` must still return `Ok(vec![])`, never `Err`, when every
/// session in the project fails its filter check.
#[test]
fn sessions_filtered_all_corrupted_returns_empty_not_err()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-sf3-all-corrupted" );

  write_corrupted_session( &p_dir, "dddddddd-0000-0000-0000-000000000004" );

  let mut project = Project::new( ProjectId::Uuid( "sf3-all-corrupted".to_string() ), p_dir );

  let filter = SessionFilter
  {
    agent_only : None,
    min_entries : Some( 1 ),
    session_id_substring : None,
  };

  let filtered = project.sessions_filtered( &filter )
    .expect( "BUG-506: an all-corrupted project must return Ok(empty), not Err" );

  assert!( filtered.is_empty(), "no sessions should match when every session is corrupted" );
}
