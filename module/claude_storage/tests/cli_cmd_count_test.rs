//! Integration tests for the `clg .count` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/04_count.md`
//!
//! ## Coverage
//!
//! - INT-1: Default count returns project count
//! - INT-2: `target::sessions` with `project::` returns session count
//! - INT-3: `target::entries` with `project::` and `session::` returns entry count
//! - INT-4: Output is a single integer line
//! - INT-5: Exit code 0 on success
//! - INT-6: Exit code 1 on invalid target value
//! - INT-7: `target::sessions` with no `project::` counts all sessions
//! - INT-8: `target::entries` with no `session::` counts all entries in project

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

/// INT-1: Default count returns project count.
///
/// ## Purpose
/// Verify that `.count` with no parameters returns the total number of
/// projects as a bare integer.
///
/// ## Coverage
/// Output is integer 3; exit 0.
///
/// ## Validation Strategy
/// Write 3 projects into temp root. Run `clg .count`.
/// Assert output parses as integer and equals 3.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-1
#[ test ]
fn int_1_default_count_returns_project_count()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "cnt1-a" );
  let p2 = root.path().join( "cnt1-b" );
  let p3 = root.path().join( "cnt1-c" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s001", 2 );
  common::write_path_project_session( root.path(), &p3, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-1: .count output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 3, "INT-1: expected project count 3; got {n}" );
}

/// INT-2: `target::sessions` with `project::` returns session count.
///
/// ## Purpose
/// Verify that `target::sessions ``project::alph``a` returns the count of sessions
/// in the specified project.
///
/// ## Coverage
/// Output is integer 4; exit 0.
///
/// ## Validation Strategy
/// Write project alpha (path contains "alpha") with 4 sessions. Run
/// `clg .count ``target::sessions`` ``project::alph``a`. Assert output is 4.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-2
#[ test ]
fn int_2_target_sessions_with_project_returns_session_count()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "alpha" );
  let enc   = common::write_path_project_session( root.path(), &alpha, "s001", 2 );
  common::write_path_project_session( root.path(), &alpha, "s002", 2 );
  common::write_path_project_session( root.path(), &alpha, "s003", 2 );
  common::write_path_project_session( root.path(), &alpha, "s004", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::sessions" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-2: .count output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 4, "INT-2: expected 4 sessions in project alpha; got {n}" );
}

/// INT-3: `target::entries` with `project::` and `session::` returns entry count.
///
/// ## Purpose
/// Verify that `target::entries ``project::alpha`` ``session::s``1` returns the
/// entry count for that specific session.
///
/// ## Coverage
/// Output is integer 7; exit 0.
///
/// ## Validation Strategy
/// Write project alpha with session s1 (7 entries). Run `clg .count
/// ``target::entries`` ``project::alpha`` ``session::s``1`. Assert output is 7.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-3
#[ test ]
fn int_3_target_entries_with_project_and_session_returns_entry_count()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "alpha" );
  let enc   = common::write_path_project_session( root.path(), &alpha, "s1", 7 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::entries" )
    .arg( format!( "project::{enc}" ) )
    .arg( "session::s1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-3: .count output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 7, "INT-3: expected 7 entries in session s1; got {n}" );
}

/// INT-4: Output is a single integer line.
///
/// ## Purpose
/// Verify that `.count` output is exactly `{n}\n` — one integer followed
/// by a newline, with nothing else.
///
/// ## Coverage
/// Trimmed output is exactly "2"; no extra text; exit 0.
///
/// ## Validation Strategy
/// Write 2 projects. Run `clg .count`. Assert trimmed stdout == "2" exactly.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-4
#[ test ]
fn int_4_output_is_single_integer_line()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "cnt4-x" );
  let p2 = root.path().join( "cnt4-y" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let raw = stdout( &out );
  let trimmed = raw.trim();
  assert_eq!(
    trimmed,
    "2",
    "INT-4: .count output must be exactly '2\\n'; got: {raw:?}"
  );
}

/// INT-5: Exit code 0 on success.
///
/// ## Purpose
/// Verify that `.count` exits with code 0 on a valid fixture.
///
/// ## Coverage
/// Integer output on stdout; exit 0.
///
/// ## Validation Strategy
/// Write 1 project. Run `clg .count`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-5
#[ test ]
fn int_5_exit_code_0_on_success()
{
  let root = TempDir::new().unwrap();
  let p = root.path().join( "cnt5-proj" );
  common::write_path_project_session( root.path(), &p, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out ).trim().to_string();
  assert!(
    s.parse::< usize >().is_ok(),
    "INT-5: .count must produce integer on stdout; got: '{s}'"
  );
}

/// INT-6: Exit code 1 on invalid target value.
///
/// ## Purpose
/// Verify that `target::widgets` (invalid) causes `.count` to fail with
/// exit code 1 and an error on stderr.
///
/// ## Coverage
/// Error on stderr; no count on stdout; exit 1.
///
/// ## Validation Strategy
/// Run `clg .count ``target::widget``s`. Assert exit 1 and stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-6
#[ test ]
fn int_6_exit_code_1_on_invalid_target_value()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::widgets" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-6: invalid target must produce error on stderr; got silence"
  );
}

/// INT-7: `target::sessions` with no `project::` counts all sessions.
///
/// ## Purpose
/// Verify that `target::sessions` without a `project::` restriction sums
/// sessions across all projects.
///
/// ## Coverage
/// Output is integer 6 (2 + 3 + 1); exit 0.
///
/// ## Validation Strategy
/// Write 3 projects with 2, 3, and 1 sessions. Run `clg .count ``target::session``s`.
/// Assert output is 6.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-7
#[ test ]
fn int_7_target_sessions_no_project_counts_all_sessions()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "cnt7-a" );
  let p2 = root.path().join( "cnt7-b" );
  let p3 = root.path().join( "cnt7-c" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p1, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s003", 2 );
  common::write_path_project_session( root.path(), &p3, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-7: .count target::sessions output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 6, "INT-7: expected 6 total sessions; got {n}" );
}

/// INT-8: `target::entries` with no `session::` counts all entries in project.
///
/// ## Purpose
/// Verify that `target::entries ``project::alph``a` without a `session::` filter
/// sums entries across all sessions in the project.
///
/// ## Coverage
/// Output is integer 8 (5 + 3); exit 0.
///
/// ## Validation Strategy
/// Write project alpha with 2 sessions: s1 (5 entries), s2 (3 entries).
/// Run `clg .count ``target::entries`` ``project::alph``a`. Assert output is 8.
///
/// ## Related Requirements
/// `tests/docs/cli/command/04_count.md` — INT-8
#[ test ]
fn int_8_target_entries_no_session_counts_all_entries_in_project()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "alpha" );
  let enc   = common::write_path_project_session( root.path(), &alpha, "s1", 5 );
  common::write_path_project_session( root.path(), &alpha, "s2", 3 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::entries" )
    .arg( format!( "project::{enc}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out ).trim().to_string();
  let n : usize = s.parse().unwrap_or_else( |_| panic!(
    "INT-8: .count output must be a bare integer; got: '{s}'"
  ) );
  assert_eq!( n, 8, "INT-8: expected 8 total entries across s1+s2; got {n}" );
}
