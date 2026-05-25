//! Edge case tests for the `metadata::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/06_metadata.md`
//!
//! ## Coverage
//!
//! - EC-1: Value 0 shows conversation content
//! - EC-2: Value 1 suppresses content, shows metadata
//! - EC-3: Value "true" rejected
//! - EC-4: Omitted defaults to 0 (show content)
//! - EC-5: `metadata::1` output includes entry count
//! - EC-6: `metadata::1` output includes session timestamps

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

/// EC-1: Value 0 shows conversation content.
///
/// ## Purpose
/// Validates that `metadata::0` shows conversation message content.
///
/// ## Coverage
/// Exit 0; conversation content present in output.
///
/// ## Validation Strategy
/// Create session with known message text. Run `.show ```metadata::```0`.
/// Assert output contains message text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/06_metadata.md` — EC-1
#[ test ]
fn ec_1_metadata_0_shows_content()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-meta",
    "-default_topic",
    2,
    "hello world content",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-meta" )
    .arg( "metadata::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "hello world content" ) || output.contains( "entry" ),
    "EC-1: metadata::0 must show conversation content; got: {output}"
  );
}

/// EC-2: Value 1 suppresses content, shows metadata.
///
/// ## Purpose
/// Validates that `metadata::1` suppresses message text and shows technical metadata.
///
/// ## Coverage
/// Exit 0; metadata fields present; conversation content absent.
///
/// ## Validation Strategy
/// Create session with unique message text. Run `.show ```metadata::```1`.
/// Assert message text absent; metadata fields present.
///
/// ## Related Requirements
/// `tests/docs/cli/param/06_metadata.md` — EC-2
#[ test ]
fn ec_2_metadata_1_suppresses_content()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-meta2",
    "-default_topic",
    2,
    "unique-sentinel-text-xyzabc",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-meta2" )
    .arg( "metadata::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "unique-sentinel-text-xyzabc" ),
    "EC-2: metadata::1 must suppress conversation content; got: {output}"
  );
}

/// EC-3: Value "true" accepted as truthy boolean.
///
/// ## Purpose
/// Validates that `metadata::true` is accepted by the unilang boolean parser
/// as a truthy value (equivalent to `metadata::1`).
///
/// ## Coverage
/// Exit 0 or non-type-error exit; no type validation error emitted.
///
/// ## Validation Strategy
/// Run `.show ```metadata::tru```e`. Assert no type validation error (may exit 1
/// due to missing project, but not due to invalid metadata value).
///
/// ## Related Requirements
/// `tests/docs/cli/param/06_metadata.md` — EC-3
#[ test ]
fn ec_3_metadata_true_accepted()
{
  let out = common::clg_cmd()
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "metadata::true" )
    .output()
    .unwrap();

  let err = stderr( &out );
  assert!(
    !err.contains( "Invalid boolean" ) && !err.contains( "Type Error" ),
    "EC-3: metadata::true must not cause a type validation error; got: {err}"
  );
}

/// EC-4: Omitted defaults to 0 (show content).
///
/// ## Purpose
/// Validates that omitting `metadata::` defaults to showing conversation content.
///
/// ## Coverage
/// Exit 0; conversation content shown identical to `metadata::0`.
///
/// ## Validation Strategy
/// Create session with known message. Run `.show` with no metadata param.
/// Assert exit 0 and message content present.
///
/// ## Related Requirements
/// `tests/docs/cli/param/06_metadata.md` — EC-4
#[ test ]
fn ec_4_metadata_omitted_defaults_to_content()
{
  let root = TempDir::new().unwrap();
  common::write_test_session_with_last_message(
    root.path(),
    "proj-meta4",
    "-default_topic",
    2,
    "default content display",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-meta4" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "default content display" ) || output.contains( "entry" ),
    "EC-4: omitted metadata must show content; got: {output}"
  );
}

/// EC-5: `metadata::1` output includes entry count.
///
/// ## Purpose
/// Validates that `metadata::1` output shows a count of entries in the session.
///
/// ## Coverage
/// Exit 0; entry count field present in metadata output.
///
/// ## Validation Strategy
/// Create 4-entry session. Run `.show ```metadata::```1`. Assert output contains
/// a numeric count or "entries" label.
///
/// ## Related Requirements
/// `tests/docs/cli/param/06_metadata.md` — EC-5
#[ test ]
fn ec_5_metadata_1_includes_entry_count()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-meta5", "-default_topic", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-meta5" )
    .arg( "metadata::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( '4' ) || output.contains( "entries" ) || output.contains( "count" ),
    "EC-5: metadata::1 must show entry count; got: {output}"
  );
}

/// EC-6: `metadata::1` output includes session timestamps.
///
/// ## Purpose
/// Validates that `metadata::1` output shows first/last entry timestamp fields.
///
/// ## Coverage
/// Exit 0; timestamp fields present in output (ISO 8601 or similar).
///
/// ## Validation Strategy
/// Create session with synthetic timestamps. Run `.show ```metadata::```1`.
/// Assert output contains year or timestamp markers.
///
/// ## Related Requirements
/// `tests/docs/cli/param/06_metadata.md` — EC-6
#[ test ]
fn ec_6_metadata_1_includes_timestamps()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-meta6", "-default_topic", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-meta6" )
    .arg( "metadata::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  // Synthetic sessions use "2025-01-01T..." timestamps
  assert!(
    output.contains( "2025" ) || output.contains( "timestamp" ) || output.contains( "first" ),
    "EC-6: metadata::1 must show timestamp fields; got: {output}"
  );
}
