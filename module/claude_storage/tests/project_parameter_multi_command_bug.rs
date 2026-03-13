//! Bug fix test for project parameter handling across multiple commands
//!
//! ## Root Cause (Finding #012)
//!
//! Commands `.count`, `.search`, and `.export` hardcode `ProjectId::uuid(proj_id)`
//! at multiple locations instead of using the `parse_project_parameter()` helper.
//! This causes path projects to fail with "Project not found" because they try
//! to find a UUID-named directory instead of decoding the path.
//!
//! This is the same bug as Finding #008, but it was only fixed for `.show` command.
//! The fix was not propagated to other commands that accept project parameter.
//!
//! **Affected locations:**
//! - `count_routine` line 1171: `storage.load_project( &ProjectId::uuid( proj_id ) )`
//! - `count_routine` line 1187: `storage.load_project( &ProjectId::uuid( proj_id ) )`
//! - `search_routine` line 1280: `storage.load_project( &ProjectId::uuid( proj_id ) )`
//! - `search_routine` line 1307: `storage.load_project( &ProjectId::uuid( proj_id ) )`
//! - `export_routine` line 1436: `storage.load_project( &ProjectId::uuid( proj_id ) )`
//!
//! ## Why Not Caught
//!
//! No tests exercised `.count`, `.search`, or `.export` commands with path projects.
//! All test scenarios used UUID projects or avoided the project parameter entirely.
//!
//! ## Fix Applied
//!
//! Replace all `ProjectId::uuid()` calls with `parse_project_parameter()` to enable:
//! - Absolute paths → `ProjectId::Path`
//! - Path-encoded strings → decode then `ProjectId::Path`
//! - Debug format `Path("...")` → extract and use `ProjectId::Path`
//! - Otherwise → `ProjectId::Uuid`
//!
//! ## Prevention
//!
//! Add comprehensive test coverage for all commands that accept project parameter:
//! - `.count` with path projects
//! - `.search` with path projects
//! - `.export` with path projects
//!
//! ## Pitfall
//!
//! When fixing a bug in one location, always grep for similar patterns across
//! the entire codebase. Bugs often exist in multiple locations that share the
//! same flawed assumption (here: "project parameter is always a UUID").

mod common;

use tempfile::TempDir;

#[ test ]
fn test_count_with_path_project()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "count-session-1",
    2,
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
    !stderr.contains( "Project not found" ),
    "Bug: .count treats path project as UUID. stderr: {stderr}"
  );

  assert!(
    output.status.success(),
    ".count should succeed with path project. stderr: {stderr}"
  );

  assert!(
    stdout.contains( '1' ),
    "Should count 1 session. stdout: {stdout}"
  );
}

#[ test ]
fn test_count_entries_with_path_project()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  let session_id = "test-entries-session-abc";
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    session_id,
    2,
  );

  let output = common::clg_cmd()
    .args( [
      ".count",
      "target::entries",
      &format!( "session::{session_id}" ),
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    !stderr.contains( "Project not found" ),
    "Bug: .count entries treats path project as UUID. stderr: {stderr}"
  );

  assert!(
    output.status.success(),
    ".count entries should succeed with path project. stderr: {stderr}"
  );

  assert!(
    stdout.contains( '2' ),
    "Should count 2 entries. stdout: {stdout}"
  );
}

#[ test ]
fn test_search_with_path_project()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  // Write a session with searchable content ("entry 0", "entry 1", etc.)
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "search-session-abc",
    2,
  );

  let output = common::clg_cmd()
    .args( [
      ".search",
      "query::entry",
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .search" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    !stderr.contains( "Project not found" ),
    "Bug: .search treats path project as UUID. stderr: {stderr}"
  );

  assert!(
    output.status.success(),
    ".search should succeed with path project. stderr: {stderr}"
  );
}

#[ test ]
fn test_export_with_path_project()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();

  let session_id = "aabbcc55-1111-2222-3333-777777777777";
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    session_id,
    2,
  );

  let out_file = out_dir.path().join( "export_path_proj.md" );

  let output = common::clg_cmd()
    .args( [
      ".export",
      &format!( "session_id::{session_id}" ),
      &format!( "output::{}", out_file.display() ),
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .export" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    !stderr.contains( "Project not found" ),
    "Bug: .export treats path project as UUID. stderr: {stderr}"
  );

  assert!(
    output.status.success(),
    ".export should succeed with path project. stderr: {stderr}"
  );
}
