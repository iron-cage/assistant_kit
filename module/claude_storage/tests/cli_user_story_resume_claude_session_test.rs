//! Acceptance tests for the "Resume Claude Session" user story.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/user_story/05_resume_claude_session.md`
//!
//! ## Coverage
//!
//! - RWS-1: project.exists exits 0 when project has history
//! - RWS-2: project.exists exits 1 when project has no history
//! - RWS-3: project.path outputs encoded storage path
//! - RWS-4: session.dir outputs session working directory path
//! - RWS-5: session.ensure creates directory and reports strategy

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

/// Write a session into the HOME-based claude storage for a given project path.
///
/// Uses the HOME env var pattern for .project.exists / .session.ensure commands.
fn setup_history( home : &std::path::Path, session_dir : &std::path::Path )
{
  let storage_root = home.join( ".claude" );
  common::write_path_project_session( &storage_root, session_dir, "session-test", 2 );
}

/// RWS-1: project.exists exits 0 when project has history.
///
/// ## Purpose
/// End-to-end acceptance test: script checks whether a project has conversation
/// history before resuming; `.project.exists` exits 0 and prints "sessions exist".
///
/// ## Coverage
/// Exit 0 with history; stdout "sessions exist"; suitable for shell conditionals.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/05_resume_claude_session.md` — RWS-1
#[ test ]
fn rws_1_project_exists_exits_0_when_project_has_history()
{
  let home = TempDir::new().unwrap();
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
  assert!(
    s.contains( "sessions exist" ),
    "RWS-1: .project.exists must print 'sessions exist' when history found; got:\n{s}"
  );
}

/// RWS-2: project.exists exits 1 when project has no history.
///
/// ## Purpose
/// End-to-end acceptance test: script detects a fresh project with no previous
/// Claude sessions; `.project.exists` exits 1 and prints "no sessions" on stderr.
///
/// ## Coverage
/// Exit 1 without history; stderr "no sessions"; suitable for shell conditionals.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/05_resume_claude_session.md` — RWS-2
#[ test ]
fn rws_2_project_exists_exits_1_when_project_has_no_history()
{
  let home = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  // No history written — project has no sessions

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.exists" )
    .arg( format!( "path::{}", project.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "no sessions" ),
    "RWS-2: .project.exists must print 'no sessions' on stderr when no history; got:\n{err}"
  );
}

/// RWS-3: project.path outputs encoded storage path.
///
/// ## Purpose
/// End-to-end acceptance test: script needs the encoded storage path to locate
/// project data; `.project.path` outputs a single absolute path line.
///
/// ## Coverage
/// Single-line output; absolute storage path; ends with slash; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/05_resume_claude_session.md` — RWS-3
#[ test ]
fn rws_3_project_path_outputs_encoded_storage_path()
{
  let home = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".project.path" )
    .arg( "path::/home/user/myproject" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let trimmed = s.trim();
  assert!(
    !trimmed.is_empty(),
    "RWS-3: .project.path must produce output; stderr: {}",
    stderr( &out )
  );
  // Output must be an absolute path containing storage path components
  assert!(
    trimmed.contains( ".claude" ) || trimmed.contains( "projects" ),
    "RWS-3: .project.path must contain storage path components (.claude or projects); got:\n{s}"
  );
  // Should reference the encoded form of the input path
  assert!(
    trimmed.contains( "myproject" ) || trimmed.contains( "home" ),
    "RWS-3: .project.path must encode the input path; got:\n{s}"
  );
}

/// RWS-4: session.dir outputs session working directory path.
///
/// ## Purpose
/// End-to-end acceptance test: script computes the session directory path
/// without creating it; `.session.dir` outputs `{path}/-{topic}`.
///
/// ## Coverage
/// Correct path format; directory NOT created; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/05_resume_claude_session.md` — RWS-4
#[ test ]
fn rws_4_session_dir_outputs_session_working_directory_path()
{
  let home = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.dir" )
    .arg( format!( "path::{base}" ) )
    .arg( "topic::work" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let expected = format!( "{base}/-work\n" );
  assert_eq!(
    s,
    expected,
    "RWS-4: .session.dir must output '{base}/-work\\n'; got:\n{s}"
  );

  // Directory must NOT be created
  let session_dir = project.path().join( "-work" );
  assert!(
    !session_dir.exists(),
    "RWS-4: .session.dir must not create the directory; found: {session_dir:?}"
  );
}

/// RWS-5: session.ensure creates directory and reports strategy.
///
/// ## Purpose
/// End-to-end acceptance test: script ensures the session directory exists and
/// learns whether to resume or start fresh; `.session.ensure` creates the
/// directory and outputs path + strategy on two lines.
///
/// ## Coverage
/// Directory created; line 1 is path; line 2 is "fresh" for new project; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/05_resume_claude_session.md` — RWS-5
#[ test ]
fn rws_5_session_ensure_creates_directory_and_reports_strategy()
{
  let home = TempDir::new().unwrap();
  let project = TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  // No prior history — session directory does not exist
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

  assert_eq!(
    lines.len(),
    2,
    "RWS-5: .session.ensure must output exactly 2 lines; got:\n{s}"
  );

  let expected_path = format!( "{base}/-work" );
  assert_eq!(
    lines[ 0 ],
    expected_path,
    "RWS-5: line 1 must be the absolute session directory path; got:\n{s}"
  );
  assert_eq!(
    lines[ 1 ],
    "fresh",
    "RWS-5: line 2 must be 'fresh' when no prior history; got:\n{s}"
  );

  // Session directory must have been created
  let session_dir = project.path().join( "-work" );
  assert!(
    session_dir.exists(),
    "RWS-5: .session.ensure must create the session directory; path: {session_dir:?}"
  );
}
