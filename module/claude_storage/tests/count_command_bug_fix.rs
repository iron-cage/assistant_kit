//! Bug fix tests for `.count` command issues (#003a, #003b, issue-017)
//!
//! ## Bug #003a: Default Behavior Mismatch
//!
//! - Location: `claude_storage/src/cli/mod.rs`
//! - The `.count` command defaults to `target::projects` when called without parameters,
//!   counting all projects globally instead of being context-aware like `.show`.
//! - Fix: When NO parameters and CWD is a project, count total entries in that project.
//!
//! ## Bug #003b: Hardcoded UUID Parsing
//!
//! - Location: `claude_storage/src/cli/mod.rs`
//! - The `count_routine` hardcodes `ProjectId::uuid()` instead of `parse_project_parameter()`.
//! - Fix: Use `parse_project_parameter()` for both UUID and path-based project parameters.
//!
//! ## issue-017: IO Error in Session Crashes `.count`
//!
//! - Location: `claude_storage/src/cli/mod.rs` — context-aware entry counting loop
//! - The context-aware `.count` loop used `?` to propagate `count_entries()` errors.
//!   Any session that raises an IO error (e.g., permission denied, filesystem error)
//!   caused the entire command to fail rather than skipping that session.
//! - Fix: Changed `?` to `match` + `eprintln!` warning, matching `project_stats()` behavior.
//!
//! ## Pitfall
//!
//! Never use `?` in a loop over user data files. Always skip erroring items with a warning.
//! Note: truncated JSONL does NOT trigger this — `count_entries()` uses byte-level search
//! and succeeds on partial lines. Only true IO errors (permission denied, unreadable file)
//! cause `count_entries()` to return `Err`.

mod common;

use std::fs;
use tempfile::TempDir;

/// Test Bug #003a: .count should be context-aware like .show
///
/// When called with NO parameters from within a project directory,
/// .count should count entries in that project, not count all projects globally.
// test_kind: bug_reproducer(issue-003a)
#[ test ]
fn test_count_default_behavior_context_aware()
{
  let storage = TempDir::new().unwrap();
  let project_cwd = TempDir::new().unwrap();

  // Write 5 conversation entries for the project whose path == project_cwd
  common::write_path_project_session(
    storage.path(),
    project_cwd.path(),
    "session-context-aware",
    5,
  );

  let output = common::clg_cmd()
    .args( [ ".count" ] )
    .current_dir( project_cwd.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".count must succeed. stderr: {stderr}"
  );

  let count_str = stdout.trim();

  // Bug #003a: Previously returned number of projects (e.g., 85)
  // Fixed: Now returns number of entries in current project (5)
  assert_eq!(
    count_str, "5",
    "Expected .count (no params) to return 5 entries in current project, got: {count_str}"
  );
}

/// Test Bug #003b: .count should handle path-based project parameters
///
/// The .count command should use `parse_project_parameter()` to handle
/// both UUID and path-based project parameters, not hardcode `ProjectId::uuid()`.
// test_kind: bug_reproducer(issue-003b)
#[ test ]
fn test_count_with_path_project_parameter()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  // Write 3 sessions (1 entry each)
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "session-path-001",
    1,
  );
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "session-path-002",
    1,
  );
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "session-path-003",
    1,
  );

  let output = common::clg_cmd()
    .args( [
      ".count",
      "target::sessions",
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".count should succeed with path parameter. stderr: {stderr}"
  );

  assert_eq!(
    stdout.trim(), "3",
    "Expected 3 sessions. stdout: {stdout}"
  );
}

/// Test `.count` skips unreadable sessions instead of failing (Bug Reproducer: issue-017)
///
/// ## Root Cause
///
/// The context-aware `.count` loop used `?` to propagate `count_entries()` errors, so
/// a session that returns `Err` (e.g., IO permission denied) caused the entire command
/// to fail. This is inconsistent with `project_stats()` which skips erroring sessions
/// with `eprintln!` and continues.
///
/// ## Why Not Caught
///
/// No tests covered the scenario where a CWD project contains a session that raises
/// an IO error. All existing tests used clean synthetic data. Real production storage
/// can have permission issues or filesystem errors during `read_to_string()`.
///
/// ## Fix Applied
///
/// Changed the session loop in `count_routine()` from `?` propagation to `match`:
/// sessions that return `Err` are skipped with `eprintln!("Warning: Skipping corrupted
/// session ...")`, matching `project.project_stats()` behavior (src/project.rs:342-355).
///
/// ## Prevention
///
/// Never use `?` inside a loop over user data files. Always use `match` + warning
/// for per-item errors so one bad file doesn't kill the whole command.
///
/// ## Pitfall
///
/// Truncated JSONL (e.g., mid-write crash) does NOT trigger this path: `count_entries()`
/// uses byte-level string search and succeeds on partial lines. Only true IO errors
/// (permission denied, read failure) cause `count_entries()` to return `Err`. Use
/// `chmod 000` in tests to simulate an IO failure cleanly.
// test_kind: bug_reproducer(issue-017)
#[ test ]
#[ cfg( unix ) ]
fn test_count_skips_unreadable_sessions()
{
  use std::os::unix::fs::PermissionsExt;

  let storage = TempDir::new().unwrap();
  let project_cwd = TempDir::new().unwrap();

  // Write the clean session (2 entries) first
  let encoded = common::write_path_project_session(
    storage.path(),
    project_cwd.path(),
    "aaaaaaaa-clean-ccccccccccc",
    2,
  );

  // Create a second session file then make it unreadable
  let unreadable_session = storage.path()
    .join( "projects" )
    .join( &encoded )
    .join( "bbbbbbbb-unreadable-dddddddddddd.jsonl" );
  fs::write( &unreadable_session, "" ).unwrap();
  fs::set_permissions(
    &unreadable_session,
    fs::Permissions::from_mode( 0o000 ),
  ).unwrap();

  let output = common::clg_cmd()
    .args( [ ".count" ] )
    .current_dir( project_cwd.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  // Restore permissions so TempDir cleanup succeeds
  fs::set_permissions(
    &unreadable_session,
    fs::Permissions::from_mode( 0o644 ),
  ).ok();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".count must succeed even with unreadable sessions. stderr: {stderr}"
  );

  assert_eq!(
    stdout.trim(), "2",
    "Expected 2 entries from clean session only. stdout: {stdout}, stderr: {stderr}"
  );

  assert!(
    stderr.contains( "Warning" ) && stderr.contains( "corrupted" ),
    "Expected 'Warning: Skipping corrupted session ...' in stderr. stderr: {stderr}"
  );
}

/// Verification test: .count with explicit `target::projects` should still work
///
/// After fixing Bug #003a to make default context-aware, the explicit
/// `target::projects` should still count all projects globally.
// test_kind: bug_reproducer(issue-003a)
#[ test ]
fn test_count_explicit_target_projects()
{
  let storage = TempDir::new().unwrap();

  // Create 2 projects
  common::write_test_session( storage.path(), "proj-alpha", "s001", 1 );
  common::write_test_session( storage.path(), "proj-beta", "s001", 1 );

  let output = common::clg_cmd()
    .args( [ ".count", "target::projects" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".count target::projects must succeed. stderr: {stderr}"
  );

  assert_eq!(
    stdout.trim(), "2",
    "Expected 2 projects. stdout: {stdout}"
  );
}

/// Test `.count target::entries session::existing-id` succeeds
///
/// Verifies that providing a valid `session` filter with `target::entries`
/// counts only entries in that specific session.
#[ test ]
fn test_count_entries_with_session_filter()
{
  let storage = TempDir::new().unwrap();

  // Create 2 sessions: one with 3 entries, one with 5
  common::write_test_session( storage.path(), "count-sess-proj", "target-session-aaa", 3 );
  common::write_test_session( storage.path(), "count-sess-proj", "other-session-bbb", 5 );

  let output = common::clg_cmd()
    .args( [
      ".count",
      "target::entries",
      "session::target-session-aaa",
      "project::count-sess-proj",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".count with session filter should succeed. stderr: {stderr}"
  );

  assert_eq!(
    stdout.trim(), "3",
    "Expected 3 entries from target session only. stdout: {stdout}"
  );
}

/// Test `.count target::entries session::` (empty) fails — framework parse error
///
/// The framework rejects `session::` (empty String parameter value) with a
/// parse error before the command reaches application code.
#[ test ]
fn test_count_entries_session_empty()
{
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".count", "target::entries", "session::", "project::some-proj" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with empty session filter. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "error" ) ||
    combined.contains( "session" ),
    "Error should be a parse or validation error. Got: {combined}"
  );
}

/// Test `.count target::entries` with partial UUID session ID (issue-019)
///
/// ## Root Cause
///
/// `count_routine` ("entries" arm) used exact string equality (`s.id() == sess_id`)
/// to locate the session, while `.show` and `.export` use prefix matching
/// (`s.id() == sid || s.id().starts_with(sid)`). A user who copies the first 8
/// characters of a UUID from `.list` output can count entries with `.show` and
/// `.export` but not with `.count target::entries`, making the UX inconsistent.
///
/// ## Why Not Caught
///
/// The partial-UUID-matching fix (issue-011) was applied to `format_session_output`
/// (used by `.show`) and to `export_routine`, but the same `find()` call in
/// `count_routine` was not updated at the same time. There was no test exercising
/// partial IDs in `target::entries` mode; all existing count tests used
/// full session IDs written by `write_test_session`.
///
/// ## Fix Applied
///
/// Changed the session lookup in `count_routine` entries arm from exact match to
/// prefix match: `s.id() == sess_id || s.id().starts_with(sess_id)`, consistent
/// with `show_routine` and `export_routine`.
///
/// ## Prevention
///
/// When applying a partial-ID-matching fix to one session lookup, grep for all
/// other `sessions.iter*().find(|s| s.id() == ...)` patterns in the codebase and
/// apply the same fix. Inconsistent ID matching across commands causes confusing
/// UX. Add tests that exercise prefix IDs in every command that accepts a
/// `session` parameter.
///
/// ## Pitfall
///
/// Partial UUID support must be applied uniformly. If `.show` accepts 8-char
/// prefixes but `.count` requires the full UUID, users who discover the shortcut
/// in one command waste time debugging why the same shortcut fails elsewhere.
// test_kind: bug_reproducer(issue-019)
#[ test ]
fn test_count_entries_partial_uuid_match()
{
  let storage = TempDir::new().unwrap();
  let session_uuid = "79f86582-1435-442c-935a-13f8d874918a";
  let session_prefix = "79f86582";

  // Write 3 entries using the full session UUID
  common::write_test_session( storage.path(), "count-partial-proj", session_uuid, 3 );

  // Bug: using 8-char prefix returns "Session not found: 79f86582"
  // Fixed: returns "3" (prefix match, consistent with .show and .export)
  let output = common::clg_cmd()
    .args( [
      ".count",
      "target::entries",
      "project::count-partial-proj",
      &format!( "session::{session_prefix}" ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".count target::entries with partial UUID must succeed. stderr: {stderr}"
  );

  assert_eq!(
    stdout.trim(), "3",
    "Expected 3 entries via partial UUID match. stdout: {stdout}"
  );
}

/// Test `.count target::invalid` exits 1 (argument error)
///
/// ## Purpose
///
/// Verifies that `.count` rejects an unrecognised `target::` value with exit
/// code 1 and an error message, per the Exit Code table in commands.md
/// (`1` = argument error).
///
/// ## Why Not Caught
///
/// All existing count tests used only valid target values (`projects`,
/// `sessions`, `entries`). No test covered the invalid-value path.
///
/// ## Prevention
///
/// Enumerated parameters must always have at least one test for an invalid
/// value to confirm the error path is exercised.
#[ test ]
fn test_count_invalid_target_exits_1()
{
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".count", "target::invalid" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    ".count with invalid target must fail. Got: {combined}"
  );

  assert_eq!(
    output.status.code(),
    Some( 1 ),
    ".count with invalid target must exit 1. Code: {:?}, output: {combined}",
    output.status.code()
  );
}
