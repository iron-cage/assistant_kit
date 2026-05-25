//! Integration tests for the `clg .project.exists` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/09_project_exists.md`
//!
//! ## Coverage
//!
//! - INT-1: cwd with history exits 0
//! - INT-2: cwd without history exits 1
//! - INT-3: `path::` with history exits 0
//! - INT-4: `path::` without history exits 1
//! - INT-5: Exit 0 prints "sessions exist" to stdout (exact)
//! - INT-6: Exit 1 prints "no sessions" to stderr (exact)
//! - INT-7: `topic::` filters to topic-specific storage
//! - INT-8: `topic::` no history exits 1
//! - INT-9: Nonexistent path exits 1 (not error)
//! - INT-10: Empty `topic::` rejected with exit 1

mod common;

use std::fs;
use std::path::PathBuf;
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

/// Create storage history for a given directory path in `{home}/.claude/projects/`.
fn setup_history( home : &std::path::Path, project_path : &std::path::Path )
{
  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode_path should succeed" );
  let storage_dir = home.join( ".claude" ).join( "projects" ).join( &encoded );
  fs::create_dir_all( &storage_dir ).unwrap();
  fs::write( storage_dir.join( "session.jsonl" ), b"fake content\n" ).unwrap();
}

// ─── INT-1 ────────────────────────────────────────────────────────────────────

/// INT-1: cwd with history exits 0 and prints "sessions exist".
#[ test ]
fn int_1_cwd_with_history_exits_0()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();

  setup_history( home.path(), project.path() );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .current_dir( project.path() )
    .arg( ".project.exists" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "sessions exist" ),
    "must print 'sessions exist' on stdout; got:\n{s}"
  );
}

// ─── INT-2 ────────────────────────────────────────────────────────────────────

/// INT-2: cwd without history exits 1 and prints "no sessions" on stderr.
#[ test ]
fn int_2_cwd_without_history_exits_1()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  // No setup_history — project has no storage entry

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .current_dir( project.path() )
    .arg( ".project.exists" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let e = stderr( &out );
  assert!(
    e.contains( "no sessions" ),
    "must print 'no sessions' on stderr; got:\n{e}"
  );
}

// ─── INT-3 ────────────────────────────────────────────────────────────────────

/// INT-3: `path::` with history exits 0.
#[ test ]
fn int_3_path_with_history_exits_0()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();

  setup_history( home.path(), project.path() );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
    .arg( format!( "path::{}", project.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "sessions exist" ), "must print 'sessions exist'; got:\n{s}" );
}

// ─── INT-4 ────────────────────────────────────────────────────────────────────

/// INT-4: `path::` without history exits 1.
#[ test ]
fn int_4_path_without_history_exits_1()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  // No history for this project

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
    .arg( format!( "path::{}", project.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let e = stderr( &out );
  assert!( e.contains( "no sessions" ), "must print 'no sessions' on stderr; got:\n{e}" );
}

// ─── INT-5 ────────────────────────────────────────────────────────────────────

/// INT-5: Exit 0 prints exactly "sessions exist\n" to stdout.
///
/// stderr must be empty. stdout must be exactly the required string.
#[ test ]
fn int_5_exit_0_stdout_exact_sessions_exist()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();

  setup_history( home.path(), project.path() );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
    .arg( format!( "path::{}", project.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert_eq!(
    stdout( &out ).as_str(),
    "sessions exist\n",
    "stdout must be exactly 'sessions exist\\n'"
  );
  assert!(
    stderr( &out ).is_empty(),
    "stderr must be empty on success; got:\n{}",
    stderr( &out )
  );
}

// ─── INT-6 ────────────────────────────────────────────────────────────────────

/// INT-6: Exit 1 prints exactly "no sessions\n" to stderr.
///
/// stdout must be empty. stderr must be exactly the required string.
#[ test ]
fn int_6_exit_1_stderr_exact_no_sessions()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  // No history

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
    .arg( format!( "path::{}", project.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert_eq!(
    stderr( &out ).as_str(),
    "no sessions\n",
    "stderr must be exactly 'no sessions\\n'"
  );
  assert!(
    stdout( &out ).is_empty(),
    "stdout must be empty when no history; got:\n{}",
    stdout( &out )
  );
}

// ─── INT-7 ────────────────────────────────────────────────────────────────────

/// INT-7: `topic::` filters to topic-specific storage.
///
/// Base path has no history; topic-specific dir has history → exit 0.
#[ test ]
fn int_7_topic_filters_to_topic_specific_storage()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();

  // Session dir for this project with topic::work is {project}/-work
  let session_dir : PathBuf = project.path().join( "-work" );
  fs::create_dir_all( &session_dir ).unwrap();
  // Store history keyed to the session dir path itself
  setup_history( home.path(), &session_dir );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
    .arg( format!( "path::{}", project.path().display() ) )
    .arg( "topic::work" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "sessions exist" ), "must detect topic-specific history; got:\n{s}" );
}

// ─── INT-8 ────────────────────────────────────────────────────────────────────

/// INT-8: `topic::` no history exits 1 even when base path has history.
#[ test ]
fn int_8_topic_no_history_exits_1()
{
  let home    = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();

  // Base path has history, but topic "nonexistent" does not
  setup_history( home.path(), project.path() );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
    .arg( format!( "path::{}", project.path().display() ) )
    .arg( "topic::nonexistent" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let e = stderr( &out );
  assert!(
    e.contains( "no sessions" ),
    "must report 'no sessions' when topic-specific storage absent; got:\n{e}"
  );
}

// ─── INT-9 ────────────────────────────────────────────────────────────────────

/// INT-9: Nonexistent path exits 1 (not error — treated as no history).
#[ test ]
fn int_9_nonexistent_path_exits_1()
{
  let home = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
    .arg( "path::/tmp/nonexistent-project-int9-xyz-abc" )
    .output()
    .unwrap();

  // Must be exit 1 (no history), not exit 2 (storage error)
  assert_exit( &out, 1 );
  let e = stderr( &out );
  assert!(
    e.contains( "no sessions" ),
    "nonexistent path must produce 'no sessions' not an error; got:\n{e}"
  );
}

// ─── INT-10 ───────────────────────────────────────────────────────────────────

/// INT-10: Empty `topic::` rejected with exit 1.
#[ test ]
fn int_10_empty_topic_rejected_with_exit_1()
{
  let home = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
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
