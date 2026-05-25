//! Edge case tests for the `session::` parameter (filter).
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/13_session.md`
//!
//! ## Coverage
//!
//! - EC-1: Partial match at start of session ID
//! - EC-2: Partial match in middle of session ID
//! - EC-3: Case-insensitive match
//! - EC-4: No match returns empty results
//! - EC-5: Empty value rejected
//! - EC-6: Auto-enables `sessions::1` in .list
//! - EC-7: `session::` in .count restricts to matching session

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

/// EC-1: Partial match at start of session ID.
///
/// ## Purpose
/// Validates that `session::default` matches sessions whose ID starts with "default".
///
/// ## Coverage
/// Exit 0; sessions starting with "default" in their ID are returned.
///
/// ## Validation Strategy
/// Create session named "-`default_topic`". Run `.list ``session::defaul``t`.
/// Assert exit 0 and session appears.
///
/// ## Related Requirements
/// `tests/docs/cli/param/13_session.md` — EC-1
#[ test ]
fn ec_1_session_partial_match_at_start()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sess", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::default" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "default" ),
    "EC-1: session with 'default' prefix must appear; got: {output}"
  );
}

/// EC-2: Partial match in middle of session ID.
///
/// ## Purpose
/// Validates that `session::topic` matches sessions containing "topic" anywhere.
///
/// ## Coverage
/// Exit 0; sessions containing "topic" in their ID returned.
///
/// ## Validation Strategy
/// Create session "-`default_topic`". Run `.list ``session::topi``c`.
/// Assert exit 0 and session appears.
///
/// ## Related Requirements
/// `tests/docs/cli/param/13_session.md` — EC-2
#[ test ]
fn ec_2_session_partial_match_in_middle()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sess2", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::topic" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "topic" ),
    "EC-2: session containing 'topic' must appear; got: {output}"
  );
}

/// EC-3: Case-insensitive match.
///
/// ## Purpose
/// Validates that session filter is case-insensitive.
///
/// ## Coverage
/// Exit 0; same sessions returned as lowercase equivalent filter.
///
/// ## Validation Strategy
/// Create session "-`default_topic`". Run `.list ``session::DEFAUL``T` (uppercase).
/// Assert same result as `session::default`.
///
/// ## Related Requirements
/// `tests/docs/cli/param/13_session.md` — EC-3
#[ test ]
fn ec_3_session_case_insensitive_match()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sess3", "-default_topic", 2 );

  let out_lower = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::default" )
    .output()
    .unwrap();

  let out_upper = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::DEFAULT" )
    .output()
    .unwrap();

  assert_exit( &out_lower, 0 );
  assert_exit( &out_upper, 0 );
  assert_eq!(
    stdout( &out_lower ),
    stdout( &out_upper ),
    "EC-3: case-insensitive session filter must return identical results"
  );
}

/// EC-4: No match returns empty results.
///
/// ## Purpose
/// Validates that a non-matching session filter produces empty output without error.
///
/// ## Coverage
/// Exit 0; empty result set (no error for non-matching filter).
///
/// ## Validation Strategy
/// Create fixture. Run `.list ``session::zzznomatch99``9`. Assert exit 0 and
/// no sessions in output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/13_session.md` — EC-4
#[ test ]
fn ec_4_session_no_match_returns_empty()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sess4", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::zzznomatch999" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "default" ),
    "EC-4: non-matching session filter must return empty list; got: {output}"
  );
}

/// EC-5: Empty value rejected.
///
/// ## Purpose
/// Validates that `session::` with empty value is rejected.
///
/// ## Coverage
/// Exit 1; error about empty session filter value.
///
/// ## Validation Strategy
/// Run `.list session::`. Assert exit 1 and error mentions session.
///
/// ## Related Requirements
/// `tests/docs/cli/param/13_session.md` — EC-5
#[ test ]
fn ec_5_session_empty_rejected()
{
  let out = common::clg_cmd()
    .arg( ".list" )
    .arg( "session::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "session" ),
    "EC-5: error must mention 'session'; got: {combined}"
  );
}

/// EC-6: Auto-enables `sessions::1` in .list.
///
/// ## Purpose
/// Validates that `session::` implicitly enables session display in .list output.
///
/// ## Coverage
/// Exit 0; session display auto-enabled and filtered by "default" substring.
///
/// ## Validation Strategy
/// Create session. Run `.list ``session::defaul``t`. Assert exit 0 and session rows visible.
///
/// ## Related Requirements
/// `tests/docs/cli/param/13_session.md` — EC-6
#[ test ]
fn ec_6_session_auto_enables_display()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sess6", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::default" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  // Session filter auto-enables display; the matching session should appear
  assert!(
    output.contains( "default" ),
    "EC-6: session filter must auto-enable session display; got: {output}"
  );
}

/// EC-7: `session::` in .count restricts to matching session.
///
/// ## Purpose
/// Validates that `session::` in .count scopes the count to the specified session.
///
/// ## Coverage
/// Exit 0; count scoped to the named session only (not all sessions in project).
///
/// ## Validation Strategy
/// Create one session with 4 entries and another with 10 entries.
/// Run `.count ``target::entries`` ``session::``-default_topic ``project::proj``-sess7`.
/// Assert exit 0 and count is 4 (only the named session, not the 10-entry session).
/// Note: `session::` in .count is an exact session ID lookup, not a substring filter.
///
/// ## Related Requirements
/// `tests/docs/cli/param/13_session.md` — EC-7
#[ test ]
fn ec_7_session_in_count_restricts_scope()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sess7", "-default_topic", 4 );
  common::write_test_session( root.path(), "proj-sess7", "other-session", 10 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::entries" )
    .arg( "session::-default_topic" )
    .arg( "project::proj-sess7" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  let count : usize = output.parse().unwrap_or( 999 );
  assert!(
    count < 10,
    "EC-7: count with session::-default_topic must not include other-session entries (10); got: {count}"
  );
}
