//! Edge case tests for the `query::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/11_query.md`
//!
//! ## Coverage
//!
//! - EC-1: Required — missing `query::` exits with 1
//! - EC-2: Empty value rejected
//! - EC-3: Single-word query accepted
//! - EC-4: Multi-word phrase query accepted (shell-quoted)
//! - EC-5: Alias `q::` accepted same as `query::`
//! - EC-6: Whitespace-only value rejected
//! - EC-7: Query with special chars (e.g., ::) accepted

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

/// EC-1: Required — missing `query::` exits with 1.
///
/// ## Purpose
/// Validates that `.search` without `query::` fails with an error.
///
/// ## Coverage
/// Exit 1; error about missing required `query::` for .search.
///
/// ## Validation Strategy
/// Run `.search` with no arguments. Assert exit 1 and error mentions query.
///
/// ## Related Requirements
/// `tests/docs/cli/param/11_query.md` — EC-1
#[ test ]
fn ec_1_query_required_missing_exits_1()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-q", "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "query" ),
    "EC-1: error must mention 'query'; got: {combined}"
  );
}

/// EC-2: Empty value rejected.
///
/// ## Purpose
/// Validates that `query::` with empty value is rejected.
///
/// ## Coverage
/// Exit 1; error message "query must be non-empty".
///
/// ## Validation Strategy
/// Run `.search query::`. Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/11_query.md` — EC-2
#[ test ]
fn ec_2_query_empty_rejected()
{
  let out = common::clg_cmd()
    .arg( ".search" )
    .arg( "query::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "query" ) || combined.contains( "empty" ),
    "EC-2: error must mention query or empty; got: {combined}"
  );
}

/// EC-3: Single-word query accepted.
///
/// ## Purpose
/// Validates that a single-word query returns results.
///
/// ## Coverage
/// Exit 0; search results returned for single-word query.
///
/// ## Validation Strategy
/// Write session with "error" in a message. Run `.search ```query::error``` ```project::proj```-q3`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/11_query.md` — EC-3
#[ test ]
fn ec_3_query_single_word_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-q3",
    "sess-q3",
    0,
    "error detected here",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::error" )
    .arg( "project::proj-q3" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "error" ),
    "EC-3: single-word query must find matching entries; got: {output}"
  );
}

/// EC-4: Multi-word phrase query accepted (shell-quoted).
///
/// ## Purpose
/// Validates that a query referencing content from a multi-word phrase is accepted.
///
/// ## Coverage
/// Exit 0; query matches content from multi-word message without format error.
///
/// ## Validation Strategy
/// Write session containing "session management topic". Run `.search ```query::management``` ```project::proj```-q4`.
/// Assert exit 0. Note: unilang parameter values cannot contain spaces; the query uses
/// a distinctive single word ("management") that only appears in the target session.
///
/// ## Related Requirements
/// `tests/docs/cli/param/11_query.md` — EC-4
#[ test ]
fn ec_4_query_multi_word_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-q4",
    "sess-q4",
    0,
    "session management topic",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::management" )
    .arg( "project::proj-q4" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-5: Alias `q::` accepted same as `query::`.
///
/// ## Purpose
/// Validates that `q::` is an alias for `query::` producing identical results.
///
/// ## Coverage
/// Exit 0; results identical to `query::` results.
///
/// ## Validation Strategy
/// Write session with "error". Run both `query::error` and `q::error` with `project::`.
/// Assert exit 0 for both and identical output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/11_query.md` — EC-5
#[ test ]
fn ec_5_query_alias_q_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-q5",
    "sess-q5",
    0,
    "error in alias test",
  );

  let out_query = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::error" )
    .arg( "project::proj-q5" )
    .output()
    .unwrap();

  let out_alias = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "q::error" )
    .arg( "project::proj-q5" )
    .output()
    .unwrap();

  assert_exit( &out_query, 0 );
  assert_exit( &out_alias, 0 );
  assert_eq!(
    stdout( &out_query ),
    stdout( &out_alias ),
    "EC-5: q:: alias must produce identical results to query::"
  );
}

/// EC-6: Whitespace-only value rejected.
///
/// ## Purpose
/// Validates that a whitespace-only query value is rejected before search begins.
///
/// ## Coverage
/// Exit 1; error about whitespace-only query value; isolated storage to prevent
/// unbounded search if validation does not fire.
///
/// ## Validation Strategy
/// Create an empty isolated storage root. Run `.search ```query::```   ` (spaces only)
/// with `CLAUDE_STORAGE_ROOT` pointing to the empty root. Assert exit 1 and
/// error mentions "query" or "empty".
///
/// ## Related Requirements
/// `tests/docs/cli/param/11_query.md` — EC-6
#[ test ]
fn ec_6_query_whitespace_only_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::   " )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "query" ) || combined.contains( "empty" ),
    "EC-6: error must mention query or empty; got: {combined}"
  );
}

/// EC-7: Query with special chars (e.g., ::) accepted.
///
/// ## Purpose
/// Validates that `::` within a query value is treated as literal content.
///
/// ## Coverage
/// Exit 0; `::` within query not treated as a second parameter delimiter.
///
/// ## Validation Strategy
/// Write session containing "`param::value`". Run `.search ```query::param```::value ```project::proj```-q7`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/11_query.md` — EC-7
#[ test ]
fn ec_7_query_special_chars_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-q7",
    "sess-q7",
    0,
    "param::value is the target",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::param::value" )
    .arg( "project::proj-q7" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "param::value" ),
    "EC-7: query with :: must search for literal param::value; got: {output}"
  );
}
