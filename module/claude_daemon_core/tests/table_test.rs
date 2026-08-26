//! Session-table tests, against real PTY-attached children.
//!
//! Each hosted session owns a real `cat` on a real pty. A stub would let the
//! table's key discipline look correct while hiding the two things that actually
//! bite: a summary reads its pid from a live process, and a session's teardown
//! has to unblock a pump thread holding a master descriptor before it can reap.
//!
//! ## Specification References
//!
//! - `docs/feature/003_session_table.md` — the table's contract
//! - `docs/feature/004_session_output.md` — output buffering and teardown order
//! - `docs/invariant/002_conversation_id_key.md` — why the key is not a pid
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | tab01 | A fresh table | Empty, length zero |
//! | tab02 | Insert then look up | The session is found |
//! | tab03 | Look up an unknown id | `Err( UnknownSession )` naming the id |
//! | tab04 | Insert twice under one id | Replaced, not duplicated; the old one is handed back |
//! | tab05 | Remove | Returns the session; a second remove fails |
//! | tab06 | Summaries | Ordered by conversation id |
//! | tab07 | A summary's fields | Match the hosted session, pid from the live child |
//! | tab08 | Two sessions re-hosted under one id | The id, not the pid, is the handle |
//! | tab09 | Mutating through `get_mut` | The change is visible in the summary |
//! | tab10 | Write to a session, then read it | The output comes back through the cursor |
//! | tab11 | Shut down a child blocked on stdin | Returns promptly; the child is reaped |
//! | tab12 | Read after shutdown | Reports `ended` |
//! | tab13 | Repeated reads without writing | Second read is empty, cursor unchanged |

use core::time::Duration;
use std::path::{ Path, PathBuf };
use std::time::Instant;

use claude_daemon_core::{ Error, HostedSession, SessionTable };
use claude_pty_core::{ PtySession, SessionConfig };

/// Longest a test waits for a child's output, or for a shutdown to return.
const TEST_TIMEOUT : Duration = Duration::from_secs( 10 );

/// Host a long-lived child under `session_id`.
///
/// `cat` blocks reading stdin, which is what an idle interactive session does —
/// so the child stays alive for the whole test rather than racing it.
fn hosted( session_id : &str, cwd : &Path ) -> HostedSession
{
  let config = SessionConfig::new( "cat" ).cwd( cwd );
  let pty = PtySession::spawn( &config ).expect( "spawn failed" );
  HostedSession::adopt( session_id, cwd, pty ).expect( "adopt failed" )
}

/// Insert, shutting down whatever session the insert displaced.
///
/// `SessionTable::insert` hands the replaced session back rather than dropping
/// it, precisely so this cannot be skipped: a dropped session leaves a live child
/// and a pump thread with no owner.
fn insert( table : &mut SessionTable, session : HostedSession )
{
  if let Some( mut replaced ) = table.insert( session )
  {
    replaced.shutdown().expect( "shutdown of the replaced session failed" );
  }
}

/// Shut every session in `table` down, so no child outlives the test.
fn drain( table : &mut SessionTable )
{
  for id in table.session_ids()
  {
    let mut session = table.remove( &id ).expect( "session vanished from table" );
    session.shutdown().expect( "shutdown failed" );
  }
}

/// Poll `session` for output until `needle` appears or [`TEST_TIMEOUT`] elapses.
///
/// Returns the accumulated text. Polling rather than blocking is the client-side
/// shape the cursor protocol is built for.
fn read_until( session : &HostedSession, needle : &str ) -> String
{
  let deadline = Instant::now() + TEST_TIMEOUT;
  let mut cursor = 0_u64;
  let mut acc = String::new();

  loop
  {
    let slice = session.read_from( cursor );
    cursor = slice.cursor;
    acc.push_str( &slice.text );
    if acc.contains( needle )
    {
      return acc;
    }
    assert!( !slice.ended, "session output ended without producing {needle:?}; got {acc:?}" );
    assert!(
      Instant::now() < deadline,
      "timed out waiting for {needle:?}; got {acc:?}",
    );
    std::thread::sleep( Duration::from_millis( 5 ) );
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
  insert( &mut table, hosted( "conv-1", dir.path() ) );

  assert_eq!( table.len(), 1 );
  assert!( !table.is_empty() );
  assert_eq!(
    table.get( "conv-1" ).expect( "inserted session not found" ).session_id(),
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
    Ok( found ) => panic!( "an empty table returned session {}", found.session_id() ),
  }
}

/// tab04: inserting under an existing id replaces, and hands the old one back.
///
/// This is the re-host path: Claude Code restarts a session with
/// `--fork-session`, the daemon hosts the replacement, and the table must end up
/// with one entry — not two, one of which points at a dead process. The displaced
/// session is returned rather than dropped because its child is still running.
#[ test ]
fn tab04_insert_under_existing_id_replaces()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut table = SessionTable::new();

  insert( &mut table, hosted( "conv-1", dir.path() ) );
  let first_pid = table.get( "conv-1" ).expect( "not found" ).pid();

  let mut displaced = table
    .insert( hosted( "conv-1", dir.path() ) )
    .expect( "inserting over an existing id returned nothing" );
  let second_pid = table.get( "conv-1" ).expect( "not found" ).pid();

  assert_eq!( table.len(), 1, "replacement left a duplicate entry" );
  assert_eq!( displaced.pid(), first_pid, "the wrong session was displaced" );
  assert_ne!( first_pid, second_pid, "the replacement is the same process — test premise broken" );

  displaced.shutdown().expect( "shutdown of the displaced session failed" );
  drain( &mut table );
}

/// tab05: removal hands the session back once.
#[ test ]
fn tab05_remove_yields_the_session_once()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut table = SessionTable::new();
  insert( &mut table, hosted( "conv-1", dir.path() ) );

  let mut removed = table.remove( "conv-1" ).expect( "remove failed" );
  assert_eq!( removed.session_id(), "conv-1" );
  assert!( table.is_empty(), "table still holds the removed session" );

  match table.remove( "conv-1" )
  {
    Err( Error::UnknownSession( id ) ) => assert_eq!( id, "conv-1" ),
    other => panic!( "expected UnknownSession on a second remove, got {other:?}" ),
  }

  removed.shutdown().expect( "shutdown failed" );
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
    insert( &mut table, hosted( id, dir.path() ) );
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
  insert( &mut table, hosted( "conv-1", &cwd ) );

  let live_pid = table.get( "conv-1" ).expect( "not found" ).pid();
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
  insert( &mut table, hosted( "conv-stable", dir.path() ) );

  let before = table.get( "conv-stable" ).expect( "not found" ).pid();

  // Re-host: the old process goes, a new one takes over the same conversation.
  let mut old = table.remove( "conv-stable" ).expect( "remove failed" );
  old.shutdown().expect( "shutdown failed" );
  insert( &mut table, hosted( "conv-stable", dir.path() ) );

  let after = table.get( "conv-stable" ).expect( "the id did not survive the re-host" ).pid();

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
  insert( &mut table, hosted( "conv-1", dir.path() ) );

  table.get_mut( "conv-1" ).expect( "not found" ).set_busy( true );

  let summaries = table.summaries();
  assert!( summaries.first().expect( "no summary produced" ).busy, "the busy flag did not stick" );

  drain( &mut table );
}

/// tab10: what goes into a session comes back out through the cursor.
///
/// The round trip a client actually performs: write, then poll `read_from` with
/// the cursor the previous read returned. `cat` echoes stdin and the terminal's
/// own line discipline echoes it too, so the payload appears more than once —
/// asserting containment rather than equality is the terminal behaving like one.
#[ test ]
fn tab10_output_round_trips_through_the_cursor()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let session = hosted( "conv-1", dir.path() );

  session.write( b"round trip\r" ).expect( "write failed" );
  let seen = read_until( &session, "round trip" );

  assert!( seen.contains( "round trip" ), "payload never came back: {seen:?}" );

  let mut session = session;
  session.shutdown().expect( "shutdown failed" );
}

/// tab11: shutting down a child blocked on stdin returns, and reaps it.
///
/// The pump thread holds a clone of the pty master, which `PtySession::shutdown`
/// cannot reach — so a teardown that closed the pty before stopping the pump
/// would wait forever for a child whose terminal is still open. The bound is the
/// assertion: a regression here hangs rather than fails.
#[ test ]
fn tab11_shutdown_ends_a_child_blocked_on_stdin()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut session = hosted( "conv-1", dir.path() );
  let pid = session.pid();

  let started = Instant::now();
  let status = session.shutdown().expect( "shutdown failed" );

  assert!(
    started.elapsed() < TEST_TIMEOUT,
    "shutdown of pid {pid} took {:?} — the pump was not released first",
    started.elapsed(),
  );
  assert!(
    status.success() || status.code().is_none(),
    "unexpected exit status: {status:?}",
  );
}

/// tab12: a read after shutdown reports the stream as ended.
///
/// Without this a client polling for output has no terminating condition and
/// spins forever on a session that has already gone.
#[ test ]
fn tab12_read_after_shutdown_reports_ended()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let mut session = hosted( "conv-1", dir.path() );
  session.shutdown().expect( "shutdown failed" );

  let slice = session.read_from( 0 );

  assert!( slice.ended, "the stream did not report itself ended after shutdown" );
  assert_eq!( slice.missed, 0, "nothing should have been evicted" );
}

/// tab13: reading twice without new output returns nothing the second time.
///
/// The cursor is what makes a read non-destructive *and* non-repeating: replaying
/// already-seen output would make a polling client print everything twice.
#[ test ]
fn tab13_second_read_without_output_is_empty()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let session = hosted( "conv-1", dir.path() );

  session.write( b"once\r" ).expect( "write failed" );
  read_until( &session, "once" );

  let settled = session.read_from( session.read_from( 0 ).cursor );
  let again = session.read_from( settled.cursor );

  assert_eq!( again.text, "", "a read with no new output replayed old output" );
  assert_eq!( again.cursor, settled.cursor, "an empty read moved the cursor" );

  let mut session = session;
  session.shutdown().expect( "shutdown failed" );
}
