//! Session-table tests, against real PTY-attached children.
//!
//! Each hosted session owns a real `cat` on a real pty. A stub would let the
//! table's key discipline look correct while hiding that a summary reads its pid
//! from a live process.
//!
//! ## Specification References
//!
//! - `docs/feature/003_session_table.md` — the table's contract
//! - `docs/invariant/002_conversation_id_key.md` — why the key is not a pid
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | tab01 | A fresh table | Empty, length zero |
//! | tab02 | Insert then look up | The session is found |
//! | tab03 | Look up an unknown id | `Err( UnknownSession )` naming the id |
//! | tab04 | Insert twice under one id | Replaced, not duplicated |
//! | tab05 | Remove | Returns the session; a second remove fails |
//! | tab06 | Summaries | Ordered by conversation id |
//! | tab07 | A summary's fields | Match the hosted session, pid from the live child |
//! | tab08 | Two sessions re-hosted under one id | The id, not the pid, is the handle |
//! | tab09 | Mutating through `get_mut` | The change is visible in the summary |

use std::path::{ Path, PathBuf };

use claude_daemon_core::{ Error, HostedSession, SessionTable };
use claude_pty_core::{ PtySession, SessionConfig };

/// Host a long-lived child under `session_id`.
///
/// `cat` blocks reading stdin, which is what an idle interactive session does —
/// so the child stays alive for the whole test rather than racing it.
fn hosted( session_id : &str, cwd : &Path ) -> HostedSession
{
  let config = SessionConfig::new( "cat" ).cwd( cwd );
  HostedSession
  {
    session_id : session_id.to_string(),
    cwd : cwd.to_path_buf(),
    pty : PtySession::spawn( &config ).expect( "spawn failed" ),
    busy : false,
  }
}

/// Shut every session in `table` down, so no child outlives the test.
fn drain( table : &mut SessionTable )
{
  for summary in table.summaries()
  {
    let mut session = table.remove( &summary.session_id ).expect( "session vanished from table" );
    session.pty.shutdown().expect( "shutdown failed" );
  }
}

/// tab01: a new table hosts nothing.
#[ test ]
fn tab01_new_table_is_empty()
{
  let table = SessionTable::new();

  assert!( table.is_empty(), "a new table is not empty" );
  assert_eq!( table.len(), 0 );
  assert!( table.summaries().is_empty(), "a new table produced summaries" );
}

/// tab02: an inserted session is reachable by its conversation id.
#[ test ]
fn tab02_insert_then_lookup()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut table = SessionTable::new();
  table.insert( hosted( "conv-1", dir.path() ) );

  assert_eq!( table.len(), 1 );
  assert!( !table.is_empty() );
  assert_eq!(
    table.get_mut( "conv-1" ).expect( "inserted session not found" ).session_id,
    "conv-1",
  );

  drain( &mut table );
}

/// tab03: an unknown id is an error that names the id.
///
/// The id is the client's handle; an error that omits it leaves the client
/// unable to tell which of its sessions the daemon lost.
#[ test ]
fn tab03_unknown_id_is_an_error()
{
  let mut table = SessionTable::new();

  match table.get_mut( "conv-missing" )
  {
    Err( Error::UnknownSession( id ) ) => assert_eq!( id, "conv-missing" ),
    Err( other ) => panic!( "expected UnknownSession, got {other:?}" ),
    Ok( found ) => panic!( "an empty table returned session {}", found.session_id ),
  }
}

/// tab04: inserting under an existing id replaces rather than duplicates.
///
/// This is the re-host path: Claude Code restarts a session with
/// `--fork-session`, the daemon hosts the replacement, and the table must end up
/// with one entry — not two, one of which points at a dead process.
#[ test ]
fn tab04_insert_under_existing_id_replaces()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut table = SessionTable::new();

  table.insert( hosted( "conv-1", dir.path() ) );
  let first_pid = table.get_mut( "conv-1" ).expect( "not found" ).pty.pid();

  table.insert( hosted( "conv-1", dir.path() ) );
  let second_pid = table.get_mut( "conv-1" ).expect( "not found" ).pty.pid();

  assert_eq!( table.len(), 1, "replacement left a duplicate entry" );
  assert_ne!( first_pid, second_pid, "the replacement is the same process — test premise broken" );

  drain( &mut table );
}

/// tab05: removal hands the session back once.
#[ test ]
fn tab05_remove_yields_the_session_once()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut table = SessionTable::new();
  table.insert( hosted( "conv-1", dir.path() ) );

  let mut removed = table.remove( "conv-1" ).expect( "remove failed" );
  assert_eq!( removed.session_id, "conv-1" );
  assert!( table.is_empty(), "table still holds the removed session" );

  match table.remove( "conv-1" )
  {
    Err( Error::UnknownSession( id ) ) => assert_eq!( id, "conv-1" ),
    other => panic!( "expected UnknownSession on a second remove, got {other:?}" ),
  }

  removed.pty.shutdown().expect( "shutdown failed" );
}

/// tab06: summaries are ordered by conversation id.
///
/// The backing map has no order, so without the sort a `list_sessions` response
/// would reshuffle between calls with nothing having changed.
#[ test ]
fn tab06_summaries_are_ordered_by_session_id()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut table = SessionTable::new();
  for id in [ "conv-c", "conv-a", "conv-b" ]
  {
    table.insert( hosted( id, dir.path() ) );
  }

  let ids : Vec< String > = table.summaries().into_iter().map( | s | s.session_id ).collect();
  assert_eq!( ids, vec![ "conv-a", "conv-b", "conv-c" ], "summaries are not sorted" );

  drain( &mut table );
}

/// tab07: a summary reports what the session actually is.
#[ test ]
fn tab07_summary_matches_the_hosted_session()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let cwd : PathBuf = dir.path().to_path_buf();
  let mut table = SessionTable::new();
  table.insert( hosted( "conv-1", &cwd ) );

  let live_pid = table.get_mut( "conv-1" ).expect( "not found" ).pty.pid();
  let summaries = table.summaries();
  let summary = summaries.first().expect( "no summary produced" );

  assert_eq!( summary.session_id, "conv-1" );
  assert_eq!( summary.pid, live_pid );
  assert_eq!( summary.cwd, cwd );
  assert!( !summary.busy );
  assert!(
    Path::new( &format!( "/proc/{}", summary.pid ) ).exists(),
    "summary reports pid {} but no such process exists",
    summary.pid,
  );

  drain( &mut table );
}

/// tab08: the conversation id is the stable handle across a re-host.
///
/// The pid changes; the key does not. A PID-keyed table would have detached here
/// — silently, since nothing errors, the client simply stops being able to reach
/// a session that is running fine.
#[ test ]
fn tab08_conversation_id_survives_a_pid_change()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut table = SessionTable::new();
  table.insert( hosted( "conv-stable", dir.path() ) );

  let before = table.get_mut( "conv-stable" ).expect( "not found" ).pty.pid();

  // Re-host: the old process goes, a new one takes over the same conversation.
  let mut old = table.remove( "conv-stable" ).expect( "remove failed" );
  old.pty.shutdown().expect( "shutdown failed" );
  table.insert( hosted( "conv-stable", dir.path() ) );

  let after = table.get_mut( "conv-stable" ).expect( "the id did not survive the re-host" ).pty.pid();

  assert_ne!( before, after, "the re-hosted session reused the pid — test premise broken" );
  assert_eq!( table.len(), 1, "re-hosting changed the table's size" );

  drain( &mut table );
}

/// tab09: `get_mut` is a real mutable borrow, not a copy.
#[ test ]
fn tab09_mutation_through_get_mut_is_visible()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut table = SessionTable::new();
  table.insert( hosted( "conv-1", dir.path() ) );

  table.get_mut( "conv-1" ).expect( "not found" ).busy = true;

  let summaries = table.summaries();
  assert!( summaries.first().expect( "no summary produced" ).busy, "the busy flag did not stick" );

  drain( &mut table );
}
