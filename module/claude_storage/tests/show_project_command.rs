//! Test coverage for `.show.project` command
//!
//! ## Purpose
//!
//! Validates the `.show.project` command: path-based project lookup, Path(...)
//! format parsing, error handling for nonexistent projects, and verbosity range
//! validation (Finding #016).
//!
//! ## Testing Strategy
//!
//! All tests use `CLAUDE_STORAGE_ROOT` + `TempDir` for full isolation.
//! No test touches the real `~/.claude/` storage directory.

mod common;

use tempfile::TempDir;

/// Test `.show.project project::/path` — path-based project lookup
///
/// Verifies that providing a filesystem path to `project::` correctly
/// identifies and displays the project stored under that path.
#[ test ]
fn test_show_project_with_path()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "test-session-1-uuid",
    2,
  );
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "test-session-2-uuid",
    2,
  );

  let output = common::clg_cmd()
    .args( [
      ".show.project",
      &format!( "project::{}", project_path.path().display() ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Command should succeed. stderr: {stderr}"
  );

  assert!(
    stdout.contains( &project_path.path().to_string_lossy().to_string() )
    || stdout.contains( "Project:" ),
    "Should display project path. stdout: {stdout}"
  );
}

/// Test `.show.project project::Path(...)` — debug-format path parsing
///
/// Verifies that the Path(...) debug format output by `.list` can be pasted
/// directly into `.show.project project::`. This is the primary UX improvement:
/// copy path from `.list`, paste into `.show.project`.
#[ test ]
fn test_show_project_from_list_output()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "test-session.jsonl",
    2,
  );

  let list_output = format!( r#"Path("{}")"#, project_path.path().display() );

  let output = common::clg_cmd()
    .args( [
      ".show.project",
      &format!( "project::{list_output}" ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .unwrap();

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should handle Path(...) format from .list. stderr: {stderr}"
  );

  assert!(
    stdout.contains( &project_path.path().to_string_lossy().to_string() )
    || stdout.contains( "Project:" ),
    "Should parse Path(...) format and display project. stdout: {stdout}"
  );
}

/// Test `.show.project` with nonexistent project fails gracefully
#[ test ]
fn test_show_project_nonexistent()
{
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".show.project", "project::/nonexistent/path/to/project-test-show" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .unwrap();

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    !output.status.success(),
    "Should fail for nonexistent project"
  );

  assert!(
    stderr.contains( "not found" ) || stderr.contains( "Project" ) || stderr.contains( "project" ),
    "Should have clear error message. stderr: {stderr}"
  );
}

/// Test `.show.project verbosity::-1` fails — negative verbosity rejected (Finding #016)
///
/// ## Root Cause
///
/// `show_project_routine` retrieved verbosity with `get_integer("verbosity").unwrap_or(1)`
/// but never validated the 0-5 range constraint. Values like -1 or 10 were silently
/// passed to `show_project_impl` and `show_project_for_cwd_impl`, unlike `status_routine`
/// and `show_routine` which validate immediately after `get_integer`.
///
/// ## Why Not Caught
///
/// No parameter validation tests existed for `.show.project` verbosity. Existing tests
/// only verified functionality with the default verbosity.
///
/// ## Fix Applied
///
/// Added explicit range check `if !(0..=5).contains(&verbosity)` in `show_project_routine`
/// immediately after `get_integer("verbosity").unwrap_or(1)`, matching the validation
/// pattern from `status_routine` (lines 88-99).
///
/// ## Prevention
///
/// Every command with a `verbosity` parameter must validate the 0-5 range at routine
/// entry before passing the value to implementation functions. When adding new commands,
/// search for all `get_integer("verbosity")` call sites and verify each has a range check.
///
/// ## Pitfall
///
/// Passing an unvalidated verbosity to impl functions propagates the invalid value into
/// format/output logic, which may produce silent errors or garbage output rather than
/// a clear validation failure. Always validate at the routine boundary, not inside impl.
///
/// Related: Finding #016, spec.md REQ-008
// test_kind: validation(finding-016)
#[ test ]
fn test_show_project_verbosity_negative()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "sess-vn-001",
    2,
  );

  let output = common::clg_cmd()
    .args( [
      ".show.project",
      &format!( "project::{}", project_path.path().display() ),
      "verbosity::-1",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .show.project" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with verbosity::-1. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "verbosity" ) ||
    combined.to_lowercase().contains( "invalid" ),
    "Error should mention verbosity or invalid. Got: {combined}"
  );
}

/// Test `.show.project verbosity::6` fails — above-range verbosity rejected (Finding #016)
///
/// Same root cause as `test_show_project_verbosity_negative`. Tests the upper boundary.
///
/// Related: Finding #016
// test_kind: validation(finding-016)
#[ test ]
fn test_show_project_verbosity_out_of_range()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "sess-vor-001",
    2,
  );

  let output = common::clg_cmd()
    .args( [
      ".show.project",
      &format!( "project::{}", project_path.path().display() ),
      "verbosity::6",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .show.project" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with verbosity::6. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "verbosity" ) ||
    combined.to_lowercase().contains( "invalid" ),
    "Error should mention verbosity or invalid. Got: {combined}"
  );
}

/// Test `.show.project verbosity::0` succeeds — minimum boundary (Finding #016)
///
/// Related: Finding #016
// test_kind: validation(finding-016)
#[ test ]
fn test_show_project_verbosity_zero()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "sess-v0-001",
    2,
  );

  let output = common::clg_cmd()
    .args( [
      ".show.project",
      &format!( "project::{}", project_path.path().display() ),
      "verbosity::0",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .show.project" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".show.project verbosity::0 should succeed. stderr: {stderr}"
  );
}

/// Test `.show.project verbosity::5` succeeds — maximum boundary (Finding #016)
///
/// Related: Finding #016
// test_kind: validation(finding-016)
#[ test ]
fn test_show_project_verbosity_max()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "sess-v5-001",
    2,
  );

  let output = common::clg_cmd()
    .args( [
      ".show.project",
      &format!( "project::{}", project_path.path().display() ),
      "verbosity::5",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .show.project" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".show.project verbosity::5 should succeed. stderr: {stderr}"
  );
}

/// Test `.show.project verbosity::1` single-entry session shows "(1 entry)" not "(1 entries)"
///
/// ## Root Cause
///
/// `show_project_routine` (verbosity 1 branch) listed each session as
/// `"  - {id} ({count} entries, last: {date})"` using the hardcoded plural "entries".
/// When a session had exactly 1 entry this produced "(1 entries, last: ...)" — grammatically
/// wrong and inconsistent with plural rules applied elsewhere.
///
/// ## Why Not Caught
///
/// All prior `.show.project` tests used sessions with ≥2 entries.  The plural branch happens
/// to be correct for count ≥2, so no existing test exposed the singular bug.
///
/// ## Fix Applied
///
/// Added `let e_noun = if session_stats.total_entries == 1 { "entry" } else { "entries" };`
/// and replaced the hardcoded "entries" in the format string with `{e_noun}`.
///
/// ## Prevention
///
/// Every count-bearing format string must derive the noun from the count.
/// Add explicit singular-value (count == 1) tests alongside plural-value tests.
///
/// ## Pitfall
///
/// "entry" is an irregular noun — "entry"/"entries", not "entry"/"entrys".
/// Hardcoding the plural form silently breaks the singular case.
// bug_reproducer(issue-028)
#[ test ]
fn test_show_project_single_entry_session_says_entry_not_entries()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  // Write a session with exactly 1 entry so the singular path is exercised
  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "sess-single-entry-028",
    1,
  );

  let output = common::clg_cmd()
    .args( [
      ".show.project",
      &format!( "project::{}", project_path.path().display() ),
      "verbosity::1",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .show.project" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".show.project verbosity::1 with 1-entry session should succeed. stderr: {stderr}"
  );

  // Before fix: "(1 entries, last: ...)" — wrong plural
  // After fix:  "(1 entry, last: ...)"   — correct singular
  assert!(
    stdout.contains( "(1 entry," ),
    "Single-entry session should show '(1 entry, ...' not '(1 entries, ...'. stdout: {stdout}"
  );

  assert!(
    !stdout.contains( "(1 entries," ),
    "Should NOT show '(1 entries, ...' for count==1. stdout: {stdout}"
  );
}

/// Regression guard: two-entry session still shows "(2 entries)" after issue-028 fix
///
/// Ensures the plural noun is still emitted correctly for count > 1 after the
/// singular-noun fix was applied.
// regression_guard(issue-028)
#[ test ]
fn test_show_project_multi_entry_session_still_says_entries()
{
  let storage = TempDir::new().unwrap();
  let project_path = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_path.path(),
    "sess-multi-entry-028",
    2,
  );

  let output = common::clg_cmd()
    .args( [
      ".show.project",
      &format!( "project::{}", project_path.path().display() ),
      "verbosity::1",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .show.project" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".show.project verbosity::1 with 2-entry session should succeed. stderr: {stderr}"
  );

  assert!(
    stdout.contains( "(2 entries," ),
    "Two-entry session should show '(2 entries, ...'. stdout: {stdout}"
  );
}
