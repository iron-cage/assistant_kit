//! Export Command Parameter Validation Tests
//!
//! ## Purpose
//!
//! Validates parameter validation for .export command per REQ-013 specification.
//! Tests ensure proper error handling for required parameters and format validation.
//!
//! ## Coverage
//!
//! Validates 5 validation requirements (V-013.1 through V-013.5):
//! - `session_id` parameter required
//! - output parameter required
//! - format accepts only markdown, json, or text
//! - session existence validation (when export executed)
//! - output directory existence validation (when export executed)
//!
//! ## Testing Strategy
//!
//! - Parameter validation tests: Run immediately (validate error messages)
//! - Integration tests: Use `CLAUDE_STORAGE_ROOT` + `TempDir` for isolation
//! - Uses same pattern as `search_command_test.rs` for consistency
//!
//! ## Related Requirements
//!
//! REQ-013: Export Command specification (spec.md:521-586)

mod common;

use tempfile::TempDir;

/// Test .export `session_id` parameter is required (V-013.1)
///
/// ## Purpose
/// Validates that .export enforces required `session_id` parameter per REQ-013 V-013.1.
///
/// ## Coverage
/// Tests missing parameter case. Verifies error message mentions "`session_id`"
/// and "required" per spec error message standard.
///
/// ## Validation Strategy
/// Execute .export without `session_id` parameter. Assert:
/// - Command fails (non-zero exit)
/// - Error contains "session" or "`session_id`"
/// - Error contains "required"
///
/// ## Related Requirements
/// REQ-013 V-013.1: Reject missing `session_id` parameter
#[test]
fn test_export_session_id_required()
{
  let output = common::clg_cmd()
    .args( [ ".export", "output::/tmp/test.md" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail when session_id missing. Got: {combined}"
  );

  assert!(
    ( combined.to_lowercase().contains( "session" ) ||
      combined.to_lowercase().contains( "session_id" ) ) &&
    combined.to_lowercase().contains( "required" ),
    "Error should mention session_id is required. Got: {combined}"
  );
}

/// Test .export output parameter is required (V-013.2)
///
/// ## Purpose
/// Validates that .export enforces required output parameter per REQ-013 V-013.2.
///
/// ## Coverage
/// Tests missing parameter case. Verifies error message mentions "output"
/// and "required" per spec error message standard.
///
/// ## Validation Strategy
/// Execute .export without output parameter. Assert:
/// - Command fails (non-zero exit)
/// - Error contains "output"
/// - Error contains "required"
///
/// ## Related Requirements
/// REQ-013 V-013.2: Reject missing output parameter
#[test]
fn test_export_output_required()
{
  let output = common::clg_cmd()
    .args( [ ".export", "session_id::test" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail when output missing. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "output" ) &&
    combined.to_lowercase().contains( "required" ),
    "Error should mention output is required. Got: {combined}"
  );
}

/// Test .export format parameter validation (V-013.3)
///
/// ## Purpose
/// Validates that format accepts only markdown, json, or text per REQ-013 V-013.3.
///
/// ## Coverage
/// Tests invalid enumerated value. Format parameter should validate against
/// allowed values (markdown, json, text) and reject others with clear error.
///
/// ## Validation Strategy
/// Execute .export with `format::csv` (not supported). Assert:
/// - Command fails (non-zero exit)
/// - Error mentions "format" or "invalid"
///
/// ## Related Requirements
/// REQ-013 V-013.3: Validate format accepts only markdown, json, or text
#[test]
fn test_export_format_invalid()
{
  let output = common::clg_cmd()
    .args( [ ".export", "session_id::test", "output::/tmp/test.csv", "format::csv" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with invalid format. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "format" ) ||
    combined.to_lowercase().contains( "invalid" ),
    "Error should mention format validation. Got: {combined}"
  );
}

/// Test .export format accepts valid values (V-013.3)
///
/// ## Purpose
/// Validates that format accepts all three valid values (markdown, json, text)
/// and that export completes successfully for each format.
///
/// ## Coverage
/// Tests all three valid format values with a real session in isolated storage.
///
/// ## Validation Strategy
/// Write a real session, run .export for each format, assert exit 0 and file created.
///
/// ## Related Requirements
/// REQ-013 V-013.3: format enumerated validation
#[test]
fn test_export_format_valid()
{
  let storage = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();

  let session_id = "aabbcc11-1111-2222-3333-444444444444";
  common::write_test_session( storage.path(), "export-proj", session_id, 4 );

  for ( format, ext ) in [ ( "markdown", "md" ), ( "json", "json" ), ( "text", "txt" ) ]
  {
    let out_file = out_dir.path().join( format!( "export.{ext}" ) );

    let output = common::clg_cmd()
      .args( [
        ".export",
        &format!( "session_id::{session_id}" ),
        &format!( "output::{}", out_file.display() ),
        &format!( "format::{format}" ),
        "project::export-proj",
      ] )
      .env( "CLAUDE_STORAGE_ROOT", storage.path() )
      .output()
      .expect( "Failed to execute .export" );

    let stderr = String::from_utf8_lossy( &output.stderr );
    let stdout = String::from_utf8_lossy( &output.stdout );
    let combined = format!( "{stderr}{stdout}" );

    assert!(
      output.status.success(),
      ".export format::{format} should succeed. Got: {combined}"
    );

    assert!(
      out_file.exists(),
      ".export format::{format} should create output file"
    );
  }
}

/// Test .export session existence validation (V-013.4)
///
/// ## Purpose
/// Validates that .export reports an error when the specified session doesn't exist.
///
/// ## Coverage
/// Tests session parameter with nonexistent session ID in a real project.
///
/// ## Validation Strategy
/// Create a real project in isolated storage. Run .export with a session ID that
/// doesn't exist. Assert exit 1 + error contains "not found".
///
/// ## Related Requirements
/// REQ-013 V-013.4: Validate session exists in project
#[test]
fn test_export_session_nonexistent()
{
  let storage = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();

  // Create a real project so the project lookup succeeds
  common::write_test_session( storage.path(), "export-proj-ne", "real-session-001", 2 );

  let out_file = out_dir.path().join( "out.md" );

  let output = common::clg_cmd()
    .args( [
      ".export",
      "session_id::nonexistent-session-99999",
      &format!( "output::{}", out_file.display() ),
      "project::export-proj-ne",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .export" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with nonexistent session. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "session" ) &&
    ( combined.to_lowercase().contains( "not found" ) ||
      combined.to_lowercase().contains( "does not exist" ) ||
      combined.to_lowercase().contains( "no session" ) ),
    "Error should mention session not found. Got: {combined}"
  );
}

/// Test .export output directory existence validation (V-013.5)
///
/// ## Purpose
/// Validates that .export fails when the output directory doesn't exist.
///
/// ## Coverage
/// Creates a real session, then runs .export with a nonexistent output path.
///
/// ## Validation Strategy
/// Create a real project + session. Run .export with output in a nonexistent directory.
/// Assert exit 1 + error mentions directory/path issue.
///
/// ## Related Requirements
/// REQ-013 V-013.5: Validate output path directory exists
#[test]
fn test_export_output_directory_nonexistent()
{
  let storage = TempDir::new().unwrap();

  let session_id = "aabbcc22-1111-2222-3333-000000000000";
  common::write_test_session( storage.path(), "export-proj-dir", session_id, 2 );

  let output = common::clg_cmd()
    .args( [
      ".export",
      &format!( "session_id::{session_id}" ),
      "output::/nonexistent/directory/path/out.md",
      "project::export-proj-dir",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .export" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with nonexistent output directory. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "directory" ) ||
    combined.to_lowercase().contains( "path" ) ||
    combined.to_lowercase().contains( "not found" ) ||
    combined.to_lowercase().contains( "does not exist" ) ||
    combined.to_lowercase().contains( "no such" ),
    "Error should mention directory/path issue. Got: {combined}"
  );
}

/// Test .export with both required parameters
///
/// ## Purpose
/// Validates that providing both required parameters passes parameter validation
/// and allows the command to proceed to session lookup.
///
/// ## Coverage
/// Creates a real session. Exports it. Verifies command succeeds.
///
/// ## Validation Strategy
/// Create real session + run .export with both required params. Assert exit 0.
///
/// ## Related Requirements
/// REQ-013 V-013.1, V-013.2: Required parameters
#[test]
fn test_export_with_required_parameters()
{
  let storage = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();

  let session_id = "aabbcc33-1111-2222-3333-555555555555";
  common::write_test_session( storage.path(), "export-proj-req", session_id, 3 );

  let out_file = out_dir.path().join( "required_params.md" );

  let output = common::clg_cmd()
    .args( [
      ".export",
      &format!( "session_id::{session_id}" ),
      &format!( "output::{}", out_file.display() ),
      "project::export-proj-req",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .export" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );

  assert!(
    output.status.success(),
    "Should succeed with both required params. stderr: {stderr}, stdout: {stdout}"
  );
}

/// Test .export output path included in error when directory missing (issue-026)
///
/// ## Root Cause
/// `export_session_to_file` used bare `?` on `File::create(output_path)`, which
/// converts `io::Error` via the blanket `From<io::Error> for Error` impl. That impl
/// always sets context to "unknown operation", producing "I/O error during unknown
/// operation: No such file or directory" with no mention of which path failed.
///
/// ## Why Not Caught
/// The existing `test_export_output_directory_nonexistent` only checked that *some*
/// directory/path-related word appeared in the error. "No such file or directory"
/// satisfies that check without requiring the output path itself to appear.
///
/// ## Fix Applied
/// Changed `File::create(output_path)?` to use `.map_err(|e| Error::io(e, format!(...)))`.
/// The context now includes the output path, producing: "I/O error during create output
/// file '/path/to/file.md': No such file or directory".
///
/// ## Prevention
/// Always use `Error::io(e, context)` when wrapping IO errors that benefit from path
/// context. Bare `?` on IO operations silently strips identifying information.
///
/// ## Pitfall
/// The blanket `From<io::Error> for Error` impl unconditionally sets context to
/// "unknown operation". Every IO error propagated via `?` loses its context.
/// Use `.map_err(|e| Error::io(e, ctx))` for any IO operation where the path matters.
#[test]
fn test_export_output_path_in_error_message()
{
  let storage = TempDir::new().unwrap();

  let session_id = "aabbcc55-1111-2222-3333-777777777777";
  common::write_test_session( storage.path(), "export-proj-err", session_id, 2 );

  let output = common::clg_cmd()
    .args( [
      ".export",
      &format!( "session_id::{session_id}" ),
      "output::/nonexistent/dir/export.md",
      "project::export-proj-err",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .export" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with nonexistent output directory. Got: {combined}"
  );

  // bug_reproducer(issue-026): before fix, error said "I/O error during unknown operation"
  // with no path context. After fix, error includes the output file path.
  assert!(
    combined.contains( "/nonexistent/dir/export.md" ),
    "Error must include the output path for diagnostics; got: {combined}"
  );
  assert!(
    !combined.contains( "unknown operation" ),
    "Error must not say 'unknown operation' — use specific file context; got: {combined}"
  );
}

/// Test .export default format is markdown
///
/// ## Purpose
/// Validates that .export uses markdown as default format when format parameter
/// not specified. Verifies the output file contains markdown content.
///
/// ## Coverage
/// Creates a real session. Exports without format parameter. Checks output file.
///
/// ## Validation Strategy
/// Create real session, run .export without format, verify output file is markdown.
///
/// ## Related Requirements
/// REQ-013: Default format is markdown
#[test]
fn test_export_default_format_markdown()
{
  let storage = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();

  let session_id = "aabbcc44-1111-2222-3333-666666666666";
  common::write_test_session( storage.path(), "export-proj-def", session_id, 2 );

  let out_file = out_dir.path().join( "default_format.md" );

  let output = common::clg_cmd()
    .args( [
      ".export",
      &format!( "session_id::{session_id}" ),
      &format!( "output::{}", out_file.display() ),
      "project::export-proj-def",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .export" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    output.status.success(),
    "Default format (markdown) should succeed. Got: {combined}"
  );

  assert!(
    out_file.exists(),
    "Output file should be created with default format"
  );

  // Markdown output should have # heading
  let content = std::fs::read_to_string( &out_file ).unwrap();
  assert!(
    content.contains( '#' ),
    "Markdown output should contain headers. Got:\n{content}"
  );
}
