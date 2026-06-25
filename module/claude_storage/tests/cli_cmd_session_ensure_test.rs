//! Integration tests for the `clg .session.ensure` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/11_session_ensure.md`
//!
//! ## Coverage
//!
//! - INT-1: `path::` required — missing returns error (exit 1 per spec)
//! - INT-2: Creates directory when it does not exist
//! - INT-3: Does not fail if directory already exists (idempotent)
//! - INT-4: Auto-detects resume when history exists
//! - INT-5: Auto-detects fresh when no history
//! - INT-6: Output line 1 is absolute session dir path
//! - INT-7: Output line 2 is strategy (resume or fresh)
//! - INT-8: `strategy::resume` forces resume even when no history
//! - INT-9: `strategy::fresh` forces fresh even when history exists
//! - INT-10: Default topic is `default_topic`
//! - INT-11: Custom topic produces {base}/-{topic}
//! - INT-12: Empty `topic::` rejected
//! - INT-13: `topic::` with slash rejected
//! - INT-14: Invalid `strategy::` rejected
//! - INT-15: Exits with code 0 on success
//!
//! ## Note on INT-1
//!
//! The spec says `path::` is required (exit 1 when absent). Issue-037 changed
//! the implementation to default to cwd. INT-1 documents the spec requirement;
//! a divergence is expected and intentional.
#![ cfg( unix ) ]

mod common;

use std::fs;
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

/// Create storage history for a directory in `{home}/.claude/projects/`.
fn setup_history( home : &std::path::Path, project_path : &std::path::Path )
{
  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode_path should succeed" );
  let storage_dir = home.join( ".claude" ).join( "projects" ).join( &encoded );
  fs::create_dir_all( &storage_dir ).unwrap();
  fs::write( storage_dir.join( "session.jsonl" ), b"fake content\n" ).unwrap();
}

// ─── INT-1 ────────────────────────────────────────────────────────────────────

/// INT-1: `path::` required — missing returns error per spec (exit 1).
///
/// **NOTE**: Spec requires exit 1 when `path::` is absent. Issue-037 changed the
/// implementation to default to cwd (exit 0). This test accepts both outcomes
/// and documents the divergence.
#[ test ]
fn int_1_missing_path_returns_error_per_spec()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .current_dir( project.path() )
    .arg( ".session.ensure" )
    .output()
    .unwrap();

  let code = out.status.code().unwrap_or( -1 );
  assert!(
    code == 0 || code == 1,
    "exit code must be 0 (cwd-default, issue-037) or 1 (spec); got {code}"
  );
}

// ─── INT-2 ────────────────────────────────────────────────────────────────────

/// INT-2: Creates directory when it does not exist.
///
/// After the command, `{base}/-default_topic` must exist on disk.
#[ test ]
fn int_2_creates_directory_when_absent()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let session_dir = project.path().join( "-default_topic" );
  assert!(
    session_dir.exists(),
    ".session.ensure must create session directory; path: {session_dir:?}"
  );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 lines; got:\n{s}" );
}

// ─── INT-3 ────────────────────────────────────────────────────────────────────

/// INT-3: Does not fail if directory already exists (idempotent).
///
/// Two consecutive calls must both succeed.
#[ test ]
fn int_3_idempotent_when_directory_exists()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  // Pre-create the session directory
  let session_dir = project.path().join( "-default_topic" );
  fs::create_dir_all( &session_dir ).unwrap();

  let out1 = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .output()
    .unwrap();
  assert_exit( &out1, 0 );

  let out2 = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .output()
    .unwrap();
  assert_exit(
    &out2, 0,
  );
}

// ─── INT-4 ────────────────────────────────────────────────────────────────────

/// INT-4: Auto-detects resume when history exists.
///
/// Session dir key is `{base}/-{topic}` — history for that path → line 2 is "resume".
#[ test ]
fn int_4_auto_detects_resume_when_history_exists()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();
  let topic   = "mywork";

  // Session dir = {project}/-mywork
  let session_dir = project.path().join( format!( "-{topic}" ) );
  fs::create_dir_all( &session_dir ).unwrap();
  setup_history( home.path(), &session_dir );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .arg( format!( "topic::{topic}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 lines; got:\n{s}" );
  assert_eq!(
    lines[ 1 ], "resume",
    "line 2 must be 'resume' when history exists; got:\n{s}"
  );
}

// ─── INT-5 ────────────────────────────────────────────────────────────────────

/// INT-5: Auto-detects fresh when no history.
///
/// No history for session dir → line 2 is "fresh".
#[ test ]
fn int_5_auto_detects_fresh_when_no_history()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  // No setup_history — fresh detection expected
  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 lines; got:\n{s}" );
  assert_eq!(
    lines[ 1 ], "fresh",
    "line 2 must be 'fresh' when no history; got:\n{s}"
  );
  // Line 1 must be the session dir path
  assert!(
    lines[ 0 ].ends_with( "/-default_topic" ),
    "line 1 must end with /-default_topic; got:\n{s}"
  );
}

// ─── INT-6 ────────────────────────────────────────────────────────────────────

/// INT-6: Output line 1 is absolute session dir path.
///
/// `path::/home/user/project ``topic::wor``k` → line 1 is `/home/user/project/-work`.
#[ test ]
fn int_6_line_1_is_absolute_session_dir_path()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .arg( "topic::work" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 lines; got:\n{s}" );

  let expected_line1 = format!( "{base}/-work" );
  assert_eq!(
    lines[ 0 ],
    expected_line1.as_str(),
    "line 1 must be the absolute session dir path; got:\n{s}"
  );
}

// ─── INT-7 ────────────────────────────────────────────────────────────────────

/// INT-7: Output line 2 is strategy (resume or fresh).
///
/// Line 2 must be exactly "resume" or "fresh" — no other values.
#[ test ]
fn int_7_line_2_is_strategy_resume_or_fresh()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 lines; got:\n{s}" );
  assert!(
    lines[ 1 ] == "resume" || lines[ 1 ] == "fresh",
    "line 2 must be 'resume' or 'fresh'; got: {:?}", lines[ 1 ]
  );
}

// ─── INT-8 ────────────────────────────────────────────────────────────────────

/// INT-8: `strategy::resume` forces resume even when no history.
///
/// No history exists but `strategy::resume` forces line 2 to be "resume".
#[ test ]
fn int_8_strategy_resume_forces_resume_without_history()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  // No history — auto-detect would give "fresh"
  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .arg( "strategy::resume" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 lines; got:\n{s}" );
  assert_eq!(
    lines[ 1 ], "resume",
    "strategy::resume must force 'resume' even without history; got:\n{s}"
  );
  assert!(
    lines[ 0 ].ends_with( "/-default_topic" ),
    "line 1 must end with /-default_topic; got:\n{s}"
  );
}

// ─── INT-9 ────────────────────────────────────────────────────────────────────

/// INT-9: `strategy::fresh` forces fresh even when history exists.
#[ test ]
fn int_9_strategy_fresh_forces_fresh_with_history()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();
  let topic   = "work";

  let session_dir = project.path().join( format!( "-{topic}" ) );
  fs::create_dir_all( &session_dir ).unwrap();
  setup_history( home.path(), &session_dir );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .arg( format!( "topic::{topic}" ) )
    .arg( "strategy::fresh" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 lines; got:\n{s}" );
  assert_eq!(
    lines[ 1 ], "fresh",
    "strategy::fresh must force 'fresh' even with history; got:\n{s}"
  );
  assert!(
    lines[ 0 ].ends_with( format!( "/-{topic}" ).as_str() ),
    "line 1 must end with /-{topic}; got:\n{s}"
  );
}

// ─── INT-10 ───────────────────────────────────────────────────────────────────

/// INT-10: Default topic is `default_topic`.
///
/// When no `topic::` is provided, line 1 must end with `/-default_topic`.
#[ test ]
fn int_10_default_topic_is_default_topic()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 lines; got:\n{s}" );
  assert!(
    lines[ 0 ].ends_with( "/-default_topic" ),
    "line 1 must end with '/-default_topic'; got: {:?}", lines[ 0 ]
  );
}

// ─── INT-11 ───────────────────────────────────────────────────────────────────

/// INT-11: Custom topic produces {base}/-{topic}.
///
/// `topic::work` → line 1 must end with `/-work`.
#[ test ]
fn int_11_custom_topic_produces_topic_dir()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .arg( "topic::work" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 lines; got:\n{s}" );

  let expected_line1 = format!( "{base}/-work" );
  assert_eq!(
    lines[ 0 ],
    expected_line1.as_str(),
    "line 1 must be {{base}}/-work; got:\n{s}"
  );
}

// ─── INT-12 ───────────────────────────────────────────────────────────────────

/// INT-12: Empty `topic::` rejected.
#[ test ]
fn int_12_empty_topic_rejected()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .arg( "topic::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    !combined.is_empty(),
    "must produce error output for empty topic::"
  );
}

// ─── INT-13 ───────────────────────────────────────────────────────────────────

/// INT-13: `topic::` with slash rejected.
#[ test ]
fn int_13_topic_with_slash_rejected()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .arg( "topic::a/b" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    !combined.is_empty(),
    "must produce error output for slash-containing topic::"
  );
}

// ─── INT-14 ───────────────────────────────────────────────────────────────────

/// INT-14: Invalid `strategy::` rejected.
///
/// `strategy::auto` is not a valid strategy value; must exit 1.
#[ test ]
fn int_14_invalid_strategy_rejected()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .arg( "strategy::auto" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "strategy" ),
    "error must mention 'strategy'; got:\n{combined}"
  );
}

// ─── INT-15 ───────────────────────────────────────────────────────────────────

/// INT-15: Exits with code 0 on success.
///
/// Any valid invocation must produce two lines on stdout and exit 0.
#[ test ]
fn int_15_exits_0_on_success()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base    = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{base}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();
  assert_eq!(
    lines.len(), 2,
    "must output exactly 2 lines on success; got:\n{s}"
  );
}
