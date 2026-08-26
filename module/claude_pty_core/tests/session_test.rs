//! Real-process tests for hosting a child on a pseudo-terminal.
//!
//! Every test spawns an actual process — `cat`, `stty`, `tty`, `sh` — and reads
//! its real output back through the master descriptor. Nothing is stubbed.
//!
//! ## Specification References
//!
//! - `docs/feature/002_session_spawn.md` — spawn, controlling terminal, scrubbing
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | sess01 | Spawn `cat` and write a line | The line comes back on the master |
//! | sess02 | Child has a controlling terminal | `tty` reports the slave path |
//! | sess03 | Child is a session leader | `/proc/$$/stat` pid == session |
//! | sess04 | `cwd()` is honoured | `pwd -P` reports the configured directory |
//! | sess05 | Configured window size reaches the child | `stty size` reports it |
//! | sess06 | Default window size when none configured | `stty size` reports 24 80 |
//! | sess07 | `CLAUDE_`-prefixed variables are scrubbed | Child sees the name unset |
//! | sess08 | Explicit `env()` survives scrubbing | Child sees the value |
//! | sess09 | `TERM` describes the pty, not the parent | Child sees `CHILD_TERM` |
//! | sess10 | `take_reader` yields the reader exactly once | Second call is `None` |
//! | sess11 | Master reaches EOF after the child exits | Read returns 0 bytes |
//! | sess12 | `shutdown()` reports a nonzero exit | Exit code 3 |
//! | sess13 | `write` after `shutdown` | `Err` — queue closed, not silently dropped |
//! | sess14 | `try_wait` on a running child | `Ok( None )` |
//! | sess15 | `pid()` names a live process | `/proc/{pid}` exists |
//! | sess16 | Two concurrent sessions | Distinct pids, slaves, and input streams |
//! | sess17 | Spawning a nonexistent program | `Err` |
//! | sess18 | `resize` after spawn | Child observes the new size |
//! | sess19 | Reported slave path matches the child's view | `tty` output contains it |
//! | sess20 | stderr reaches the master | Output visible on the master |
//! | sess21 | `shutdown` on a child blocked reading stdin | Returns promptly rather than hanging |
//! | sess22 | `resize` after `shutdown` | `Error::SessionClosed` |
//! | sess23 | `slave_path` and a second `shutdown` afterwards | Path survives; status repeats |
//! | sess24 | The child's open descriptors | Only the slave — no `/dev/ptmx` |

use core::time::Duration;
use std::io::Read;
use std::time::Instant;

use claude_pty_core::{ PtySession, SessionConfig, WinSize };

/// Longest a test will wait for a child's output before failing.
const READ_TIMEOUT : Duration = Duration::from_secs( 10 );

/// Read from `reader` until `needle` appears, EOF, or [`READ_TIMEOUT`] elapses.
///
/// Reads are blocking, so a child that produces nothing would hang the suite
/// rather than fail it. The deadline is checked between reads: enough to turn a
/// silent child into a diagnosable failure without a non-blocking rewrite.
fn read_until( reader : &mut impl Read, needle : &str ) -> String
{
  let deadline = Instant::now() + READ_TIMEOUT;
  let mut acc = String::new();
  let mut chunk = [ 0_u8; 1024 ];

  loop
  {
    let n = reader.read( &mut chunk ).unwrap_or( 0 );
    assert!( n != 0, "child reached EOF without producing {needle:?}; got: {acc:?}" );
    acc.push_str( &String::from_utf8_lossy( &chunk[ ..n ] ) );
    if acc.contains( needle )
    {
      return acc;
    }
    assert!(
      Instant::now() < deadline,
      "timed out after {READ_TIMEOUT:?} waiting for {needle:?}; got so far: {acc:?}",
    );
  }
}

/// Read everything until EOF, bounded by [`READ_TIMEOUT`].
fn read_to_eof( reader : &mut impl Read ) -> String
{
  let deadline = Instant::now() + READ_TIMEOUT;
  let mut acc = String::new();
  let mut chunk = [ 0_u8; 1024 ];

  loop
  {
    let n = reader.read( &mut chunk ).unwrap_or( 0 );
    if n == 0
    {
      return acc;
    }
    acc.push_str( &String::from_utf8_lossy( &chunk[ ..n ] ) );
    assert!( Instant::now() < deadline, "timed out waiting for EOF; got: {acc:?}" );
  }
}

/// Spawn `config` and return everything the child prints before exiting.
fn output_of( config : SessionConfig ) -> String
{
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let mut reader = session.take_reader().expect( "reader already taken" );
  let out = read_to_eof( &mut reader );
  session.shutdown().expect( "shutdown failed" );
  out
}

/// `sh -c <script>` on a pty, with the default window size.
fn shell( script : &str ) -> SessionConfig
{
  SessionConfig::new( "sh" ).arg( "-c" ).arg( script )
}

/// sess01: a line written to the child comes back through the master.
///
/// `cat` echoes stdin to stdout, and the pty's own line discipline also echoes
/// input, so the payload appears more than once. Asserting containment rather
/// than equality is deliberate — the echo is the terminal behaving like one.
#[ test ]
fn sess01_write_round_trips_through_cat()
{
  let config = SessionConfig::new( "cat" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let mut reader = session.take_reader().expect( "reader already taken" );

  session.write( b"round trip\r" ).expect( "write failed" );

  let seen = read_until( &mut reader, "round trip" );
  assert!( seen.contains( "round trip" ), "payload not echoed back: {seen:?}" );

  // `cat` exits on EOF, and EOF arrives only when the last master descriptor
  // closes — including this one, which `shutdown` cannot reach because
  // `take_reader` gave it away. Dropping it is the caller's obligation.
  drop( reader );
  session.shutdown().expect( "shutdown failed" );
}

/// sess02: the child's controlling terminal is the pty slave.
///
/// This is what an interactive program checks before it will run at all — the
/// reason this crate exists.
#[ test ]
fn sess02_child_has_controlling_terminal()
{
  let config = shell( "tty" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let expected = session.slave_path().to_string();

  let mut reader = session.take_reader().expect( "reader already taken" );
  let out = read_to_eof( &mut reader );
  session.shutdown().expect( "shutdown failed" );

  assert!(
    out.contains( &expected ),
    "`tty` reported {out:?}; expected it to name the slave {expected:?} — \
     the child did not acquire the pty as its controlling terminal",
  );
}

/// sess03: the child leads its own session.
///
/// `setsid()` must run before `TIOCSCTTY`: a process still in its parent's
/// session cannot acquire a new controlling terminal. Field 1 of
/// `/proc/{pid}/stat` is the pid and field 6 is the session id; for a session
/// leader they are equal.
#[ test ]
fn sess03_child_is_session_leader()
{
  let out = output_of( shell( "awk '{ print $1, $6 }' /proc/$$/stat" ) );

  let line = out.lines().find( | l | !l.trim().is_empty() ).unwrap_or_default();
  let mut fields = line.split_whitespace();
  let pid = fields.next().unwrap_or( "pid?" );
  let sid = fields.next().unwrap_or( "sid?" );

  assert_eq!(
    pid, sid,
    "child pid {pid} != session id {sid} — setsid() did not take effect (full output: {out:?})",
  );
}

/// sess04: the configured working directory is where the child starts.
#[ test ]
fn sess04_cwd_is_honoured()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  // The temp root is often itself a symlink; compare against the resolved path
  // the child will actually report.
  let expected = dir.path().canonicalize().expect( "cannot canonicalize temp dir" );

  let out = output_of( shell( "pwd -P" ).cwd( &expected ) );

  assert!(
    out.contains( expected.to_str().expect( "non-UTF-8 temp path" ) ),
    "`pwd -P` reported {out:?}, expected {expected:?}",
  );
}

/// sess05: the window size configured before spawn is what the child sees.
#[ test ]
fn sess05_window_size_reaches_child()
{
  let out = output_of( shell( "stty size" ).win_size( WinSize::new( 40, 132 ) ) );

  assert!( out.contains( "40 132" ), "`stty size` reported {out:?}, expected \"40 132\"" );
}

/// sess06: with no size configured, the child sees the historical 24x80 default.
#[ test ]
fn sess06_default_window_size_is_24x80()
{
  let out = output_of( shell( "stty size" ) );

  assert!( out.contains( "24 80" ), "`stty size` reported {out:?}, expected the 24x80 default" );
}

/// sess07: a `CLAUDE_`-prefixed variable does not reach the child.
///
/// The parent's environment is scrubbed by name at spawn time, so this asserts
/// against a marker matched by the prefix rule rather than by enumeration.
/// Nothing here mutates the test process's own environment, so a parallel test
/// cannot observe a transient global change.
#[ test ]
fn sess07_claude_prefixed_vars_are_scrubbed()
{
  const MARKER : &str = "CLAUDE_PTY_CORE_TEST_MARKER";

  assert!(
    claude_pty_core::env_scrub::is_scrubbed( MARKER ),
    "{MARKER} is not recognized as scrubbable — the CLAUDE_ prefix rule regressed",
  );

  let out = output_of( shell( &format!( "printf 'MARKER=[%s]\\n' \"${{{MARKER}:-}}\"" ) ) );

  assert!(
    out.contains( "MARKER=[]" ),
    "child saw {MARKER} set: {out:?} — CLAUDE_-prefixed variables must be scrubbed",
  );
}

/// sess08: an explicit `env()` entry survives, because it is applied after scrubbing.
///
/// The escape hatch: a caller that genuinely wants a scrubbed name set can set
/// it, and the library does not second-guess that.
#[ test ]
fn sess08_explicit_env_survives_scrubbing()
{
  const MARKER : &str = "CLAUDE_PTY_CORE_DELIBERATE";

  let out = output_of(
    shell( &format!( "printf 'MARKER=[%s]\\n' \"${{{MARKER}:-}}\"" ) ).env( MARKER, "kept" )
  );

  assert!( out.contains( "MARKER=[kept]" ), "explicit env() did not survive scrubbing: {out:?}" );
}

/// sess09: `TERM` describes the pty the child has, not the parent's emulator.
#[ test ]
fn sess09_term_describes_the_pty()
{
  let expected = format!( "TERM=[{}]", claude_pty_core::env_scrub::CHILD_TERM );
  let out = output_of( shell( "printf 'TERM=[%s]\\n' \"${TERM:-}\"" ) );

  assert!( out.contains( &expected ), "child TERM is {out:?}, expected {expected:?}" );
}

/// sess10: the reader is handed out exactly once.
///
/// Two owners of the same master read half would each receive an arbitrary
/// portion of the child's output, which is worse than an error.
#[ test ]
fn sess10_take_reader_yields_once()
{
  let config = SessionConfig::new( "cat" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );

  assert!( session.take_reader().is_some(), "first take_reader returned None" );
  assert!( session.take_reader().is_none(), "second take_reader returned a reader" );

  session.shutdown().expect( "shutdown failed" );
}

/// sess11: the master reaches EOF once the child exits.
///
/// EOF only arrives because `spawn` closed every slave descriptor in the parent.
/// Had any been retained, the terminal would still have a writer and this read
/// would block forever — an exited child would be indistinguishable from a
/// silent live one.
#[ test ]
fn sess11_master_reaches_eof_after_child_exits()
{
  let config = shell( "printf 'done\\n'" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let mut reader = session.take_reader().expect( "reader already taken" );

  let out = read_to_eof( &mut reader );
  assert!( out.contains( "done" ), "child output missing: {out:?}" );

  let status = session.shutdown().expect( "shutdown failed" );
  assert!( status.success(), "child exited with {status:?}" );
}

/// sess12: a nonzero child exit is reported rather than swallowed.
#[ test ]
fn sess12_nonzero_exit_is_reported()
{
  let config = shell( "exit 3" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let mut reader = session.take_reader().expect( "reader already taken" );
  let _ = read_to_eof( &mut reader );

  let status = session.shutdown().expect( "shutdown failed" );
  assert_eq!( status.code(), Some( 3 ), "expected exit code 3, got {status:?}" );
}

/// sess13: a write after shutdown fails rather than vanishing.
///
/// `shutdown` drops the queue's sender, so a later `send` has nowhere to go.
/// Reporting that is the point: input silently discarded by a supervisor is
/// indistinguishable, from the user's side, from a session that ignored them.
#[ test ]
fn sess13_write_after_shutdown_reports_writer_gone()
{
  let config = shell( "exit 0" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let mut reader = session.take_reader().expect( "reader already taken" );
  let _ = read_to_eof( &mut reader );
  session.shutdown().expect( "shutdown failed" );

  assert!(
    session.write( b"too late\r" ).is_err(),
    "write to a shut-down session succeeded — it should report WriterGone",
  );
}

/// sess14: `try_wait` reports a running child as running.
#[ test ]
fn sess14_try_wait_reports_running_child()
{
  let config = SessionConfig::new( "cat" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );

  assert!(
    session.try_wait().expect( "try_wait failed" ).is_none(),
    "try_wait reported a freshly spawned `cat` as exited",
  );

  session.shutdown().expect( "shutdown failed" );
}

/// sess15: `pid()` names a process that actually exists.
#[ test ]
fn sess15_pid_names_a_live_process()
{
  let config = SessionConfig::new( "cat" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );

  let pid = session.pid();
  assert!( pid > 0, "pid() returned {pid}" );
  assert!(
    std::path::Path::new( &format!( "/proc/{pid}" ) ).exists(),
    "/proc/{pid} does not exist for a child that was just spawned",
  );

  session.shutdown().expect( "shutdown failed" );
}

/// sess16: two concurrent sessions do not share identity or input.
#[ test ]
fn sess16_concurrent_sessions_are_independent()
{
  let first_config = SessionConfig::new( "cat" );
  let second_config = SessionConfig::new( "cat" );
  let mut first = PtySession::spawn( &first_config ).expect( "first spawn failed" );
  let mut second = PtySession::spawn( &second_config ).expect( "second spawn failed" );

  assert_ne!( first.slave_path(), second.slave_path(), "concurrent sessions share a slave path" );
  assert_ne!( first.pid(), second.pid(), "concurrent sessions share a pid" );

  let mut first_reader = first.take_reader().expect( "reader already taken" );
  let mut second_reader = second.take_reader().expect( "reader already taken" );

  first.write( b"alpha\r" ).expect( "write failed" );
  second.write( b"beta\r" ).expect( "write failed" );

  let a = read_until( &mut first_reader, "alpha" );
  let b = read_until( &mut second_reader, "beta" );

  assert!( !a.contains( "beta" ), "first session saw the second's input: {a:?}" );
  assert!( !b.contains( "alpha" ), "second session saw the first's input: {b:?}" );

  drop( first_reader );
  drop( second_reader );
  first.shutdown().expect( "first shutdown failed" );
  second.shutdown().expect( "second shutdown failed" );
}

/// sess17: spawning a nonexistent program fails instead of yielding a dead session.
#[ test ]
fn sess17_spawn_of_missing_program_fails()
{
  let config = SessionConfig::new( "claude_pty_core_no_such_program_exists" );

  assert!( PtySession::spawn( &config ).is_err(), "spawning a nonexistent program succeeded" );
}

/// sess18: resizing a live session reaches the child.
#[ test ]
fn sess18_resize_after_spawn_reaches_child()
{
  let config = shell( "read _ ; stty size" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let mut reader = session.take_reader().expect( "reader already taken" );

  session.resize( WinSize::new( 50, 120 ) ).expect( "resize failed" );
  session.write( b"\r" ).expect( "write failed" );

  let out = read_until( &mut reader, "50 120" );
  assert!( out.contains( "50 120" ), "child reported {out:?} after resize" );

  session.shutdown().expect( "shutdown failed" );
}

/// sess19: the slave path a session reports is the one its child sees.
#[ test ]
fn sess19_reported_slave_path_matches_child_view()
{
  let config = shell( "tty" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let reported = session.slave_path().to_string();
  let mut reader = session.take_reader().expect( "reader already taken" );

  let out = read_to_eof( &mut reader );
  session.shutdown().expect( "shutdown failed" );

  assert!( out.contains( &reported ), "session reports slave {reported:?} but child sees {out:?}" );
}

/// sess20: stderr reaches the master too, not only stdout.
///
/// The three slave descriptors are independent, so this also confirms none was
/// closed as a side effect of another.
#[ test ]
fn sess20_stderr_reaches_the_master()
{
  let out = output_of( shell( "printf 'to-stderr\\n' >&2" ) );

  assert!( out.contains( "to-stderr" ), "stderr did not reach the master: {out:?}" );
}

/// sess21: `shutdown` returns for a child that is blocked reading stdin.
///
/// The child only exits when its stdin reaches EOF, and stdin only reaches EOF
/// when the *last* master descriptor closes. A `shutdown` that stops the writer
/// thread but leaves the session's own master open would leave the child's
/// terminal apparently connected, and this call would never return. That is not
/// an edge case: `cat` here stands in for an idle interactive session, which is
/// the state every hosted session spends most of its life in.
///
/// The 10-second bound is the assertion — a regression hangs rather than fails,
/// so the timeout is what turns it back into a failure.
#[ test ]
fn sess21_shutdown_hangs_up_a_child_blocked_on_stdin()
{
  let config = SessionConfig::new( "cat" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let pid = session.pid();

  assert!(
    session.try_wait().expect( "try_wait failed" ).is_none(),
    "`cat` exited before the test could shut it down",
  );

  let started = Instant::now();
  let status = session.shutdown().expect( "shutdown failed" );
  let elapsed = started.elapsed();

  // How the child ends is deliberately not asserted. Losing a terminal races two
  // kernel events: `SIGHUP` to the foreground process group, and `EIO` returned
  // to the `read` the child is blocked in. `cat` dies by the signal when it wins
  // and exits 1 on the I/O error when it does not — both have been observed from
  // this very test on one machine, minutes apart. What the daemon depends on is
  // that `shutdown` returns a status promptly instead of hanging; *which* status
  // is sess12's claim, where draining to `EOF` first leaves the child's own exit
  // as the only possible ending.
  assert!(
    elapsed < READ_TIMEOUT,
    "shutdown of pid {pid} took {elapsed:?} (ended {status:?}) — the master was not closed",
  );
}

/// sess22: resizing after shutdown reports the closed session instead of lying.
///
/// There is no master left to issue `TIOCSWINSZ` against. Returning `Ok` would
/// tell a caller its resize took effect on a terminal that no longer exists.
#[ test ]
fn sess22_resize_after_shutdown_reports_session_closed()
{
  let config = shell( "exit 0" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  session.shutdown().expect( "shutdown failed" );

  match session.resize( WinSize::new( 30, 100 ) )
  {
    Err( claude_pty_core::Error::SessionClosed ) => {}
    other => panic!( "expected SessionClosed, got {other:?}" ),
  }
}

/// sess24: the child inherits the slave and nothing else.
///
/// The direct form of what sess21 tests behaviorally. A master descriptor opened
/// without `O_CLOEXEC` survives `exec`, so the child ends up holding a copy of
/// the master to its own terminal — and then no amount of closing in the parent
/// produces `EOF` on the slave, because the child is keeping its own terminal
/// alive. `/proc/{pid}/fd` shows that directly: an inherited master resolves to
/// `/dev/ptmx`, the device it was allocated from.
///
/// Worth asserting on its own because the behavioral symptom is a hang, and a
/// hang reports as "slow" long before it reports as "wrong".
#[ test ]
fn sess24_child_does_not_inherit_the_master()
{
  let config = SessionConfig::new( "cat" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let pid = session.pid();
  let expected_slave = session.slave_path().to_string();

  let entries = std::fs::read_dir( format!( "/proc/{pid}/fd" ) )
    .expect( "cannot enumerate the child's descriptors" );
  let targets : Vec< String > = entries
    .flatten()
    .filter_map( | entry | std::fs::read_link( entry.path() ).ok() )
    .map( | target | target.display().to_string() )
    .collect();

  assert!( !targets.is_empty(), "the child appears to hold no descriptors at all" );
  assert!(
    !targets.iter().any( | t | t == "/dev/ptmx" ),
    "the child inherited the pty master: {targets:?} — shutdown can never hang it up",
  );
  assert!(
    targets.contains( &expected_slave ),
    "the child does not hold its own slave {expected_slave:?}: {targets:?}",
  );

  session.shutdown().expect( "shutdown failed" );
}

/// sess23: the slave path outlives shutdown, and shutdown is idempotent.
///
/// The path is what a log line or an error message names, so losing it exactly
/// when the session ends would make shutdown the hardest event to attribute. The
/// second `shutdown` returns the status recorded by the first rather than
/// failing on an already-reaped child.
///
/// `cat` rather than a script that exits on its own: a child racing its own exit
/// against the hangup reports 7 on a fast machine and "killed by `SIGHUP`" on a
/// loaded one, and neither is wrong. A child that can only ever end by hangup has
/// one outcome, whatever it turns out to be — and the assertion here is that the
/// second call repeats it, not what it is. sess12 covers the orderly exit code,
/// where draining the master to `EOF` first makes the child's own exit the only
/// possible ending.
#[ test ]
fn sess23_slave_path_survives_shutdown_and_shutdown_repeats()
{
  let config = SessionConfig::new( "cat" );
  let mut session = PtySession::spawn( &config ).expect( "spawn failed" );
  let before = session.slave_path().to_string();

  let first = session.shutdown().expect( "first shutdown failed" );
  let second = session.shutdown().expect( "second shutdown failed" );

  assert_eq!( session.slave_path(), before, "slave path changed across shutdown" );
  assert_eq!( first, second, "repeated shutdown reported {second:?} after {first:?}" );
}
