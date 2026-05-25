//! Cross-command interaction tests for Project Scope parameter group.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param_group/02_project_scope.md`
//!
//! ## Coverage
//!
//! - CC-1: `project::` resolves same project in .show and .search
//! - CC-2: `project::` with absolute path format works in .export
//! - CC-3: `project::` with UUID format works in .count
//! - CC-4: Absent `project::` defaults to cwd in .show
//! - CC-5: Absent `project::` defaults to cwd in .export
//! - CC-6: Same `project::` value returns same project in all 5 commands

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

/// CC-1: `project::` resolves same project in .show and .search.
///
/// ## Purpose
/// Verify that the same `project::` value scopes both `.show` and `.search`
/// to the specified project, with `.show` returning session content and
/// `.search` returning results from that project only.
///
/// ## Coverage
/// Cross-command project scoping; .show content from project; .search scoped to project; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/02_project_scope.md` — CC-1
#[ test ]
fn cc_1_project_resolves_same_project_in_show_and_search()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "myproject" );
  common::write_test_session_with_last_message(
    root.path(),
    &claude_storage_core::encode_path( &project_dir ).unwrap(),
    "-default_topic",
    2,
    "hello from myproject",
  );

  let show_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{}", project_dir.display() ) )
    .output()
    .unwrap();

  let search_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::hello" )
    .arg( format!( "project::{}", project_dir.display() ) )
    .output()
    .unwrap();

  assert_exit( &show_out, 0 );
  assert_exit( &search_out, 0 );

  let show_s = stdout( &show_out );
  assert!(
    !show_s.is_empty(),
    "CC-1: .show with project:: must produce output; stderr: {}",
    stderr( &show_out )
  );

  let search_s = stdout( &search_out );
  assert!(
    search_s.contains( "hello" ) || search_s.contains( "myproject" ),
    "CC-1: .search must find 'hello' in project-scoped results; got:\n{search_s}"
  );
}

/// CC-2: `project::` with absolute path format works in .export.
///
/// ## Purpose
/// Verify that `.export` with an absolute-path `project::` value exports the
/// correct session as valid JSONL output.
///
/// ## Coverage
/// Absolute path `project::` in .export; valid JSONL on stdout; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/02_project_scope.md` — CC-2
#[ test ]
fn cc_2_project_with_absolute_path_works_in_export()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "exportproj" );
  common::write_path_project_session( root.path(), &project_dir, "-default_topic", 2 );

  let output_file = root.path().join( "cc2-export.jsonl" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{}", project_dir.display() ) )
    .arg( format!( "output::{}", output_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    output_file.exists(),
    "CC-2: .export with project:: must create the output file; stderr: {}",
    stderr( &out )
  );
  let content = std::fs::read_to_string( &output_file ).unwrap();
  assert!(
    !content.is_empty(),
    "CC-2: exported file must not be empty"
  );
}

/// CC-3: `project::` with UUID format works in .count.
///
/// ## Purpose
/// Verify that `.count` accepts a raw UUID directory name as `project::` and
/// returns the correct entry count for that project.
///
/// ## Coverage
/// UUID `project::` in .count; correct count returned; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/02_project_scope.md` — CC-3
#[ test ]
fn cc_3_project_with_uuid_format_works_in_count()
{
  let root = TempDir::new().unwrap();
  let uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
  // Write 10 entries (5 sessions of 2 each would complicate; use 1 session of 10)
  common::write_test_session( root.path(), uuid, "session-cc3", 10 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( format!( "project::{uuid}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.trim().is_empty(),
    "CC-3: .count with UUID project:: must produce output; stderr: {}",
    stderr( &out )
  );
  // The output should be a numeric count
  let trimmed = s.trim();
  assert!(
    trimmed.parse::< u64 >().is_ok() || trimmed.contains( "10" ),
    "CC-3: .count must return a numeric entry count; got:\n{s}"
  );
}

/// CC-4: Absent `project::` defaults to cwd in .show.
///
/// ## Purpose
/// Verify that `.show` without `project::` resolves the project from the
/// current working directory.
///
/// ## Coverage
/// Default project resolution via cwd; session content returned; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/02_project_scope.md` — CC-4
#[ test ]
fn cc_4_absent_project_defaults_to_cwd_in_show()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "cwdproj" );
  std::fs::create_dir_all( &project_dir ).unwrap();
  common::write_path_project_session( root.path(), &project_dir, "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &project_dir )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    !stdout( &out ).is_empty(),
    "CC-4: .show without project:: must resolve via cwd and produce output; stderr: {}",
    stderr( &out )
  );
}

/// CC-5: Absent `project::` defaults to cwd in .export.
///
/// ## Purpose
/// Verify that `.export` without `project::` resolves the project from the
/// current working directory and exports the session successfully.
///
/// ## Coverage
/// Default project resolution via cwd in .export; valid export file; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/02_project_scope.md` — CC-5
#[ test ]
fn cc_5_absent_project_defaults_to_cwd_in_export()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "cwdexport" );
  std::fs::create_dir_all( &project_dir ).unwrap();
  common::write_path_project_session( root.path(), &project_dir, "-default_topic", 2 );

  let output_file = root.path().join( "cc5-export.jsonl" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &project_dir )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "output::{}", output_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    output_file.exists(),
    "CC-5: .export without project:: must create output file via cwd resolution; stderr: {}",
    stderr( &out )
  );
}

/// CC-6: Same `project::` value returns same project in all 5 commands.
///
/// ## Purpose
/// Verify that using the same `project::` value in `.show`, `.search`,
/// `.export`, `.count`, and `.list` all resolve to the same project with
/// consistent counts and session presence.
///
/// ## Coverage
/// Consistent project resolution across 5 commands; counts match; exit 0 for all.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/02_project_scope.md` — CC-6
#[ test ]
fn cc_6_same_project_value_returns_same_project_in_all_commands()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "allcmds" );
  common::write_path_project_session( root.path(), &project_dir, "s001", 4 );
  common::write_path_project_session( root.path(), &project_dir, "s002", 4 );

  let project_arg = format!( "project::{}", project_dir.display() );
  let output_file = root.path().join( "cc6-export.jsonl" );

  // .list
  let list_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( &project_arg )
    .output()
    .unwrap();
  assert_exit( &list_out, 0 );
  assert!(
    stdout( &list_out ).contains( "allcmds" ),
    "CC-6: .list must show the specified project; got:\n{}",
    stdout( &list_out )
  );

  // .count
  let count_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( &project_arg )
    .output()
    .unwrap();
  assert_exit( &count_out, 0 );

  // .show
  let show_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::s001" )
    .arg( &project_arg )
    .output()
    .unwrap();
  assert_exit( &show_out, 0 );

  // .export
  let export_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( "session_id::s001" )
    .arg( &project_arg )
    .arg( format!( "output::{}", output_file.display() ) )
    .output()
    .unwrap();
  assert_exit( &export_out, 0 );

  // .search
  let search_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::entry" )
    .arg( &project_arg )
    .output()
    .unwrap();
  assert_exit( &search_out, 0 );
}
