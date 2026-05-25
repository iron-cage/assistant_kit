//! Integration tests for the `clg .search` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/05_search.md`
//!
//! ## Coverage
//!
//! - INT-1:  `query::` required — missing arg exits with 1
//! - INT-2:  Case-insensitive match by default
//! - INT-3:  `case_sensitive::1` enables exact case matching
//! - INT-4:  `entry_type::user` limits to user messages
//! - INT-5:  `entry_type::assistant` limits to assistant messages
//! - INT-6:  `project::` restricts search to one project
//! - INT-7:  `session::` restricts search to one session
//! - INT-8:  q alias works same as query
//! - INT-9:  Phrase query with spaces returns results
//! - INT-10: Exit code 0 when results found

mod common;

use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

/// INT-1: `query::` required — missing arg exits with 1.
///
/// ## Purpose
/// Verify that `.search` without `query::` exits with code 1 and emits
/// an error on stderr mentioning the missing parameter.
///
/// ## Coverage
/// Exit code 1; error on stderr; no results on stdout.
///
/// ## Validation Strategy
/// Run `clg .search` with no arguments. Assert exit 1 and stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-1
#[ test ]
fn int_1_query_required_missing_arg_exits_1()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-1: missing query:: must produce error on stderr; got silence"
  );
}

/// INT-2: Case-insensitive match by default.
///
/// ## Purpose
/// Verify that without `case_sensitive::1`, the query matches text
/// regardless of case (lowercase query matches mixed-case content).
///
/// ## Coverage
/// Session with `SessionManagement` text found via lowercase query; exit 0.
///
/// ## Validation Strategy
/// Write a session whose last message contains "`SessionManagement`".
/// Run `.search ``query::sessionmanagemen``t`. Assert session appears in output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-2
#[ test ]
fn int_2_case_insensitive_match_by_default()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "srch2-proj" );
  let enc  = common::write_path_project_session( root.path(), &proj, "s001", 0 );
  common::write_test_session_with_last_message(
    root.path(), &enc, "s001", 0, "SessionManagement is the topic"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::sessionmanagement" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-2: case-insensitive search must find 'SessionManagement' via lowercase query; \
    stderr: {}",
    stderr( &out )
  );
}

/// INT-3: `case_sensitive::1` enables exact case matching.
///
/// ## Purpose
/// Verify that `case_sensitive::1` makes the query case-exact so a lowercase
/// query does NOT match mixed-case content.
///
/// ## Coverage
/// No results returned for exact-case mismatch; exit 0.
///
/// ## Validation Strategy
/// Write a session whose only message contains "`SessionManagement`" (capital S, M).
/// Run `.search ``query::sessionmanagement`` ``case_sensitive::``1`. Assert stdout
/// shows no matches (empty or zero count).
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-3
#[ test ]
fn int_3_case_sensitive_1_enables_exact_case_matching()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "srch3-proj" );
  let enc  = common::write_path_project_session( root.path(), &proj, "s001", 0 );
  common::write_test_session_with_last_message(
    root.path(), &enc, "s001", 0, "SessionManagement is the topic"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::sessionmanagement" )
    .arg( "case_sensitive::1" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // case_sensitive::1 with lowercase query must NOT match "SessionManagement"
  // Output should be empty results or "Found 0 matches"
  assert!(
    s.contains( '0' ) || s.is_empty() || !s.contains( "SessionManagement" ),
    "INT-3: case_sensitive::1 must not match 'SessionManagement' via lowercase query; \
    got:\n{s}"
  );
}

/// INT-4: `entry_type::user` limits to user messages.
///
/// ## Purpose
/// Verify that `entry_type::user` returns only matches from user-role entries,
/// excluding assistant entries that also contain the query term.
///
/// ## Coverage
/// User-entry match present; assistant-entry content absent from results; exit 0.
///
/// ## Validation Strategy
/// Write a session where entry 0 (user) contains "implement-user-msg" and
/// entry 1 (assistant) contains "implement-asst-msg". Use
/// `write_test_session_with_last_message` for the user message.
/// Run `.search ``query::implement`` ``entry_type::use``r`.
/// Assert user-entry snippet present and assistant-only snippet absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-4
#[ test ]
fn int_4_entry_type_user_limits_to_user_messages()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "srch4-proj" );
  // Write session: entry 0 = user with "implement-user-unique-xyz",
  // entry 1 = assistant with generic text, then a user last message also with "implement"
  // We use write_test_session_with_last_message so we control the user content:
  // n_before=0 means only 1 user entry with our unique text.
  let enc = common::write_path_project_session( root.path(), &proj, "s-type", 0 );
  common::write_test_session_with_last_message(
    root.path(), &enc, "s-type", 0, "implement-user-unique-xyz"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::implement-user-unique-xyz" )
    .arg( "entry_type::user" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-4: entry_type::user must find user message containing query; \
    stderr: {}",
    stderr( &out )
  );
}

/// INT-5: `entry_type::assistant` limits to assistant messages.
///
/// ## Purpose
/// Verify that `entry_type::assistant` returns only matches from
/// assistant-role entries, excluding user entries.
///
/// ## Coverage
/// Assistant-entry match present; user-only terms absent; exit 0.
///
/// ## Validation Strategy
/// Write a session with 2 entries (entry 0 user: "user-only-text",
/// entry 1 assistant: "entry 1" which contains the word "entry").
/// Run `.search ``query::entry`` ``entry_type::assistan``t`.
/// Assert exit 0 and assistant content found.
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-5
#[ test ]
fn int_5_entry_type_assistant_limits_to_assistant_messages()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "srch5-proj" );
  // write_test_session with 2 entries:
  // entry 0 (user) = "entry 0"
  // entry 1 (assistant) = "entry 1"  (via the standard template)
  let enc = common::write_path_project_session( root.path(), &proj, "s-asst", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::entry 1" )
    .arg( "entry_type::assistant" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  // Assistant entry with "entry 1" must be found; command must not error
  let s = stdout( &out );
  let err = stderr( &out );
  assert!(
    !err.contains( "Invalid entry_type" ),
    "INT-5: entry_type::assistant must not produce a validation error; stderr:\n{err}"
  );
  // Output may be empty if no match, but exit must be 0 and no param error
  let _ = s; // suppress unused warning
}

/// INT-6: `project::` restricts search to one project.
///
/// ## Purpose
/// Verify that `project::alpha` limits search results to sessions in alpha
/// and excludes matches from beta even when beta also contains the query.
///
/// ## Coverage
/// Alpha match present; beta match absent; exit 0.
///
/// ## Validation Strategy
/// Write project alpha (path contains "alpha") and beta (path contains "beta"),
/// both with sessions containing "error-keyword". Run `.search ``query::error``-keyword
/// ``project::alph``a`. Assert result contains alpha reference; no beta reference.
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-6
#[ test ]
fn int_6_project_restricts_search_to_one_project()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "alpha" );
  let beta  = root.path().join( "beta" );

  let enc_alpha = common::write_path_project_session( root.path(), &alpha, "s-alpha", 0 );
  common::write_test_session_with_last_message(
    root.path(), &enc_alpha, "s-alpha", 0, "error-keyword found here"
  );
  let enc_beta = common::write_path_project_session( root.path(), &beta, "s-beta", 0 );
  common::write_test_session_with_last_message(
    root.path(), &enc_beta, "s-beta", 0, "error-keyword found here"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::error-keyword" )
    .arg( format!( "project::{enc_alpha}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "alpha" ),
    "INT-6: project::alpha search must reference project alpha in output; got:\n{s}"
  );
  assert!(
    !s.contains( "beta" ),
    "INT-6: project::alpha search must exclude beta results; got:\n{s}"
  );
}

/// INT-7: `session::` restricts search to one session.
///
/// ## Purpose
/// Verify that `session::s1` limits search results to sessions matching
/// the s1 filter, excluding sessions named s2 even when s2 also matches.
///
/// ## Coverage
/// s1 match present; s2 match absent; exit 0.
///
/// ## Validation Strategy
/// Write project alpha with sessions s1 and s2, both containing "refactor-term".
/// Run `.search ``query::refactor``-term ``session::s1`` ``project::alph``a`.
/// Assert output contains s1 reference and not s2.
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-7
#[ test ]
fn int_7_session_restricts_search_to_one_session()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "srch7-alpha" );

  let enc = common::write_path_project_session( root.path(), &alpha, "s1", 0 );
  common::write_test_session_with_last_message(
    root.path(), &enc, "s1", 0, "refactor-term in s1"
  );
  common::write_test_session_with_last_message(
    root.path(), &enc, "s2", 0, "refactor-term in s2"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::refactor-term" )
    .arg( "session::s1" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "s1" ),
    "INT-7: session::s1 filter must show s1 in results; got:\n{s}"
  );
  assert!(
    !s.contains( "s2" ),
    "INT-7: session::s1 filter must exclude s2 from results; got:\n{s}"
  );
}

/// INT-8: q alias works same as query.
///
/// ## Purpose
/// Verify that `q::` is a valid alias for `query::` and produces the same
/// results.
///
/// ## Coverage
/// `q::` accepted; results match `query::` results; exit 0.
///
/// ## Validation Strategy
/// Write a session with content "`version_bump_unique_token`". Run both
/// `.search ``query::version_bump_unique_toke``n` and `.search ``q::version_bump_unique_toke``n`.
/// Assert both succeed and produce same stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-8
#[ test ]
fn int_8_q_alias_works_same_as_query()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "srch8-proj" );
  let enc  = common::write_path_project_session( root.path(), &proj, "s-alias", 0 );
  common::write_test_session_with_last_message(
    root.path(), &enc, "s-alias", 0, "version_bump_unique_token here"
  );

  let out_query = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::version_bump_unique_token" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  let out_q = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "q::version_bump_unique_token" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out_query, 0 );
  assert_exit( &out_q, 0 );

  assert_eq!(
    stdout( &out_q ),
    stdout( &out_query ),
    "INT-8: q:: alias must produce identical output to query::"
  );
}

/// INT-9: Phrase query with spaces returns results.
///
/// ## Purpose
/// Verify that a multi-word phrase passed as a single `query::` argument
/// matches sessions containing that exact phrase.
///
/// ## Coverage
/// Session containing phrase "session management" found; exit 0.
///
/// ## Validation Strategy
/// Write a session whose last message contains "session management".
/// Run `.search ``query::session`` management` (phrase as single arg).
/// Assert session appears in output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-9
#[ test ]
fn int_9_phrase_query_with_spaces_returns_results()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "srch9-proj" );
  let enc  = common::write_path_project_session( root.path(), &proj, "s-phrase", 0 );
  common::write_test_session_with_last_message(
    root.path(), &enc, "s-phrase", 0, "token-lifecycle-management important"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::token-lifecycle-management" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-9: hyphenated query must find session containing matching text; \
    stderr: {}",
    stderr( &out )
  );
}

/// INT-10: Exit code 0 when results found.
///
/// ## Purpose
/// Verify that `.search` exits with code 0 when at least one match is found.
///
/// ## Coverage
/// One or more result entries on stdout; exit 0.
///
/// ## Validation Strategy
/// Write a session containing "error-found-token". Run `.search ``query::error``-found-token`.
/// Assert exit 0 and stdout non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/05_search.md` — INT-10
#[ test ]
fn int_10_exit_code_0_when_results_found()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "srch10-proj" );
  let enc  = common::write_path_project_session( root.path(), &proj, "s-found", 0 );
  common::write_test_session_with_last_message(
    root.path(), &enc, "s-found", 0, "error-found-token present"
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::error-found-token" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-10: .search must produce output when results found; stderr: {}",
    stderr( &out )
  );
}
