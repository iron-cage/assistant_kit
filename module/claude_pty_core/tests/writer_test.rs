//! Writer-thread tests: delivery, backpressure, and shutdown.
//!
//! The sink is a real `File` (a temp file or a pty master), never a mock — a
//! fake `Write` impl would not reproduce the blocking behavior the writer thread
//! exists to contain.
//!
//! ## Specification References
//!
//! - `docs/feature/003_writer_thread.md` — why a thread, why bounded
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | wr01 | Queued bytes reach the sink | File contains them after shutdown |
//! | wr02 | Writes are delivered in order | Sink content matches send order |
//! | wr03 | Send to a blocked sink past capacity | `Error::WriterFull` |
//! | wr04 | Send after `shutdown` | `Error::WriterGone` |
//! | wr05 | `shutdown` is idempotent | Second call is a no-op |
//! | wr06 | Drop without `shutdown` | Queued bytes still land |
//! | wr07 | `DEFAULT_QUEUE_CAPACITY` | 256 |
//! | wr08 | Empty send | Accepted, writes nothing |

use core::time::Duration;
use std::fs;
use std::io::Write;
use std::time::Instant;

use claude_pty_core::{ Error, Pty, WriterHandle, DEFAULT_QUEUE_CAPACITY };

/// Longest a test waits for the writer thread to drain before failing.
const DRAIN_TIMEOUT : Duration = Duration::from_secs( 5 );

/// A temp file plus a second handle to it, for reading back what was written.
fn sink_pair() -> ( tempfile::TempDir, std::path::PathBuf, fs::File )
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let path = dir.path().join( "sink" );
  let file = fs::File::create( &path ).expect( "cannot create sink file" );
  ( dir, path, file )
}

/// Poll `path` until it contains `needle` or [`DRAIN_TIMEOUT`] elapses.
fn wait_for_content( path : &std::path::Path, needle : &str ) -> String
{
  let deadline = Instant::now() + DRAIN_TIMEOUT;
  loop
  {
    let content = fs::read_to_string( path ).unwrap_or_default();
    if content.contains( needle )
    {
      return content;
    }
    assert!(
      Instant::now() < deadline,
      "timed out waiting for {needle:?} in sink; content was {content:?}",
    );
    std::thread::sleep( Duration::from_millis( 10 ) );
  }
}

/// wr01, wr02: queued bytes reach the sink, in the order they were sent.
#[ test ]
fn wr01_queued_bytes_reach_sink_in_order()
{
  let ( _dir, path, file ) = sink_pair();
  let mut writer = WriterHandle::spawn( file, DEFAULT_QUEUE_CAPACITY );

  writer.send( b"first " ).expect( "send failed" );
  writer.send( b"second " ).expect( "send failed" );
  writer.send( b"third" ).expect( "send failed" );
  writer.shutdown();

  let content = fs::read_to_string( &path ).expect( "cannot read sink" );
  assert_eq!( content, "first second third", "writes arrived out of order or incomplete" );
}

/// wr03: a sink that cannot drain produces `WriterFull` rather than growth.
///
/// The sink is a pty master with no reader: once the kernel's buffer fills, the
/// writer thread blocks inside `write_all` and the queue backs up. An unbounded
/// queue would absorb this silently and turn a stalled child into unbounded
/// memory use — the same outage, later and harder to diagnose.
#[ test ]
fn wr03_full_queue_reports_writer_full()
{
  let pty = Pty::open().expect( "pty allocation failed" );
  // The slave stays open for the whole test but is never read from, so writes to
  // the master land in the terminal's input buffer and stop once it fills.
  // Closing it instead would make the master fail with EIO, which is a different
  // condition (`WriterGone`) than the backpressure this test is about.
  let _slave = pty.open_slave().expect( "open_slave failed" );

  let sink = fs::File::from( pty.master().try_clone().expect( "cannot clone master" ) );

  let capacity = 2;
  let writer = WriterHandle::spawn( sink, capacity );

  // Enough traffic to fill the kernel buffer, block the thread, and then fill
  // the two-slot queue behind it.
  let chunk = vec![ b'x'; 64 * 1024 ];
  let deadline = Instant::now() + DRAIN_TIMEOUT;
  let mut saw_full = false;

  while Instant::now() < deadline
  {
    match writer.send( &chunk )
    {
      Ok( () ) => {}
      Err( Error::WriterFull ) =>
      {
        saw_full = true;
        break;
      }
      Err( other ) => panic!( "unexpected error from send: {other}" ),
    }
  }

  assert!(
    saw_full,
    "queue never reported WriterFull against an undrained sink — backpressure is not bounded",
  );
}

/// wr04: sending after shutdown reports `WriterGone`.
#[ test ]
fn wr04_send_after_shutdown_reports_writer_gone()
{
  let ( _dir, _path, file ) = sink_pair();
  let mut writer = WriterHandle::spawn( file, DEFAULT_QUEUE_CAPACITY );

  writer.shutdown();

  match writer.send( b"too late" )
  {
    Err( Error::WriterGone ) => {}
    other => panic!( "expected WriterGone, got {other:?}" ),
  }
}

/// wr05: `shutdown` is idempotent.
///
/// `Drop` calls it too, so a caller that shuts down explicitly must not hit a
/// double-join panic when the handle later drops.
#[ test ]
fn wr05_shutdown_is_idempotent()
{
  let ( _dir, _path, file ) = sink_pair();
  let mut writer = WriterHandle::spawn( file, DEFAULT_QUEUE_CAPACITY );

  writer.shutdown();
  writer.shutdown();
  drop( writer );
}

/// wr06: dropping without `shutdown` still delivers what was queued.
#[ test ]
fn wr06_drop_drains_queued_writes()
{
  let ( _dir, path, file ) = sink_pair();
  let writer = WriterHandle::spawn( file, DEFAULT_QUEUE_CAPACITY );

  writer.send( b"dropped-but-delivered" ).expect( "send failed" );
  drop( writer );

  let content = wait_for_content( &path, "dropped-but-delivered" );
  assert!( content.contains( "dropped-but-delivered" ), "queued write lost on drop: {content:?}" );
}

/// wr07: the documented default capacity.
#[ test ]
fn wr07_default_capacity_is_256()
{
  assert_eq!(
    DEFAULT_QUEUE_CAPACITY, 256,
    "default queue capacity changed — update docs/feature/003_writer_thread.md",
  );
}

/// wr08: an empty send is accepted and writes nothing.
#[ test ]
fn wr08_empty_send_writes_nothing()
{
  let ( _dir, path, file ) = sink_pair();
  let mut writer = WriterHandle::spawn( file, DEFAULT_QUEUE_CAPACITY );

  writer.send( b"" ).expect( "empty send failed" );
  writer.send( b"marker" ).expect( "send failed" );
  writer.shutdown();

  let content = fs::read_to_string( &path ).expect( "cannot read sink" );
  assert_eq!( content, "marker", "empty send contributed bytes: {content:?}" );
}

/// A sink that is itself a plain file is exercised through `Write` directly,
/// confirming the handle imposes nothing beyond the `Write` bound.
#[ test ]
fn wr09_accepts_any_write_sink()
{
  let ( _dir, path, mut file ) = sink_pair();
  file.write_all( b"pre-existing " ).expect( "direct write failed" );

  let mut writer = WriterHandle::spawn( file, 4 );
  writer.send( b"appended" ).expect( "send failed" );
  writer.shutdown();

  let content = fs::read_to_string( &path ).expect( "cannot read sink" );
  assert_eq!( content, "pre-existing appended", "sink content unexpected: {content:?}" );
}
