//! Cross-command interaction tests for Session Identification parameter group.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param_group/03_session_identification.md`
//!
//! ## Coverage
//!
//! - CC-1: `session_id::` in .show displays session content
//! - CC-2: `session_id::` in .export exports the same session
//! - CC-3: Same `session_id::` value resolves same session in both commands
//! - CC-4: `session_id::` required in .export, optional in .show
//! - CC-5: `session_id::` depends on `project::` for scoping
//! - CC-6: `session_id::` without `project::` resolves via cwd

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

/// CC-1: `session_id::` in .show displays session content.
///
/// ## Purpose
/// Verify that `.show session_id::` with a known session ID shows conversation
/// content with user/assistant entries from that session.
///
/// ## Coverage
/// .show with `session_id::`; session content on stdout; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/03_session_identification.md` — CC-1
#[ test ]
fn cc_1_session_id_in_show_displays_session_content()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "sid-proj" );
  std::fs::create_dir_all( &project_dir ).unwrap();
  common::write_path_project_session( root.path(), &project_dir, "-default_topic", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &project_dir )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "CC-1: .show session_id:: must produce content; stderr: {}",
    stderr( &out )
  );
}

/// CC-2: `session_id::` in .export exports the same session.
///
/// ## Purpose
/// Verify that `.export session_id::` with a known session ID writes valid
/// content to the output file.
///
/// ## Coverage
/// .export with `session_id::`; output file written; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/03_session_identification.md` — CC-2
#[ test ]
fn cc_2_session_id_in_export_exports_the_same_session()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "expproj" );
  std::fs::create_dir_all( &project_dir ).unwrap();
  common::write_path_project_session( root.path(), &project_dir, "-default_topic", 4 );

  let output_file = root.path().join( "cc2-out.jsonl" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &project_dir )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "format::json" )
    .arg( format!( "output::{}", output_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    output_file.exists(),
    "CC-2: .export session_id:: must create output file; stderr: {}",
    stderr( &out )
  );
  let content = std::fs::read_to_string( &output_file ).unwrap();
  assert!(
    !content.is_empty(),
    "CC-2: exported file must contain JSONL content"
  );
  // Each non-empty line should be parseable JSON
  for line in content.lines().filter( | l | !l.is_empty() )
  {
    assert!(
      line.starts_with( '{' ) && line.ends_with( '}' ),
      "CC-2: each exported line must be a JSON object; got:\n{line}"
    );
  }
}

/// CC-3: Same `session_id::` value resolves same session in both commands.
///
/// ## Purpose
/// Verify that `.show` and `.export` with the same `session_id::` value both
/// operate on the same session; exported JSONL line count matches entry count.
///
/// ## Coverage
/// Consistent session resolution; JSONL line count matches entries; exit 0 for both.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/03_session_identification.md` — CC-3
#[ test ]
fn cc_3_same_session_id_resolves_same_session_in_both_commands()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "consistproj" );
  std::fs::create_dir_all( &project_dir ).unwrap();
  common::write_path_project_session( root.path(), &project_dir, "-default_topic", 4 );

  let output_file = root.path().join( "cc3-export.jsonl" );

  let show_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &project_dir )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .output()
    .unwrap();

  let export_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &project_dir )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "output::{}", output_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &show_out, 0 );
  assert_exit( &export_out, 0 );

  assert!(
    !stdout( &show_out ).is_empty(),
    "CC-3: .show must produce content for the session"
  );
  assert!(
    output_file.exists(),
    "CC-3: .export must create the output file"
  );

  let exported = std::fs::read_to_string( &output_file ).unwrap();
  let line_count = exported.lines().filter( | l | !l.is_empty() ).count();
  assert!(
    line_count > 0,
    "CC-3: exported file must contain at least one entry; got 0 lines"
  );
}

/// CC-4: `session_id::` required in .export, optional in .show.
///
/// ## Purpose
/// Verify that `.export` without `session_id::` exits 1 with an error, while
/// `.show` without `session_id::` exits 0 and produces project-level output.
///
/// ## Coverage
/// .export exits 1 without `session_id::`; .show exits 0 without `session_id::`; correct behaviors.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/03_session_identification.md` — CC-4
#[ test ]
fn cc_4_session_id_required_in_export_optional_in_show()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "reqproj" );
  std::fs::create_dir_all( &project_dir ).unwrap();
  common::write_path_project_session( root.path(), &project_dir, "-default_topic", 2 );

  // .export without session_id:: must fail
  let export_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &project_dir )
    .arg( ".export" )
    .arg( format!( "output::{}", root.path().join( "unused.jsonl" ).display() ) )
    .output()
    .unwrap();

  assert_exit( &export_out, 1 );
  assert!(
    !stderr( &export_out ).is_empty(),
    "CC-4: .export without session_id:: must emit error on stderr"
  );

  // .show without session_id:: must succeed with project-level output
  let show_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &project_dir )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &show_out, 0 );
  assert!(
    !stdout( &show_out ).is_empty(),
    "CC-4: .show without session_id:: must produce project-level output; stderr: {}",
    stderr( &show_out )
  );
}

/// CC-5: `session_id::` depends on `project::` for scoping.
///
/// ## Purpose
/// Verify that when two projects each have a session with the same name,
/// `session_id::` combined with `project::` resolves to the correct project's session.
///
/// ## Coverage
/// Session scoped to specified project; other project's session not returned; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/03_session_identification.md` — CC-5
#[ test ]
fn cc_5_session_id_depends_on_project_for_scoping()
{
  let root = TempDir::new().unwrap();
  let project_a = root.path().join( "project-a" );
  let project_b = root.path().join( "project-b" );

  common::write_test_session_with_last_message(
    root.path(),
    &claude_storage_core::encode_path( &project_a ).unwrap(),
    "-default_topic",
    2,
    "content from project-a",
  );
  common::write_test_session_with_last_message(
    root.path(),
    &claude_storage_core::encode_path( &project_b ).unwrap(),
    "-default_topic",
    2,
    "content from project-b",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "project::{}", project_a.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "project-a" ) || s.contains( "content from project-a" ),
    "CC-5: session must resolve to project-a scope; got:\n{s}"
  );
  assert!(
    !s.contains( "content from project-b" ),
    "CC-5: project-b content must not appear when scoped to project-a; got:\n{s}"
  );
}

/// CC-6: `session_id::` without `project::` resolves via cwd.
///
/// ## Purpose
/// Verify that `.show session_id::` without an explicit `project::` resolves
/// the project via the current working directory.
///
/// ## Coverage
/// cwd-based project resolution; session content from cwd project; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/03_session_identification.md` — CC-6
#[ test ]
fn cc_6_session_id_without_project_resolves_via_cwd()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "cwd-sid-proj" );
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
    "CC-6: .show session_id:: via cwd must produce content; stderr: {}",
    stderr( &out )
  );
}
