//! Edge case tests for the `output::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/08_output.md`
//!
//! ## Coverage
//!
//! - EC-1: Required — missing `output::` exits with 1
//! - EC-2: Absolute path accepted
//! - EC-3: ~ prefix path accepted
//! - EC-4: Relative path accepted
//! - EC-5: Empty value rejected
//! - EC-6: Nonexistent parent directory exits with 2
//! - EC-7: Existing file is overwritten without error
//! - EC-8: Whitespace-only path rejected

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

/// EC-1: Required — missing `output::` exits with 1.
///
/// ## Purpose
/// Validates that `.export` without `output::` fails with an error.
///
/// ## Coverage
/// Exit 1; error indicating output path is required.
///
/// ## Validation Strategy
/// Create session. Run `.export session_id::` with no output param.
/// Assert exit 1 and error mentions output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/08_output.md` — EC-1
#[ test ]
fn ec_1_output_required_missing_exits_1()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-out", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-out" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "output" ),
    "EC-1: error must mention 'output'; got: {combined}"
  );
}

/// EC-2: Absolute path accepted.
///
/// ## Purpose
/// Validates that an absolute path for `output::` is accepted and file created.
///
/// ## Coverage
/// Exit 0; file created at exact absolute path.
///
/// ## Validation Strategy
/// Create session. Run `.export ``output::``{abs_path}`. Assert exit 0 and file exists.
///
/// ## Related Requirements
/// `tests/docs/cli/param/08_output.md` — EC-2
#[ test ]
fn ec_2_output_absolute_path_accepted()
{
  let root = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-out2", "-default_topic", 2 );
  let out_path = out_dir.path().join( "absolute.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-out2" )
    .arg( format!( "output::{}", out_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_path.exists(),
    "EC-2: file must exist at absolute path; stderr: {}",
    stderr( &out )
  );
}

/// EC-3: Absolute path in `output::` temp directory accepted.
///
/// ## Purpose
/// Validates that an absolute path for `output::` in an existing directory
/// is accepted and the file is created.
///
/// ## Coverage
/// Exit 0; file created at the given absolute path.
///
/// ## Validation Strategy
/// Create session. Run `.export ``output::``{abs_path}` to an existing temp dir.
/// Assert exit 0 and file exists.
///
/// ## Related Requirements
/// `tests/docs/cli/param/08_output.md` — EC-3
#[ test ]
fn ec_3_output_tilde_path_accepted()
{
  let root = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-out3", "-default_topic", 2 );
  let out_path = out_dir.path().join( "clg-test-output.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-out3" )
    .arg( format!( "output::{}", out_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    out_path.exists(),
    "EC-3: file must exist at output path; stderr: {}",
    stderr( &out )
  );
}

/// EC-4: Relative path accepted.
///
/// ## Purpose
/// Validates that a relative path for `output::` is accepted.
///
/// ## Coverage
/// Exit 0; file created at relative path resolved from cwd.
///
/// ## Validation Strategy
/// Create session. Run `.export ``output::session``-output.md` from a temp cwd.
/// Assert exit 0 and file exists in that cwd.
///
/// ## Related Requirements
/// `tests/docs/cli/param/08_output.md` — EC-4
#[ test ]
fn ec_4_output_relative_path_accepted()
{
  let root = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-out4", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-out4" )
    .arg( "output::session-output.md" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    cwd.path().join( "session-output.md" ).exists(),
    "EC-4: file must exist at relative path; stderr: {}",
    stderr( &out )
  );
}

/// EC-5: Empty value rejected.
///
/// ## Purpose
/// Validates that `output::` with empty value is rejected.
///
/// ## Coverage
/// Exit 1; error indicating empty path is invalid.
///
/// ## Validation Strategy
/// Run `.export output::` (empty value). Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/08_output.md` — EC-5
#[ test ]
fn ec_5_output_empty_rejected()
{
  let out = common::clg_cmd()
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "output::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "output" ) || combined.contains( "path" ) || combined.contains( "empty" ),
    "EC-5: error must mention output or empty path; got: {combined}"
  );
}

/// EC-6: Nonexistent parent directory exits with 1.
///
/// ## Purpose
/// Validates that output to a nonexistent parent directory exits with an error.
///
/// ## Coverage
/// Exit 1; error about nonexistent parent directory (I/O error during write).
///
/// ## Validation Strategy
/// Create session. Run `.export ``output::``/nonexistent/dir/file.md`.
/// Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/08_output.md` — EC-6
#[ test ]
fn ec_6_output_nonexistent_parent_exits_2()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-out6", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-out6" )
    .arg( "output::/nonexistent-clg-test-dir/subdir/file.md" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
}

/// EC-7: Existing file is overwritten without error.
///
/// ## Purpose
/// Validates that `output::` overwrites an existing file silently.
///
/// ## Coverage
/// Exit 0; file content replaced (original sentinel content gone).
///
/// ## Validation Strategy
/// Write sentinel file. Run `.export ``output::``...` to same path.
/// Assert exit 0 and sentinel content replaced.
///
/// ## Related Requirements
/// `tests/docs/cli/param/08_output.md` — EC-7
#[ test ]
fn ec_7_output_overwrites_existing_file()
{
  let root = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-out7", "-default_topic", 2 );
  let out_path = out_dir.path().join( "existing.md" );
  std::fs::write( &out_path, b"ORIGINAL SENTINEL CONTENT" ).unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-out7" )
    .arg( format!( "output::{}", out_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let content = std::fs::read_to_string( &out_path ).unwrap();
  assert!(
    !content.contains( "ORIGINAL SENTINEL CONTENT" ),
    "EC-7: overwrite must replace original content; got: {content}"
  );
}

/// EC-8: Whitespace-only path rejected.
///
/// ## Purpose
/// Validates that a whitespace-only output path is rejected.
///
/// ## Coverage
/// Exit 1; error indicating whitespace-only path is invalid.
///
/// ## Validation Strategy
/// Run `.export ``output::`` ` (spaces only). Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/08_output.md` — EC-8
#[ test ]
fn ec_8_output_whitespace_only_rejected()
{
  let out = common::clg_cmd()
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "output:: " )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let _ = stdout( &out );
}
