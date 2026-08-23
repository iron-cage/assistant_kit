//! Unit tests for `src/cli/liveness.rs` — attachment-state detection.
//!
//! Relocated out of a `#[ cfg( test ) ]` module in the source file: every test
//! in this crate lives under `tests/`. The units under test are reached through
//! `claude_storage::cli::liveness`, which is `#[ doc( hidden ) ] pub` for
//! exactly this purpose (see the note in `src/cli/mod.rs`).
//!
//! Each test builds a real `/proc`-shaped directory and a real history file in
//! a `TempDir`, so `LivenessMap::probe` performs the same `read_dir`,
//! `read_to_string`, and `read_link` calls it makes against the kernel's own
//! filesystem — no mocking, no stand-ins.

use claude_storage::cli::liveness::{ Liveness, LivenessMap, display_key };
use std::path::Path;
use std::time::SystemTime;
use core::time::Duration;

/// Build a real `/proc`-shaped directory: one numeric subdirectory per
/// process, each holding a `comm` file and a `cwd` symlink.
///
/// This is the genuine article rather than a stand-in — `read_attached`
/// performs the same `read_dir`/`read_to_string`/`read_link` calls it makes
/// against the kernel's own filesystem.
fn fake_proc( root : &Path, processes : &[ ( &str, &str, &Path ) ] )
{
  for ( pid, comm, cwd ) in processes
  {
    let dir = root.join( pid );
    std::fs::create_dir_all( &dir ).unwrap();
    std::fs::write( dir.join( "comm" ), format!( "{comm}\n" ) ).unwrap();
    std::os::unix::fs::symlink( cwd, dir.join( "cwd" ) ).unwrap();
  }
}

fn history_line( project : &str, session_id : &str ) -> String
{
  format!( r#"{{"display":"hi","project":"{project}","sessionId":"{session_id}","timestamp":"1"}}"# )
}

fn now() -> SystemTime { SystemTime::now() }
fn ago( secs : u64 ) -> SystemTime { SystemTime::now() - Duration::from_secs( secs ) }
fn ahead( secs : u64 ) -> SystemTime { SystemTime::now() + Duration::from_secs( secs ) }

/// An empty probe reports nothing rather than reporting everything dead.
#[ test ]
fn test_absent_process_table_reports_nothing()
{
  let tmp = tempfile::tempdir().unwrap();
  let map = LivenessMap::probe( &tmp.path().join( "no-such-proc" ), None );

  assert!( !map.any_attached(), "an unreadable process table must not claim knowledge" );
  assert_eq!( map.project_state( "~/anything", now() ), None );
}

/// A `claude` process' cwd marks its project attached; unrelated processes do not.
#[ test ]
fn test_attached_project_detected_from_process_cwd()
{
  let tmp = tempfile::tempdir().unwrap();
  let proc_dir = tmp.path().join( "proc" );
  let live = tmp.path().join( "live_project" );
  let other = tmp.path().join( "other_project" );
  std::fs::create_dir_all( &live ).unwrap();
  std::fs::create_dir_all( &other ).unwrap();
  fake_proc( &proc_dir, &[ ( "101", "claude", &live ), ( "102", "bash", &other ) ] );

  let map = LivenessMap::probe( &proc_dir, None );
  let live_key = display_key( &live ).unwrap();
  let other_key = display_key( &other ).unwrap();

  assert!( map.any_attached() );
  assert!( map.project_state( &live_key, now() ).is_some(), "cwd of a claude process is live" );
  assert_eq!( map.project_state( &other_key, now() ), None, "a non-claude process must not mark a project" );
}

/// Recency splits an attached project into working and waiting — and, crucially,
/// a long-idle attached project stays live rather than decaying to nothing.
#[ test ]
fn test_attached_project_splits_working_from_waiting()
{
  let tmp = tempfile::tempdir().unwrap();
  let proc_dir = tmp.path().join( "proc" );
  let project = tmp.path().join( "project" );
  std::fs::create_dir_all( &project ).unwrap();
  fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

  let map = LivenessMap::probe( &proc_dir, None );
  let key = display_key( &project ).unwrap();

  assert_eq!( map.project_state( &key, now() ), Some( Liveness::Working ) );
  assert_eq!( map.project_state( &key, ago( 3_600 ) ), Some( Liveness::Waiting ),
    "an hour of silence with a process attached is waiting, never absent" );
}

/// An mtime ahead of the local clock is the freshest write there is, not the
/// oldest.
///
/// `duration_since` reports a future timestamp as `Err`, and the obvious
/// reading of that error — "no measurable age, so not fresh" — inverts the
/// answer: clock skew against an NFS or container host would make the one
/// session being actively written the only one reported quiet.
#[ test ]
fn test_future_mtime_is_working_not_waiting()
{
  let tmp = tempfile::tempdir().unwrap();
  let proc_dir = tmp.path().join( "proc" );
  let project = tmp.path().join( "project" );
  std::fs::create_dir_all( &project ).unwrap();
  fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

  let map = LivenessMap::probe( &proc_dir, None );
  let key = display_key( &project ).unwrap();

  assert_eq!( map.project_state( &key, ahead( 5 ) ), Some( Liveness::Working ),
    "a few seconds of skew must not read as an idle terminal" );
  assert_eq!( map.project_state( &key, ahead( 86_400 ) ), Some( Liveness::Working ),
    "and neither must a wholly wrong clock — the direction of the error is what matters" );
}

/// History pins the driven session even when it is not the newest by mtime —
/// the case a recency heuristic gets backwards.
#[ test ]
fn test_history_pins_driven_session_over_newer_sibling()
{
  let tmp = tempfile::tempdir().unwrap();
  let proc_dir = tmp.path().join( "proc" );
  let project = tmp.path().join( "project" );
  std::fs::create_dir_all( &project ).unwrap();
  fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

  let history = tmp.path().join( "history.jsonl" );
  std::fs::write( &history, format!( "{}\n", history_line( project.to_str().unwrap(), "driven-id" ) ) ).unwrap();

  let map = LivenessMap::probe( &proc_dir, Some( &history ) );
  let key = display_key( &project ).unwrap();

  // rank 1 — an older session by mtime, yet the one actually driven.
  assert_eq!( map.session_state( &key, "driven-id", 1, ago( 3_000 ) ), Some( Liveness::Waiting ) );
  // rank 0 — the newest session, but history says it is not the live one.
  assert_eq!( map.session_state( &key, "newer-id", 0, now() ), None );
}

/// With no history record (a headless `--print` session), the newest session
/// by mtime stands in, bounded by the attached process count.
#[ test ]
fn test_missing_history_falls_back_to_mtime_rank()
{
  let tmp = tempfile::tempdir().unwrap();
  let proc_dir = tmp.path().join( "proc" );
  let project = tmp.path().join( "project" );
  std::fs::create_dir_all( &project ).unwrap();
  fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

  let map = LivenessMap::probe( &proc_dir, None );
  let key = display_key( &project ).unwrap();

  assert_eq!( map.session_state( &key, "any-id", 0, now() ), Some( Liveness::Working ) );
  assert_eq!( map.session_state( &key, "any-id", 1, now() ), None,
    "only as many sessions as there are processes may be called live" );
}

/// Two processes in one project mark two driven sessions, not one.
#[ test ]
fn test_two_processes_drive_two_sessions()
{
  let tmp = tempfile::tempdir().unwrap();
  let proc_dir = tmp.path().join( "proc" );
  let project = tmp.path().join( "project" );
  std::fs::create_dir_all( &project ).unwrap();
  fake_proc( &proc_dir, &[ ( "101", "claude", &project ), ( "102", "claude", &project ) ] );

  let path = project.to_str().unwrap();
  let history = tmp.path().join( "history.jsonl" );
  std::fs::write(
    &history,
    format!( "{}\n{}\n{}\n",
      history_line( path, "oldest-id" ),
      history_line( path, "second-id" ),
      history_line( path, "newest-id" ) ),
  ).unwrap();

  let map = LivenessMap::probe( &proc_dir, Some( &history ) );
  let key = display_key( &project ).unwrap();

  assert!( map.session_state( &key, "newest-id", 0, now() ).is_some() );
  assert!( map.session_state( &key, "second-id", 1, now() ).is_some() );
  assert_eq!( map.session_state( &key, "oldest-id", 2, now() ), None,
    "history is read newest-first and capped at the attached process count" );
}

/// History for a project with no attached process is ignored entirely.
#[ test ]
fn test_history_without_attached_process_is_ignored()
{
  let tmp = tempfile::tempdir().unwrap();
  let proc_dir = tmp.path().join( "proc" );
  let attached = tmp.path().join( "attached" );
  let exited = tmp.path().join( "exited" );
  std::fs::create_dir_all( &attached ).unwrap();
  std::fs::create_dir_all( &exited ).unwrap();
  fake_proc( &proc_dir, &[ ( "101", "claude", &attached ) ] );

  let history = tmp.path().join( "history.jsonl" );
  std::fs::write( &history, format!( "{}\n", history_line( exited.to_str().unwrap(), "ghost-id" ) ) ).unwrap();

  let map = LivenessMap::probe( &proc_dir, Some( &history ) );
  let exited_key = display_key( &exited ).unwrap();

  assert_eq!( map.session_state( &exited_key, "ghost-id", 0, now() ), None );
}

/// A malformed history line is skipped without discarding the records around it.
#[ test ]
fn test_malformed_history_line_is_skipped()
{
  let tmp = tempfile::tempdir().unwrap();
  let proc_dir = tmp.path().join( "proc" );
  let project = tmp.path().join( "project" );
  std::fs::create_dir_all( &project ).unwrap();
  fake_proc( &proc_dir, &[ ( "101", "claude", &project ) ] );

  let history = tmp.path().join( "history.jsonl" );
  std::fs::write(
    &history,
    format!( "not json at all\n{{\"partial\":true}}\n{}\n", history_line( project.to_str().unwrap(), "good-id" ) ),
  ).unwrap();

  let map = LivenessMap::probe( &proc_dir, Some( &history ) );
  let key = display_key( &project ).unwrap();

  assert!( map.session_state( &key, "good-id", 5, now() ).is_some(),
    "a valid record must survive malformed neighbours" );
}

/// Labels stay in step with the width the column reserves for them.
#[ test ]
fn test_labels_fit_the_reserved_column_width()
{
  let width = Liveness::column_width();
  for state in [ Liveness::Working, Liveness::Waiting ]
  {
    assert!( state.label().chars().count() <= width, "{} overflows reserved width", state.label() );
  }
}
