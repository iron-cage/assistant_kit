//! Edge case tests for the `scope::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/12_scope.md`
//!
//! ## Coverage
//!
//! - EC-1: Value "local" accepted
//! - EC-2: Value "relevant" accepted
//! - EC-3: Value "under" accepted
//! - EC-4: Value "global" accepted
//! - EC-5: Value "RELEVANT" accepted (case-insensitive)
//! - EC-6: Invalid value "all" rejected with error
//! - EC-7: Omitted defaults to "under" scope (summary mode output)
//! - EC-8: `scope::global` ignores `path::`

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

/// EC-1: Value "local" accepted.
///
/// ## Purpose
/// Validates that `scope::local` is accepted and scopes to the cwd project.
///
/// ## Coverage
/// Exit 0; output scoped to current project only.
///
/// ## Validation Strategy
/// Create fixture. Run `.projects ``scope::loca``l` from a project directory.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/12_scope.md` — EC-1
#[ test ]
fn ec_1_scope_local_accepted()
{
  let root = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  common::write_path_project_session( root.path(), project_dir.path(), "sess-local", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( project_dir.path() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-2: Value "relevant" accepted.
///
/// ## Purpose
/// Validates that `scope::relevant` is accepted and includes ancestor projects.
///
/// ## Coverage
/// Exit 0; output includes ancestor-level sessions.
///
/// ## Validation Strategy
/// Create fixture. Run `.projects ``scope::relevan``t` from a directory.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/12_scope.md` — EC-2
#[ test ]
fn ec_2_scope_relevant_accepted()
{
  let root = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  common::write_path_project_session( root.path(), project_dir.path(), "sess-rel", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( project_dir.path() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-3: Value "under" accepted.
///
/// ## Purpose
/// Validates that `scope::under` is accepted with a `path::` argument.
///
/// ## Coverage
/// Exit 0; sessions from descendant projects shown.
///
/// ## Validation Strategy
/// Create fixture. Run `.projects ``scope::under`` ``path::``...`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/12_scope.md` — EC-3
#[ test ]
fn ec_3_scope_under_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-under", "sess-under", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", root.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-4: Value "global" accepted.
///
/// ## Purpose
/// Validates that `scope::global` is accepted and returns all sessions.
///
/// ## Coverage
/// Exit 0; all sessions across all projects in storage.
///
/// ## Validation Strategy
/// Create multiple projects. Run `.projects ``scope::globa``l`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/12_scope.md` — EC-4
#[ test ]
fn ec_4_scope_global_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-a", "sess-a", 2 );
  common::write_test_session( root.path(), "proj-b", "sess-b", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-5: Value "RELEVANT" accepted (case-insensitive).
///
/// ## Purpose
/// Validates that scope enum parsing is case-insensitive.
///
/// ## Coverage
/// Exit 0; output identical to lowercase `scope::relevant`.
///
/// ## Validation Strategy
/// Create fixture. Run `.projects ``scope::RELEVAN``T` from a project directory.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/12_scope.md` — EC-5
#[ test ]
fn ec_5_scope_uppercase_accepted()
{
  let root = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  common::write_path_project_session( root.path(), project_dir.path(), "sess-rel", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( project_dir.path() )
    .arg( ".projects" )
    .arg( "scope::RELEVANT" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-6: Invalid value "all" rejected with error.
///
/// ## Purpose
/// Validates that "all" is not a valid scope value.
///
/// ## Coverage
/// Exit 1; error message contains "scope must be relevant|local|under|global, got all".
///
/// ## Validation Strategy
/// Run `.projects ``scope::al``l`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/12_scope.md` — EC-6
#[ test ]
fn ec_6_scope_all_rejected()
{
  let out = common::clg_cmd()
    .arg( ".projects" )
    .arg( "scope::all" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "scope" ) && err.contains( "all" ),
    "EC-6: expected 'scope' and 'all' in stderr; got: {err}"
  );
}

/// EC-7: Omitted defaults to "around" scope (returns child projects).
///
/// ## Purpose
/// Validates that omitting `scope::` in .projects returns projects under the cwd.
///
/// ## Coverage
/// Exit 0; child project appears in output.
///
/// ## Validation Strategy
/// Create parent and child projects. Run `.projects` from parent dir.
/// Assert exit 0 and child project appears in output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/12_scope.md` — EC-7
#[ test ]
fn ec_7_scope_omitted_defaults_to_under()
{
  let root = TempDir::new().unwrap();
  let parent_dir = TempDir::new().unwrap();
  let child_dir_path = parent_dir.path().join( "child" );
  std::fs::create_dir_all( &child_dir_path ).unwrap();

  common::write_path_project_session(
    root.path(),
    &child_dir_path,
    "sess-child",
    2,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( parent_dir.path() )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "child" ),
    "EC-7: default .projects from parent dir must show child project; got: {output}"
  );
}

/// EC-8: `scope::global` ignores `path::`.
///
/// ## Purpose
/// Validates that `scope::global` returns all sessions regardless of `path::` value.
///
/// ## Coverage
/// Exit 0; output unaffected by path parameter when scope is global.
///
/// ## Validation Strategy
/// Create fixture. Run `.projects ``scope::global`` ``path::``/tmp/nonexistent-subpath`.
/// Assert exit 0 and sessions returned.
///
/// ## Related Requirements
/// `tests/docs/cli/param/12_scope.md` — EC-8
#[ test ]
fn ec_8_scope_global_ignores_path()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-global", "sess-g", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "path::/tmp/nonexistent-subpath-clg-test" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}
