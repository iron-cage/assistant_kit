//! Status Command Path Parameter Tests
//!
//! ## Purpose
//!
//! Validates path parameter functionality for .status command.
//! Tests ensure proper handling of default path, custom paths, and error cases.
//!
//! ## Coverage
//!
//! Validates path parameter behavior:
//! - Default path usage (no parameter specified)
//! - Custom path support (`path::` parameter)
//! - Nonexistent path handling (error case)
//! - Empty path validation (error case)
//! - Path with verbosity interaction (parameter combination)
//!
//! ## Testing Strategy
//!
//! - Feature tests: Run immediately (verify existing path parameter functionality)
//! - Uses real filesystem with tempfile crate for integration testing
//! - Follows same pattern as `search_command_test.rs` and `export_command_test.rs`
//!
//! ## Related Requirements
//!
//! .status path parameter documentation (spec.md:272-276)

mod common;

use std::fs;
use tempfile::TempDir;

/// Test `.status` uses `CLAUDE_STORAGE_ROOT` when no path parameter specified
///
/// ## Purpose
/// Validates that `.status` command respects the `CLAUDE_STORAGE_ROOT` env var
/// when no path parameter is provided (covers the default-path code path).
///
/// ## Coverage
/// Tests default parameter behavior with isolated storage. Creates 2 known
/// projects in temp storage, verifies `.status` reports them correctly.
///
/// ## Validation Strategy
/// Write 2 projects to temp storage. Run `.status` with `CLAUDE_STORAGE_ROOT`.
/// Assert exit 0 and output shows project information.
///
/// ## Related Requirements
/// `.status` path parameter: default behavior uses `CLAUDE_STORAGE_ROOT` or `~/.claude/`
#[test]
fn test_status_default_path()
{
  let storage = TempDir::new().expect( "create temp storage" );

  // Create 2 projects so we have something meaningful to report
  common::write_test_session( storage.path(), "status-proj-alpha", "s001", 2 );
  common::write_test_session( storage.path(), "status-proj-beta", "s001", 2 );

  let output = common::clg_cmd()
    .args( [ ".status" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .status" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    output.status.success(),
    "Should succeed with default path via CLAUDE_STORAGE_ROOT. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "project" ) ||
    combined.to_lowercase().contains( "storage" ),
    "Should show storage information. Got: {combined}"
  );
}

/// Test .status accepts custom path parameter
///
/// ## Purpose
/// Validates that .status command can use custom storage path via `path::` parameter.
///
/// ## Coverage
/// Tests custom path parameter functionality. Should successfully execute
/// using specified storage location instead of default.
///
/// ## Validation Strategy
/// Setup: Create temp directory with projects/ subdirectory structure
/// Execute .status with `path::{temp_dir`}. Assert:
/// - Command succeeds (zero exit)
/// - Output shows 0 projects (empty storage)
///
/// ## Related Requirements
/// .status path parameter: accepts custom storage location
#[test]
fn test_status_custom_path()
{
  // Create temp directory structure
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let storage_path = temp_dir.path();
  let projects_dir = storage_path.join( "projects" );
  fs::create_dir_all( &projects_dir ).expect( "Failed to create projects dir" );

  let output = common::clg_cmd()
    .args( [ ".status", &format!( "path::{}", storage_path.display() ) ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    output.status.success(),
    "Should succeed with custom path. Got: {combined}"
  );

  // Should report 0 projects in empty storage
  assert!(
    combined.contains( '0' ) && combined.to_lowercase().contains( "project" ),
    "Should show 0 projects in empty storage. Got: {combined}"
  );
}

/// Test .status handles nonexistent path gracefully
///
/// ## Purpose
/// Validates that .status command succeeds with nonexistent path and reports
/// empty storage (0 projects) rather than failing.
///
/// ## Coverage
/// Tests graceful handling of nonexistent paths. Command should succeed and
/// report 0 projects for nonexistent paths, allowing users to check status
/// of new/empty storage locations.
///
/// ## Validation Strategy
/// Execute .status with `path::/nonexistent/path/12345`. Assert:
/// - Command succeeds (zero exit)
/// - Reports 0 projects (empty storage)
///
/// ## Related Requirements
/// .status path parameter: gracefully handles nonexistent paths
#[test]
fn test_status_nonexistent_path()
{
  let output = common::clg_cmd()
    .args( [ ".status", "path::/nonexistent/path/12345" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    output.status.success(),
    "Should succeed with nonexistent path. Got: {combined}"
  );

  // Should report 0 projects for nonexistent path
  assert!(
    combined.contains( '0' ) && combined.to_lowercase().contains( "project" ),
    "Should show 0 projects for nonexistent path. Got: {combined}"
  );
}

/// Test .status rejects empty path parameter
///
/// ## Purpose
/// Validates that .status command rejects empty path parameter value.
///
/// ## Coverage
/// Tests empty string edge case. Empty parameter values should be rejected
/// with clear error message.
///
/// ## Validation Strategy
/// Execute .status with `path::` (empty value). Assert:
/// - Command fails (non-zero exit)
/// - Error mentions "path" or "expected value"
///
/// ## Related Requirements
/// .status path parameter: rejects empty path values
#[test]
fn test_status_empty_path()
{
  let output = common::clg_cmd()
    .args( [ ".status", "path::" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with empty path. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "path" ) ||
    combined.to_lowercase().contains( "expected value" ),
    "Error should mention path validation. Got: {combined}"
  );
}

/// Test .status `path::`. resolves to current directory (Finding #014)
///
/// ## Root Cause
/// `status_routine` at line 74 passes path directly to `Storage::with_root()` without
/// resolving special path markers. While `list_routine` uses `resolve_path_parameter()`,
/// `status_routine` does not, causing ".", "..", "~" to be used literally.
///
/// ## Why Not Caught
/// Existing tests used only explicit full paths or temp directories. No tests
/// exercised special path markers (".", "..", "~") in the path parameter.
///
/// ## Fix Applied
/// Added `resolve_path_parameter()` call in `status_routine` before passing to
/// `Storage::with_root()`, consistent with `list_routine` pattern.
///
/// ## Prevention
/// When multiple commands share similar parameters, ensure they use the same
/// helper functions. The `resolve_path_parameter()` helper exists specifically
/// for this purpose but was not consistently applied.
///
/// ## Pitfall
/// Adding new commands by copying existing code without understanding shared
/// utilities leads to inconsistent behavior between commands.
#[test]
fn test_status_path_dot_resolves_to_cwd()
{
  let manifest_dir = env!( "CARGO_MANIFEST_DIR" );

  let output = common::clg_cmd()
    .args( [ ".status", "path::." ] )
    .current_dir( manifest_dir )
    .output()
    .expect( "Failed to execute command" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );
  let combined = format!( "{stdout}{stderr}" );

  // Bug behavior: Shows Storage: "."
  // Fixed behavior: Shows resolved path like Storage: "/home/.../claude_storage"

  let has_literal_dot = combined.contains( r#"Storage: ".""# );

  assert!(
    !has_literal_dot,
    "Bug: path::. not resolved, shows literal '.' in Storage.\n\
    Expected: Resolved absolute path\n\
    Got: {combined}"
  );
}

/// Test .status `path::`~ resolves to home directory (Finding #014)
///
/// ## Purpose
/// Validates that `status_routine` resolves ~ to home directory.
#[test]
fn test_status_path_tilde_resolves_to_home()
{
  let manifest_dir = env!( "CARGO_MANIFEST_DIR" );

  let output = common::clg_cmd()
    .args( [ ".status", "path::~" ] )
    .current_dir( manifest_dir )
    .output()
    .expect( "Failed to execute command" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );
  let combined = format!( "{stdout}{stderr}" );

  // Bug behavior: Shows Storage: "~"
  // Fixed behavior: Shows resolved path like Storage: "/home/user"

  let has_literal_tilde = combined.contains( r#"Storage: "~""# );

  assert!(
    !has_literal_tilde,
    "Bug: path::~ not resolved, shows literal '~' in Storage.\n\
    Expected: Resolved home directory path\n\
    Got: {combined}"
  );
}

/// Test .status path parameter works with verbosity parameter
///
/// ## Purpose
/// Validates that .status command correctly handles path and verbosity
/// parameters together, ensuring no parameter interaction issues.
///
/// ## Coverage
/// Tests parameter combination. Path and verbosity should work independently
/// without conflicts.
///
/// ## Validation Strategy
/// Setup: Create temp directory with projects/ subdirectory
/// Execute .status with `path::{temp`} and `verbosity::2`. Assert:
/// - Command succeeds (zero exit)
/// - Output shows storage information
/// - Both parameters are respected
///
/// ## Related Requirements
/// .status path parameter: works with other parameters
#[test]
fn test_status_path_with_verbosity()
{
  // Create temp directory structure
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let storage_path = temp_dir.path();
  let projects_dir = storage_path.join( "projects" );
  fs::create_dir_all( &projects_dir ).expect( "Failed to create projects dir" );

  let output = common::clg_cmd()
    .args( [ ".status", &format!( "path::{}", storage_path.display() ), "verbosity::2" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    output.status.success(),
    "Should succeed with path and verbosity. Got: {combined}"
  );

  // Should show storage information
  assert!(
    combined.to_lowercase().contains( "project" ) ||
    combined.to_lowercase().contains( "storage" ),
    "Should show storage information. Got: {combined}"
  );
}

/// Test .status `v::2` correctly reports User/Assistant entry breakdown (`bug_reproducer` issue-020)
///
/// ## Root Cause
/// `global_stats()` in `storage.rs` aggregated `total_entries` from `project_stats()`
/// but never aggregated `total_user_entries` or `total_assistant_entries`. The comment
/// "For now, we approximate this from total entries" was in the code but no approximation
/// was implemented — both fields were left at their zero-initialized defaults.
/// `ProjectStats` had no user/assistant breakdown fields at all, so `global_stats()`
/// had no source to aggregate them from.
///
/// ## Why Not Caught
/// The existing `test_status_path_with_verbosity` only checked that `verbosity::2` exits 0
/// and contains "project"/"storage" — it never asserted the entry breakdown values.
/// A test that doesn't check output values cannot catch wrong output values.
///
/// ## Fix Applied
/// Added `total_user_entries` / `total_assistant_entries` fields to `ProjectStats`.
/// Populated them in `project.project_stats()` from session stats.
/// Aggregated them in `storage.global_stats()`.
///
/// ## Prevention
/// When adding stats output to a command, write a test that checks the NUMERIC VALUES
/// in that output, not just that the output exists or the command succeeds.
/// A stats test that doesn't verify numbers is not a stats test.
///
/// ## Pitfall
/// "For now, we approximate" comments in code are time-bombs. They signal incomplete
/// implementation. Always leave a failing test or a TODO compile error (via
/// `compile_error!`) rather than a silent wrong value.
#[test]
fn test_status_verbosity2_user_assistant_counts_bug_reproducer_issue_020()
{
  let storage = TempDir::new().expect( "create temp dir" );

  // 2 user + 2 assistant entries across 2 sessions
  common::write_test_session( storage.path(), "status-test-proj", "sess-count-a", 4 );

  let output = common::clg_cmd()
    .args( [ ".status", "v::2" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .status" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    output.status.success(),
    ".status v::2 should succeed. Got: {combined}"
  );

  // Must NOT show "User: 0, Assistant: 0" — both must be non-zero for a session
  // with both user and assistant entries
  assert!(
    !combined.contains( "User: 0, Assistant: 0" ),
    ".status v::2 must report non-zero user/assistant counts. Got: {combined}"
  );

  // Must show correct breakdown: 2 user + 2 assistant = 4 total
  assert!(
    combined.contains( "User: 2" ),
    ".status v::2 must report User: 2. Got: {combined}"
  );

  assert!(
    combined.contains( "Assistant: 2" ),
    ".status v::2 must report Assistant: 2. Got: {combined}"
  );
}
