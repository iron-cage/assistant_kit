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
