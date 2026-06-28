//! Acceptance tests for the "Query Storage Programmatically" user story.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/user_story/04_query_storage_programmatically.md`
//!
//! ## Coverage
//!
//! - RWS-1: status outputs key=value pairs
//! - RWS-2: count outputs bare integer
//! - RWS-3: count `target::` specifies what to count
//! - RWS-4: `path::` scopes query to alternate storage root
//! - RWS-5: Non-existent storage root exits non-zero

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

/// RWS-1: status outputs key=value pairs.
///
/// ## Purpose
/// End-to-end acceptance test: script captures storage stats for a dashboard;
/// `.status` outputs `projects: 2, sessions: 5` format.
///
/// ## Coverage
/// Parseable key=value output; no decorations; exact counts match fixture; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/04_query_storage_programmatically.md` — RWS-1
#[ test ]
fn rws_1_status_outputs_key_value_pairs()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "qsp1-proj-a" );
  let p2 = root.path().join( "qsp1-proj-b" );
  // 2 projects, 5 sessions total
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p1, "s002", 2 );
  common::write_path_project_session( root.path(), &p1, "s003", 2 );
  common::write_path_project_session( root.path(), &p2, "s004", 2 );
  common::write_path_project_session( root.path(), &p2, "s005", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.to_lowercase().contains( "projects: 2" ),
    "RWS-1: .status must output 'projects: 2' (case-insensitive); got:\n{s}"
  );
  assert!(
    s.to_lowercase().contains( "sessions: 5" ),
    "RWS-1: .status must output 'sessions: 5' (case-insensitive); got:\n{s}"
  );
  // No table borders or decorative headers
  assert!(
    !s.contains( "===" ) && !s.contains( "│" ) && !s.contains( "┌" ),
    "RWS-1: .status must contain no decorative characters; got:\n{s}"
  );
}

/// RWS-2: count outputs bare integer.
///
/// ## Purpose
/// End-to-end acceptance test: script checks project count for a threshold
/// alert; `.count` outputs a single integer usable in shell arithmetic.
///
/// ## Coverage
/// Single bare integer on stdout; no labels or decorators; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/04_query_storage_programmatically.md` — RWS-2
#[ test ]
fn rws_2_count_outputs_bare_integer()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "qsp2-proj-a" );
  let p2 = root.path().join( "qsp2-proj-b" );
  let p3 = root.path().join( "qsp2-proj-c" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p3, "s003", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let trimmed = s.trim();
  assert!(
    trimmed.parse::< u64 >().is_ok(),
    "RWS-2: .count must output a bare integer; got:\n{s}"
  );
  assert_eq!(
    trimmed,
    "3",
    "RWS-2: project count must be 3 for 3-project fixture; got:\n{s}"
  );
}

/// RWS-3: count `target::` specifies what to count.
///
/// ## Purpose
/// End-to-end acceptance test: script needs session count rather than project
/// count; `.count ```target::session```s` returns total sessions across all projects.
///
/// ## Coverage
/// Session count returned; correct total from multi-project fixture; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/04_query_storage_programmatically.md` — RWS-3
#[ test ]
fn rws_3_count_target_specifies_what_to_count()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "qsp3-proj-a" );
  let p2 = root.path().join( "qsp3-proj-b" );
  // Project A: 3 sessions, Project B: 2 sessions = 5 total
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p1, "s002", 2 );
  common::write_path_project_session( root.path(), &p1, "s003", 2 );
  common::write_path_project_session( root.path(), &p2, "s004", 2 );
  common::write_path_project_session( root.path(), &p2, "s005", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let trimmed = s.trim();
  assert!(
    trimmed.parse::< u64 >().is_ok(),
    "RWS-3: .count target::sessions must output a bare integer; got:\n{s}"
  );
  assert_eq!(
    trimmed,
    "5",
    "RWS-3: session count must be 5 for the 5-session fixture; got:\n{s}"
  );
}

/// RWS-4: `path::` scopes query to alternate storage root.
///
/// ## Purpose
/// End-to-end acceptance test: script monitors a secondary storage location;
/// `.count path::` reads from the alternate root and returns its project count.
///
/// ## Coverage
/// Alternate storage root via `path::`; count from alternate fixture; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/04_query_storage_programmatically.md` — RWS-4
#[ test ]
fn rws_4_path_scopes_query_to_alternate_storage_root()
{
  let alt_root = TempDir::new().unwrap();
  let proj = alt_root.path().join( "qsp4-alt-proj" );
  common::write_path_project_session( alt_root.path(), &proj, "s001", 2 );

  let out = common::clg_cmd()
    .arg( ".count" )
    .arg( format!( "path::{}", alt_root.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let trimmed = s.trim();
  assert!(
    trimmed.parse::< u64 >().is_ok(),
    "RWS-4: .count path:: must output a bare integer; got:\n{s}"
  );
  assert_eq!(
    trimmed,
    "1",
    "RWS-4: count must be 1 for the single-project alternate root; got:\n{s}"
  );
}

/// RWS-5: Non-existent storage root exits non-zero.
///
/// ## Purpose
/// End-to-end acceptance test: script handles inaccessible storage gracefully;
/// `.status` on a non-existent path exits with code 2 and stderr error.
///
/// ## Coverage
/// Exit code 2; error on stderr; handles inaccessible path.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/04_query_storage_programmatically.md` — RWS-5
#[ test ]
fn rws_5_non_existent_storage_root_exits_non_zero()
{
  let out = common::clg_cmd()
    .arg( ".status" )
    .arg( "path::/tmp/does-not-exist-clg-rws5-test-xyz" )
    .output()
    .unwrap();

  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "RWS-5: .status on non-existent path must emit error on stderr; stdout: {}",
    stdout( &out )
  );
}
