//! Bug reproducer: `.search session::` rejects partial UUID (issue-020)
//!
//! ## Root Cause
//!
//! `search_routine` uses exact equality (`s.id() == sess_id`) to locate
//! the session when a `session::` filter is provided.  `.show` and `.export`
//! both use prefix matching (`s.id() == sid || s.id().starts_with(sid)`),
//! so a user who discovers the 8-char UUID shortcut in one command expects
//! it to work across all commands that take a session parameter.
//!
//! ## Why Not Caught
//!
//! The partial-UUID fix (issue-011) was applied to `format_session_output`
//! and `export_routine` but not to `search_routine`.  All existing search
//! tests used full session IDs written by `write_test_session`, so the
//! exact-only path was never exercised with a real UUID prefix.
//!
//! ## Fix Applied
//!
//! Changed the session lookup in `search_routine` from exact match to prefix
//! match: `s.id() == sess_id || s.id().starts_with(sess_id)`, consistent
//! with `show_routine` and `export_routine`.
//!
//! ## Prevention
//!
//! When applying a partial-ID-matching fix to one session lookup, grep for
//! every other `sessions.iter*().find(|s| s.id() == ...)` call in the
//! codebase and apply the same change.  Add a prefix-ID test for every
//! command that accepts a `session` parameter so regressions are caught
//! immediately.
//!
//! ## Pitfall
//!
//! Partial UUID support must be applied uniformly across all commands.  If
//! `.show` accepts 8-char prefixes but `.search` requires the full UUID,
//! users who discover the shortcut in one command will waste time debugging
//! why the identical input fails in another.

mod common;

use tempfile::TempDir;

/// Test `.search session::PREFIX` finds matches using a partial UUID (issue-020)
///
/// ## Root Cause
///
/// `search_routine` used `s.id() == sess_id` (exact match) while `.show` and
/// `.export` use `s.id().starts_with(sess_id)`.  Providing the first 8 chars
/// of a UUID as the session filter returned "Session not found" instead of
/// searching the matched session.
///
/// ## Why Not Caught
///
/// All existing search tests supplied complete session IDs.  The inconsistency
/// with `.show` partial-UUID support was not exercised until manual corner-case
/// testing revealed the discrepancy.
///
/// ## Fix Applied
///
/// Changed `find(|s| s.id() == sess_id)` to
/// `find(|s| s.id() == sess_id || s.id().starts_with(sess_id))` in the session-
/// scoped branch of `search_routine`.
///
/// ## Prevention
///
/// Any command that looks up a session by ID must use prefix matching.  After
/// applying any session-lookup fix, search for all other `.find(|s| s.id() ==`
/// patterns and verify they also use prefix matching.
///
/// ## Pitfall
///
/// Copying the `find` predicate from a freshly written routine without checking
/// whether existing commands already support prefix matching introduces
/// inconsistency.  Always grep for existing patterns before writing a new find.
// test_kind: bug_reproducer(issue-020)
#[ test ]
fn test_search_session_partial_uuid_match()
{
  let storage = TempDir::new().unwrap();
  let session_uuid = "79f86582-1435-442c-935a-13f8d874918a";
  let session_prefix = "79f86582";

  // Write 2 entries so the search has something to find
  common::write_test_session( storage.path(), "search-partial-proj", session_uuid, 2 );

  // Bug: partial prefix returns "Session not found: 79f86582"
  // Fixed: partial prefix matches the full UUID and returns search results
  let output = common::clg_cmd()
    .args( [
      ".search",
      "query::entry",
      "project::search-partial-proj",
      &format!( "session::{session_prefix}" ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .search" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".search with partial session UUID must succeed. stderr: {stderr}"
  );

  assert!(
    !stdout.contains( "Session not found" ),
    "Must not report 'Session not found' for a valid prefix. stdout: {stdout}"
  );

  // The synthetic entries contain "entry N" — at least 1 match expected
  assert!(
    stdout.contains( "match" ) || stdout.contains( "entry" ),
    "Expected search results for query 'entry'. stdout: {stdout}, stderr: {stderr}"
  );
}

/// Regression: full UUID still works after partial-match fix (issue-020)
///
/// ## Purpose
///
/// Confirms that adding `starts_with` to the predicate does not break
/// exact-UUID lookups, which must continue to work as before.
///
/// ## Coverage
///
/// Full UUID in `session::` parameter returns correct results.
///
/// ## Validation Strategy
///
/// Exact same session UUID as in the prefix test — verifies the `==` branch
/// still fires and returns results.
///
/// ## Related Requirements
///
/// Consistent session lookup behaviour across all commands (issue-020 fix).
#[ test ]
fn test_search_session_full_uuid_still_works()
{
  let storage = TempDir::new().unwrap();
  let session_uuid = "79f86582-1435-442c-935a-13f8d874918a";

  common::write_test_session( storage.path(), "search-full-uuid-proj", session_uuid, 2 );

  let output = common::clg_cmd()
    .args( [
      ".search",
      "query::entry",
      "project::search-full-uuid-proj",
      &format!( "session::{session_uuid}" ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .search" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    ".search with full UUID must still succeed. stderr: {stderr}"
  );

  assert!(
    stdout.contains( "match" ) || stdout.contains( "entry" ),
    "Expected search results. stdout: {stdout}"
  );
}
