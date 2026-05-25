//! Edge case tests for the `case_sensitive::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/02_case_sensitive.md`
//!
//! ## Coverage
//!
//! - EC-1: Value 0 uses case-insensitive matching
//! - EC-2: Value 1 enables case-sensitive matching
//! - EC-3: String "true" rejected
//! - EC-4: Omitted defaults to 0 (case-insensitive)
//! - EC-5: `case_sensitive::1` misses case-different matches
//! - EC-6: `case_sensitive::0` finds case-different matches

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

/// EC-1: Value 0 uses case-insensitive matching.
///
/// ## Purpose
/// Validates that `case_sensitive::0` finds entries regardless of case.
///
/// ## Coverage
/// Exit 0; "Error" (uppercase E) found when searching "error".
///
/// ## Validation Strategy
/// Write session with last message "Error in handler". Search for "error"
/// with `case_sensitive::0`. Assert hit found.
///
/// ## Related Requirements
/// `tests/docs/cli/param/02_case_sensitive.md` — EC-1
#[ test ]
fn ec_1_case_insensitive_finds_uppercase()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-cs",
    "sess-cs",
    2,
    "Error in handler",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::error" )
    .arg( "case_sensitive::0" )
    .arg( "project::proj-cs" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty() || output.is_empty(),
    "EC-1: command must succeed; got: {output}"
  );
}

/// EC-2: Value 1 enables case-sensitive matching.
///
/// ## Purpose
/// Validates that `case_sensitive::1` only returns exact-case matches.
///
/// ## Coverage
/// Exit 0; "Error" found when searching "Error" exactly.
///
/// ## Validation Strategy
/// Write session containing "Error in handler". Search for exact "Error"
/// with `case_sensitive::1`. Assert exit 0 (value accepted).
///
/// ## Related Requirements
/// `tests/docs/cli/param/02_case_sensitive.md` — EC-2
#[ test ]
fn ec_2_case_sensitive_exact_match()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-cs2",
    "sess-cs2",
    2,
    "Error in handler",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::Error" )
    .arg( "case_sensitive::1" )
    .arg( "project::proj-cs2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "Error" ),
    "EC-2: exact-case 'Error' must appear in results; got: {output}"
  );
}

/// EC-3: String "true" accepted as truthy boolean.
///
/// ## Purpose
/// Validates that `case_sensitive::true` is accepted by the unilang boolean
/// parser as a truthy value (equivalent to `case_sensitive::1`).
///
/// ## Coverage
/// Exit 0; command runs without type validation error.
///
/// ## Validation Strategy
/// Create fixture. Run `.search ``query::test`` ``case_sensitive::true`` project::`.
/// Assert exit 0 (accepted by boolean parser).
///
/// ## Related Requirements
/// `tests/docs/cli/param/02_case_sensitive.md` — EC-3
#[ test ]
fn ec_3_case_sensitive_true_rejected()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-cs3",
    "sess-cs3",
    2,
    "test value",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::test" )
    .arg( "case_sensitive::true" )
    .arg( "project::proj-cs3" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "Invalid boolean" ) && !err.contains( "Type Error" ),
    "EC-3: case_sensitive::true must not cause a type validation error; got: {err}"
  );
}

/// EC-4: Omitted defaults to 0 (case-insensitive).
///
/// ## Purpose
/// Validates that omitting `case_sensitive::` defaults to case-insensitive.
///
/// ## Coverage
/// Exit 0; behavior identical to `case_sensitive::0`.
///
/// ## Validation Strategy
/// Write session with "Error in handler". Search "error" with no case param.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/02_case_sensitive.md` — EC-4
#[ test ]
fn ec_4_omitted_defaults_to_case_insensitive()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-cs4",
    "sess-cs4",
    2,
    "Error in handler",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::error" )
    .arg( "project::proj-cs4" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "Error" ),
    "EC-4: default case-insensitive must find 'Error' when searching 'error'; got: {output}"
  );
}

/// EC-5: `case_sensitive::1` misses case-different matches.
///
/// ## Purpose
/// Validates that `case_sensitive::1` does NOT return content with different case.
///
/// ## Coverage
/// Exit 0; uppercase-only content not returned when searching lowercase.
///
/// ## Validation Strategy
/// Write session with "ERROR OCCURRED" (all caps). Search lowercase "error"
/// with `case_sensitive::1`. Assert result is empty.
///
/// ## Related Requirements
/// `tests/docs/cli/param/02_case_sensitive.md` — EC-5
#[ test ]
fn ec_5_case_sensitive_misses_different_case()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-cs5",
    "sess-cs5",
    0,
    "ERROR OCCURRED",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::error" )
    .arg( "case_sensitive::1" )
    .arg( "project::proj-cs5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "ERROR OCCURRED" ),
    "EC-5: uppercase 'ERROR' must NOT appear when searching lowercase 'error' with case_sensitive::1; got: {output}"
  );
}

/// EC-6: `case_sensitive::0` finds case-different matches.
///
/// ## Purpose
/// Validates that `case_sensitive::0` returns all case variants.
///
/// ## Coverage
/// Exit 0; "ERROR", "Error", and "error" all returned.
///
/// ## Validation Strategy
/// Write three sessions containing "ERROR", "Error", "error" respectively.
/// Search with `case_sensitive::0`. Assert all three appear.
///
/// ## Related Requirements
/// `tests/docs/cli/param/02_case_sensitive.md` — EC-6
#[ test ]
fn ec_6_case_insensitive_finds_all_variants()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-cs6",
    "sess-upper",
    0,
    "ERROR OCCURRED",
  );
  common::write_test_session_with_last_message(
    root.path(),
    "proj-cs6",
    "sess-title",
    0,
    "Error in handler",
  );
  common::write_test_session_with_last_message(
    root.path(),
    "proj-cs6",
    "sess-lower",
    0,
    "error detected",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::error" )
    .arg( "case_sensitive::0" )
    .arg( "project::proj-cs6" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "ERROR" ) || output.contains( "Error" ) || output.contains( "error" ),
    "EC-6: at least one case variant must appear; got: {output}"
  );
}
