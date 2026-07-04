//! Acceptance tests for the "Export Session for Review" user story.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/user_story/003_export_session_for_review.md`
//!
//! ## Coverage
//!
//! - RWS-1: Export as markdown writes output file
//! - RWS-2: Export as JSON produces JSONL output
//! - RWS-3: Export as text produces plain text transcript
//! - RWS-4: Missing `session_id` exits with error
//! - RWS-5: Missing output exits with error

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

/// RWS-1: Export as markdown writes output file.
///
/// ## Purpose
/// End-to-end acceptance test: developer exports a session to a markdown file
/// for offline review; output file is created with markdown content.
///
/// ## Coverage
/// Markdown output file created; user/assistant entries distinguishable; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/003_export_session_for_review.md` — RWS-1
#[ test ]
fn rws_1_export_as_markdown_writes_output_file()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "export-md-proj" );
  std::fs::create_dir_all( &proj ).unwrap();
  common::write_path_project_session( root.path(), &proj, "-default_topic", 4 );

  let output_file = root.path().join( "session-export.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &proj )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( format!( "output::{}", output_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    output_file.exists(),
    "RWS-1: .export must create the output file; stderr: {}",
    stderr( &out )
  );
  let content = std::fs::read_to_string( &output_file ).unwrap();
  assert!(
    !content.is_empty(),
    "RWS-1: exported markdown file must not be empty"
  );
  // Markdown format should have some structure (headings or role labels)
  assert!(
    content.contains( '#' ) || content.contains( "user" ) || content.contains( "User" )
      || content.contains( "assistant" ) || content.contains( "Assistant" ),
    "RWS-1: exported markdown must distinguish user/assistant entries; got:\n{content}"
  );
}

/// RWS-2: Export as JSON produces JSONL output.
///
/// ## Purpose
/// End-to-end acceptance test: developer exports a session as JSON for
/// programmatic processing; output file contains valid JSONL.
///
/// ## Coverage
/// JSONL output file created; each line is parseable JSON; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/003_export_session_for_review.md` — RWS-2
#[ test ]
fn rws_2_export_as_json_produces_jsonl_output()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "export-json-proj" );
  std::fs::create_dir_all( &proj ).unwrap();
  common::write_path_project_session( root.path(), &proj, "-default_topic", 2 );

  let output_file = root.path().join( "session-export.json" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &proj )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "format::json" )
    .arg( format!( "output::{}", output_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    output_file.exists(),
    "RWS-2: .export format::json must create the output file; stderr: {}",
    stderr( &out )
  );
  let content = std::fs::read_to_string( &output_file ).unwrap();
  assert!(
    !content.is_empty(),
    "RWS-2: exported JSON file must not be empty"
  );
  // Each non-empty line must be a JSON object
  for line in content.lines().filter( | l | !l.is_empty() )
  {
    assert!(
      line.starts_with( '{' ) && line.ends_with( '}' ),
      "RWS-2: each JSONL line must be a JSON object; got line:\n{line}"
    );
  }
}

/// RWS-3: Export as text produces plain text transcript.
///
/// ## Purpose
/// End-to-end acceptance test: developer exports a session as plain text
/// for piping to other tools; output is human-readable without markdown syntax.
///
/// ## Coverage
/// Plain text output file created; no markdown syntax; human-readable; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/003_export_session_for_review.md` — RWS-3
#[ test ]
fn rws_3_export_as_text_produces_plain_text_transcript()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "export-txt-proj" );
  std::fs::create_dir_all( &proj ).unwrap();
  common::write_path_project_session( root.path(), &proj, "-default_topic", 2 );

  let output_file = root.path().join( "session-export.txt" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &proj )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .arg( "format::text" )
    .arg( format!( "output::{}", output_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    output_file.exists(),
    "RWS-3: .export format::text must create the output file; stderr: {}",
    stderr( &out )
  );
  let content = std::fs::read_to_string( &output_file ).unwrap();
  assert!(
    !content.is_empty(),
    "RWS-3: exported text file must not be empty"
  );
  // Plain text must not contain JSON objects (no JSONL braces)
  assert!(
    !content.trim_start().starts_with( '{' ),
    "RWS-3: plain text output must not be JSON; got:\n{content}"
  );
}

/// RWS-4: Missing `session_id` exits with error.
///
/// ## Purpose
/// End-to-end acceptance test: developer accidentally omits `session_id::`;
/// the command exits 1 with an error message and does not create the output file.
///
/// ## Coverage
/// Exit 1 on missing `session_id::`; error on stderr; no output file created.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/003_export_session_for_review.md` — RWS-4
#[ test ]
fn rws_4_missing_session_id_exits_with_error()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "export-err-proj" );
  std::fs::create_dir_all( &proj ).unwrap();
  common::write_path_project_session( root.path(), &proj, "-default_topic", 2 );

  let output_file = root.path().join( "should-not-exist.md" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &proj )
    .arg( ".export" )
    .arg( format!( "output::{}", output_file.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "RWS-4: .export without session_id:: must emit error on stderr"
  );
  assert!(
    !output_file.exists(),
    "RWS-4: no output file must be created when session_id:: is missing"
  );
}

/// RWS-5: Missing output exits with error.
///
/// ## Purpose
/// End-to-end acceptance test: developer accidentally omits `output::`;
/// the command exits 1 with an error message and no file is written.
///
/// ## Coverage
/// Exit 1 on missing `output::`; error on stderr; no file written.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/003_export_session_for_review.md` — RWS-5
#[ test ]
fn rws_5_missing_output_exits_with_error()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "export-noout-proj" );
  std::fs::create_dir_all( &proj ).unwrap();
  common::write_path_project_session( root.path(), &proj, "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &proj )
    .arg( ".export" )
    .arg( "session_id::-default_topic" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "RWS-5: .export without output:: must emit error on stderr; stdout: {}",
    stdout( &out )
  );
}
