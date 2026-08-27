//! Registration tests — learning a spawned session's conversation id.
//!
//! Two halves, deliberately separated. [`lookup`] is one scan of a fixture
//! directory with no timing in it at all, so the matching rules can be pinned
//! exactly. [`await_session_id`] is the wait around it, and the only thing worth
//! asserting there is *when it stops* — early on a dead child, at the deadline on
//! a live one, immediately once the record lands.
//!
//! The ordering case (reg08) is the one that looks like a detail and is not: the
//! registry is scanned before liveness is consulted, so a process that registered
//! and then exited is still reported. Reversing those two lines loses a
//! conversation id that is sitting readable on disk.
//!
//! ## Specification References
//!
//! - `docs/feature/005_session_registration.md` — waiting for a conversation id
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | reg01 | Lookup against a directory that does not exist | No record, no error |
//! | reg02 | Several records, one matching pid | That record's conversation id |
//! | reg03 | Records, none matching | No record |
//! | reg04 | Waiting on an already-registered pid | Returns the id without waiting |
//! | reg05 | A record written during the wait | Picked up on a later poll |
//! | reg06 | The child dies without registering | Gives up early, not at the deadline |
//! | reg07 | A live child that never registers | `NoRegistration` at the deadline |
//! | reg08 | A registered pid whose child has since died | Still reports the id |
//! | reg09 | An unparseable file beside a good one | The good record is still found |

use core::time::Duration;
use std::fs;
use std::path::Path;
use std::time::Instant;

use claude_daemon_core::registration::lookup;
use claude_daemon_core::{ await_session_id, Error };

/// Long enough to clear several poll intervals, short enough that the two
/// timeout cases do not dominate the suite.
const SHORT_TIMEOUT : Duration = Duration::from_millis( 400 );

/// For cases that must return well before their deadline — generous, so that
/// missing it means the logic is wrong rather than the machine slow.
const PATIENT_TIMEOUT : Duration = Duration::from_secs( 10 );

/// Write one registry record in the shape Claude Code writes it.
///
/// `procStart` is a string here because it is a string on disk — a fixture that
/// quietly corrects the format would stop testing the parser that has to cope
/// with it.
fn register( dir : &Path, pid : u32, session_id : &str )
{
  let text = format!
  (
    "{{ \"pid\": {pid}, \"sessionId\": \"{session_id}\", \"cwd\": \"/tmp\", \"procStart\": \"99\" }}"
  );
  fs::write( dir.join( format!( "{pid}.json" ) ), text ).expect( "writing a registry record failed" );
}

/// reg01: a registry directory that was never created is empty, not broken.
///
/// The first session on a machine runs before anything has created it.
#[ test ]
fn reg01_lookup_in_a_missing_directory_finds_nothing()
{
  let root = tempfile::tempdir().expect( "tempdir failed" );
  let missing = root.path().join( "never-created" );

  let found = lookup( &missing, 1234 ).expect( "a missing registry must not be an error" );

  assert_eq!( found, None );
}

/// reg02: the record naming our pid is the one that answers.
#[ test ]
fn reg02_lookup_matches_on_pid()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  register( dir.path(), 100, "first" );
  register( dir.path(), 200, "second" );
  register( dir.path(), 300, "third" );

  let found = lookup( dir.path(), 200 ).expect( "lookup failed" );

  assert_eq!( found.as_deref(), Some( "second" ) );
}

/// reg03: other people's sessions are not ours.
#[ test ]
fn reg03_lookup_ignores_other_pids()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  register( dir.path(), 100, "first" );
  register( dir.path(), 200, "second" );

  let found = lookup( dir.path(), 999 ).expect( "lookup failed" );

  assert_eq!( found, None );
}

/// reg04: a record already on disk is returned on the first scan.
#[ test ]
fn reg04_await_returns_an_existing_record_immediately()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  register( dir.path(), 4242, "already-there" );

  let started = Instant::now();
  let id = await_session_id( dir.path(), 4242, PATIENT_TIMEOUT, || true )
    .expect( "an existing record was not found" );

  assert_eq!( id, "already-there" );
  assert!
  (
    started.elapsed() < Duration::from_secs( 1 ),
    "an existing record took {:?} to find — the first scan is happening after a sleep",
    started.elapsed(),
  );
}

/// reg05: registration that lands mid-wait is picked up.
///
/// The real shape of the problem: the daemon starts waiting before the child has
/// written anything, because the child cannot write it until it has started.
#[ test ]
fn reg05_a_record_written_during_the_wait_is_picked_up()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );

  let id = std::thread::scope( | scope |
  {
    scope.spawn( ||
    {
      std::thread::sleep( Duration::from_millis( 80 ) );
      register( dir.path(), 5150, "late-arrival" );
    });
    await_session_id( dir.path(), 5150, PATIENT_TIMEOUT, || true )
  })
  .expect( "a record that appeared during the wait was never seen" );

  assert_eq!( id, "late-arrival" );
}

/// reg06: a child that dies during startup fails now, not in thirty seconds.
///
/// This is the whole reason `alive` is a parameter rather than something the
/// wait works out for itself. The registry cannot distinguish "has not written
/// yet" from "will never write"; only the caller holding the child handle can.
#[ test ]
fn reg06_a_dead_child_gives_up_early()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );

  let started = Instant::now();
  let error = await_session_id( dir.path(), 7, PATIENT_TIMEOUT, || false )
    .expect_err( "a dead child that never registered was reported as registered" );
  let elapsed = started.elapsed();

  assert!( matches!( error, Error::NoRegistration { pid } if pid == 7 ), "unexpected error: {error}" );
  assert!
  (
    elapsed < Duration::from_secs( 2 ),
    "gave up after {elapsed:?} — a dead child should not be waited out",
  );
}

/// reg07: a live child that never registers is abandoned at the deadline.
#[ test ]
fn reg07_a_live_child_that_never_registers_times_out()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );

  let started = Instant::now();
  let error = await_session_id( dir.path(), 4242, SHORT_TIMEOUT, || true )
    .expect_err( "an unregistered pid was reported as registered" );
  let elapsed = started.elapsed();

  assert!( matches!( error, Error::NoRegistration { pid } if pid == 4242 ), "unexpected error: {error}" );
  assert!( elapsed >= SHORT_TIMEOUT, "returned after {elapsed:?}, before the {SHORT_TIMEOUT:?} deadline" );
  assert!( elapsed < SHORT_TIMEOUT * 10, "overran the deadline by an order of magnitude: {elapsed:?}" );
}

/// reg08: a record outlives the process that wrote it, and still counts.
///
/// A short-lived session can register and exit before the first poll comes
/// round. Its id is on disk and correct; refusing to read it because the process
/// is gone would lose a conversation to a race the caller cannot influence.
#[ test ]
fn reg08_a_record_from_a_dead_child_is_still_reported()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  register( dir.path(), 8080, "wrote-then-exited" );

  let id = await_session_id( dir.path(), 8080, PATIENT_TIMEOUT, || false )
    .expect( "a readable record was discarded because its process had exited" );

  assert_eq!( id, "wrote-then-exited" );
}

/// reg09: one unreadable file does not hide the rest of the registry.
///
/// Claude Code rewrites these files in place, so a scan can catch a torn write.
/// Failing the whole lookup over one would make registration flaky in exactly
/// the busy conditions where sessions are being created.
#[ test ]
fn reg09_an_unparseable_file_does_not_hide_a_good_record()
{
  let dir = tempfile::tempdir().expect( "tempdir failed" );
  fs::write( dir.path().join( "torn.json" ), "{ \"pid\": 90" ).expect( "writing the torn record failed" );
  register( dir.path(), 9000, "intact" );

  let found = lookup( dir.path(), 9000 ).expect( "lookup failed" );

  assert_eq!( found.as_deref(), Some( "intact" ) );
}
