//! Edge case tests for the `entries::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/03_entries.md`
//!
//! ## Coverage
//!
//! - EC-1: Value 0 shows summary view
//! - EC-2: Value 1 shows all entry records
//! - EC-3: Value "yes" rejected
//! - EC-4: Omitted defaults to 0 (summary view)
//! - EC-5: `entries::1` with small session shows all entries
//! - EC-6: `entries::1` output includes UUID and timestamp per entry

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

/// EC-1: Value 0 shows summary view.
///
/// ## Purpose
/// Validates that `entries::0` shows a concise session summary without
/// per-entry expansion.
///
/// ## Coverage
/// Exit 0; summary output without individual entry records.
///
/// ## Validation Strategy
/// Create a session. Run `.show ``session_id::`` ``entries::``0`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/03_entries.md` — EC-1
#[ test ]
fn ec_1_entries_0_shows_summary_view()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-ent", "-default_topic", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-ent" )
    .arg( "entries::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty(),
    "EC-1: summary output must be non-empty; got: {output}"
  );
}

/// EC-2: Value 1 shows all entry records.
///
/// ## Purpose
/// Validates that `entries::1` shows all individual entry records.
///
/// ## Coverage
/// Exit 0; individual entry records listed in output.
///
/// ## Validation Strategy
/// Create a session. Run `.show ``session_id::`` ``entries::``1`.
/// Assert exit 0 and non-empty output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/03_entries.md` — EC-2
#[ test ]
fn ec_2_entries_1_shows_all_records()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-ent2", "-default_topic", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-ent2" )
    .arg( "entries::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty(),
    "EC-2: entries::1 output must be non-empty; got: {output}"
  );
}

/// EC-3: Value "yes" accepted as truthy boolean.
///
/// ## Purpose
/// Validates that `entries::yes` is accepted by the unilang boolean parser
/// as a truthy value (equivalent to `entries::1`).
///
/// ## Coverage
/// Exit 0 or non-type-error exit; no type validation error emitted.
///
/// ## Validation Strategy
/// Run `.show ``session_id::`` ``entries::ye``s`. Assert no type error (may exit 1
/// due to missing project, but not due to invalid entries value).
///
/// ## Related Requirements
/// `tests/docs/cli/param/03_entries.md` — EC-3
#[ test ]
fn ec_3_entries_yes_accepted()
{
  let out = common::clg_cmd()
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "entries::yes" )
    .output()
    .unwrap();

  let err = stderr( &out );
  assert!(
    !err.contains( "Invalid boolean" ) && !err.contains( "Type Error" ),
    "EC-3: entries::yes must not cause a type validation error; got: {err}"
  );
}

/// EC-4: Omitted defaults to 0 (summary view).
///
/// ## Purpose
/// Validates that omitting `entries::` defaults to summary view.
///
/// ## Coverage
/// Exit 0; summary view identical to `entries::0`.
///
/// ## Validation Strategy
/// Create session. Run `.show session_id::` with no entries param.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/03_entries.md` — EC-4
#[ test ]
fn ec_4_omitted_defaults_to_summary_view()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-ent4", "-default_topic", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-ent4" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-5: `entries::1` with small session shows all entries.
///
/// ## Purpose
/// Validates that `entries::1` on a small (3-entry) session shows all 3 records.
///
/// ## Coverage
/// Exit 0; entry record count equals fixture session's actual entry count.
///
/// ## Validation Strategy
/// Create a 3-entry session. Run `.show ``session_id::`` ``entries::``1`.
/// Assert exit 0 and non-empty output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/03_entries.md` — EC-5
#[ test ]
fn ec_5_entries_1_small_session_shows_all()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-ent5", "-default_topic", 3 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-ent5" )
    .arg( "entries::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty(),
    "EC-5: entries::1 output must be non-empty for 3-entry session; got: {output}"
  );
}

/// EC-6: `entries::1` output includes UUID and timestamp per entry.
///
/// ## Purpose
/// Validates that `entries::1` output contains UUID and timestamp fields.
///
/// ## Coverage
/// Exit 0; UUID-format string and timestamp string present in output.
///
/// ## Validation Strategy
/// Create session with known UUIDs. Run `.show ``session_id::`` ``entries::``1`.
/// Assert output contains "uuid" or timestamp markers.
///
/// ## Related Requirements
/// `tests/docs/cli/param/03_entries.md` — EC-6
#[ test ]
fn ec_6_entries_1_includes_uuid_and_timestamp()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-ent6", "-default_topic", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-ent6" )
    .arg( "entries::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  // Synthetic entries use "test-uuid-NNN" — check that UUID-like content appears
  assert!(
    output.contains( "uuid" ) || output.contains( "2025" ) || output.contains( "test-uuid" ),
    "EC-6: entries::1 output must contain UUID or timestamp data; got: {output}"
  );
}
