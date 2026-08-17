//! Bug Reproducer (BUG-503): `Session::search()` and `export::export_json()` diverge from the
//! established per-line graceful-degradation policy when a single JSONL line fails to decode
//! as UTF-8 (e.g. a crash-truncated write leaving binary garbage mid-file)
//!
//! ## Root Cause
//!
//! Both functions stream the session file line-by-line via `BufReader::lines()`, which yields
//! `io::Result<String>` per line and can fail independently for a single line even when every
//! other line in the same file is well-formed. `search()`'s loop hard-propagated that failure via
//! `let line = line?;`, discarding every match already collected for the session. `export_json()`'s
//! `.map_while( std::io::Result::ok )` silently stopped collecting at the first bad line, dropping
//! every subsequent line (not just the bad one) with no error or warning. Both diverge from the
//! silent-skip-and-continue policy already used by sibling per-line handling in the same file —
//! `load_entries()`, `stats()` (BUG-489), and `search()`'s own `Entry::from_json_line` skip 11
//! lines below its own bug.
//!
//! ## Why Not Caught
//!
//! No existing test for either function constructs a session file with a line that fails to
//! decode as UTF-8 — every fixture used well-formed lines only (malformed-JSON fixtures like
//! `stats_malformed_line_bug.rs`'s `"{"` are still valid UTF-8, so they never exercise the
//! `BufReader::lines()` read-level failure path at all, only the JSON-parse-level path).
//!
//! ## Fix Applied
//!
//! `search()`: changed `let line = line?;` to skip the unreadable line and continue (mirroring
//! the `entry_index += 1; continue;` shape already used by this same function's other two
//! skip-paths). `export_json()`: changed `.map_while( std::io::Result::ok )` to
//! `.filter_map( std::io::Result::ok )` — `filter_map` omits `None`-producing elements without
//! stopping the iterator, `map_while` stops at the first one.
//!
//! ## Prevention
//!
//! A per-line JSONL reader must distinguish "this one line is unreadable" from "the whole file
//! is unreadable" — only the latter should hard-fail (`File::open()`/`fs::read_to_string()`
//! failures, per `docs/invariant/001_safety_guarantees.md`'s "Format validation" guarantee).
//!
//! ## Pitfall
//!
//! `BufReader::lines()`'s `io::Result<String>` per line looks like a single opaque failure, but
//! `map_while`/`?` treat it as fatal-for-the-whole-stream while `filter_map`/skip-and-continue
//! treat it as fatal-for-just-that-line only — the iterator adaptor choice silently encodes a
//! graceful-degradation policy decision; picking the wrong one (as both functions did) doesn't
//! show up as a compile error or a clippy lint, only as data loss on real-world corrupted input.

// `core` has no `io` module — `Cursor`'s std::io::{Read,Write} impls require std; no core equivalent exists.
#![ allow( clippy::std_instead_of_core ) ]

use std::fs;
use std::io::Cursor;
use tempfile::TempDir;
use claude_storage_core::{ Session, SearchFilter, ExportFormat, export_session };

/// Helper: create a project directory in `projects_dir`
fn create_project( projects_dir : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let p = projects_dir.join( name );
  fs::create_dir_all( &p ).expect( "create project dir" );
  p
}

/// Helper: write a session file with a valid entry, then an invalid-UTF-8 line, then a valid entry.
///
/// 0xFF is never valid UTF-8 in any position — guarantees the middle line fails to decode,
/// matching the established fixture technique from `sessions_filtered_corrupted_session_bug.rs`.
/// The "before"/"after" lines carry every field `Entry::from_json_line()` requires (`uuid`, `cwd`,
/// `sessionId`, `version`, `userType`, etc., per `entry.rs`'s `from_json_line`) — a minimal
/// `type`+`message`+`timestamp`-only line fails to parse as an `Entry` regardless of the bad middle
/// line, which would mask the fix under test behind an unrelated parse failure.
fn write_session_with_bad_line_in_middle( project_dir : &std::path::Path, session_id : &str, needle : &str )
{
  let mut content : Vec< u8 > = Vec::new();
  content.extend_from_slice
  (
    format!( r#"{{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{{"role":"user","content":"{needle} before"}}}}"# ).as_bytes()
  );
  content.push( b'\n' );
  content.extend_from_slice( &[ 0xFFu8, 0xFE, 0x00, 0xFF ] );
  content.push( b'\n' );
  content.extend_from_slice
  (
    format!( r#"{{"type":"user","uuid":"u2","parentUuid":"u1","timestamp":"2026-01-01T00:00:01Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{{"role":"user","content":"{needle} after"}}}}"# ).as_bytes()
  );
  content.push( b'\n' );

  let path = project_dir.join( format!( "{session_id}.jsonl" ) );
  fs::write( &path, &content ).expect( "write session file with bad middle line" );
}

/// Test `search()` skips an unreadable (invalid-UTF-8) line instead of hard-failing the whole search.
///
/// ## Coverage
///
/// A session with a matching entry before the bad line and a matching entry after it. Before the
/// fix, `search()` returned `Err` on the bad line, discarding the first match too. After the fix,
/// it must return `Ok` with both matches — proving the bad line is skipped, not fatal to the stream.
#[test]
fn search_skips_line_with_invalid_utf8_and_finds_matches_around_it()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-sf5-search-project" );

  write_session_with_bad_line_in_middle( &p_dir, "aaaaaaaa-1111-0000-0000-000000000001", "needle" );

  let session_path = p_dir.join( "aaaaaaaa-1111-0000-0000-000000000001.jsonl" );
  let mut session = Session::load( &session_path ).expect( "load session" );

  let filter = SearchFilter::new( "needle" );

  // Before fix: Err( ... ) from the invalid-UTF-8 line, discarding the "before" match too.
  // After fix: Ok( 2 matches ), the bad line skipped, search continuing to the "after" entry.
  let matches = session.search( &filter )
    .expect( "BUG-503: an unreadable line must not abort the whole search" );

  assert_eq!( matches.len(), 2, "should find matches both before and after the unreadable line" );
  assert!( matches.iter().any( | m | m.excerpt().contains( "before" ) ), "missing the pre-corruption match" );
  assert!( matches.iter().any( | m | m.excerpt().contains( "after" ) ), "missing the post-corruption match" );
}

/// Test `export_json()` includes entries both before AND after an unreadable line, instead of
/// silently truncating everything from the bad line onward.
///
/// ## Coverage
///
/// Before the fix, `.map_while( std::io::Result::ok )` stopped collecting lines at the first
/// invalid-UTF-8 line, so the exported JSON contained only the "before" entry — the "after"
/// entry was silently dropped with no error or warning. After the fix, both entries must appear.
#[test]
fn export_json_includes_entries_after_invalid_utf8_line()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-sf5-export-project" );

  write_session_with_bad_line_in_middle( &p_dir, "aaaaaaaa-2222-0000-0000-000000000002", "marker" );

  let session_path = p_dir.join( "aaaaaaaa-2222-0000-0000-000000000002.jsonl" );
  let mut session = Session::load( &session_path ).expect( "load session" );

  let mut output = Cursor::new( Vec::new() );
  export_session( &mut session, ExportFormat::Json, &mut output ).expect( "export must not hard-fail on an unreadable line" );

  let result = String::from_utf8( output.into_inner() ).expect( "export output must be valid UTF-8" );

  // Before fix: only "marker before" appears — "marker after" silently truncated.
  // After fix: both appear, proving the bad line was skipped, not treated as end-of-stream.
  assert!( result.contains( "marker before" ), "BUG-503: entry before the unreadable line must still be exported" );
  assert!( result.contains( "marker after" ), "BUG-503: entry after the unreadable line must not be silently truncated" );
}
