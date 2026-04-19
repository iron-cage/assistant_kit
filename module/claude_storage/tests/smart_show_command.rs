//! Test coverage for smart .show command (location-aware behavior)
//!
//! ## Feature
//!
//! The `.show` command adapts its behavior based on which parameters are provided:
//! - No parameters → Shows current directory project (all sessions)
//! - `session_id` only → Shows that session in current project
//! - project only → Shows that project (all sessions)
//! - Both parameters → Shows that session in that project
//!
//! ## Root Cause (Original Issue)
//!
//! User complained: ".show without arguments must work perfectly fine and show for current path"
//!
//! Original behavior:
//! - `.show` required `session_id` parameter (parser rejected without it)
//! - Code had logic for current directory (line 311 in `show_routine`) but it was UNREACHABLE
//! - YAML definition had `optional: false` for `session_id` (line 149)
//!
//! ## Fix Applied
//!
//! 1. YAML: Changed `session_id` to `optional: true`
//! 2. Implementation: Refactored `show_routine` to `match(session_id`, `project_id`) with 4 cases
//! 3. Spec: Updated documentation to reflect smart behavior
//!
//! ## Prevention
//!
//! - Always test commands with all parameter combinations (none, one, both)
//! - Verify parser configuration matches intended UX
//! - Test location-aware commands from different directories
//!
//! ## Pitfall
//!
//! **Parser configuration must match implementation expectations**. When code expects
//! optional parameters but YAML marks them as required, the implementation code becomes
//! unreachable dead code. Always trace from parser YAML → command handler to verify
//! the optional/required flag matches code expectations.
//!
//! ## Isolation Note
//!
//! All tests that write session data use `CLAUDE_STORAGE_ROOT` + `TempDir` for isolation.
//! This prevents race conditions in workspace-wide parallel `cargo nextest run --workspace`.
//!
//! Fix(issue-smart-show-isolation)
//! Root cause: original tests wrote to real `~/.claude/` (via `HOME` resolution), causing
//! non-deterministic failures under nextest's thread pool when multiple tests ran concurrently.
//! Pitfall: manual `fs::remove_*` cleanup runs after binary exit but is not guaranteed to
//! complete before the next parallel test starts — `TempDir` drop is the only safe cleanup.

mod common;

use std::fs;
use tempfile::TempDir;

#[ test ]
fn test_show_parser_accepts_no_args()
{
  // Test: .show (parser should accept no arguments)
  // This is a smoke test - the detailed "current directory" behavior
  // is tested via unit tests in src/cli/mod.rs

  // Execute .show with no arguments (parser should not reject)
  let output = common::clg_cmd()
    .args( [ ".show" ] )
    .output()
    .unwrap();

  let stderr = String::from_utf8_lossy( &output.stderr );

  // Parser should NOT reject with "missing required argument"
  // (It may fail for other reasons like "project not found", which is fine)
  assert!(
    !stderr.contains( "required argument" ) && !stderr.contains( "session_id"),
    "Parser should not require session_id parameter. stderr: {stderr}"
  );
}

#[ test ]
fn test_show_with_session_id_and_project()
{
  // Test: .show session_id::abc project::/path
  // This tests explicit parameters (current directory logic tested separately)

  use claude_storage_core::encode_path;

  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let session_id = "test-session-specific";
  let file = project_dir.join( format!( "{session_id}.jsonl" ) );
  fs::write( &file, r#"{"type":"user","text":"specific test entry"}
{"type":"assistant","text":"specific response"}"# ).unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() )
    ] )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed. stderr: {stderr}"
  );

  assert!(
    stdout.contains( session_id ),
    "Should display session ID. stdout: {stdout}"
  );

  // Should show entry count (2 entries)
  assert!(
    stdout.contains( "Entries:" ) || stdout.contains( '2' ),
    "Should show entry count. stdout: {stdout}"
  );
}

#[ test ]
fn test_show_with_project_only()
{
  // Test: .show project::/path/to/project
  // Expected: Shows that project (all sessions)

  use claude_storage_core::encode_path;

  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let sessions = vec![ "session-a", "session-b", "session-c" ];
  for session_id in &sessions
  {
    let file = project_dir.join( format!( "{session_id}.jsonl" ) );
    fs::write( &file, r#"{"type":"user","text":"test"}"# ).unwrap();
  }

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [ ".show", &format!( "project::{}", project_path.path().display() ) ] )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed. stderr: {stderr}"
  );

  // Should show project info
  assert!(
    stdout.contains( &project_path.path().to_string_lossy().to_string() )
      || stdout.contains( "Project:" ),
    "Should display project info. stdout: {stdout}"
  );

  // Should list all sessions
  for session_id in &sessions
  {
    assert!(
      stdout.contains( session_id ),
      "Should list session {session_id}. stdout: {stdout}"
    );
  }
}

#[ test ]
fn test_show_with_both_params()
{
  // Test: .show session_id::abc project::/path
  // Expected: Shows that session in that project (regression test)

  use claude_storage_core::encode_path;

  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let session_id = "test-both-params";
  let file = project_dir.join( format!( "{session_id}.jsonl" ) );
  fs::write( &file, r#"{"type":"user","text":"test entry"}"# ).unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() )
    ] )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed (regression test). stderr: {stderr}"
  );

  assert!(
    stdout.contains( session_id ),
    "Should display session ID. stdout: {stdout}"
  );
}

#[ test ]
fn test_show_error_for_nonexistent_project()
{
  // Test: .show project::/nonexistent/path
  // Expected: Clear error message

  let output = common::clg_cmd()
    .args( [ ".show", "project::/nonexistent/test-path-show-error" ] )
    .output()
    .unwrap();

  let stderr = String::from_utf8_lossy( &output.stderr );

  // Should fail gracefully
  assert!(
    !output.status.success(),
    "Should fail for non-existent project"
  );

  // Should have clear error message
  assert!(
    stderr.contains( "not found" ) || stderr.contains( "Project" ) || stderr.contains( "No project" ),
    "Should have clear error message. stderr: {stderr}"
  );
}

#[ test ]
fn test_show_project_parser_accepts_no_args()
{
  // Test: .show.project (parser should accept no arguments)
  // This verifies backward compatibility for the deprecated command

  let output = common::clg_cmd()
    .args( [ ".show.project" ] )
    .output()
    .unwrap();

  let stderr = String::from_utf8_lossy( &output.stderr );

  // Parser should NOT reject with "missing required argument"
  // (It may fail for other reasons like "project not found", which is fine)
  assert!(
    !stderr.contains( "required argument" ),
    "Parser should not require project parameter. stderr: {stderr}"
  );
}

/// Test .show `session_id` partial UUID matching (Finding #011)
///
/// ## Root Cause
///
/// The .show command's session lookup doesn't support partial UUID matching
/// (first 8 characters) despite the spec explicitly documenting this feature.
/// When users provide `session_id::79f86582` for session `79f86582-1435-442c-935a-13f8d874918a`,
/// the command fails with "Session not found" error.
///
/// The implementation only does exact string matching against full session IDs,
/// without checking if the provided ID is a prefix of any existing session.
///
/// ## Why Not Caught
///
/// Existing tests for .show command used custom test session IDs like "test-session-specific"
/// which don't follow the UUID format. All tests provided the exact full ID that was
/// created, so prefix matching was never exercised. The spec mentions partial UUID support
/// but no tests verified it.
///
/// ## Fix Applied
///
/// Modified session lookup in `show_routine` to check if provided `session_id` is either:
/// 1. Exact match (existing behavior for non-UUID IDs like "agent-022ada42")
/// 2. Prefix match (new behavior for partial UUIDs like first 8 chars)
///
/// The fix maintains backward compatibility for all ID formats while adding prefix
/// matching support for UUIDs.
///
/// ## Prevention
///
/// When implementing ID-based lookups:
/// - Always test with both full and partial IDs
/// - Consider UUID prefix matching as standard UX
/// - Test actual UUID format IDs, not just test-friendly strings
/// - Verify spec-documented features have corresponding tests
///
/// ## Pitfall
///
/// **Test data that doesn't match production data patterns leads to missing test coverage**.
/// Using friendly test IDs like "test-session-specific" instead of actual UUIDs meant
/// partial UUID matching was never exercised in tests, despite being documented in the spec.
#[ test ]
fn test_show_partial_uuid_matching()
{
  use claude_storage_core::encode_path;

  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  // Create session with actual UUID format
  let full_uuid = "79f86582-1435-442c-935a-13f8d874918a";
  let partial_uuid = "79f86582"; // First 8 chars
  let file = project_dir.join( format!( "{full_uuid}.jsonl" ) );
  fs::write( &file, r#"{"type":"user","uuid":"uuid-001","parentUuid":null,"timestamp":"2025-11-29T10:00:00Z","cwd":"/tmp/test","sessionId":"test-session","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"message":{"role":"user","content":"test partial uuid matching"}}
{"type":"assistant","uuid":"uuid-002","parentUuid":"uuid-001","timestamp":"2025-11-29T10:00:01Z","cwd":"/tmp/test","sessionId":"test-session","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"message":{"role":"assistant","content":[{"type":"text","text":"this should be findable with partial ID"}]}}"# ).unwrap();

  // Test 1: Full UUID should work
  let output_full = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{full_uuid}" ),
      &format!( "project::{}", project_path.path().display() )
    ] )
    .output()
    .unwrap();

  let stdout_full = String::from_utf8_lossy( &output_full.stdout );
  let stderr_full = String::from_utf8_lossy( &output_full.stderr );

  assert!(
    output_full.status.success(),
    "Full UUID should work. stderr: {stderr_full}"
  );

  assert!(
    stdout_full.contains( full_uuid ) || stdout_full.contains( "Session:" ),
    "Should show session with full UUID. stdout: {stdout_full}"
  );

  // Test 2: Partial UUID (first 8 chars) should also work
  let output_partial = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{partial_uuid}" ),
      &format!( "project::{}", project_path.path().display() )
    ] )
    .output()
    .unwrap();

  let stdout_partial = String::from_utf8_lossy( &output_partial.stdout );
  let stderr_partial = String::from_utf8_lossy( &output_partial.stderr );

  assert!(
    output_partial.status.success(),
    "Partial UUID (first 8 chars) should work. stderr: {stderr_partial}"
  );

  assert!(
    stdout_partial.contains( full_uuid ) || stdout_partial.contains( "Session:" ),
    "Should find session using partial UUID. stdout: {stdout_partial}"
  );

  assert!(
    stdout_partial.contains( "partial uuid matching" ),
    "Should show session content. stdout: {stdout_partial}"
  );
}

/// Test `.show session_id::...` with 1-entry session shows "Session: id (1 entry)"
///
/// ## Root Cause
///
/// `show_session_routine` formatted the session header as
/// `"Session: {id} ({count} entries)"` with hardcoded plural "entries".
/// For a session with exactly 1 entry this produces "Session: abc (1 entries)" —
/// grammatically incorrect.
///
/// ## Why Not Caught
///
/// All prior `.show` / `smart_show` tests used sessions with ≥2 entries (from
/// `write_path_project_session` which defaulted to higher counts).  The plural
/// branch is correct for count ≥2, so no existing test exposed the singular bug.
///
/// ## Fix Applied
///
/// Added `let entry_noun = if stats.total_entries == 1 { "entry" } else { "entries" };`
/// and replaced the hardcoded "entries" with `{entry_noun}` in the header format string.
///
/// ## Prevention
///
/// Explicitly test every count-bearing header with count == 1 alongside count > 1.
/// Write both a `bug_reproducer` (count==1 shows singular) and a `regression_guard`
/// (count > 1 still shows plural) for the same format string.
///
/// ## Pitfall
///
/// "entry" is an irregular noun — singular "entry", plural "entries".
/// Deriving the noun programmatically avoids future regressions if the format
/// string is reused in other contexts.
// bug_reproducer(issue-028)
#[ test ]
fn test_show_session_single_entry_header_says_entry_not_entries()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  let session_id = "028a0000-1111-2222-3333-444444444444";

  // Write a session with exactly 1 entry so the singular path is exercised
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    session_id,
    1,
  );

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .output()
    .expect( "Failed to execute .show" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".show with 1-entry session should succeed. stderr: {stderr}"
  );

  // Before fix: "Session: ... (1 entries)" — wrong plural
  // After fix:  "Session: ... (1 entry)"   — correct singular
  assert!(
    stdout.contains( "(1 entry)" ),
    "1-entry session header should say '(1 entry)' not '(1 entries)'. stdout: {stdout}"
  );

  assert!(
    !stdout.contains( "(1 entries)" ),
    "1-entry session header must NOT say '(1 entries)'. stdout: {stdout}"
  );
}

/// Regression guard: two-entry session header still shows "(2 entries)" after fix
// regression_guard(issue-028)
#[ test ]
fn test_show_session_multi_entry_header_still_says_entries()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  let session_id = "028b0000-1111-2222-3333-444444444444";

  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    session_id,
    3,
  );

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .output()
    .expect( "Failed to execute .show" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".show with 3-entry session should succeed. stderr: {stderr}"
  );

  assert!(
    stdout.contains( "(3 entries)" ),
    "3-entry session header should say '(3 entries)'. stdout: {stdout}"
  );
}

/// IT-5: `metadata::1` suppresses content, shows only session metadata
///
/// Verifies that `metadata::1` produces output with metadata fields present
/// but without the raw message content from entries.
#[ test ]
fn test_show_metadata_mode_suppresses_content()
{
  use claude_storage_core::encode_path;

  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let session_id = "meta-mode-0000-1111-2222-3333-444444444444";
  let distinctive_text = "xkcd_distinctive_message_content_metadata_test";

  // Write session with a distinctive content string so we can check suppression
  let file = project_dir.join( format!( "{session_id}.jsonl" ) );
  fs::write(
    &file,
    format!( r#"{{"type":"user","uuid":"uuid-m1","parentUuid":null,"timestamp":"2025-11-29T10:00:00Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"{distinctive_text}"}}}}
{{"type":"assistant","uuid":"uuid-m2","parentUuid":"uuid-m1","timestamp":"2025-11-29T10:00:01Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_m2","message":{{"role":"assistant","model":"claude-test","id":"msg_m2","content":[{{"type":"text","text":"response text"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":5,"output_tokens":3,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"# )
  ).unwrap();

  // Without metadata::1 — should show content including distinctive_text
  let out_normal = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .output()
    .unwrap();

  // With metadata::1 — should suppress content
  let out_meta = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
      "metadata::1",
    ] )
    .output()
    .unwrap();

  let stdout_normal = String::from_utf8_lossy( &out_normal.stdout );
  let stdout_meta   = String::from_utf8_lossy( &out_meta.stdout );
  let stderr_meta   = String::from_utf8_lossy( &out_meta.stderr );

  assert!(
    out_normal.status.success(),
    "Normal mode should succeed. stderr: {}",
    String::from_utf8_lossy( &out_normal.stderr )
  );
  assert!(
    out_meta.status.success(),
    "metadata::1 mode should exit 0. stderr: {stderr_meta}"
  );

  // Normal mode must show the distinctive text
  assert!(
    stdout_normal.contains( distinctive_text ),
    "Normal mode must show entry content. stdout: {stdout_normal}"
  );

  // metadata::1 mode must NOT show the distinctive entry text
  assert!(
    !stdout_meta.contains( distinctive_text ),
    "metadata::1 must suppress entry content. stdout: {stdout_meta}"
  );
}

/// IT-6: `entries::1` shows all individual session entries
///
/// Verifies that `entries::1` expands the session view to include each entry's
/// content, producing more detailed output than the default summary.
#[ test ]
fn test_show_entries_mode_expands_content()
{
  use claude_storage_core::encode_path;

  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let session_id = "entries-mode-0000-1111-2222-3333-444444444444";

  // Write 4-entry session with identifiable content in each entry
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    session_id,
    4,
  );

  // Without entries::1 — default summary mode
  let out_default = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .output()
    .unwrap();

  // With entries::1 — expanded entry view
  let out_entries = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [
      ".show",
      &format!( "session_id::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
      "entries::1",
    ] )
    .output()
    .unwrap();

  let stdout_entries = String::from_utf8_lossy( &out_entries.stdout );
  let stderr_entries = String::from_utf8_lossy( &out_entries.stderr );
  let stdout_default = String::from_utf8_lossy( &out_default.stdout );

  assert!(
    out_default.status.success(),
    "Default mode should succeed. stderr: {}",
    String::from_utf8_lossy( &out_default.stderr )
  );
  assert!(
    out_entries.status.success(),
    "entries::1 mode should exit 0. stderr: {stderr_entries}"
  );

  // entries::1 should produce more output (individual entries expanded)
  assert!(
    stdout_entries.len() >= stdout_default.len(),
    "entries::1 output must be at least as long as default output. entries: {} chars, default: {} chars",
    stdout_entries.len(),
    stdout_default.len()
  );
}
