//! Integration test for content-first display (REQ-011)
//!
//! ## Test Organization
//!
//! **Root Cause**: Users were seeing UUIDs instead of conversation content
//!
//! **Why Not Caught**: No prior tests for default display behavior
//!
//! **Fix Applied**: REQ-011 implements content-first display paradigm
//!
//! **Prevention**: This test validates actual CLI output format
//!
//! **Pitfall**: Always test the user-facing output, not just internal functions
//!
//! This integration test validates that:
//! 1. Default `.show session_id::X` displays conversation content
//! 2. `metadata::1` parameter shows only metadata (old behavior)
//! 3. Content format is readable (timestamps, role labels, text)
//!
//! Fix(issue-content-display-isolation)
//! Root cause: original tests used `Path::new("/home/user1/.claude/projects")` and silently
//! returned when the path didn't exist — violating "Never silently pass due to missing
//! tokens/resources". Tests ran green on CI (path absent) while never actually asserting anything.
//! Pitfall: `if !path.exists() { return; }` is indistinguishable from a real pass in test output.
//! Use `CLAUDE_STORAGE_ROOT` + `TempDir` to guarantee assertions execute on every run.

mod common;

use std::fs;
use tempfile::TempDir;

/// Test: .show displays conversation content by default (not UUIDs)
///
/// ## Test Organization
///
/// **Root Cause**: Default behavior showed UUIDs, not content
///
/// **Why Not Caught**: No integration tests for CLI output format
///
/// **Fix Applied**: Content-first display in `format_session_output`
///
/// **Prevention**: Test actual CLI invocation and output parsing
///
/// **Pitfall**: Integration tests must use real storage, not mocks
#[test]
fn show_displays_content_by_default()
{
  use claude_storage_core::encode_path;

  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let session_id = "test-content-display-default";
  let session_file = project_dir.join( format!( "{session_id}.jsonl" ) );
  // Use full JSONL format with timestamps and content so display can render chat-log
  fs::write(
    &session_file,
    concat!(
      "{\"type\":\"user\",\"uuid\":\"uuid-001\",\"parentUuid\":null,",
      "\"timestamp\":\"2025-11-29T10:00:00Z\",\"cwd\":\"/tmp\",",
      "\"sessionId\":\"test-session\",\"version\":\"2.0.0\",\"gitBranch\":\"master\",",
      "\"userType\":\"external\",\"isSidechain\":false,",
      "\"message\":{\"role\":\"user\",\"content\":\"hello content display\"}}\n",
      "{\"type\":\"assistant\",\"uuid\":\"uuid-002\",\"parentUuid\":\"uuid-001\",",
      "\"timestamp\":\"2025-11-29T10:00:01Z\",\"cwd\":\"/tmp\",",
      "\"sessionId\":\"test-session\",\"version\":\"2.0.0\",\"gitBranch\":\"master\",",
      "\"userType\":\"external\",\"isSidechain\":false,",
      "\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"assistant reply here\"}]}}\n"
    ),
  )
  .unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .output()
    .expect( "Failed to execute .show" );

  let show_output = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!( output.status.success(), "Command should succeed. stderr: {stderr}" );
  assert!(
    show_output.contains( "Session:" ),
    "Should show session header. stdout: {show_output}"
  );
  assert!(
    show_output.contains( "━" ),
    "Should show separator line (content-first). stdout: {show_output}"
  );

  // Chat-log format: "[2025-11-29 10:00] User: ..." or "[...] Assistant: ..."
  let has_timestamp_pattern = show_output.contains( "[20" )
    && ( show_output.contains( "] User:" ) || show_output.contains( "] Assistant:" ) );
  assert!(
    has_timestamp_pattern,
    "Should contain chat-log format with timestamps and roles. stdout: {show_output}"
  );

  // Old broken format: "1. [User] uuid (timestamp)" — should NOT appear
  let has_uuid_list_pattern = show_output.lines().any( | line |
  {
    line.trim().chars().next().is_some_and( | c | c.is_ascii_digit() )
      && line.contains( "[User]" )
      && line.contains( '-' )
      && !line.contains( ':' )
  } );
  assert!(
    !has_uuid_list_pattern,
    "Should NOT show UUID list format (old entries::1 behavior). stdout: {show_output}"
  );
}

/// Test: `metadata::1` parameter shows only metadata (old behavior)
///
/// ## Test Organization
///
/// **Root Cause**: Need backward compatibility for metadata-only view
///
/// **Why Not Caught**: New parameter, needs integration test
///
/// **Fix Applied**: `metadata::1` preserves old behavior
///
/// **Prevention**: Test all parameter combinations
///
/// **Pitfall**: Backward compat modes must be tested, not just new defaults
#[test]
fn show_metadata_only_parameter()
{
  use claude_storage_core::encode_path;

  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let session_id = "test-metadata-only-param";
  let session_file = project_dir.join( format!( "{session_id}.jsonl" ) );
  fs::write(
    &session_file,
    concat!(
      "{\"type\":\"user\",\"uuid\":\"uuid-001\",\"parentUuid\":null,",
      "\"timestamp\":\"2025-11-29T10:00:00Z\",\"cwd\":\"/tmp\",",
      "\"sessionId\":\"test-session\",\"version\":\"2.0.0\",\"gitBranch\":\"master\",",
      "\"userType\":\"external\",\"isSidechain\":false,",
      "\"message\":{\"role\":\"user\",\"content\":\"test for metadata mode\"}}\n",
      "{\"type\":\"assistant\",\"uuid\":\"uuid-002\",\"parentUuid\":\"uuid-001\",",
      "\"timestamp\":\"2025-11-29T10:00:01Z\",\"cwd\":\"/tmp\",",
      "\"sessionId\":\"test-session\",\"version\":\"2.0.0\",\"gitBranch\":\"master\",",
      "\"userType\":\"external\",\"isSidechain\":false,",
      "\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"metadata test reply\"}]}}\n"
    ),
  )
  .unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
      "metadata::1",
    ] )
    .output()
    .expect( "Failed to execute .show metadata::1" );

  let show_output = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!( output.status.success(), "Command should succeed. stderr: {stderr}" );
  assert!(
    show_output.contains( "Session:" ),
    "Should show session header. stdout: {show_output}"
  );
  assert!(
    show_output.contains( "Path:" ),
    "Should show path (metadata field). stdout: {show_output}"
  );
  assert!(
    show_output.contains( "Total Entries:" ),
    "Should show entry count (metadata field). stdout: {show_output}"
  );
  // Content-first mode uses separator; metadata-only must NOT use it
  assert!(
    !show_output.contains( "━" ),
    "Should NOT show separator (content-first feature). stdout: {show_output}"
  );

  let has_chat_format = show_output.contains( "] User:" ) || show_output.contains( "] Assistant:" );
  assert!(
    !has_chat_format,
    "Should NOT show chat-log format with metadata::1. stdout: {show_output}"
  );
}

/// Test: `verbosity::0` is equivalent to `metadata::1`
///
/// ## Test Organization
///
/// **Root Cause**: `verbosity::0` should show minimal metadata only
///
/// **Why Not Caught**: New verbosity semantics need validation
///
/// **Fix Applied**: `verbosity::0` triggers metadata-only mode
///
/// **Prevention**: Test verbosity level equivalences
///
/// **Pitfall**: Don't have multiple ways to specify same behavior without tests
#[test]
fn show_verbosity_zero_is_metadata_only()
{
  use claude_storage_core::encode_path;

  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let session_id = "test-verbosity-zero-mode";
  let session_file = project_dir.join( format!( "{session_id}.jsonl" ) );
  fs::write(
    &session_file,
    concat!(
      "{\"type\":\"user\",\"uuid\":\"uuid-001\",\"parentUuid\":null,",
      "\"timestamp\":\"2025-11-29T10:00:00Z\",\"cwd\":\"/tmp\",",
      "\"sessionId\":\"test-session\",\"version\":\"2.0.0\",\"gitBranch\":\"master\",",
      "\"userType\":\"external\",\"isSidechain\":false,",
      "\"message\":{\"role\":\"user\",\"content\":\"test verbosity zero\"}}\n",
      "{\"type\":\"assistant\",\"uuid\":\"uuid-002\",\"parentUuid\":\"uuid-001\",",
      "\"timestamp\":\"2025-11-29T10:00:01Z\",\"cwd\":\"/tmp\",",
      "\"sessionId\":\"test-session\",\"version\":\"2.0.0\",\"gitBranch\":\"master\",",
      "\"userType\":\"external\",\"isSidechain\":false,",
      "\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"verbosity reply\"}]}}\n"
    ),
  )
  .unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
      "verbosity::0",
    ] )
    .output()
    .expect( "Failed to execute .show verbosity::0" );

  let show_output = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!( output.status.success(), "Command should succeed. stderr: {stderr}" );
  assert!(
    show_output.contains( "Session:" ),
    "Should show session header. stdout: {show_output}"
  );
  assert!(
    show_output.contains( "Path:" ) || show_output.contains( "Total Entries:" ),
    "Should show metadata fields. stdout: {show_output}"
  );

  let has_chat_format = show_output.contains( "] User:" ) || show_output.contains( "] Assistant:" );
  assert!(
    !has_chat_format,
    "verbosity::0 should NOT show chat-log format. stdout: {show_output}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// show session_id:: — Topic Project Directory Search (issue-036)
//
// Root Cause: show_session_in_cwd_impl calls storage.load_project_for_cwd(),
// which does an exact match on the base encoded path. Sessions recorded in
// topic dirs (e.g. -commit, -default_topic) live under storage dirs with
// --commit / --default-topic suffixes; load_project_for_cwd() never returns
// these, so .show session_id:: fails with "Session not found" even when
// .projects shows the session under the current project.
//
// Why Not Caught: No test exercised .show session_id:: with a session in a
// topic project dir. All existing tests supply project:: (Case 4), bypassing
// show_session_in_cwd_impl (Case 2) entirely.
//
// Fix Applied: Replace load_project_for_cwd() with a list_projects() scan
// filtered by the scope::local predicate:
//   dir_name == eb || dir_name.starts_with(&format!("{eb}--"))
// The double-hyphen prevents matching sibling directories.
//
// Prevention: Every test of .show session_id:: (Case 2, no project parameter)
// must cover both the base-dir and topic-dir cases.
//
// Pitfall: Use double-hyphen ({eb}--) for the topic predicate, not
// single-hyphen ({eb}-). Single-hyphen matches sibling directories whose
// encoded name shares the base prefix (e.g. myproject-extra matches myproject).
// ─────────────────────────────────────────────────────────────────────────────

#[test]
// bug_reproducer(issue-036)
fn show_finds_session_in_topic_dir()
{
  use claude_storage_core::encode_path;

  let storage     = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();  // simulated CWD
  let eb          = encode_path( project_dir.path() ).unwrap();
  let topic_id    = format!( "{eb}--commit" );

  // Write session into the topic project dir. Base dir {eb} is absent.
  common::write_test_session( storage.path(), &topic_id, "session-t05-topic-show", 4 );

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .current_dir( project_dir.path() )  // CWD is the project path
    .args( [ ".show", "session_id::session-t05-topic-show" ] )
    .output()
    .unwrap();

  let s    = String::from_utf8_lossy( &output.stdout );
  let serr = String::from_utf8_lossy( &output.stderr );
  assert!(
    output.status.success(),
    ".show must find session in topic project dir; stderr: {serr}\nstdout: {s}"
  );
  assert!( s.contains( "Session:" ), "output must have Session header; got:\n{s}" );
}

#[test]
fn show_finds_session_in_base_dir()
{
  use claude_storage_core::encode_path;

  let storage     = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  let eb          = encode_path( project_dir.path() ).unwrap();

  // Write session into the base project dir (non-regression).
  common::write_test_session( storage.path(), &eb, "session-t06-base-show", 4 );

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .current_dir( project_dir.path() )
    .args( [ ".show", "session_id::session-t06-base-show" ] )
    .output()
    .unwrap();

  let s    = String::from_utf8_lossy( &output.stdout );
  let serr = String::from_utf8_lossy( &output.stderr );
  assert!(
    output.status.success(),
    ".show must find session in base project dir; stderr: {serr}\nstdout: {s}"
  );
  assert!( s.contains( "Session:" ), "output must have Session header; got:\n{s}" );
}

#[test]
fn show_session_not_found_in_unrelated_dir()
{
  use claude_storage_core::encode_path;

  let storage     = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();  // CWD — no sessions here
  let unrelated   = TempDir::new().unwrap();
  let unrelated_id = encode_path( unrelated.path() ).unwrap();

  // Write session into unrelated project (completely different path).
  common::write_test_session( storage.path(), &unrelated_id, "session-t07-unrelated", 2 );

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .current_dir( project_dir.path() )  // CWD has no sessions
    .args( [ ".show", "session_id::session-t07-unrelated" ] )
    .output()
    .unwrap();

  assert!(
    !output.status.success(),
    ".show must fail when session is in an unrelated project dir"
  );
}

#[test]
// bug_reproducer(issue-036)
fn show_finds_session_in_default_topic_dir()
{
  use claude_storage_core::encode_path;

  let storage     = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  let eb          = encode_path( project_dir.path() ).unwrap();
  // "--default-topic" is the storage suffix for the -default_topic working dir.
  let topic_id    = format!( "{eb}--default-topic" );

  common::write_test_session( storage.path(), &topic_id, "session-t08-default-topic-show", 4 );

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .current_dir( project_dir.path() )
    .args( [ ".show", "session_id::session-t08-default-topic-show" ] )
    .output()
    .unwrap();

  let s    = String::from_utf8_lossy( &output.stdout );
  let serr = String::from_utf8_lossy( &output.stderr );
  assert!(
    output.status.success(),
    ".show must find session in --default-topic dir; stderr: {serr}\nstdout: {s}"
  );
  assert!( s.contains( "Session:" ), "output must have Session header; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// show session_id:: — Single-Hyphen Sibling Not Matched (issue-036, T09)
//
// Root Cause: show_session_in_cwd_impl uses `dir_name.starts_with(&format!("{eb}--"))`
// (double-hyphen) as the topic predicate. A storage dir named `{eb}-extra`
// (single hyphen) must NOT be matched — that encodes a completely different
// path (e.g., `base_extra`), not a topic subdirectory of `base`.
//
// Why Not Caught: T07 only tested a completely unrelated path (independent
// TempDir with no prefix overlap). The single-vs-double-hyphen distinction
// was never explicitly validated.
//
// Fix Applied: The double-hyphen predicate is already correct. This test
// guards against regression where the predicate might be weakened to
// single-hyphen.
//
// Prevention: Any change to the topic-prefix predicate in show_session_in_cwd_impl
// must keep T09 passing.
//
// Pitfall: Using single-hyphen `{eb}-` would allow sibling paths like
// `base_extra` to match — the double-hyphen `{eb}--` is critical to
// limiting the scan to genuine topic subdirectories of the current project.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn show_session_not_found_in_single_hyphen_sibling_dir()
{
  use claude_storage_core::encode_path;

  let storage     = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();  // CWD
  let eb          = encode_path( project_dir.path() ).unwrap();

  // Craft a storage dir named "{eb}-extra" — single hyphen, NOT double.
  // This simulates a sibling path whose encoded form shares the base prefix.
  let sibling_id  = format!( "{eb}-extra" );
  common::write_test_session( storage.path(), &sibling_id, "session-t09-single-hyphen-sibling", 2 );

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .current_dir( project_dir.path() )
    .args( [ ".show", "session_id::session-t09-single-hyphen-sibling" ] )
    .output()
    .unwrap();

  assert!(
    !output.status.success(),
    ".show must NOT find session in single-hyphen sibling dir (double-hyphen predicate required)"
  );
}
