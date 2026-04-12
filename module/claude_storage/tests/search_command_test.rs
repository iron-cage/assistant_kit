//! Search Command Parameter Validation Tests
//!
//! ## Purpose
//!
//! Validates parameter validation for .search command per REQ-012 specification.
//! Tests ensure proper error handling before command implementation.
//!
//! ## Coverage
//!
//! Validates 7 validation requirements (V-012.1 through V-012.7):
//! - query parameter required and non-empty
//! - `case_sensitive` accepts only 0 or 1
//! - `entry_type` accepts only user, assistant, or all
//! - verbosity range 0-5
//! - project existence validation (when search implemented)
//! - session existence validation (when search implemented)
//!
//! ## Testing Strategy
//!
//! - Parameter validation tests: Run immediately (command will fail, we check error messages)
//! - Integration tests: Run against the `search_routine` implementation in `src/cli/mod.rs`
//! - Uses same pattern as `parameter_validation_test.rs` for consistency
//!
//! ## Related Requirements
//!
//! REQ-012: Search Command specification (spec.md:458-519)

mod common;

/// Test .search query parameter is required (V-012.1)
///
/// ## Purpose
/// Validates that .search enforces required query parameter per REQ-012 V-012.1.
///
/// ## Coverage
/// Tests missing parameter case. Verifies error message mentions "query"
/// and "required" per spec error message standard.
///
/// ## Validation Strategy
/// Execute .search without query parameter. Assert:
/// - Command fails (non-zero exit)
/// - Error contains "query"
/// - Error contains "required"
///
/// ## Related Requirements
/// REQ-012 V-012.1: Reject missing query parameter
#[test]
fn test_search_query_required()
{
  let output = common::clg_cmd()
    .args( [ ".search" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail when query missing. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "query" ) &&
    combined.to_lowercase().contains( "required" ),
    "Error should mention query is required. Got: {combined}"
  );
}

/// Test .search query parameter cannot be empty (V-012.2)
///
/// ## Purpose
/// Validates that .search rejects empty query string per REQ-012 V-012.2.
///
/// ## Coverage
/// Tests empty string edge case. Empty parameter values should be rejected
/// with clear error message.
///
/// ## Validation Strategy
/// Execute .search with empty query (`query::`). Assert:
/// - Command fails (non-zero exit)
/// - Error contains "query"
/// - Error contains "empty" or "cannot be empty"
///
/// ## Related Requirements
/// REQ-012 V-012.2: Reject empty query string
#[test]
fn test_search_query_empty()
{
  let output = common::clg_cmd()
    .args( [ ".search", "query::" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail when query empty. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "query" ) &&
    ( combined.to_lowercase().contains( "empty" ) ||
      combined.to_lowercase().contains( "expected value" ) ),
    "Error should mention query validation. Got: {combined}"
  );
}

/// Test .search `case_sensitive` parameter validation (V-012.3)
///
/// ## Purpose
/// Validates that `case_sensitive` accepts only 0 or 1 per REQ-012 V-012.3.
///
/// ## Coverage
/// Tests invalid boolean value. Boolean parameters should only accept
/// 0 (false) or 1 (true).
///
/// ## Validation Strategy
/// Execute .search with `case_sensitive::2` (invalid boolean). Assert:
/// - Command fails (non-zero exit)
/// - Error mentions "`case_sensitive`" or "invalid"
///
/// ## Related Requirements
/// REQ-012 V-012.3: Validate `case_sensitive` accepts only 0 or 1
#[test]
fn test_search_case_sensitive_invalid()
{
  let output = common::clg_cmd()
    .args( [ ".search", "query::test", "case_sensitive::2" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with invalid case_sensitive value. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "case" ) ||
    combined.to_lowercase().contains( "invalid" ),
    "Error should mention case_sensitive or invalid. Got: {combined}"
  );
}

/// Test .search `entry_type` parameter validation (V-012.4)
///
/// ## Purpose
/// Validates that `entry_type` accepts only user, assistant, or all per REQ-012 V-012.4.
///
/// ## Coverage
/// Tests invalid enumerated value. Enumerated parameters should validate
/// against allowed values and reject invalid ones with clear error message.
///
/// ## Validation Strategy
/// Execute .search with `entry_type::invalid`. Assert:
/// - Command fails (non-zero exit)
/// - Error mentions "`entry_type`" or "invalid"
///
/// ## Related Requirements
/// REQ-012 V-012.4: Validate `entry_type` accepts only user, assistant, or all
#[test]
fn test_search_entry_type_invalid()
{
  let output = common::clg_cmd()
    .args( [ ".search", "query::test", "entry_type::invalid" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with invalid entry_type. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "entry" ) ||
    combined.to_lowercase().contains( "type" ) ||
    combined.to_lowercase().contains( "invalid" ),
    "Error should mention entry_type or invalid. Got: {combined}"
  );
}

/// Test .search `entry_type` accepts valid values (V-012.4)
///
/// ## Purpose
/// Validates that `entry_type` accepts all valid values (user, assistant, all) per REQ-012 V-012.4.
///
/// ## Coverage
/// Tests all three valid `entry_type` values with real isolated session data.
///
/// ## Validation Strategy
/// Write a session with user+assistant entries. Search with each `entry_type`.
/// Assert exit 0 and no validation error in stderr.
///
/// ## Related Requirements
/// REQ-012 V-012.4: `entry_type` enumerated validation — valid values: user, assistant, all
#[test]
fn test_search_entry_type_valid()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  // Write session with alternating user/assistant entries containing "entry"
  common::write_test_session( storage.path(), "search-proj-et", "sess-et-001", 4 );

  // Valid entry_type values: "user", "assistant", and "all"
  // "all" is equivalent to omitting the parameter — searches both types
  for entry_type in [ "user", "assistant", "all" ]
  {
    let output = common::clg_cmd()
      .args( [
        ".search",
        "query::entry",
        &format!( "entry_type::{entry_type}" ),
        "project::search-proj-et",
      ] )
      .env( "CLAUDE_STORAGE_ROOT", storage.path() )
      .output()
      .expect( "Failed to execute .search" );

    let stderr = String::from_utf8_lossy( &output.stderr );

    assert!(
      output.status.success(),
      "Search with entry_type::{entry_type} should succeed. stderr: {stderr}"
    );

    let has_validation_error =
      stderr.to_lowercase().contains( "entry" ) &&
      stderr.to_lowercase().contains( "type" ) &&
      stderr.to_lowercase().contains( "invalid" );

    assert!(
      !has_validation_error,
      "Should not fail on entry_type validation for '{entry_type}'. stderr: {stderr}"
    );
  }
}

/// Test .search `entry_type::all` is a valid value (`bug_reproducer` issue-021)
///
/// ## Root Cause
/// `search_routine` only handled "user" and "assistant" in the `entry_type` match,
/// treating "all" as invalid despite the YAML spec documenting it as valid
/// ("Filter by entry type (user, assistant, or all)"). The match arm fell through
/// to the error branch for any value other than "user" or "assistant".
///
/// ## Why Not Caught
/// The existing `test_search_entry_type_valid` test commented "all is NOT supported"
/// and only iterated over `["user", "assistant"]`, documenting the broken behavior
/// instead of testing against the YAML spec.
///
/// ## Fix Applied
/// Added "all" match arm in `search_routine` (src/cli/mod.rs) that skips calling
/// `filter.match_entry_type()`, making it equivalent to omitting the parameter.
///
/// ## Prevention
/// Enumerated parameter validation tests must cover ALL values documented in the
/// YAML spec description, not just values the developer remembered to implement.
/// When a help description says "user, assistant, or all", test all three.
///
/// ## Pitfall
/// Documenting missing functionality as "not supported" in tests instead of fixing
/// it. Test comments that say "X is not valid" without a spec reference are a smell —
/// always check the YAML spec first.
// test_kind: bug_reproducer(issue-021)
#[test]
fn test_search_entry_type_all_is_valid_bug_reproducer_issue_021()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  common::write_test_session( storage.path(), "search-proj-all", "sess-all-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".search", "query::entry", "entry_type::all", "project::search-proj-all" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .search" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    output.status.success(),
    "entry_type::all must be accepted as valid per YAML spec. Got: {combined}"
  );

  assert!(
    !combined.contains( "Invalid entry_type" ),
    "entry_type::all must not produce invalid-value error. Got: {combined}"
  );
}

/// Test .search verbosity parameter range validation (Finding #010)
///
/// ## Root Cause
/// `search_routine` in src/cli/mod.rs:1171 retrieved verbosity parameter without
/// validating the 0-5 range constraint, unlike `status_routine` and `show_routine`
/// which include explicit range validation. This inconsistency allowed invalid
/// values like -1 or 10 to be accepted and used.
///
/// ## Why Not Caught
/// .search command had no parameter validation tests. The existing search tests
/// only verified functionality with valid parameters. No tests checked edge cases
/// or invalid parameter values.
///
/// ## Fix Applied
/// Added explicit verbosity range validation (0-5) in `search_routine` at line 1190,
/// matching the validation pattern used in `status_routine` (line 18) and
/// `show_routine` (line 650). Returns clear error message with actual value and
/// valid range when validation fails.
///
/// ## Prevention
/// All parameters with constrained ranges must validate at routine entry, not
/// just in commands added later. When adding new commands, audit existing commands
/// for similar parameters and apply consistent validation patterns. Parameters
/// with defaults still require validation since users can override with invalid values.
///
/// ## Pitfall
/// Don't assume default values prevent invalid input. A parameter with `default::1`
/// can still receive invalid values from user input. Validation is required even
/// when defaults are sensible.
///
/// Related: REQ-012 V-012.5
#[test]
fn test_search_verbosity_invalid()
{
  // Test negative value
  let output = common::clg_cmd()
    .args( [ ".search", "query::test", "verbosity::-1" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with verbosity::-1. Got: {combined}"
  );

  // Test value too large
  let output = common::clg_cmd()
    .args( [ ".search", "query::test", "verbosity::10" ] )
    .current_dir( env!( "CARGO_MANIFEST_DIR" ) )
    .output()
    .expect( "Failed to execute command" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with verbosity::10. Got: {combined}"
  );
}

/// Test .search project existence validation (V-012.6)
///
/// ## Purpose
/// Validates that .search checks project exists when specified per REQ-012 V-012.6.
///
/// ## Coverage
/// Tests project parameter with nonexistent project ID in isolated empty storage.
///
/// ## Validation Strategy
/// Set `CLAUDE_STORAGE_ROOT` to empty temp dir. Run `.search` with nonexistent project ID.
/// Assert exit 1 + error mentions "project" and "not found".
///
/// ## Related Requirements
/// REQ-012 V-012.6: Validate project exists when specified
#[test]
fn test_search_project_nonexistent()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".search", "query::test", "project::nonexistent-uuid-12345" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .search" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with nonexistent project. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "project" ) &&
    ( combined.to_lowercase().contains( "not found" ) ||
      combined.to_lowercase().contains( "does not exist" ) ||
      combined.to_lowercase().contains( "no project" ) ),
    "Error should mention project not found. Got: {combined}"
  );
}

/// Test .search session existence validation (V-012.7)
///
/// ## Purpose
/// Validates that .search checks session exists when specified per REQ-012 V-012.7.
///
/// ## Coverage
/// Creates a real project in isolated storage. Runs .search with nonexistent session ID.
///
/// ## Validation Strategy
/// Create real project + session. Search with a different, nonexistent session ID.
/// Assert exit 1 + error mentions "session" and "not found".
///
/// ## Related Requirements
/// REQ-012 V-012.7: Validate session exists when specified
#[test]
fn test_search_session_nonexistent()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  // Create a real project so the project lookup succeeds
  common::write_test_session( storage.path(), "search-proj-sne", "real-session-9999", 2 );

  let output = common::clg_cmd()
    .args( [
      ".search",
      "query::test",
      "session::nonexistent-session-id-xyz",
      "project::search-proj-sne",
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .search" );

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

// ─────────────────────────────────────────────────────────────────────────────
// issue-025 regression: "Found 1 matches:" uses wrong plural — must be
// "Found 1 match:" (singular).
//
// Root Cause: search_routine always formats the count noun as "matches"
// regardless of count. English grammar requires singular when count == 1.
//
// Why Not Caught: No existing search test asserted the exact header form.
// Tests only verified success status or presence of "Found" keyword.
//
// Fix Applied: Derive noun ("match" vs "matches") from count, use in header.
//
// Prevention: Assert exact-string header form in integration tests, not just
// command success status.
//
// Pitfall: "Found 0 matches:" stays plural — zero takes plural in English.
// ─────────────────────────────────────────────────────────────────────────────

/// Test `.search` outputs singular "Found 1 match:" when exactly 1 match found.
///
/// Uses `project::` to restrict search to the single test project (avoids
/// `load_project_for_cwd()` which requires CWD to match a real storage project).
///
/// bug_reproducer(issue-025)
// test_kind: bug_reproducer(issue-025)
#[test]
fn test_search_singular_noun_one_match()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  // 2 entries: "entry 0" (user) + "response" (assistant).
  // Query "entry 0" matches only the first entry → exactly 1 match.
  common::write_test_session( storage.path(), "search-proj-sing", "sess-sing-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".search", "query::entry 0", "project::search-proj-sing", "verbosity::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .search" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!(
    output.status.success(),
    ".search must succeed; stderr: {stderr}"
  );
  assert!(
    stdout.contains( "Found 1 match:" ),
    "with 1 match, header must use singular 'match' (not 'matches'); got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "Found 1 matches:" ),
    "with 1 match, header must NOT use plural 'matches'; got:\n{stdout}"
  );
}

/// Test `.search` outputs plural "Found N matches:" when multiple matches found.
///
/// bug_reproducer(issue-025)
// test_kind: bug_reproducer(issue-025)
#[test]
fn test_search_plural_noun_multiple_matches()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  // 4 entries — all contain "entry" in their content: "entry 0", "entry 1", etc.
  common::write_test_session( storage.path(), "search-proj-plur", "sess-plur-001", 4 );

  let output = common::clg_cmd()
    .args( [ ".search", "query::entry", "project::search-proj-plur", "verbosity::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .search" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!(
    output.status.success(),
    ".search must succeed; stderr: {stderr}"
  );
  // 4 entries all contain "entry" — must produce plural header
  assert!(
    !stdout.contains( "Found 1 match:" ),
    "with multiple matches, must not show singular 'match'; got:\n{stdout}"
  );
  if let Some( line ) = stdout.lines().find( | l | l.starts_with( "Found" ) )
  {
    assert!(
      line.contains( "matches:" ),
      "multi-match header must use plural 'matches'; got: {line}"
    );
  }
}
