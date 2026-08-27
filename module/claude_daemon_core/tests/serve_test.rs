//! End-to-end tests: a real socket, a real client, real children.
//!
//! Nothing here is stubbed. The daemon runs on a background thread with a bound
//! `Listener`, the client connects over the socket the way `clr` will, and the
//! sessions are actual PTY-attached processes. The spawner runs `cat` instead of
//! `claude` because that is the one thing a test cannot afford to run — and it
//! writes the registry record itself, which is what Claude Code does a few
//! milliseconds after it starts.
//!
//! Output is waited on until it *settles*, never until a substring first shows
//! up. A terminal echoes what is written to it, so a prompt appears twice: once
//! from the line discipline and once from the child. A test that stops at the
//! first sighting is holding a cursor that still has the second copy after it,
//! and any later assertion about what a turn did or did not contain becomes a
//! race — one that passes on a quiet machine.
//!
//! ## Specification References
//!
//! - `docs/feature/006_serving_clients.md` — dispatch, framing, and the client
//! - `docs/feature/002_wire_protocol.md` — the request and response shapes
//! - `docs/feature/004_session_output.md` — what `send` and `read` are built on
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | srv01 | `ping` | The daemon's version |
//! | srv02 | `list_sessions` with none hosted | An empty list |
//! | srv03 | `spawn` | A conversation id, and the session is listed |
//! | srv04 | `send` then `read` | The child's output comes back |
//! | srv05 | The cursor `send` returns | Reading from it yields only that turn |
//! | srv06 | A request naming no known session | `ok:false`, connection still answered |
//! | srv07 | A line that is not a request | A well-formed error, not a dropped connection |
//! | srv08 | `shutdown` | Exit status reported, session no longer listed |
//! | srv09 | `resize` on a hosted session | Accepted; an absent one is refused |
//! | srv10 | `spawn` carrying a prompt | Delivered without a separate `send` |
//! | srv11 | `stop_daemon` | Answered first, then the loop ends and sessions go |
//! | srv12 | `spawn` whose child never registers | The request fails *and* the child is dead |
//! | srv13 | The return that submits a prompt | Sent well after the text, not in the same burst |
//! | srv14 | `context_summary`, hosted and absent | Read from the recorded `cwd`; an unhosted id refused |

use core::sync::atomic::{ AtomicBool, Ordering };
use core::time::Duration;
use std::path::{ Path, PathBuf };
use std::sync::{ Arc, Mutex };
use std::thread::JoinHandle;
use std::time::Instant;

use claude_daemon_core::
{
  acquire, client, Daemon, Error, InstanceLock, Listener, Request, Response, Result,
};
use claude_pty_core::{ PtySession, SessionConfig };

/// How long a test waits for output before calling it a failure.
const TEST_TIMEOUT : Duration = Duration::from_secs( 10 );

/// How long between polls while waiting for output.
const POLL : Duration = Duration::from_millis( 25 );

/// Consecutive empty reads that count as a turn having finished.
///
/// Three, so the gap between the terminal's echo and the child's own copy has to
/// close before anything is asserted about where a turn ended.
const QUIET_READS : u32 = 3;

/// Boxed so the daemon's type can be named — the spawner is a closure otherwise.
type Spawner = Box< dyn FnMut( &Path ) -> Result< PtySession > + Send >;

/// A daemon, its socket, and the thread serving it.
struct Harness
{
  socket : PathBuf,
  server : JoinHandle< Daemon< Spawner > >,
  stop : Arc< AtomicBool >,
  // Held, not read: dropping the lock early would let a second daemon bind over
  // this one's socket mid-test, and dropping the directory would take the
  // registry with it.
  _lock : InstanceLock,
  _dir : tempfile::TempDir,
}

impl Harness
{
  /// Start a daemon serving on its own thread until [`Harness::finish`].
  ///
  /// The registry record is written before the spawner returns, so the first
  /// scan always finds it. A long timeout would only ever be spent on a genuine
  /// failure, and spending it would turn that into a hang.
  fn start() -> Self
  {
    Self::start_with( spawner, Duration::from_secs( 5 ) )
  }

  /// [`Harness::start`], with the spawner and registration timeout chosen by the
  /// caller — for the tests about a spawn that does not go well.
  fn start_with
  (
    make_spawner : impl FnOnce( PathBuf ) -> Spawner,
    registration_timeout : Duration,
  ) -> Self
  {
    let dir = tempfile::tempdir().expect( "tempdir failed" );
    let sessions_dir = dir.path().join( "sessions" );
    std::fs::create_dir_all( &sessions_dir ).expect( "creating the registry dir failed" );

    let lock = acquire( &dir.path().join( "instance.lock" ) ).expect( "acquiring the lock failed" );
    let socket = dir.path().join( "daemon.sock" );
    let listener = Listener::bind( &socket, &lock ).expect( "bind failed" );

    let mut daemon = Daemon::new( sessions_dir.clone(), make_spawner( sessions_dir ) )
      .with_registration_timeout( registration_timeout );

    let stop = Arc::new( AtomicBool::new( false ) );
    let flag = Arc::clone( &stop );
    let server = std::thread::spawn( move ||
    {
      // Deliberately the same shape a real main loop has: serve, then ask
      // whether that request was the one asking the daemon to stop. The extra
      // `flag` is the test's own way out, since nothing else would end this.
      while !flag.load( Ordering::Relaxed )
      {
        claude_daemon_core::serve_once( &listener, &mut daemon ).expect( "serving failed" );
        if daemon.stop_requested()
        {
          break;
        }
      }
      daemon
    } );

    Self { socket, server, stop, _lock : lock, _dir : dir }
  }

  /// Issue one request, expecting the daemon to succeed.
  fn call( &self, request : &Request ) -> serde_json::Value
  {
    client::call( &self.socket, request )
      .unwrap_or_else( | error | panic!( "{request:?} failed: {error}" ) )
  }

  /// Issue one request, keeping whichever answer comes back.
  fn request( &self, request : &Request ) -> Response
  {
    client::request( &self.socket, request )
      .unwrap_or_else( | error | panic!( "{request:?} did not complete: {error}" ) )
  }

  /// Read from `cursor` until `needle` has arrived *and* output has gone quiet.
  ///
  /// Returns what was read and the cursor past all of it — a cursor taken at the
  /// first sighting still has the terminal's second copy after it.
  fn read_settled( &self, session_id : &str, cursor : u64, needle : &str ) -> ( String, u64 )
  {
    let started = Instant::now();
    let mut at = cursor;
    let mut seen = String::new();
    let mut quiet = 0_u32;

    while started.elapsed() < TEST_TIMEOUT
    {
      let slice = self.call( &Request::Read { session_id : session_id.into(), cursor : at } );
      at = slice[ "cursor" ].as_u64().expect( "read reported no cursor" );
      let text = slice[ "text" ].as_str().expect( "read reported no text" );

      if text.is_empty()
      {
        quiet += 1;
      }
      else
      {
        quiet = 0;
        seen.push_str( text );
      }

      if seen.contains( needle ) && quiet >= QUIET_READS
      {
        return ( seen, at );
      }
      std::thread::sleep( POLL );
    }
    panic!( "waited {TEST_TIMEOUT:?} for {needle:?} to settle; saw {seen:?}" );
  }

  /// Stop the server, shut down every session it still holds, and hand it back.
  fn finish( self ) -> Daemon< Spawner >
  {
    self.stop.store( true, Ordering::Relaxed );
    // The thread is parked in `accept`; a connection that immediately hangs up
    // is served as a clean end-of-stream and lets it see the flag. Ignored if it
    // fails — the thread may already have left the loop on its own, taking the
    // socket with it, which is the same outcome by a different route.
    drop( std::os::unix::net::UnixStream::connect( &self.socket ) );

    let mut daemon = self.server.join().expect( "the server thread panicked" );
    daemon.shutdown_all().expect( "shutting sessions down failed" );
    daemon
  }
}

/// A spawner that starts `cat` and registers it, standing in for `claude`.
///
/// Registering synchronously is the fast end of what really happens: Claude Code
/// publishes its conversation id shortly after start, and the slower case —
/// including never — is `registration_test.rs`.
fn spawner( sessions_dir : PathBuf ) -> Spawner
{
  let mut minted = 0_u32;
  Box::new( move | cwd : &Path |
  {
    let config = SessionConfig::new( "cat" ).cwd( cwd );
    let pty = PtySession::spawn( &config ).map_err( Error::Pty )?;
    minted += 1;
    let record = format!
    (
      "{{ \"pid\": {}, \"sessionId\": \"conv-{minted}\", \"cwd\": \"{}\" }}",
      pty.pid(),
      cwd.display(),
    );
    std::fs::write( sessions_dir.join( format!( "{}.json", pty.pid() ) ), record )
      .expect( "writing the registry record failed" );
    Ok( pty )
  } )
}

/// Ask for a session and return its conversation id.
fn spawn_session( harness : &Harness ) -> String
{
  let result = harness.call( &Request::Spawn { cwd : PathBuf::from( "/tmp" ), prompt : None } );
  result[ "session_id" ]
    .as_str()
    .expect( "spawn reported no session_id" )
    .to_string()
}

/// srv01: the daemon answers a liveness probe with its version.
#[ test ]
fn srv01_ping_reports_the_version()
{
  let harness = Harness::start();

  let result = harness.call( &Request::Ping );

  assert_eq!( result[ "version" ], env!( "CARGO_PKG_VERSION" ) );
  harness.finish();
}

/// srv02: a daemon hosting nothing says so, rather than erroring.
#[ test ]
fn srv02_list_sessions_starts_empty()
{
  let harness = Harness::start();

  let result = harness.call( &Request::ListSessions );

  assert_eq!( result.as_array().expect( "list_sessions is not an array" ).len(), 0 );
  harness.finish();
}

/// srv03: a spawned session is named and immediately addressable.
#[ test ]
fn srv03_spawn_registers_a_session()
{
  let harness = Harness::start();

  let session_id = spawn_session( &harness );
  let listed = harness.call( &Request::ListSessions );

  let sessions = listed.as_array().expect( "list_sessions is not an array" );
  assert_eq!( sessions.len(), 1, "expected exactly one session, got {listed}" );
  assert_eq!( sessions[ 0 ][ "session_id" ], session_id.as_str() );
  assert_eq!( sessions[ 0 ][ "cwd" ], "/tmp" );
  assert!( sessions[ 0 ][ "pid" ].as_u64().is_some_and( | pid | pid > 0 ) );
  harness.finish();
}

/// srv04: what goes in through `send` comes back out through `read`.
///
/// The round trip the whole crate exists for: a client writes a prompt and reads
/// what the session produced, over one socket, without holding the daemon for
/// the duration of the turn.
#[ test ]
fn srv04_send_output_comes_back_through_read()
{
  let harness = Harness::start();
  let session_id = spawn_session( &harness );

  harness.call( &Request::Send { session_id : session_id.clone(), text : "marco".into() } );
  let ( seen, _ ) = harness.read_settled( &session_id, 0, "marco" );

  assert!( seen.contains( "marco" ), "output was {seen:?}" );
  harness.finish();
}

/// srv05: the cursor `send` returns is where that turn's output begins.
///
/// Exact, not approximate — the daemon is single-threaded, so nothing can have
/// written to the session between reading the cursor and queueing the text.
/// Without it a client has to guess, and guessing means either replaying the
/// previous turn or truncating this one.
#[ test ]
fn srv05_send_reports_the_cursor_its_output_starts_at()
{
  let harness = Harness::start();
  let session_id = spawn_session( &harness );

  harness.call( &Request::Send { session_id : session_id.clone(), text : "first".into() } );
  harness.read_settled( &session_id, 0, "first" );

  let second = harness.call( &Request::Send
  {
    session_id : session_id.clone(),
    text : "second".into(),
  } );
  let cursor = second[ "cursor" ].as_u64().expect( "send reported no cursor" );
  assert!( cursor > 0, "the second send started at the beginning of the session" );

  let ( seen, _ ) = harness.read_settled( &session_id, cursor, "second" );

  assert!( !seen.contains( "first" ), "the second turn replayed the first: {seen:?}" );
  harness.finish();
}

/// srv06: an unknown session is answered, not hung up on.
#[ test ]
fn srv06_unknown_session_is_an_error_response()
{
  let harness = Harness::start();

  let response = harness.request( &Request::Send
  {
    session_id : "conv-nonexistent".into(),
    text : "hello".into(),
  } );

  match response
  {
    Response::Err { error, .. } => assert!
    (
      error.contains( "conv-nonexistent" ),
      "the error does not name the session: {error}",
    ),
    Response::Ok { result, .. } => panic!( "an unknown session succeeded: {result}" ),
  }
  harness.finish();
}

/// srv07: an unparseable line still gets a well-formed answer.
///
/// The client is owed a response even when what it sent was nonsense. Closing
/// the connection instead makes a client bug look like a daemon that died.
#[ test ]
fn srv07_a_malformed_line_gets_an_error_response()
{
  use std::io::{ BufRead, Write };

  let harness = Harness::start();

  let mut stream = std::os::unix::net::UnixStream::connect( &harness.socket )
    .expect( "connect failed" );
  stream.write_all( b"this is not json\n" ).expect( "write failed" );
  stream.flush().expect( "flush failed" );

  let mut reply = String::new();
  std::io::BufReader::new( &stream ).read_line( &mut reply ).expect( "read failed" );

  let response : Response = serde_json::from_str( reply.trim() ).unwrap_or_else
  (
    | error | panic!( "the daemon's reply was not a Response ({error}): {reply:?}" ),
  );
  assert!( matches!( response, Response::Err { .. } ), "nonsense was accepted: {response:?}" );
  harness.finish();
}

/// srv08: shutting a session down reports how it ended and forgets it.
#[ test ]
fn srv08_shutdown_reports_and_removes()
{
  let harness = Harness::start();
  let session_id = spawn_session( &harness );

  let result = harness.call( &Request::Shutdown { session_id : session_id.clone() } );

  // `cat` handed end-of-input exits cleanly. Deterministic here, unlike a child
  // that loses its terminal without warning — the ladder sends `Ctrl-D` first
  // precisely so the child gets to end on its own terms.
  assert_eq!( result[ "exit_code" ], 0, "unexpected ending: {result}" );

  let listed = harness.call( &Request::ListSessions );
  assert_eq!( listed.as_array().expect( "not an array" ).len(), 0, "still listed: {listed}" );
  harness.finish();
}

/// srv09: a hosted session can be resized; an absent one cannot.
#[ test ]
fn srv09_resize_reaches_a_hosted_session()
{
  let harness = Harness::start();
  let session_id = spawn_session( &harness );

  let result = harness.call( &Request::Resize { session_id, rows : 40, cols : 132 } );
  assert!( result.is_null(), "resize returned a payload: {result}" );

  let absent = harness.request( &Request::Resize
  {
    session_id : "conv-nonexistent".into(),
    rows : 40,
    cols : 132,
  } );
  assert!( matches!( absent, Response::Err { .. } ), "resizing nothing succeeded: {absent:?}" );
  harness.finish();
}

/// srv10: a prompt carried by `spawn` is delivered without a second call.
///
/// The shape a one-shot client wants: one request in, then poll for output. Two
/// round trips to start a session would leave a window in which the session
/// exists and nobody has told it anything.
#[ test ]
fn srv10_spawn_delivers_its_prompt()
{
  let harness = Harness::start();

  let result = harness.call( &Request::Spawn
  {
    cwd : PathBuf::from( "/tmp" ),
    prompt : Some( "polo".into() ),
  } );
  let session_id = result[ "session_id" ].as_str().expect( "no session_id" ).to_string();

  let ( seen, _ ) = harness.read_settled( &session_id, 0, "polo" );

  assert!( seen.contains( "polo" ), "the prompt never reached the session: {seen:?}" );
  harness.finish();
}

/// srv11: the daemon answers the request to stop before acting on it.
///
/// The ordering is the point. Tearing sessions down inside the request would
/// spend an unbounded amount of a client's wait on children that may be slow to
/// die — and a client that asked the daemon to stop would learn nothing about
/// whether it is going to. The flag is set, the answer goes out, and the loop
/// ends on the next check.
#[ test ]
fn srv11_stop_daemon_answers_then_stops()
{
  let harness = Harness::start();
  let session_id = spawn_session( &harness );
  let listed = harness.call( &Request::ListSessions );
  assert_eq!( listed.as_array().expect( "not an array" ).len(), 1, "premise: {listed}" );

  let result = harness.call( &Request::StopDaemon );
  assert!( result[ "stopping" ].as_bool().unwrap_or( false ), "unexpected answer: {result}" );

  let daemon = harness.finish();

  assert!( daemon.stop_requested(), "the request did not set the flag" );
  assert!(
    daemon.sessions().is_empty(),
    "{session_id} outlived the daemon it was hosted by",
  );
}

/// srv12: a child that never registers is killed, not left running.
///
/// The failure this pins down is silent and permanent. `PtySession` has no
/// `Drop`, and `std::process::Child` deliberately does not kill on drop either —
/// so a spawn that gives up on registration used to *return an error and leave
/// the child running*, reparented to init, holding a terminal, addressable by
/// nobody. Nothing in the daemon's own state would show it: the session table
/// never had it, so `list_sessions` is empty either way, and that is exactly
/// what the assertion below is for.
///
/// Found by an end-to-end smoke against a real `claude` that sat in a first-run
/// prompt instead of registering — the case the `cat` spawner cannot produce,
/// reproduced here by simply not writing the record.
#[ test ]
fn srv12_an_unregistered_child_is_killed()
{
  // Shared with the spawner so the test can name the process it is asserting
  // about. The daemon never reports the pid — the whole point is that this child
  // is one the daemon does not know it has.
  let spawned : Arc< Mutex< Vec< u32 > > > = Arc::new( Mutex::new( Vec::new() ) );
  let recorder = Arc::clone( &spawned );

  let harness = Harness::start_with
  (
    move | _sessions_dir |
    {
      Box::new( move | cwd : &Path |
      {
        // Deliberately not `cat`, and this is the whole test. `cat` reads its
        // terminal, so closing the master end kills it for free — and a test
        // whose subject dies for free proves nothing, which was measured, not
        // assumed: srv12 passed with the fix commented out until the child
        // became this one.
        //
        // So: a child that ignores the hangup and never reads the terminal, and
        // therefore outlives every incidental teardown. Ignored dispositions
        // survive `execve`, so `sleep` inherits the trap and only SIGKILL —
        // which is what the fix sends — will end it. `sleep`'s own duration is
        // just an upper bound on the mess a failure leaves behind.
        let config = SessionConfig::new( "sh" )
          .arg( "-c" )
          .arg( "trap '' HUP; exec sleep 30" )
          .cwd( cwd );
        let pty = PtySession::spawn( &config ).map_err( Error::Pty )?;
        recorder.lock().expect( "poisoned" ).push( pty.pid() );
        Ok( pty )
      } )
    },
    // Short: this timeout is spent in full on every run of this test, and there
    // is nothing to wait for.
    Duration::from_millis( 300 ),
  );

  let response = harness.request( &Request::Spawn { cwd : PathBuf::from( "/tmp" ), prompt : None } );
  assert!(
    matches!( response, Response::Err { .. } ),
    "a child that never registers must not produce a session: {response:?}",
  );

  let pid = *spawned.lock().expect( "poisoned" ).first().expect( "the spawner never ran" );

  // Reaping happens inside the failed request, so this is already true when the
  // error arrives; the deadline is only there so a scheduling hiccup reports a
  // failure instead of causing one.
  let deadline = Instant::now() + Duration::from_secs( 2 );
  while claude_session_core::pid_alive( pid, None ) && Instant::now() < deadline
  {
    std::thread::sleep( Duration::from_millis( 20 ) );
  }

  assert!(
    !claude_session_core::pid_alive( pid, None ),
    "pid {pid} outlived the spawn that failed — a leaked session nobody can address",
  );

  harness.finish();
}

/// The floor `send` must clear, well under the real gap and well over socket noise.
///
/// Deliberately not `SUBMIT_GAP` itself: asserting a constant against itself passes
/// no matter what the constant is set to, including zero.
const SUBMIT_FLOOR : Duration = Duration::from_millis( 100 );

#[ test ]
fn srv13_the_submitting_return_is_not_sent_with_the_text()
{
  let harness = Harness::start();
  let session_id = spawn_session( &harness );

  let started = Instant::now();
  harness.call( &Request::Send
  {
    session_id : session_id.clone(),
    // Over the threshold where this actually mattered in production: prompts
    // below roughly 55 bytes submitted fine and everything longer did not.
    text : "a prompt long enough that its arrival looks like a paste rather than typing".into(),
  } );
  let elapsed = started.elapsed();

  // Why a stopwatch rather than an observation. What the fix guarantees is that
  // Claude Code's input handler reads the text and the return as two separate
  // events — but the daemon is single-threaded, so while `send` is holding them
  // apart it cannot answer a `read` that would catch it in the act. From outside,
  // the gap has exactly one signature, and this is it.
  //
  // What the stopwatch cannot see is *where* the pause sits. That the text goes
  // first is settled by the two writes below; that the pause is between them and
  // not after both is settled by reading `send`. Neither is free — deleting the
  // sleep silently reverts every prompt over ~55 bytes to landing in the input
  // box and staying there, with no error anywhere. `tests/manual/readme.md`
  // covers the end of that story against a real `claude`, which is the only
  // place the paste heuristic itself exists.
  assert!
  (
    elapsed >= SUBMIT_FLOOR,
    "`send` returned in {elapsed:?} — the text and the return went out together, \
     and a prompt this long will sit unsubmitted in the input box",
  );

  // And the return really did follow the text, in that order.
  let ( seen, _ ) = harness.read_settled( &session_id, 0, "looks like a paste" );
  let text_at = seen.find( "looks like a paste" ).expect( "the prompt never echoed" );
  assert!
  (
    seen[ text_at.. ].contains( '\r' ),
    "no carriage return after the prompt — nothing submitted it. Saw {seen:?}",
  );

  harness.finish();
}

/// srv14: a context summary is resolved through the table, not from the request.
///
/// The request carries a session id and nothing else. The working directory the
/// transcript is found under comes from the daemon's own record of that session,
/// so a client cannot name one session and be served a path of its own choosing.
///
/// The transcript is planted at the path that `cwd` implies and nowhere else,
/// and the hosted half must *succeed*. That is what makes this a test of the
/// routing rather than of the error: a dispatch that used any other working
/// directory would not find this file, and would answer `NoTranscript` — which
/// is exactly what a session with no transcript answers, and so would prove
/// nothing if it were all this asserted.
#[ test ]
fn srv14_context_summary_resolves_through_the_table()
{
  // Point the transcript lookup at a directory this test owns, so the answer
  // does not depend on what the machine running it has in its real home.
  let home = tempfile::tempdir().expect( "tempdir failed" );
  std::env::set_var( "CLAUDE_HOME", home.path() );

  let harness = Harness::start();
  let session_id = spawn_session( &harness );

  // `spawn_session` asks for `/tmp`, so that is what the daemon recorded and
  // that is the only encoding under which this transcript is reachable.
  let encoded = claude_storage_core::encode_path( Path::new( "/tmp" ) )
    .expect( "/tmp should encode" );
  let project_dir = home.path().join( "projects" ).join( encoded );
  std::fs::create_dir_all( &project_dir ).expect( "creating the project dir failed" );
  std::fs::write
  (
    project_dir.join( format!( "{session_id}.jsonl" ) ),
    format!
    (
      "{{\"type\":\"attachment\",\"sessionId\":\"{session_id}\",\
        \"attachment\":{{\"type\":\"deferred_tools_delta\",\"addedNames\":[\"WebFetch\"]}}}}\n"
    ),
  ).expect( "writing the transcript failed" );

  let summary = harness.call( &Request::ContextSummary
  {
    session_id : session_id.clone(),
  } );
  assert_eq!( summary[ "session_id" ], session_id.as_str() );
  assert_eq!
  (
    summary[ "deferred_tools" ],
    serde_json::json!( [ "WebFetch" ] ),
    "the summary did not come from the transcript at the recorded cwd: {summary}",
  );

  // And an id the daemon does not host is refused, rather than sent to the
  // filesystem to be looked up on a client-supplied path.
  let absent = harness.request( &Request::ContextSummary
  {
    session_id : "conv-nonexistent".into(),
  } );
  match absent
  {
    Response::Err { error, .. } => assert!
    (
      error.contains( "conv-nonexistent" ),
      "the error does not name the session: {error}",
    ),
    Response::Ok { result, .. } => panic!( "an unhosted session summarized: {result}" ),
  }

  harness.finish();
}
