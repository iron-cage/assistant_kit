//! Bug Reproducer (BUG-489): `Session::stats()` hard-fails on the first malformed JSONL line
//!
//! ## Root Cause
//!
//! `Session::stats()` parsed each JSONL line via `parse_json(line).map_err(...)?`, propagating
//! any parse error immediately through `?`. Its sibling `load_entries()` instead uses
//! `if let Ok(entry) = Entry::from_json_line(line) { entries.push(entry); }`, silently skipping
//! any line it can't parse. A single malformed/truncated line anywhere in a session file made
//! `stats()` hard-fail even though the rest of the file was well-formed and `load_entries()`
//! would process the same file without complaint.
//!
//! ## Why Not Caught
//!
//! Tests used synthetic sessions built entirely from well-formed JSONL lines. Real Claude Code
//! sessions occasionally contain a truncated line (e.g. from an interrupted write); `load_entries()`
//! already tolerates this, but `stats()` was never exercised against the same kind of fixture.
//!
//! ## Fix Applied
//!
//! Changed the hard-propagating `parse_json(line)?` to `let Ok(json) = parse_json(line) else { continue; };`,
//! mirroring `load_entries()`'s established graceful-degradation pattern exactly.
//!
//! ## Prevention
//!
//! Any per-line JSONL processing function must skip a malformed line silently, never hard-fail via `?`
//! — production data always contains some noise, and occasionally a genuinely truncated write.
//!
//! ## Pitfall
//!
//! Sibling functions reading the same JSONL data (`load_entries()`, `stats()`) must handle malformed
//! input consistently. One graceful and one hard-failing creates command-dependent brittleness: `.show`
//! and `.export` (which call `stats()`) would fail on a file that `.tail` (which calls `load_entries()`)
//! reads successfully.

use std::fs;
use tempfile::TempDir;

/// Helper: create a project directory in `projects_dir`
fn create_project( projects_dir : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let p = projects_dir.join( name );
  fs::create_dir_all( &p ).expect( "create project dir" );
  p
}

/// Test `stats()` skips a malformed JSONL line instead of hard-failing.
///
/// ## Coverage
///
/// A session with a valid user entry, then one syntactically-invalid line (unterminated JSON
/// object, matching `json.rs`'s own `test_malformed_json` fixture style: `"{"`), then a valid
/// assistant entry. Before the fix, `stats()` returned `Err` on the malformed line. After the
/// fix, it must return `Ok` and count only the 2 valid entries — proving the malformed line is
/// skipped and processing continues (not aborted) for the entries after it.
#[test]
fn stats_skips_malformed_line()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-test-project" );

  let content = concat!(
    r#"{"type":"user","message":{"role":"user","content":"hello"},"timestamp":"2026-01-01T00:00:00Z"}"#, "\n",
    "{", "\n",
    r#"{"type":"assistant","message":{"role":"assistant","content":"hi"},"timestamp":"2026-01-01T00:00:01Z"}"#, "\n",
  );

  let session_path = p_dir.join( "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.jsonl" );
  fs::write( &session_path, content ).expect( "write session file" );

  let mut session = claude_storage_core::Session::load( &session_path ).expect( "load session" );

  // Before fix: Err( "JSON parse error: ..." ) from the second line.
  // After fix: Ok, having skipped the malformed line and kept processing.
  let stats = session.stats().expect( "stats() must skip a malformed line, not hard-fail" );

  assert_eq!( stats.user_entries, 1, "should count the 1 valid user entry" );
  assert_eq!( stats.assistant_entries, 1, "should count the 1 valid assistant entry after the malformed line" );
  assert_eq!( stats.total_entries, 2, "total should reflect only the 2 valid entries, excluding the malformed line" );
}

/// Test `stats()` on a session containing only malformed lines returns empty (not `Err`) stats.
///
/// ## Coverage
///
/// The all-malformed edge case: every line fails to parse. `stats()` must still return `Ok`
/// with all-zero counts, rather than failing on the first line — mirroring how `load_entries()`
/// would return an empty entry list for the same file, never an error.
#[test]
fn stats_all_malformed_lines_returns_empty_stats()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-all-malformed" );

  let content = concat!( "{", "\n", "[", "\n", r#"{"key"}"#, "\n" );

  let session_path = p_dir.join( "bbbbbbbb-cccc-dddd-eeee-ffffffffffff.jsonl" );
  fs::write( &session_path, content ).expect( "write session file" );

  let mut session = claude_storage_core::Session::load( &session_path ).expect( "load session" );

  let stats = session.stats().expect( "stats() must not hard-fail even when every line is malformed" );

  assert_eq!( stats.total_entries, 0, "an all-malformed session has 0 conversation entries" );
  assert_eq!( stats.user_entries, 0 );
  assert_eq!( stats.assistant_entries, 0 );
}
