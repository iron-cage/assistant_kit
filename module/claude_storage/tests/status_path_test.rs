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
#[ test ]
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
#[ test ]
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

/// Test .status rejects nonexistent path with an error
///
/// ## Purpose
/// Validates that .status command fails with a clear error message when
/// the specified storage root does not exist.
///
/// ## Coverage
/// Tests error handling for nonexistent paths. Command should fail (exit non-zero)
/// and emit an error message on stderr describing the missing path.
///
/// ## Validation Strategy
/// Execute .status with `path::/nonexistent/path/12345`. Assert:
/// - Command fails (non-zero exit)
/// - Error message present on stderr
///
/// ## Related Requirements
/// .status path parameter: rejects nonexistent storage roots with exit 2
#[ test ]
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
    !output.status.success(),
    "Should fail with nonexistent path. Got: {combined}"
  );

  // Error message must be present
  assert!(
    !stderr.is_empty(),
    "Should emit error on stderr for nonexistent path. Got stdout: {stdout}"
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
#[ test ]
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
#[ test ]
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
#[ test ]
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

