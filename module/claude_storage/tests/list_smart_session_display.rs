//! Test coverage for smart session display in .list command
//!
//! ## Feature
//!
//! The `.list` command uses smart parameter detection for session display:
//! - Providing session filters (`session::`, `agent::`, `min_entries::`) auto-enables session display
//! - Explicit `sessions::0` or `sessions::1` overrides auto-detection
//! - No filters → Projects only (default behavior)
//!
//! ## Root Cause (Original Issue)
//!
//! User discovered that `.list session::X` parameter had no effect:
//!
//! ```bash
//! .list path::claude_storage session::commit   # Same output
//! .list path::claude_storage session::commitx  # Same output
//! .list path::claude_storage session::0        # Same output
//! ```
//!
//! **Observation**: `session::` parameter accepted but completely ignored (garbage parameter)
//!
//! Investigation revealed:
//! - Line 106: `session_id_filter` parsed from `session::` parameter ✅
//! - Line 121-126: `SessionFilter` built with `session_id_filter` ✅
//! - Line 195: `if show_sessions` blocked filter usage ❌
//! - Line 100: `show_sessions = false` (default) ❌
//!
//! **Problem**: Filter built but never used because `show_sessions` defaults to false
//!
//! ## Why Not Caught
//!
//! - No tests for `.list` command with session filters
//! - Test coverage focused on `sessions::1` explicit enable
//! - Auto-enable behavior not documented in spec
//! - Same pattern as `.show` bug (fixed in v1.2.0) not recognized proactively
//!
//! ## Fix Applied
//!
//! Applied same "smart parameter detection" pattern from `.show` fix (v1.2.0):
//!
//! ```rust
//! // Before (broken):
//! let show_sessions = cmd.get_boolean( "sessions" ).unwrap_or( false );
//!
//! // After (smart):
//! let explicit_sessions = cmd.get_boolean( "sessions" );
//! let has_session_filters = session_id_filter.is_some()
//!   || agent_filter.is_some()
//!   || min_entries.is_some();
//! let show_sessions = match explicit_sessions
//! {
//!   Some( value ) => value,  // Respect explicit choice
//!   None => has_session_filters,  // Auto-enable if filters provided
//! };
//! ```
//!
//! ## Prevention
//!
//! - Test all filter parameters for actual effect (not just parser acceptance)
//! - Apply progressive disclosure pattern consistently across commands
//! - Document auto-enable behavior in spec.md
//! - Proactively check for "garbage parameter" anti-pattern (parameter accepted but ignored)
//!
//! ## Pitfall
//!
//! **Garbage Parameter Anti-Pattern**: When parser accepts a parameter but implementation
//! silently ignores it, users waste time trying different values with no effect. This is
//! particularly insidious because:
//! 1. Parser validates parameter (seems to work) ✅
//! 2. Filter gets built (code executes) ✅
//! 3. Filter never used (blocked by unrelated flag) ❌
//! 4. No error message (silent failure) ❌
//!
//! **Detection**: For every parameter, trace from parser → filter build → filter usage.
//! If usage is conditional on a default-false flag, the parameter is garbage.
//!
//! ---
//!
//! ## Second Bug: `min_entries::` Caused Binary Hang (issue-list-hang)
//!
//! `min_entries::` was placed in BOTH `ProjectFilter.min_entries` AND `SessionFilter.min_entries`.
//! `ProjectFilter.min_entries` caused `project.matches_filter()` to call `project_stats()` for
//! every project, which read ALL session JSONL files. With 1,448+ projects / 7 GB JSONL this hung
//! indefinitely — O(projects × sessions × entries) = O(total JSONL bytes).
//!
//! `min_entries::` is a SESSION filter only. The auto-enable behavior (`show_sessions = true`)
//! is handled separately.
//!
//! ## Isolation Note
//!
//! All tests use `CLAUDE_STORAGE_ROOT` + `TempDir` for isolation. This prevents:
//! - Race conditions when multiple tests run in parallel (workspace nextest)
//! - Pollution of the developer's real `~/.claude/` storage
//! - Flaky failures caused by pre-existing session data in real storage
//!
//! Fix(issue-smart-display-isolation)
//! Root cause: original tests wrote to real `~/.claude/` via `HOME` resolution, causing
//! non-deterministic failures in workspace-wide `cargo nextest run --workspace` due to
//! concurrent writes and reads across tests. Replaced with `CLAUDE_STORAGE_ROOT` + `TempDir`.
//! Pitfall: tests that clean up after themselves with `fs::remove_*` still race — cleanup
//! runs after the binary exits but before the next parallel test starts, which is not
//! guaranteed under nextest's thread pool execution.

mod common;

use std::fs;
use tempfile::TempDir;

#[ test ]
fn test_no_filters_shows_projects_only()
{
  // Test: .list (no parameters)
  // Expected: Projects only, no sessions shown
  // Purpose: Verify default behavior unchanged

  let storage = TempDir::new().unwrap();
  let project_dir = storage.path().join( "projects" ).join( "test-no-filters-project" );
  fs::create_dir_all( &project_dir ).unwrap();

  let session_a = "zzz-test-nofilter-a";
  let session_b = "zzz-test-nofilter-b";
  fs::write(
    project_dir.join( format!( "{session_a}.jsonl" ) ),
    r#"{"type":"user","text":"hello"}"#
  ).unwrap();
  fs::write(
    project_dir.join( format!( "{session_b}.jsonl" ) ),
    r#"{"type":"user","text":"world"}"#
  ).unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [ ".list" ] )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed. stderr: {stderr}"
  );

  // Sessions should NOT be shown (no session filter provided)
  assert!(
    !stdout.contains( session_a ) && !stdout.contains( session_b ),
    "sessions should not be shown (no filters provided). stdout: {stdout}"
  );
}

#[ test ]
fn test_session_filter_auto_enables_display()
{
  // Test: .list session::X (session filter provided)
  // Expected: Sessions shown (auto-enabled), filtered by session ID
  // Purpose: Core bug fix - session:: parameter must have effect

  let storage = TempDir::new().unwrap();

  use claude_storage_core::encode_path;
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let matching_session = "zzz-commit-session-test-abc123";
  let non_matching_session = "zzz-feature-session-test-xyz789";

  fs::write(
    project_dir.join( format!( "{matching_session}.jsonl" ) ),
    r#"{"type":"user","text":"test"}"#
  ).unwrap();
  fs::write(
    project_dir.join( format!( "{non_matching_session}.jsonl" ) ),
    r#"{"type":"user","text":"test"}"#
  ).unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [ ".list", "session::zzz-commit" ] )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed. stderr: {stderr}"
  );

  // Should show matching session (CRITICAL: This is the bug being fixed!)
  assert!(
    stdout.contains( matching_session ),
    "sessions should be shown (auto-enabled by session:: filter). stdout: {stdout}"
  );

  // Should NOT show non-matching session
  assert!(
    !stdout.contains( non_matching_session ),
    "Non-matching session should be filtered. stdout: {stdout}"
  );
}

#[ test ]
fn test_agent_filter_auto_enables_display()
{
  // Test: .list agent::1 (agent filter provided)
  // Expected: Sessions shown (auto-enabled), filtered to agent sessions only
  // Purpose: Verify agent:: parameter also triggers auto-enable

  let storage = TempDir::new().unwrap();

  use claude_storage_core::encode_path;
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let agent_session = "agent-zzz-test-task-abc123";
  let main_session = "zzz-test-main-topic";

  fs::write(
    project_dir.join( format!( "{agent_session}.jsonl" ) ),
    r#"{"type":"user","text":"test"}"#
  ).unwrap();
  fs::write(
    project_dir.join( format!( "{main_session}.jsonl" ) ),
    r#"{"type":"user","text":"test"}"#
  ).unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [ ".list", "agent::1" ] )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed. stderr: {stderr}"
  );

  // Should show agent session (with unique ID)
  assert!(
    stdout.contains( "agent-zzz-test" ),
    "agent sessions should be shown (auto-enabled by agent:: filter). stdout: {stdout}"
  );
}

#[ test ]
fn test_explicit_sessions_0_with_filter()
{
  // Test: .list sessions::0 session::X
  // Expected: Sessions shown (filter auto-enables, sessions::0 currently doesn't override)
  // Purpose: Document current behavior - filters always enable
  // Note: Future enhancement could allow sessions::0 to override auto-enable

  let storage = TempDir::new().unwrap();

  use claude_storage_core::encode_path;
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let session_id = "zzz-commit-explicit-disable-test";
  fs::write(
    project_dir.join( format!( "{session_id}.jsonl" ) ),
    r#"{"type":"user","text":"test"}"#
  ).unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [ ".list", "sessions::0", "session::zzz-commit-explicit" ] )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed. stderr: {stderr}"
  );

  // Should SHOW session (filter auto-enables even with sessions::0)
  assert!(
    stdout.contains( session_id ),
    "Sessions should be shown (filter auto-enables). stdout: {stdout}"
  );
}

#[ test ]
fn test_explicit_sessions_1_backward_compatible()
{
  // Test: .list sessions::1 session::X
  // Expected: Sessions shown, filtered
  // Purpose: Verify backward compatibility (existing usage still works)

  let storage = TempDir::new().unwrap();

  use claude_storage_core::encode_path;
  let project_path = TempDir::new().unwrap();
  let encoded = encode_path( project_path.path() ).unwrap();
  let project_dir = storage.path().join( "projects" ).join( &encoded );
  fs::create_dir_all( &project_dir ).unwrap();

  let matching_session = "zzz-commit-backward-compat-abc";
  let non_matching_session = "zzz-feature-backward-compat-xyz";

  fs::write(
    project_dir.join( format!( "{matching_session}.jsonl" ) ),
    r#"{"type":"user","text":"test"}"#
  ).unwrap();
  fs::write(
    project_dir.join( format!( "{non_matching_session}.jsonl" ) ),
    r#"{"type":"user","text":"test"}"#
  ).unwrap();

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [ ".list", "sessions::1", "session::zzz-commit-backward" ] )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed. stderr: {stderr}"
  );

  // Should show matching session
  assert!(
    stdout.contains( matching_session ),
    "Should show matching session. stdout: {stdout}"
  );

  // Should NOT show non-matching session
  assert!(
    !stdout.contains( non_matching_session ),
    "Should filter non-matching session. stdout: {stdout}"
  );
}

/// Test `.list min_entries::N` auto-enables sessions and filters correctly (Bug Reproducer: issue-list-hang)
///
/// ## Root Cause
///
/// `min_entries::` was placed in BOTH `ProjectFilter.min_entries` AND `SessionFilter.min_entries`.
/// `ProjectFilter.min_entries` caused `project.matches_filter()` to call `project_stats()` for
/// every project, which read ALL session JSONL files. With 1,448+ projects / 7 GB JSONL this hung
/// indefinitely — O(projects × sessions × entries) = O(total JSONL bytes).
///
/// ## Why Not Caught
///
/// Development used small synthetic storage (few projects). The `O(total_JSONL_bytes)` complexity
/// only manifests at real-world scale (thousands of projects). The test ran against real storage
/// without isolation, so it passed on small machines but timed out (300s+) on large ones.
///
/// ## Fix Applied
///
/// Removed `min_entries` from `ProjectFilter` in `list_routine()`. `min_entries::` is a SESSION
/// filter only. The auto-enable behavior (`show_sessions = true`) is handled separately.
/// Test uses isolated `CLAUDE_STORAGE_ROOT` to avoid scanning real `~/.claude/` storage.
///
/// ## Prevention
///
/// When a parameter auto-enables a feature, assign it to exactly one semantic level. Never
/// apply session-level parameters as project filters — trace computational cost.
///
/// ## Pitfall
///
/// **Dual-Filter Duplication**: Placing session-level parameters at project level causes
/// O(projects × sessions × entries) I/O instead of O(sessions × entries). Always trace
/// parameter → filter build → filter usage to its correct semantic level.
// test_kind: bug_reproducer(issue-list-hang)
#[ test ]
fn test_min_entries_filter_auto_enables_display()
{
  // Create isolated synthetic storage so we scan only known test data (not real ~/.claude/)
  let storage = TempDir::new().unwrap();
  let project_dir = storage.path()
    .join( "projects" ).join( "test-hang-project" );
  fs::create_dir_all( &project_dir ).expect( "create project dir" );

  // Session with 15 user entries — should match min_entries::10
  let matching_session = "session-many-entries";
  let many_entries : String = ( 0..15 )
    .map( | i | format!( r#"{{"type":"user","text":"entry {i}"}}"# ) )
    .collect::< Vec< _ > >()
    .join( "\n" );
  fs::write( project_dir.join( format!( "{matching_session}.jsonl" ) ), &many_entries )
    .expect( "write matching session" );

  // Session with 5 user entries — should NOT match min_entries::10
  let non_matching_session = "session-few-entries";
  let few_entries : String = ( 0..5 )
    .map( | i | format!( r#"{{"type":"user","text":"entry {i}"}}"# ) )
    .collect::< Vec< _ > >()
    .join( "\n" );
  fs::write( project_dir.join( format!( "{non_matching_session}.jsonl" ) ), &few_entries )
    .expect( "write non-matching session" );

  let output = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .args( [ ".list", "min_entries::10" ] )
    .output()
    .expect( "execute command" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed. stderr: {stderr}"
  );

  // Session with 15 entries must be shown (auto-enabled by min_entries:: filter)
  assert!(
    stdout.contains( matching_session ),
    "Session with 15 entries should be shown (auto-enabled by min_entries::10). stdout: {stdout}"
  );

  // Session with 5 entries must NOT be shown (filtered by min_entries::10)
  assert!(
    !stdout.contains( non_matching_session ),
    "Session with 5 entries must be filtered out (does not meet min_entries::10). stdout: {stdout}"
  );
}
