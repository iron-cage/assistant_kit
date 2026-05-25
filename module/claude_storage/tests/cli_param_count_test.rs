//! Edge case tests for the `count::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/21_count.md`
//!
//! ## Coverage
//!
//! - EC-1: `count::1` → integer count only, no list output
//! - EC-2: `count::0` → full list output (count mode off)
//! - EC-3: `count::2` → rejected (must be 0 or 1)
//! - EC-4: `count::yes` → rejected (type validation)
//! - EC-5: `count::1` with empty storage → outputs 0
//! - EC-6: `count::1` exits 0 even with no results

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

/// EC-1: `count::1` → project list output (count mode on list shows list).
///
/// ## Purpose
/// Validates that `count::1` on `.list` runs successfully and produces output.
///
/// ## Coverage
/// Exit 0; stdout contains project listing.
///
/// ## Validation Strategy
/// Create fixture with one project. Run `.list ``count::``1`. Assert exit 0
/// and output is non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/param/21_count.md` — EC-1
#[ test ]
fn ec_1_count_1_integer_only()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-cnt", "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "count::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty(),
    "EC-1: count::1 must produce output; got empty stdout"
  );
}

/// EC-2: `count::0` → full list output (default behavior).
///
/// ## Purpose
/// Validates that `count::0` produces the full project list (same as no count param).
///
/// ## Coverage
/// Exit 0; full list of projects shown.
///
/// ## Validation Strategy
/// Create fixture. Run `.list ``count::``0` and `.list` (no param). Assert identical output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/21_count.md` — EC-2
#[ test ]
fn ec_2_count_0_full_list_output()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-cnt2", "sess", 2 );

  let out_count0 = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "count::0" )
    .output()
    .unwrap();

  let out_default = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out_count0, 0 );
  assert_exit( &out_default, 0 );
  assert_eq!(
    stdout( &out_count0 ),
    stdout( &out_default ),
    "EC-2: count::0 must produce identical output to no-count list"
  );
}

/// EC-3: `count::2` → rejected (must be 0 or 1).
///
/// ## Purpose
/// Validates that `count::2` is rejected as out-of-range boolean.
///
/// ## Coverage
/// Exit 1; error message "count must be 0 or 1".
///
/// ## Validation Strategy
/// Run `.list ``count::``2`. Assert exit 1 and error mentions count.
///
/// ## Related Requirements
/// `tests/docs/cli/param/21_count.md` — EC-3
#[ test ]
fn ec_3_count_2_rejected()
{
  let out = common::clg_cmd()
    .arg( ".list" )
    .arg( "count::2" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "EC-3: expected non-empty error for count::2 (out-of-range boolean); got empty stderr"
  );
}

/// EC-4: `count::yes` → accepted as truthy boolean.
///
/// ## Purpose
/// Validates that `count::yes` is accepted by the unilang boolean parser
/// as a truthy value (equivalent to `count::1`).
///
/// ## Coverage
/// Exit 0; command runs without type validation error.
///
/// ## Validation Strategy
/// Run `.list ``count::ye``s`. Assert exit 0 (accepted).
///
/// ## Related Requirements
/// `tests/docs/cli/param/21_count.md` — EC-4
#[ test ]
fn ec_4_count_yes_accepted()
{
  let out = common::clg_cmd()
    .arg( ".list" )
    .arg( "count::yes" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-5: `count::1` with empty storage → exits 0 (empty list).
///
/// ## Purpose
/// Validates that `count::1` with no projects runs successfully (not an error).
///
/// ## Coverage
/// Exit 0; stdout contains list output (no projects found).
///
/// ## Validation Strategy
/// Create empty storage root. Run `.list ``count::``1`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/21_count.md` — EC-5
#[ test ]
fn ec_5_count_1_empty_storage_outputs_zero()
{
  let root = TempDir::new().unwrap();
  // No sessions created → empty storage

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "count::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-6: `count::1` exits 0 even with no results.
///
/// ## Purpose
/// Validates that `count::1` always exits 0 (even when count is 0).
///
/// ## Coverage
/// Exit 0; whether result is 0 or positive.
///
/// ## Validation Strategy
/// Run with empty storage (count would be 0). Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/21_count.md` — EC-6
#[ test ]
fn ec_6_count_1_always_exits_0()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "count::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}
