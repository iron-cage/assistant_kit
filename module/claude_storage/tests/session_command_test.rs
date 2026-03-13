//! `.session` command tests
//!
//! Validates that `.session` detects conversation history for a given directory.
//! Uses `claude_storage_core::continuation::check_continuation()` under the hood.
//!
//! ## Coverage
//!
//! - Default path (current directory) with no history -> "no history"
//! - Explicit `path::` with history -> "has history"
//! - Explicit `path::` with no history -> "no history"
//! - Help output mentions `.session`

mod common;

use std::fs;

/// Test .session with no history reports "no history"
///
/// Uses a temp directory as HOME and a temp directory as `session_dir`.
/// Since no `.claude/projects/` storage exists, `check_continuation` returns false.
#[ test ]
fn test_session_no_history()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session", &format!( "path::{}", project.path().display() ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}, stdout: {stdout}"
  );
  assert!(
    stdout.contains( "no history" ),
    "Should report 'no history'. Got: {stdout}"
  );
}

/// Test .session with history reports "has history"
///
/// Sets up a fake HOME with the right `.claude/projects/{encoded}/` structure
/// containing a non-empty .jsonl file, then verifies "has history".
#[ test ]
fn test_session_has_history()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  // Encode the project path the same way check_continuation does
  let encoded = claude_storage_core::encode_path( project.path() )
    .expect( "encode_path should succeed" );

  // Create storage directory with a non-empty .jsonl file
  let storage_dir = home.path().join( ".claude" ).join( "projects" ).join( &encoded );
  fs::create_dir_all( &storage_dir ).unwrap();
  fs::write( storage_dir.join( "session.jsonl" ), b"some content\n" ).unwrap();

  let output = common::clg_cmd()
    .args( [ ".session", &format!( "path::{}", project.path().display() ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}, stdout: {stdout}"
  );
  assert!(
    stdout.contains( "has history" ),
    "Should report 'has history'. Got: {stdout}"
  );
}

/// Test .session with empty .jsonl file reports "no history"
///
/// Claude Code creates 0-byte .jsonl files during initialization.
/// These should NOT count as conversation history.
#[ test ]
fn test_session_empty_file_no_history()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  let encoded = claude_storage_core::encode_path( project.path() )
    .expect( "encode_path should succeed" );

  let storage_dir = home.path().join( ".claude" ).join( "projects" ).join( &encoded );
  fs::create_dir_all( &storage_dir ).unwrap();
  // Empty file — should be skipped
  fs::write( storage_dir.join( "session.jsonl" ), b"" ).unwrap();

  let output = common::clg_cmd()
    .args( [ ".session", &format!( "path::{}", project.path().display() ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!(
    stdout.contains( "no history" ),
    "Empty .jsonl should not count as history. Got: {stdout}"
  );
}

/// Test `.session path::/nonexistent/path` reports "no history" (P)
///
/// A nonexistent filesystem path has no encoded project directory in storage,
/// so `check_continuation` returns false → "no history". The command must
/// succeed (exit 0) rather than error on a nonexistent path.
#[ test ]
fn test_session_path_nonexistent()
{
  let home = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session", "path::/nonexistent/path/for-session-test-xyz" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0 for nonexistent path. stderr: {stderr}, stdout: {stdout}"
  );
  assert!(
    stdout.contains( "no history" ),
    "Nonexistent path should report 'no history'. Got: {stdout}"
  );
}

/// Test `.session path::` (empty value) fails — framework parse error
///
/// The unilang framework rejects empty String parameter values with a parse
/// error. No application-level validation is needed for this case.
#[ test ]
fn test_session_path_empty()
{
  let output = common::clg_cmd()
    .args( [ ".session", "path::" ] )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with empty path. Got: {combined}"
  );

  assert!(
    combined.contains( "path" ),
    "Error should mention path parameter. Got: {combined}"
  );
}

/// Test help output mentions .session command
#[ test ]
fn test_help_mentions_session()
{
  let output = common::clg_cmd()
    .args( [ "help" ] )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );
  let combined = format!( "{stdout}{stderr}" );

  assert!(
    combined.contains( ".session" ),
    "Help should mention .session command. Got: {combined}"
  );
}
