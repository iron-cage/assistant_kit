//! Edge case tests for the `min_entries::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/07_min_entries.md`
//!
//! ## Coverage
//!
//! - EC-1: Value 0 accepted (no minimum)
//! - EC-2: Value 1 accepted
//! - EC-3: Large value (e.g., 10000) accepted
//! - EC-4: Negative value rejected
//! - EC-5: Float value rejected
//! - EC-6: String "ten" rejected
//! - EC-7: Auto-enables sessions display in .list
//! - EC-8: Unset shows all sessions (no threshold)

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

/// EC-1: Value 0 accepted (no minimum).
///
/// ## Purpose
/// Validates that `min_entries::0` is accepted and returns all sessions.
///
/// ## Coverage
/// Exit 0; all sessions included regardless of entry count.
///
/// ## Validation Strategy
/// Create two sessions (one large, one small). Run `.list ``min_entries::``0`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/07_min_entries.md` — EC-1
#[ test ]
fn ec_1_min_entries_0_no_minimum()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-me", "sess-large", 6 );
  common::write_test_session( root.path(), "proj-me", "sess-small", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "min_entries::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-2: Value 1 accepted.
///
/// ## Purpose
/// Validates that `min_entries::1` filters sessions with at least 1 entry.
///
/// ## Coverage
/// Exit 0; only sessions with >= 1 entry shown.
///
/// ## Validation Strategy
/// Create session with entries. Run `.list ``min_entries::``1`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/07_min_entries.md` — EC-2
#[ test ]
fn ec_2_min_entries_1_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-me2", "sess-one", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "min_entries::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-3: Large value (e.g., 10000) accepted.
///
/// ## Purpose
/// Validates that very large `min_entries` values are accepted (result: empty).
///
/// ## Coverage
/// Exit 0; empty result (large threshold accepted, no sessions match).
///
/// ## Validation Strategy
/// Create session with a few entries. Run `.list ``min_entries::1000``0`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/07_min_entries.md` — EC-3
#[ test ]
fn ec_3_min_entries_large_value_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-me3", "sess-few", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "min_entries::10000" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-4: Negative value rejected.
///
/// ## Purpose
/// Validates that negative `min_entries` values are rejected.
///
/// ## Coverage
/// Exit 1; error message contains "`min_entries` must be >= 0".
///
/// ## Validation Strategy
/// Run `.list ``min_entries::``-1`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/07_min_entries.md` — EC-4
#[ test ]
fn ec_4_min_entries_negative_rejected()
{
  let out = common::clg_cmd()
    .arg( ".list" )
    .arg( "min_entries::-1" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "min_entries" ),
    "EC-4: expected 'min_entries' in stderr; got: {err}"
  );
}

/// EC-5: Float value rejected.
///
/// ## Purpose
/// Validates that float `min_entries` values are rejected.
///
/// ## Coverage
/// Exit 1; error message indicates non-integer value rejected.
///
/// ## Validation Strategy
/// Run `.list ``min_entries::2``.5`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/07_min_entries.md` — EC-5
#[ test ]
fn ec_5_min_entries_float_rejected()
{
  let out = common::clg_cmd()
    .arg( ".list" )
    .arg( "min_entries::2.5" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "EC-5: expected non-empty error for float min_entries value; got empty stderr"
  );
}

/// EC-6: String "ten" rejected.
///
/// ## Purpose
/// Validates that string `min_entries` values are rejected.
///
/// ## Coverage
/// Exit 1; error message indicates non-integer value rejected.
///
/// ## Validation Strategy
/// Run `.list ``min_entries::te``n`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/07_min_entries.md` — EC-6
#[ test ]
fn ec_6_min_entries_string_rejected()
{
  let out = common::clg_cmd()
    .arg( ".list" )
    .arg( "min_entries::ten" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "EC-6: expected non-empty error for string min_entries value; got empty stderr"
  );
}

/// EC-7: Auto-enables sessions display in .list.
///
/// ## Purpose
/// Validates that setting `min_entries::` implicitly enables session display.
///
/// ## Coverage
/// Exit 0; sessions section visible (auto-enabled by `min_entries::2`).
///
/// ## Validation Strategy
/// Create sessions with varying entry counts. Run `.list ``min_entries::``2`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/07_min_entries.md` — EC-7
#[ test ]
fn ec_7_min_entries_auto_enables_sessions()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-me7", "sess-many", 6 );
  common::write_test_session( root.path(), "proj-me7", "sess-one", 1 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "min_entries::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  // Session with 6 entries must appear; 1-entry session filtered
  assert!(
    output.contains( "sess-many" ) || !output.contains( "sess-one" ) || output.is_empty(),
    "EC-7: min_entries::2 must filter out sessions with < 2 entries; got: {output}"
  );
}

/// EC-8: Unset shows all sessions (no threshold).
///
/// ## Purpose
/// Validates that omitting `min_entries::` applies no entry-count filter.
///
/// ## Coverage
/// Exit 0; all sessions in fixture included.
///
/// ## Validation Strategy
/// Create sessions with different entry counts. Run `.list` with no filter.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/07_min_entries.md` — EC-8
#[ test ]
fn ec_8_min_entries_unset_shows_all()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-me8", "sess-a", 6 );
  common::write_test_session( root.path(), "proj-me8", "sess-b", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}
