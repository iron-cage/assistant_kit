//! Integration tests for the `.tail` command.
//!
//! ## Source
//!
//! - Command spec: `tests/docs/cli/command/12_tail.md`
//! - Param spec: `tests/docs/cli/param/25_tail.md`
//!
//! ## Coverage
//!
//! - INT-1: No args prints last 4 entries of `default_topic` session (also covers EC-1)
//! - INT-2: `tail::N` controls entry count (also covers EC-2)
//! - INT-3: `tail::0` prints all entries (also covers EC-3)
//! - INT-4: `topic::` resolves a non-default session
//! - INT-5: `path::` resolves a different directory's project
//! - INT-6: Fewer entries than requested prints all available (also covers EC-6)
//! - INT-7: Exit code 2 when cwd has no project
//! - INT-8: Negative `tail::` is rejected with exit code 1 (also covers EC-4)
//! - EC-5: Empty value rejected
//! - EC-7: Non-integer value rejected
// BUG-002 — real assertions replacing the "didn't hang" cheating tests

mod common;

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

/// INT-1: No args prints last 4 entries of `default_topic` session.
///
/// ## Purpose
/// Validates the zero-parameter default: current directory's project,
/// `-default_topic` session, last 4 entries.
///
/// ## Coverage
/// Exit 0; last 4 of 6 entries shown (entries 2-5), oldest-first; entries 0-1 absent.
/// Also covers EC-1 (`tests/docs/cli/param/25_tail.md`) — identical scenario.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-1
/// `tests/docs/cli/param/25_tail.md` — EC-1
#[ test ]
fn int_1_no_args_shows_last_4_of_default_topic()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 2..6
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
  for i in 0..2
  {
    assert!( !text.contains( &format!( "entry {i}" ) ), "did not expect entry {i} in output: {text}" );
  }
}

/// INT-2: `tail::N` controls entry count.
///
/// ## Purpose
/// Validates that `tail::2` shows exactly the last 2 entries.
///
/// ## Coverage
/// Exit 0; last 2 of 6 entries shown (entries 4-5); entries 0-3 absent.
/// Also covers EC-2 (`tests/docs/cli/param/25_tail.md`) — identical scenario.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-2
/// `tests/docs/cli/param/25_tail.md` — EC-2
#[ test ]
fn int_2_tail_n_controls_entry_count()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "tail::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 4..6
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
  for i in 0..4
  {
    assert!( !text.contains( &format!( "entry {i}" ) ), "did not expect entry {i} in output: {text}" );
  }
}

/// INT-3: `tail::0` prints all entries.
///
/// ## Purpose
/// Validates that `tail::0` disables the cap and shows every entry.
///
/// ## Coverage
/// Exit 0; all 6 entries shown, oldest-first.
/// Also covers EC-3 (`tests/docs/cli/param/25_tail.md`) — identical scenario.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-3
/// `tests/docs/cli/param/25_tail.md` — EC-3
#[ test ]
fn int_3_tail_zero_prints_all_entries()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "tail::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 0..6
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
}

/// INT-4: `topic::` resolves a non-default session.
///
/// ## Purpose
/// Validates that `topic::work` reads the `-work` session instead of `-default_topic`.
///
/// ## Coverage
/// Exit 0; `-work` session's distinct content shown; `-default_topic` content absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-4
#[ test ]
fn int_4_topic_resolves_non_default_session()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  let encoded = claude_storage_core::encode_path( cwd.path() ).unwrap();
  common::write_test_session_with_last_message( root.path(), &encoded, "-default_topic", 1, "DEFAULTTOPICMARKER" );
  common::write_test_session_with_last_message( root.path(), &encoded, "-work", 1, "WORKTOPICMARKER" );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "topic::work" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "WORKTOPICMARKER" ), "expected -work session marker in output: {text}" );
  assert!( !text.contains( "DEFAULTTOPICMARKER" ), "did not expect -default_topic marker in output: {text}" );
}

/// INT-5: `path::` resolves a different directory's project.
///
/// ## Purpose
/// Validates that `path::DIR` loads DIR's project instead of the process's cwd.
///
/// ## Coverage
/// Exit 0; last 4 of 6 entries from the `path::`-specified project's `-default_topic`
/// session are shown, even though the process cwd is a different, unrelated directory.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-5
#[ test ]
fn int_5_path_resolves_different_directory_project()
{
  let root          = tempfile::TempDir::new().unwrap();
  let alpha_dir     = tempfile::TempDir::new().unwrap();
  let unrelated_cwd = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), alpha_dir.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( unrelated_cwd.path() )
    .arg( ".tail" )
    .arg( format!( "path::{}", alpha_dir.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 2..6
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
}

/// INT-6: Fewer entries than requested prints all available.
///
/// ## Purpose
/// Validates that requesting more entries than exist shows all available, no error.
///
/// ## Coverage
/// Exit 0; all 3 entries shown when `tail::10` is requested against a 3-entry session.
/// Also covers EC-6 (`tests/docs/cli/param/25_tail.md`) — same boundary condition.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-6
/// `tests/docs/cli/param/25_tail.md` — EC-6
#[ test ]
fn int_6_fewer_entries_than_requested_shows_all()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 3 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "tail::10" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  for i in 0..3
  {
    assert!( text.contains( &format!( "entry {i}" ) ), "expected entry {i} in output: {text}" );
  }
}

/// INT-7: Exit code 2 when cwd has no project.
///
/// ## Purpose
/// Validates the "not found = usage error" convention: running from a directory
/// with no matching storage project exits 2, not the standard error exit 1.
///
/// ## Coverage
/// Exit 2; stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-7
#[ test ]
fn int_7_exit_2_when_no_project_for_cwd()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  // No fixture written — CLAUDE_STORAGE_ROOT has no project for `cwd`.

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .output()
    .unwrap();

  assert_exit( &out, 2 );
  assert!( !stderr( &out ).is_empty(), "INT-7: expected non-empty stderr for missing project" );
}

/// INT-8: Negative `tail::` is rejected with exit code 1.
///
/// ## Purpose
/// Validates the exact stderr wording and exit code for negative tail counts.
///
/// ## Coverage
/// Exit 1; stderr exactly `"tail must be non-negative"`. Rejection happens before
/// entries (or the project) are loaded — a valid project/session fixture is present
/// to prove the rejection is not a side effect of a missing project.
/// Also covers EC-4 (`tests/docs/cli/param/25_tail.md`) — same scenario, stricter assertion.
///
/// ## Related Requirements
/// `tests/docs/cli/command/12_tail.md` — INT-8
/// `tests/docs/cli/param/25_tail.md` — EC-4
#[ test ]
fn int_8_negative_tail_rejected_exit_1()
{
  let root = tempfile::TempDir::new().unwrap();
  let cwd  = tempfile::TempDir::new().unwrap();

  common::write_path_project_session( root.path(), cwd.path(), "-default_topic", 6 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( cwd.path() )
    .arg( ".tail" )
    .arg( "tail::-1" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert_eq!( stderr( &out ).trim_end(), "tail must be non-negative" );
}

/// EC-5: Empty `tail::` value is rejected.
///
/// ## Purpose
/// Validates that an empty value for the Integer-typed `tail` parameter is
/// rejected by the framework's own type parsing, before the routine ever runs.
///
/// ## Coverage
/// Exit 1; stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/param/25_tail.md` — EC-5
#[ test ]
fn ec_5_empty_tail_value_rejected()
{
  let out = common::clg_cmd()
    .arg( ".tail" )
    .arg( "tail::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "EC-5: expected non-empty stderr for empty tail value" );
}

/// EC-7: Non-integer `tail::` value is rejected.
///
/// ## Purpose
/// Validates that a non-integer value for the Integer-typed `tail` parameter is
/// rejected by the framework's own type parsing, before the routine ever runs.
///
/// ## Coverage
/// Exit 1; stderr non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/param/25_tail.md` — EC-7
#[ test ]
fn ec_7_non_integer_tail_value_rejected()
{
  let out = common::clg_cmd()
    .arg( ".tail" )
    .arg( "tail::four" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "EC-7: expected non-empty stderr for non-integer tail value" );
}
