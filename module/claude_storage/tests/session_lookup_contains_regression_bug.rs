//! Bug reproducer: session lookup matches a non-prefix substring anywhere in
//! the ID instead of only as a Git-style prefix (BUG-490)
//!
//! ## Root Cause
//!
//! `find_session_mut()` (`src/cli/storage.rs`) and `count_routine`'s own
//! duplicated "entries" lookup (`src/cli/count.rs`) both use
//! `s.id() == id || s.id().contains(id)`. Commit `a405168a` ("docs: restructure
//! CLI documentation...") accidentally changed both call sites from
//! `starts_with` to `contains` while performing unrelated refactor work in the
//! same commit — the doc comment directly above `find_session_mut` still reads
//! "Checks `s.id() == session_id || s.id().starts_with(session_id)`", and three
//! separate CLI docs (`03_show.md`, `05_search.md`, `06_export.md`) still
//! describe "prefix matching" as the contract.
//!
//! ## Why Not Caught
//!
//! Every existing partial-ID test (`test_show_partial_uuid_matching`,
//! `test_search_session_partial_uuid_match`) supplies a genuine prefix (the
//! UUID's own first 8 characters), which matches under both `starts_with` and
//! `contains` — so neither test can distinguish the two behaviors. No test
//! supplied a substring drawn from the middle of a session ID, which is the
//! only input that exposes the divergence.
//!
//! ## Fix Applied
//!
//! Reverted both call sites from `s.id().contains(id)` back to
//! `s.id().starts_with(id)`, restoring Git-style prefix matching for
//! `.show`, `.export`, `.search`, `.tail` (all via the shared
//! `find_session_mut`) and `.count target::entries` (via its own duplicate).
//!
//! ## Prevention
//!
//! When a session/entity lookup predicate is meant to support prefix-only
//! matching, add a negative test asserting that a substring which is NOT a
//! prefix does NOT match — a positive-prefix test alone cannot detect
//! `starts_with`/`contains` regressions.
//!
//! ## Pitfall
//!
//! A commit's stated scope (e.g. "docs: restructure...") is not proof of its
//! actual diff — a mechanical, unrelated one-line logic change can ride along
//! inside a large, legitimately-docs-focused commit and go unnoticed because
//! nothing in the message or review flags it as a behavior change.

mod common;

use tempfile::TempDir;

/// Test `.show` with a mid-string (non-prefix) substring must NOT match (BUG-490)
///
/// ## Root Cause
///
/// `find_session_mut` used `s.id().contains(id)`, so any substring anywhere in
/// the ID matched — not just a leading prefix.
///
/// ## Why Not Caught
///
/// All existing partial-UUID tests used the ID's own first 8 characters (a
/// genuine prefix), which passes under `contains` too.
///
/// ## Fix Applied
///
/// Reverted to `s.id().starts_with(id)` in `find_session_mut`.
///
/// ## Prevention
///
/// Any prefix-matching lookup needs a negative test using a non-leading
/// substring, not only a positive prefix test.
///
/// ## Pitfall
///
/// A `.contains()` predicate silently accepts wrong-but-plausible matches;
/// with short filters and many sessions this can resolve to the wrong session
/// with no indication to the user that a different one was found.
// test_kind: bug_reproducer(BUG-490)
#[ test ]
fn test_show_mid_string_substring_must_not_match()
{
  let storage = TempDir::new().unwrap();
  let session_uuid = "11112222-3333-4444-5555-666677778888";
  // Drawn from the middle of the UUID — not a prefix.
  let mid_substring = "5555-6666";

  common::write_test_session( storage.path(), "contains-regression-proj", session_uuid, 1 );

  let output = common::clg_cmd()
    .args( [ ".show", &format!( "session_id::{mid_substring}" ), "project::contains-regression-proj" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .show" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    !output.status.success(),
    "A non-prefix substring must not resolve to a session. stdout: {stdout}, stderr: {stderr}"
  );

  assert!(
    stdout.contains( "Session not found" ) || stderr.contains( "Session not found" ),
    "Expected 'Session not found' for a non-prefix substring. stdout: {stdout}, stderr: {stderr}"
  );
}

/// Regression: genuine prefix still matches after the fix (BUG-490)
///
/// ## Purpose
///
/// Confirms reverting to `starts_with` does not break the documented
/// Git-style 8-char UUID prefix feature.
///
/// ## Coverage
///
/// `.show` with the session's own leading 8 characters.
///
/// ## Validation Strategy
///
/// Same session as the mid-string test, filtered by its actual prefix.
///
/// ## Related Requirements
///
/// `03_show.md` Algorithm step 3: "prefix matching for partial UUIDs
/// (Git-style 8-char prefix)".
#[ test ]
fn test_show_leading_prefix_still_matches()
{
  let storage = TempDir::new().unwrap();
  let session_uuid = "11112222-3333-4444-5555-666677778888";
  let leading_prefix = "11112222";

  common::write_test_session( storage.path(), "contains-regression-prefix-proj", session_uuid, 1 );

  let output = common::clg_cmd()
    .args( [ ".show", &format!( "session_id::{leading_prefix}" ), "project::contains-regression-prefix-proj" ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .show" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "A genuine leading prefix must still resolve the session. stderr: {stderr}"
  );

  assert!(
    !stdout.contains( "Session not found" ),
    "Leading prefix must not report 'Session not found'. stdout: {stdout}"
  );
}

/// Test `.count target::entries` with a mid-string substring must NOT match (BUG-490)
///
/// ## Root Cause
///
/// `count_routine`'s own duplicated lookup (`src/cli/count.rs`) independently
/// used `s.id().contains(sess_id)`, the same regression as `find_session_mut`
/// but in a separately-maintained copy of the logic.
///
/// ## Why Not Caught
///
/// `count_command_bug_fix.rs`'s only `session::` test uses a full exact
/// session ID (caught by the `==` branch), never a substring.
///
/// ## Fix Applied
///
/// Reverted to `s.id().starts_with(sess_id)` in `count_routine`.
///
/// ## Prevention
///
/// Duplicated lookup logic must carry the same negative-substring test as the
/// shared implementation it was copied from.
///
/// ## Pitfall
///
/// Duplicating a lookup predicate instead of calling the shared helper means
/// a regression fixed in one copy silently persists in the other.
// test_kind: bug_reproducer(BUG-490)
#[ test ]
fn test_count_entries_mid_string_substring_must_not_match()
{
  let storage = TempDir::new().unwrap();
  let session_uuid = "11112222-3333-4444-5555-666677778888";
  let mid_substring = "5555-6666";

  common::write_test_session( storage.path(), "contains-regression-count-proj", session_uuid, 3 );

  let output = common::clg_cmd()
    .args( [
      ".count",
      "target::entries",
      "project::contains-regression-count-proj",
      &format!( "session::{mid_substring}" ),
    ] )
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .output()
    .expect( "failed to execute .count" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    !output.status.success(),
    "A non-prefix substring must not resolve to a session. stdout: {stdout}, stderr: {stderr}"
  );

  assert!(
    stdout.contains( "Session not found" ) || stderr.contains( "Session not found" ),
    "Expected 'Session not found' for a non-prefix substring. stdout: {stdout}, stderr: {stderr}"
  );
}
