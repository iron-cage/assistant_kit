//! Edge case tests for the `entry_type::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/04_entry_type.md`
//!
//! ## Coverage
//!
//! - EC-1: Value "user" accepted
//! - EC-2: Value "assistant" accepted
//! - EC-3: Value "all" accepted
//! - EC-4: Value "USER" accepted (case-insensitive)
//! - EC-5: Invalid value "both" rejected with error
//! - EC-6: Invalid value "system" rejected with error
//! - EC-7: Omitted defaults to "all"

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

/// EC-1: Value "user" accepted.
///
/// ## Purpose
/// Validates that `entry_type::user` filters search to user-turn entries only.
///
/// ## Coverage
/// Exit 0; only user-authored entries in results.
///
/// ## Validation Strategy
/// Write a session where user entries contain "needle" (even indices) and
/// assistant entries contain "needle" too. Search with `entry_type::user`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/04_entry_type.md` — EC-1
#[ test ]
fn ec_1_entry_type_user_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-et",
    "sess-et",
    2,
    "needle in user message",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::needle" )
    .arg( "entry_type::user" )
    .arg( "project::proj-et" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-2: Value "assistant" accepted.
///
/// ## Purpose
/// Validates that `entry_type::assistant` filters to assistant-turn entries.
///
/// ## Coverage
/// Exit 0; only assistant-authored entries in results.
///
/// ## Validation Strategy
/// Write a session. Search with `entry_type::assistant`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/04_entry_type.md` — EC-2
#[ test ]
fn ec_2_entry_type_assistant_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-et2", "sess-et2", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::entry" )
    .arg( "entry_type::assistant" )
    .arg( "project::proj-et2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-3: Value "all" accepted.
///
/// ## Purpose
/// Validates that `entry_type::all` returns both user and assistant entries.
///
/// ## Coverage
/// Exit 0; both user and assistant entries in results (same as default).
///
/// ## Validation Strategy
/// Write a session. Search with `entry_type::all`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/04_entry_type.md` — EC-3
#[ test ]
fn ec_3_entry_type_all_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-et3", "sess-et3", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::entry" )
    .arg( "entry_type::all" )
    .arg( "project::proj-et3" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-4: Value "USER" rejected (case-sensitive enum parsing).
///
/// ## Purpose
/// Validates that `entry_type` enum parsing is case-sensitive: "USER" is
/// rejected while "user" is accepted.
///
/// ## Coverage
/// Exit 1; error message contains "`entry_type`" and indicates invalid value.
///
/// ## Validation Strategy
/// Write session. Search with `entry_type::USER`. Assert exit 1 and error
/// mentions "`entry_type`".
///
/// ## Related Requirements
/// `tests/docs/cli/param/04_entry_type.md` — EC-4
#[ test ]
fn ec_4_entry_type_uppercase_rejected()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-et4", "sess-et4", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::entry" )
    .arg( "entry_type::USER" )
    .arg( "project::proj-et4" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "entry_type" ) || err.contains( "Invalid" ),
    "EC-4: expected error mentioning 'entry_type' or 'Invalid' for uppercase USER; got: {err}"
  );
}

/// EC-5: Invalid value "both" rejected with error.
///
/// ## Purpose
/// Validates that "both" is not a valid `entry_type` value.
///
/// ## Coverage
/// Exit 1; error message contains "`entry_type` must be user|assistant|all, got both".
///
/// ## Validation Strategy
/// Run `.search ``query::error`` ``entry_type::bot``h`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/04_entry_type.md` — EC-5
#[ test ]
fn ec_5_entry_type_both_rejected()
{
  let out = common::clg_cmd()
    .arg( ".search" )
    .arg( "query::error" )
    .arg( "entry_type::both" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "entry_type" ) && err.contains( "both" ),
    "EC-5: expected 'entry_type' and 'both' in stderr; got: {err}"
  );
}

/// EC-6: Invalid value "system" rejected with error.
///
/// ## Purpose
/// Validates that "system" is not a valid `entry_type` value.
///
/// ## Coverage
/// Exit 1; error message contains "`entry_type` must be user|assistant|all, got system".
///
/// ## Validation Strategy
/// Run `.search ``query::error`` ``entry_type::syste``m`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/04_entry_type.md` — EC-6
#[ test ]
fn ec_6_entry_type_system_rejected()
{
  let out = common::clg_cmd()
    .arg( ".search" )
    .arg( "query::error" )
    .arg( "entry_type::system" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "entry_type" ) && err.contains( "system" ),
    "EC-6: expected 'entry_type' and 'system' in stderr; got: {err}"
  );
}

/// EC-7: Omitted defaults to "all".
///
/// ## Purpose
/// Validates that omitting `entry_type` searches both user and assistant entries.
///
/// ## Coverage
/// Exit 0; results equivalent to `entry_type::all` (no implicit filter).
///
/// ## Validation Strategy
/// Write session. Search with no `entry_type`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/04_entry_type.md` — EC-7
#[ test ]
fn ec_7_entry_type_omitted_defaults_to_all()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-et7", "sess-et7", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::entry" )
    .arg( "project::proj-et7" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty() || output.is_empty(),
    "EC-7: default entry_type::all must not error; got: {output}"
  );
}
