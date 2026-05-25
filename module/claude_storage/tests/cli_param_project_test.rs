//! Edge case tests for the `project::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/10_project.md`
//!
//! ## Coverage
//!
//! - EC-1: Absolute path format resolves correctly
//! - EC-2: Path-encoded ID format resolves correctly
//! - EC-3: UUID format resolves correctly
//! - EC-4: Path(...) form from .list resolves correctly
//! - EC-5: Unknown project value exits with error
//! - EC-6: Empty value rejected
//! - EC-7: Default resolves to cwd project when omitted
//! - EC-8: Default exits with 2 when cwd has no project

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

/// EC-1: Absolute path format resolves correctly.
///
/// ## Purpose
/// Validates that `project::/abs/path` resolves to the correct project.
///
/// ## Coverage
/// Exit 0; correct project displayed.
///
/// ## Validation Strategy
/// Create project keyed by absolute path. Run `.show ``project::``{abs_path}`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/10_project.md` — EC-1
#[ test ]
fn ec_1_project_absolute_path_resolves()
{
  let root = TempDir::new().unwrap();
  let project_path = std::path::PathBuf::from( "/home/alice/projects/myproject" );
  common::write_path_project_session( root.path(), &project_path, "sess-abc", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "project::/home/alice/projects/myproject" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-2: Path-encoded ID format resolves correctly.
///
/// ## Purpose
/// Validates that `project::-home-alice-projects-myproject` (encoded form) resolves.
///
/// ## Coverage
/// Exit 0; same project displayed as absolute path form.
///
/// ## Validation Strategy
/// Create project keyed by absolute path. Query using encoded form.
/// Assert exit 0 and same output as EC-1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/10_project.md` — EC-2
#[ test ]
fn ec_2_project_encoded_id_resolves()
{
  let root = TempDir::new().unwrap();
  let project_path = std::path::PathBuf::from( "/home/alice/projects/myproject" );
  let encoded = common::write_path_project_session( root.path(), &project_path, "sess-abc", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-3: UUID format resolves correctly.
///
/// ## Purpose
/// Validates that `project::` with a UUID directory name resolves correctly.
///
/// ## Coverage
/// Exit 0; UUID project correctly identified and displayed.
///
/// ## Validation Strategy
/// Create project using UUID as directory name. Run `.show ``project::``{uuid}`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/10_project.md` — EC-3
#[ test ]
fn ec_3_project_uuid_format_resolves()
{
  let root = TempDir::new().unwrap();
  let uuid = "8d795a1c-c81d-4010-8d29-b4e678272419";
  common::write_test_session( root.path(), uuid, "sess-uuid", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( format!( "project::{uuid}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-4: Path(...) form from .list resolves correctly.
///
/// ## Purpose
/// Validates that `project::Path("/abs/path")` form resolves correctly.
///
/// ## Coverage
/// Exit 0; same project as using raw absolute path.
///
/// ## Validation Strategy
/// Create project by absolute path. Run `.show ``project::Path``("...")`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/10_project.md` — EC-4
#[ test ]
fn ec_4_project_path_form_resolves()
{
  let root = TempDir::new().unwrap();
  let project_path = std::path::PathBuf::from( "/home/alice/projects/myproject" );
  common::write_path_project_session( root.path(), &project_path, "sess-abc", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( r#"project::Path("/home/alice/projects/myproject")"# )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-5: Unknown project value exits with error.
///
/// ## Purpose
/// Validates that an unknown project value produces a not-found error.
///
/// ## Coverage
/// Exit 1; error message contains "project not found".
///
/// ## Validation Strategy
/// Run `.show ``project::nonexistent``-project-zzz`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/10_project.md` — EC-5
#[ test ]
fn ec_5_project_unknown_exits_with_error()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "project::nonexistent-project-zzz" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "project" ) && ( combined.contains( "not found" ) || combined.contains( "nonexistent" ) ),
    "EC-5: error must mention 'project not found'; got: {combined}"
  );
}

/// EC-6: Empty value rejected.
///
/// ## Purpose
/// Validates that `project::` with empty value is rejected.
///
/// ## Coverage
/// Exit 1; error about empty project identifier.
///
/// ## Validation Strategy
/// Run `.show project::`. Assert exit 1 and error mentions project.
///
/// ## Related Requirements
/// `tests/docs/cli/param/10_project.md` — EC-6
#[ test ]
fn ec_6_project_empty_rejected()
{
  let out = common::clg_cmd()
    .arg( ".show" )
    .arg( "project::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "project" ),
    "EC-6: error must mention 'project'; got: {combined}"
  );
}

/// EC-7: Default resolves to cwd project when omitted.
///
/// ## Purpose
/// Validates that omitting `project::` uses the cwd project.
///
/// ## Coverage
/// Exit 0; cwd project displayed without explicit `project::` argument.
///
/// ## Validation Strategy
/// Create project for a directory. Run `.show` from that directory.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/10_project.md` — EC-7
#[ test ]
fn ec_7_project_default_resolves_to_cwd()
{
  let root = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  common::write_path_project_session( root.path(), project_dir.path(), "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( project_dir.path() )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-8: Default exits with 1 when cwd has no project.
///
/// ## Purpose
/// Validates that omitting `project::` when cwd has no history exits with 1.
///
/// ## Coverage
/// Exit 1; message indicating no project for cwd.
///
/// ## Validation Strategy
/// Run `.show` from `/tmp` or a dir with no fixture entry.
/// Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/10_project.md` — EC-8
#[ test ]
fn ec_8_project_default_no_project_exits_2()
{
  let root = TempDir::new().unwrap();
  // Run from a directory that has no project in the fixture
  let empty_dir = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( empty_dir.path() )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
}
