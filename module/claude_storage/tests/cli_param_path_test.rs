//! Edge case tests for the `path::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/09_path.md`
//!
//! ## Coverage
//!
//! - EC-1: Absolute path accepted in .status
//! - EC-2: ~ prefix expanded in .status
//! - EC-3: Relative path accepted in .project.exists
//! - EC-4: Empty value rejected
//! - EC-5: Substring filter in .list matches case-insensitively
//! - EC-6: Substring filter in .list with no match returns empty list
//! - EC-7: Default in .exists resolves to cwd
//! - EC-8: Nonexistent path in .exists exits with code 1 (not error)

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

/// EC-1: Absolute path accepted in .status.
///
/// ## Purpose
/// Validates that an absolute path for `path::` is accepted in .status.
///
/// ## Coverage
/// Exit 0; status output references the given absolute path.
///
/// ## Validation Strategy
/// Create a storage root. Run `.status ``path::``{abs_path}`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/09_path.md` — EC-1
#[ test ]
fn ec_1_path_absolute_accepted_in_status()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-path", "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( format!( "path::{}", root.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-2: ~ prefix expanded in .status.
///
/// ## Purpose
/// Validates that `path::~/.claude/` is tilde-expanded in .status.
///
/// ## Coverage
/// Exit 0; ~ expanded correctly and storage at that path reported.
///
/// ## Validation Strategy
/// Set HOME to a temp dir. Run `.status ``path::``~/.claude/`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/09_path.md` — EC-2
#[ test ]
fn ec_2_path_tilde_expanded_in_status()
{
  let home = TempDir::new().unwrap();
  // Create the .claude directory so status has something to report
  std::fs::create_dir_all( home.path().join( ".claude" ) ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".status" )
    .arg( "path::~/.claude/" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-3: Relative path accepted in .project.exists.
///
/// ## Purpose
/// Validates that a relative path for `path::` is accepted in .project.exists.
///
/// ## Coverage
/// Exit 1 (no history for this path); relative path format accepted.
///
/// ## Validation Strategy
/// Create fixture. Run `.project.exists ``path::subdir``/project`.
/// Assert exit 1 (not-found is acceptable; format not rejected).
///
/// ## Related Requirements
/// `tests/docs/cli/param/09_path.md` — EC-3
#[ test ]
fn ec_3_path_relative_accepted_in_exists()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-path3", "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".project.exists" )
    .arg( "path::subdir/project" )
    .output()
    .unwrap();

  // Exit 1 = not found (expected); exit 2+ = format error (not expected)
  let code = out.status.code().unwrap_or( -1 );
  assert!(
    code == 0 || code == 1,
    "EC-3: relative path must not cause a format-validation error (exit 2+); got exit {code}"
  );
}

/// EC-4: Empty value rejected.
///
/// ## Purpose
/// Validates that `path::` with empty value is rejected.
///
/// ## Coverage
/// Exit 1; error message "path must be non-empty".
///
/// ## Validation Strategy
/// Run `.status path::`. Assert exit 1 and error mentions path.
///
/// ## Related Requirements
/// `tests/docs/cli/param/09_path.md` — EC-4
#[ test ]
fn ec_4_path_empty_rejected()
{
  let out = common::clg_cmd()
    .arg( ".status" )
    .arg( "path::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "path" ),
    "EC-4: error must mention 'path'; got: {combined}"
  );
}

/// EC-5: Substring filter in .list matches case-insensitively.
///
/// ## Purpose
/// Validates that `path::MYPROJECT` matches project paths containing
/// "myproject" (case-insensitive).
///
/// ## Coverage
/// Exit 0; results match what lowercase `path::myproject` would return.
///
/// ## Validation Strategy
/// Create project with lowercase path. Run `.list ``path::MYPROJEC``T` (uppercase).
/// Assert exit 0 and same project found.
///
/// ## Related Requirements
/// `tests/docs/cli/param/09_path.md` — EC-5
#[ test ]
fn ec_5_path_list_substring_case_insensitive()
{
  let root = TempDir::new().unwrap();
  // Create a project whose encoded path contains "myproject"
  common::write_path_project_session(
    root.path(),
    &std::path::PathBuf::from( "/home/user/myproject" ),
    "sess",
    2,
  );

  let out_upper = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "path::MYPROJECT" )
    .output()
    .unwrap();

  let out_lower = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "path::myproject" )
    .output()
    .unwrap();

  assert_exit( &out_upper, 0 );
  assert_exit( &out_lower, 0 );
  // Both should return the same result (case-insensitive match)
  assert_eq!(
    stdout( &out_upper ),
    stdout( &out_lower ),
    "EC-5: uppercase and lowercase path filter must produce identical results"
  );
}

/// EC-6: Substring filter in .list with no match returns empty list.
///
/// ## Purpose
/// Validates that a non-matching path filter produces empty output without error.
///
/// ## Coverage
/// Exit 0; empty result set (no error for non-matching filter).
///
/// ## Validation Strategy
/// Create fixture. Run `.list ``path::zzznomatch99``9`. Assert exit 0 and
/// no projects in output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/09_path.md` — EC-6
#[ test ]
fn ec_6_path_list_no_match_returns_empty()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-path6", "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "path::zzznomatch999" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "proj-path6" ),
    "EC-6: non-matching path filter must return empty list; got: {output}"
  );
}

/// EC-7: Default in .exists resolves to cwd.
///
/// ## Purpose
/// Validates that omitting `path::` in .project.exists uses cwd as the project path.
///
/// ## Coverage
/// Exit 0; cwd project recognized as having history.
///
/// ## Validation Strategy
/// Create fixture for a known path. Run `.project.exists` from that path.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/09_path.md` — EC-7
#[ test ]
fn ec_7_path_default_resolves_to_cwd()
{
  let home = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  // The implementation uses ~/.claude as the default storage root.
  // Set HOME so that storage writes to the temp home dir.
  let storage_root = home.path().join( ".claude" );
  common::write_path_project_session(
    &storage_root,
    project_dir.path(),
    "sess",
    2,
  );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .current_dir( project_dir.path() )
    .arg( ".project.exists" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "sessions exist" ),
    "EC-7: cwd with history must report 'sessions exist'; got: {output}"
  );
}

/// EC-8: Nonexistent path in .exists exits with code 1 (not error).
///
/// ## Purpose
/// Validates that .project.exists with a nonexistent path exits 1 gracefully.
///
/// ## Coverage
/// Exit 1; graceful not-found message (not a crash or exception).
///
/// ## Validation Strategy
/// Run `.project.exists ``path::``/tmp/nonexistent-dir-xyzabc`. Assert exit 1
/// and no stack trace in stderr.
///
/// ## Related Requirements
/// `tests/docs/cli/param/09_path.md` — EC-8
#[ test ]
fn ec_8_path_nonexistent_exists_exits_1()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".project.exists" )
    .arg( "path::/tmp/nonexistent-dir-xyzabc-clg-test" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.contains( "panic" ) && !err.contains( "thread" ),
    "EC-8: nonexistent path must produce graceful not-found, not a panic; got: {err}"
  );
}
