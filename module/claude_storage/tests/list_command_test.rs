//! List Command Parameter Validation Tests
//!
//! ## Purpose
//!
//! Validates parameter bounds and combinations for the `.list` command.
//! Covers Finding #015 (verbosity range) and session filter empty-string validation.
//!
//! ## Coverage
//!
//! - Verbosity range validation (0-5): values outside range rejected (Finding #015)
//! - Agent Boolean parameter: only 0 or 1 accepted (framework validation)
//! - Session filter: non-empty string required when provided
//! - Sessions Boolean: only 0 or 1 accepted (framework validation)
//! - Pairwise combinations: valid and invalid parameter combinations
//!
//! ## Testing Strategy
//!
//! All tests use `CLAUDE_STORAGE_ROOT` + `TempDir` for full isolation.
//! No test touches the real `~/.claude/` storage directory.
//!
//! ## Related Requirements
//!
//! REQ-010: List Command specification (spec.md)

mod common;

/// Test `.list verbosity::-1` fails — negative verbosity rejected (Finding #015)
///
/// ## Root Cause
///
/// `list_routine` retrieved verbosity with `get_integer("verbosity").unwrap_or(1)`
/// but never validated the 0-5 range constraint. Values like -1 or 10 were silently
/// accepted, unlike `status_routine` and `show_routine` which include explicit range
/// validation after the `get_integer` call.
///
/// ## Why Not Caught
///
/// The `.list` command had no parameter validation tests. Existing list tests only
/// verified functionality with valid parameters. No tests covered out-of-range values
/// or boundary conditions for the verbosity parameter.
///
/// ## Fix Applied
///
/// Added explicit range check `if !(0..=5).contains(&verbosity)` in `list_routine`
/// immediately after `get_integer("verbosity").unwrap_or(1)`, matching the pattern
/// from `status_routine` (lines 88-99). Returns `"Invalid verbosity: N. Valid range: 0-5"`.
///
/// ## Prevention
///
/// Every command with a `verbosity` parameter must validate the 0-5 range at routine
/// entry. When adding new commands, audit existing commands for inconsistent validation.
/// Parameters with defaults still require validation — `unwrap_or(1)` does not prevent
/// a user from passing `verbosity::-1` or `verbosity::10`.
///
/// ## Pitfall
///
/// `get_integer("verbosity").unwrap_or(1)` only substitutes the default when the
/// parameter is absent. When the user explicitly provides an out-of-range value, the
/// function returns that value — range validation is always the caller's responsibility.
///
/// Related: Finding #015, spec.md REQ-010
// test_kind: validation(finding-015)
#[ test ]
fn test_list_verbosity_negative()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".list", "verbosity::-1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

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

/// Test `.list verbosity::6` fails — above-range verbosity rejected (Finding #015)
///
/// Same root cause and fix as `test_list_verbosity_negative`. Tests the upper
/// boundary: `verbosity::5` is the maximum, `verbosity::6` must be rejected.
///
/// Related: Finding #015
// test_kind: validation(finding-015)
#[ test ]
fn test_list_verbosity_out_of_range()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".list", "verbosity::6" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

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

/// Test `.list verbosity::0` succeeds — minimum boundary accepted (Finding #015)
///
/// Verifies that the lower boundary of the valid range (0) is accepted after
/// Fix(issue-015). `verbosity::0` suppresses most output but is a valid value.
///
/// Related: Finding #015
// test_kind: validation(finding-015)
#[ test ]
fn test_list_verbosity_zero()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-proj-v0", "sess-v0-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "verbosity::0" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list verbosity::0 should succeed. stderr: {stderr}"
  );
}

/// Test `.list verbosity::3` succeeds — mid-range value accepted (Finding #015)
///
/// Related: Finding #015
// test_kind: validation(finding-015)
#[ test ]
fn test_list_verbosity_mid_range()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-proj-v3", "sess-v3-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "verbosity::3" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list verbosity::3 should succeed. stderr: {stderr}"
  );
}

/// Test `.list verbosity::5` succeeds — maximum boundary accepted (Finding #015)
///
/// Related: Finding #015
// test_kind: validation(finding-015)
#[ test ]
fn test_list_verbosity_max()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-proj-v5", "sess-v5-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "verbosity::5" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list verbosity::5 should succeed. stderr: {stderr}"
  );
}

/// Test `.list agent::2` fails — Boolean parameter rejects non-Boolean value
///
/// `agent` is declared as `Boolean` type in the CLI YAML specification. The
/// framework validates Boolean parameters and only accepts 0 or 1. Any other
/// value (including 2) must be rejected.
#[ test ]
fn test_list_agent_invalid()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".list", "agent::2" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with invalid Boolean agent::2. Got: {combined}"
  );
}

/// Test `.list agent::0` succeeds — Boolean false value
#[ test ]
fn test_list_agent_zero()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".list", "agent::0" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list agent::0 should succeed. stderr: {stderr}"
  );
}

/// Test `.list agent::1` succeeds — Boolean true value
#[ test ]
fn test_list_agent_one()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-proj-ag", "sess-ag-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "agent::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list agent::1 should succeed. stderr: {stderr}"
  );
}

/// Test `.list session::abc` succeeds — non-empty session substring filter
#[ test ]
fn test_list_session_filter()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-proj-sf", "sess-sf-abc", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "session::abc" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list session::abc should succeed. stderr: {stderr}"
  );
}

/// Test `.list session::` fails — empty session filter rejected by framework
///
/// The framework rejects `session::` (empty value for a String parameter) with a
/// parse error: "Expected value for named argument 'session' but found end of
/// instruction". No application-level validation is needed; the framework enforces
/// that String parameters must have a non-empty value.
///
/// ## Pitfall
///
/// Don't assume String parameters accept empty values. The unilang framework
/// treats `param::` (no value after `::`) as a parse error, not as `Some("")`.
/// This means application code never sees the empty-string case for String params.
// test_kind: validation(session-empty-filter)
#[ test ]
fn test_list_session_filter_empty()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".list", "session::" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with empty session filter. Got: {combined}"
  );

  assert!(
    combined.contains( "session" ),
    "Error should mention session parameter. Got: {combined}"
  );
}

/// Test `.list sessions::2` fails — Boolean parameter rejects non-Boolean value
///
/// `sessions` is declared as `Boolean` type. Framework validation rejects
/// any value other than 0 or 1.
#[ test ]
fn test_list_sessions_invalid()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".list", "sessions::2" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with invalid Boolean sessions::2. Got: {combined}"
  );
}

/// Pairwise: `.list type::uuid sessions::1` succeeds
#[ test ]
fn test_list_type_uuid_with_sessions()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-pair-uuid", "sess-uuid-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "type::uuid", "sessions::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list type::uuid sessions::1 should succeed. stderr: {stderr}"
  );
}

/// Pairwise: `.list type::path verbosity::2` succeeds
#[ test ]
fn test_list_type_path_with_verbosity()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-pair-path", "sess-path-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "type::path", "verbosity::2" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list type::path verbosity::2 should succeed. stderr: {stderr}"
  );
}

/// Pairwise: `.list sessions::0 verbosity::-1` fails — invalid verbosity
///
/// When multiple parameters are provided with one invalid value, the command
/// must fail. Verbosity range validation happens before storage access.
// test_kind: validation(finding-015)
#[ test ]
fn test_list_sessions_with_invalid_verbosity()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".list", "sessions::0", "verbosity::-1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with verbosity::-1 even alongside valid sessions::0. Got: {combined}"
  );
}

/// Test `.list` with no parameters succeeds (base case, covers agent absent)
#[ test ]
fn test_list_no_params()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-base-proj", "sess-base-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list with no params should succeed. stderr: {stderr}"
  );
}

/// Test `.list sessions::0` succeeds — explicit sessions-off
#[ test ]
fn test_list_sessions_zero()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-s0-proj", "sess-s0-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "sessions::0" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list sessions::0 should succeed. stderr: {stderr}"
  );
}

/// Test `.list sessions::1` succeeds — explicit sessions-on
#[ test ]
fn test_list_sessions_one()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-s1-proj", "sess-s1-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "sessions::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list sessions::1 should succeed. stderr: {stderr}"
  );
}

/// Test `.list min_entries::0` succeeds — zero `min_entries` is valid lower bound
#[ test ]
fn test_list_min_entries_zero()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-me-proj", "sess-me-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "min_entries::0" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list min_entries::0 should succeed. stderr: {stderr}"
  );
}

/// Test `.list path::/tmp` succeeds — substring path filter
#[ test ]
fn test_list_path_filter()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".list", "path::/tmp" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list path::/tmp should succeed. stderr: {stderr}"
  );
}

/// Test `.list type::all` succeeds — explicit all-types filter
#[ test ]
fn test_list_type_all()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-tall-proj", "sess-tall-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "type::all" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list type::all should succeed. stderr: {stderr}"
  );
}

/// Test `.list type::notvalid` fails — type parameter validates against allowed values
///
/// `list_routine` validates `type` against "uuid", "path", and "all". Any other
/// value returns `"Invalid type: X. Valid values: uuid, path, all"`.
#[ test ]
fn test_list_type_invalid()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".list", "type::notvalid" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with invalid type. Got: {combined}"
  );

  assert!(
    combined.to_lowercase().contains( "type" ) ||
    combined.to_lowercase().contains( "invalid" ),
    "Error should mention type or invalid. Got: {combined}"
  );
}

/// Test `.list verbosity::1` succeeds — default value accepted explicitly
#[ test ]
fn test_list_verbosity_default_explicit()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "list-vdef-proj", "sess-vdef-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "verbosity::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".list verbosity::1 (default) should succeed. stderr: {stderr}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// issue-025 regression: "Found 1 projects:" uses wrong plural — must be
// "Found 1 project:" (singular).
//
// Root Cause: list_routine always formats the count noun as "projects"
// regardless of count. English grammar requires singular when count == 1.
//
// Why Not Caught: No existing test asserted the exact singular/plural form of
// the "Found N projects:" header — only that the command succeeds.
//
// Fix Applied: Derive noun ("project" vs "projects") from the count, use
// it in the header format string.
//
// Prevention: Assert exact-string header form, not just command success.
//
// Pitfall: "Found 0 projects:" stays plural — zero takes plural in English.
// ─────────────────────────────────────────────────────────────────────────────

/// Test `.list` outputs singular "Found 1 project:" when exactly 1 project exists.
///
/// bug_reproducer(issue-025)
// test_kind: bug_reproducer(issue-025)
#[ test ]
fn test_list_singular_noun_one_project()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "only-proj", "sess-sing-001", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "verbosity::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!(
    output.status.success(),
    ".list with 1 project should succeed; stderr: {}",
    String::from_utf8_lossy( &output.stderr )
  );
  assert!(
    stdout.contains( "Found 1 project:" ),
    "with 1 project, header must use singular 'project'; got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "Found 1 projects:" ),
    "with 1 project, header must NOT use plural 'projects'; got:\n{stdout}"
  );
}

/// Test `.list` outputs plural "Found 2 projects:" when 2 projects exist.
///
/// bug_reproducer(issue-025)
// test_kind: bug_reproducer(issue-025)
#[ test ]
fn test_list_plural_noun_multiple_projects()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "proj-a", "sess-plur-a", 2 );
  common::write_test_session( storage.path(), "proj-b", "sess-plur-b", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "verbosity::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!(
    output.status.success(),
    ".list with 2 projects should succeed; stderr: {}",
    String::from_utf8_lossy( &output.stderr )
  );
  assert!(
    stdout.contains( "Found 2 projects:" ),
    "with 2 projects, header must use plural 'projects'; got:\n{stdout}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// issue-027 regression: ".list sessions::1" with 1 session shows "(1 sessions)"
// — wrong plural in per-project session count label (verbosity 1).
//
// Root Cause: list_routine verbosity==1 branch used hardcoded plural "sessions"
// in the `"{:?} ({} sessions)"` format string regardless of session_count value.
// English requires singular "session" when count == 1.
//
// Why Not Caught: The issue-025 fix addressed the "Found N X:" header noun but
// missed the per-project session count label on the same verbosity level. These
// are two separate format strings — fixing one left the other broken.
//
// Fix Applied: Derive noun ("session" vs "sessions") from session_count before
// the writeln! call, matching the pattern used for the "Found N X:" header fix.
//
// Prevention: When fixing plural nouns in a routine, audit ALL format strings
// that embed counts, not just the most visible one.
//
// Pitfall: Multiple format strings in the same routine can have the same bug.
// A targeted fix for one occurrence may miss siblings with identical patterns.
// ─────────────────────────────────────────────────────────────────────────────

/// Test `.list sessions::1` shows "(1 session)" (singular) when project has 1 session.
///
/// bug_reproducer(issue-027)
// test_kind: bug_reproducer(issue-027)
#[ test ]
fn test_list_session_count_singular_when_one_session()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  common::write_test_session( storage.path(), "sing-proj", "sess-sing-only", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "sessions::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!(
    output.status.success(),
    ".list sessions::1 should succeed; stderr: {}",
    String::from_utf8_lossy( &output.stderr )
  );
  assert!(
    stdout.contains( "(1 session)" ),
    "with 1 session, project label must use singular '(1 session)'; got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "(1 sessions)" ),
    "with 1 session, project label must NOT use plural '(1 sessions)'; got:\n{stdout}"
  );
}

/// Test `.list sessions::1` shows "(2 sessions)" (plural) when project has 2 sessions.
///
/// Regression guard for issue-027: plural form must remain correct for counts > 1.
// test_kind: regression_guard(issue-027)
#[ test ]
fn test_list_session_count_plural_when_multiple_sessions()
{
  use tempfile::TempDir;
  let storage = TempDir::new().unwrap();
  // Two sessions in the same project
  common::write_test_session( storage.path(), "plur-proj", "sess-plur-x", 2 );
  common::write_test_session( storage.path(), "plur-proj", "sess-plur-y", 2 );

  let output = common::clg_cmd()
    .args( [ ".list", "sessions::1" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "Failed to execute .list" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!(
    output.status.success(),
    ".list sessions::1 should succeed; stderr: {}",
    String::from_utf8_lossy( &output.stderr )
  );
  assert!(
    stdout.contains( "(2 sessions)" ),
    "with 2 sessions, project label must use plural '(2 sessions)'; got:\n{stdout}"
  );
}
