//! Bug Reproducer (BUG-508): `Session::entries()` (via `load_entries()`), `Session::count_entries()`,
//! and `Session::stats()` hard-fail the WHOLE session file on a single JSONL line that fails to
//! decode as UTF-8 (e.g. a crash-truncated write leaving binary garbage mid-file), instead of
//! skipping just that one line the way `Session::search()` and `export::export_json()` already do
//! (BUG-503).
//!
//! ## Root Cause
//!
//! All three functions read the session file via `fs::read_to_string()`, which validates UTF-8
//! across the ENTIRE file in one pass — a single invalid byte anywhere fails the whole call before
//! even one line is examined. This is strictly worse than BUG-503's pre-fix behavior (which at
//! least processed lines up to the bad one): here, zero lines are ever processed, regardless of how
//! many valid lines surround the corrupted one. `docs/invariant/001_safety_guarantees.md` already
//! documented this as a known, unfixed divergence from `search()`/`export_json()`'s per-line
//! degradation (established by BUG-489 at the JSON-parse layer, BUG-503 at the UTF-8-decode layer).
//!
//! ## Why Not Caught
//!
//! No existing test for any of the three functions constructs a session file with a line that
//! fails to decode as UTF-8 — `stats_malformed_line_bug.rs` (BUG-489) uses still-valid-UTF-8
//! malformed JSON (`"{"`), which never exercises the read/decode layer this bug fixes, only the
//! parse layer one hop later.
//!
//! ## Fix Applied
//!
//! All three functions swapped `fs::read_to_string()` (whole-file) for `BufReader::new(File::open(..)).lines()`
//! (per-line), with `let Ok(line) = line else { continue; };` skipping any line that fails to
//! decode, mirroring `Session::search()`'s and `export::export_json()`'s already-fixed shape.
//!
//! ## Prevention
//!
//! A per-line JSONL reader must distinguish "this one line is unreadable" from "the whole file is
//! unreadable" at EVERY layer of the loop (read/decode, then parse) — fixing one layer (as BUG-489
//! did for the parse layer) does not imply an earlier layer (read/decode) is also fixed.
//!
//! ## Pitfall
//!
//! `fs::read_to_string()` and `BufReader::lines()` both look like "read this JSONL file," but only
//! the latter validates UTF-8 per-line rather than across the whole buffer at once — the former
//! can never recover from one bad byte anywhere in an otherwise-valid file.

// `core` has no `io` module — `BufReader`'s std::io::{Read,BufRead} impls require std; no core equivalent exists.
#![ allow( clippy::std_instead_of_core ) ]

use std::fs;
use tempfile::TempDir;
use claude_storage_core::Session;

/// Helper: create a project directory in `projects_dir`
fn create_project( projects_dir : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let p = projects_dir.join( name );
  fs::create_dir_all( &p ).expect( "create project dir" );
  p
}

/// Helper: write a session file with a valid entry, then an invalid-UTF-8 line, then a valid entry.
///
/// 0xFF is never valid UTF-8 in any position — guarantees the middle line fails to decode, matching
/// the established fixture technique from `search_export_line_read_failure_bug.rs`. Both valid
/// entries carry every field `Entry::from_json_line()` requires, so a parse failure never masks the
/// read-layer failure under test.
fn write_session_with_bad_line_in_middle( project_dir : &std::path::Path, session_id : &str )
{
  let mut content : Vec< u8 > = Vec::new();
  content.extend_from_slice
  (
    format!( r#"{{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{{"role":"user","content":"before"}}}}"# ).as_bytes()
  );
  content.push( b'\n' );
  content.extend_from_slice( &[ 0xFFu8, 0xFE, 0x00, 0xFF ] );
  content.push( b'\n' );
  content.extend_from_slice
  (
    format!( r#"{{"type":"assistant","uuid":"u2","parentUuid":"u1","timestamp":"2026-01-01T00:00:01Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"requestId":"req1","message":{{"role":"assistant","model":"claude-test","id":"msg1","content":[{{"type":"text","text":"after"}}]}}}}"# ).as_bytes()
  );
  content.push( b'\n' );

  let path = project_dir.join( format!( "{session_id}.jsonl" ) );
  fs::write( &path, &content ).expect( "write session file with bad middle line" );
}

/// Test `Session::entries()` (via `load_entries()`) skips an unreadable line instead of hard-failing
/// the whole session.
///
/// ## Coverage
///
/// Before the fix, `.entries()` returned `Err` from the invalid-UTF-8 line, losing both surrounding
/// entries. After the fix, it must return `Ok` with both entries — proving the bad line is skipped,
/// not fatal to the whole file.
#[test]
fn entries_skips_line_with_invalid_utf8_and_finds_entries_around_it()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-sf6-entries-project" );

  write_session_with_bad_line_in_middle( &p_dir, "aaaaaaaa-3333-0000-0000-000000000003" );

  let session_path = p_dir.join( "aaaaaaaa-3333-0000-0000-000000000003.jsonl" );
  let mut session = Session::load( &session_path ).expect( "load session" );

  let entries = session.entries()
    .expect( "BUG-508: an unreadable line must not abort the whole entries() read" );

  assert_eq!( entries.len(), 2, "should find entries both before and after the unreadable line" );
}

/// Test `Session::count_entries()` skips an unreadable line instead of hard-failing the whole count.
///
/// ## Coverage
///
/// Before the fix, `count_entries()` returned `Err` from the invalid-UTF-8 line, losing the count
/// of both surrounding entries (0 instead of 2). After the fix, it must return `Ok(2)`.
#[test]
fn count_entries_skips_line_with_invalid_utf8_and_counts_entries_around_it()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-sf6-count-project" );

  write_session_with_bad_line_in_middle( &p_dir, "aaaaaaaa-4444-0000-0000-000000000004" );

  let session_path = p_dir.join( "aaaaaaaa-4444-0000-0000-000000000004.jsonl" );
  let session = Session::load( &session_path ).expect( "load session" );

  let count = session.count_entries()
    .expect( "BUG-508: an unreadable line must not abort the whole count_entries() read" );

  assert_eq!( count, 2, "should count entries both before and after the unreadable line" );
}

/// Test `Session::stats()` skips an unreadable line instead of hard-failing the whole stats read.
///
/// ## Coverage
///
/// Before the fix, `stats()` returned `Err` from the invalid-UTF-8 line, losing both surrounding
/// entries' contribution to `total_entries`. After the fix, it must return `Ok` with `total_entries == 2`.
#[test]
fn stats_skips_line_with_invalid_utf8_and_counts_entries_around_it()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-sf6-stats-project" );

  write_session_with_bad_line_in_middle( &p_dir, "aaaaaaaa-5555-0000-0000-000000000005" );

  let session_path = p_dir.join( "aaaaaaaa-5555-0000-0000-000000000005.jsonl" );
  let mut session = Session::load( &session_path ).expect( "load session" );

  let stats = session.stats()
    .expect( "BUG-508: an unreadable line must not abort the whole stats() read" );

  assert_eq!( stats.total_entries, 2, "should count entries both before and after the unreadable line" );
  assert_eq!( stats.user_entries, 1, "the 'before' entry is type:user" );
  assert_eq!( stats.assistant_entries, 1, "the 'after' entry is type:assistant" );
}
