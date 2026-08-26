//! Liveness tests — real processes, including a real zombie and a real thread id.
//!
//! Every clause of [`claude_session_core::pid_alive`] was paid for by a
//! production bug (BUG-479, BUG-488). Each one is reproduced here against an
//! actual `/proc` occupant rather than a fabricated string, because the whole
//! point of the predicate is that the obvious `/proc/{pid}` existence check
//! agrees with it right up until it matters.
//!
//! ## Specification References
//!
//! - `docs/invariant/001_liveness_four_clauses.md` — the four clauses
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | live01 | This test process | `true` |
//! | live02 | A pid number no process can hold | `false` — clause (a) |
//! | live03 | A real unreaped zombie | `false` — clause (b) |
//! | live04 | A real non-leader thread id | `false` — clause (c) |
//! | live05 | Recorded start time matches / does not | `true` / `false` — clause (d) |
//! | live06 | Recorded start time absent | `true` — clause (d) inert |
//! | live07 | `comm` containing `)` and spaces | Parsed from the LAST `)` |
//! | live08 | `proc_starttime` of a dead pid | `None` |
//! | live09 | `proc_starttime` is stable | Same value across calls |

use core::time::Duration;
use std::fs;
use std::path::{ Path, PathBuf };
use std::process::Command;
use std::time::Instant;

use claude_session_core::{ pid_alive, proc_starttime };

/// Longest a test waits for a spawned process to reach an expected state.
const STATE_TIMEOUT : Duration = Duration::from_secs( 10 );

/// A pid number no live process can hold — the highest `u32`.
///
/// Linux caps pids well below this (`/proc/sys/kernel/pid_max`), so `/proc` can
/// never have an entry for it. That makes it a stable stand-in for "gone"
/// without racing a real pid that might be reused mid-test.
const IMPOSSIBLE_PID : u32 = u32::MAX;

/// Locate a standard utility, failing loudly rather than silently skipping.
fn program( name : &str ) -> PathBuf
{
  for dir in [ "/bin", "/usr/bin" ]
  {
    let candidate = Path::new( dir ).join( name );
    if candidate.exists()
    {
      return candidate;
    }
  }
  panic!( "{name} not found in /bin or /usr/bin — the test environment lacks a required utility" );
}

/// The single-character state field of `/proc/{pid}/stat`, read the same way the
/// crate reads it.
fn proc_state( pid : u32 ) -> Option< char >
{
  let stat = fs::read_to_string( format!( "/proc/{pid}/stat" ) ).ok()?;
  stat.rsplit_once( ')' )
    .and_then( | ( _, rest ) | rest.trim_start().chars().next() )
}

/// Poll until `pid` reports `state`, or fail after [`STATE_TIMEOUT`].
fn wait_for_state( pid : u32, state : char )
{
  let deadline = Instant::now() + STATE_TIMEOUT;
  loop
  {
    if proc_state( pid ) == Some( state )
    {
      return;
    }
    assert!(
      Instant::now() < deadline,
      "pid {pid} never reached state {state:?}; last seen {:?}",
      proc_state( pid ),
    );
    std::thread::sleep( Duration::from_millis( 10 ) );
  }
}

/// live01: the process running this test is alive by every clause.
#[ test ]
fn live01_this_process_is_alive()
{
  let pid = std::process::id();

  assert!( pid_alive( pid, None ), "this process reported itself dead" );
}

/// live02: clause (a) — no `/proc` entry at all.
#[ test ]
fn live02_impossible_pid_is_not_alive()
{
  assert!( !pid_alive( IMPOSSIBLE_PID, None ), "an unreachable pid reported alive" );
}

/// live03: clause (b) — an exited-but-unreaped child.
///
/// This is BUG-479 reproduced. The child has exited; its `/proc/{pid}` directory
/// is still there because this process has not called `wait()`. A bare existence
/// probe reads it as running, which is exactly how every dead slot owner became
/// permanent under a non-reaping supervisor.
#[ test ]
fn live03_zombie_is_not_alive()
{
  let mut child = Command::new( program( "sh" ) )
    .arg( "-c" )
    .arg( "exit 0" )
    .spawn()
    .expect( "cannot spawn child" );
  let pid = child.id();

  wait_for_state( pid, 'Z' );

  // The directory is still there — this is the trap.
  assert!(
    Path::new( &format!( "/proc/{pid}" ) ).exists(),
    "test premise broken: the zombie's /proc entry is already gone",
  );
  assert!( !pid_alive( pid, None ), "an unreaped zombie reported alive" );

  child.wait().expect( "cannot reap child" );
}

/// live04: clause (c) — a non-leader thread id.
///
/// This is BUG-488's first half. Linux resolves a direct `/proc/<tid>` lookup for
/// a thread id that `readdir` never lists, so a number belonging to some other
/// process's worker thread passes clauses (a) and (b) unchanged. The thread here
/// belongs to this very process, which makes the distinction unmistakable: the
/// number is live, and it is still not the recorded process.
#[ test ]
fn live04_non_leader_thread_id_is_not_alive()
{
  let ( release, parked ) = std::sync::mpsc::channel::< () >();
  let ( ready, started ) = std::sync::mpsc::channel::< () >();
  let thread = std::thread::spawn( move ||
  {
    ready.send( () ).expect( "cannot signal readiness" );
    // Blocks until `release` is dropped, keeping the thread id occupied.
    let _ = parked.recv();
  } );
  started.recv().expect( "worker thread never started" );

  let pid = std::process::id();
  let tid = fs::read_dir( "/proc/self/task" )
    .expect( "cannot enumerate this process's threads" )
    .flatten()
    .filter_map( | entry | entry.file_name().to_str()?.parse::< u32 >().ok() )
    .find( | candidate | *candidate != pid )
    .expect( "no non-leader thread id found — the worker thread did not register" );

  assert!(
    fs::read_to_string( format!( "/proc/{tid}/stat" ) ).is_ok(),
    "test premise broken: the thread id has no /proc entry to be fooled by",
  );
  assert!( !pid_alive( tid, None ), "a non-leader thread id reported alive" );

  drop( release );
  thread.join().expect( "worker thread panicked" );
}

/// live05, live06: clause (d) — the incarnation check, and its inert form.
///
/// This is BUG-488's second half. A pid number outlives the process that held it;
/// binding a record to `( pid, starttime )` is what makes it name a process
/// rather than a number. Absence of the recorded value is deliberately *not* a
/// mismatch, so a record written before the field existed keeps the earlier
/// semantics instead of mass-reclaiming live sessions.
#[ test ]
fn live05_start_time_identifies_the_incarnation()
{
  let pid = std::process::id();
  let start = proc_starttime( pid ).expect( "cannot read this process's start time" );

  assert!( pid_alive( pid, Some( start ) ), "the correct start time was rejected" );
  assert!( !pid_alive( pid, Some( start + 1 ) ), "a wrong start time was accepted" );
  assert!( pid_alive( pid, None ), "a record without a start time should stay usable" );
}

/// live07: the state field follows the LAST `)`, not the first.
///
/// `/proc/{pid}/stat` puts the executable name in field 2, unquoted and
/// unescaped, so a program named `x) Z 9 9` produces
/// `PID (x) Z 9 9) S ...`. Splitting on the first `)` reads the state as `Z` and
/// declares a healthy process dead. The binary here is a real copy of `sleep`
/// under that name — an actual occupant of `/proc`, not a synthetic string.
#[ test ]
fn live07_comm_containing_parenthesis_is_parsed_correctly()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  // Under 15 characters, so the kernel does not truncate away the closing paren.
  let disguised = dir.path().join( "x) Z 9 9" );
  fs::copy( program( "sleep" ), &disguised ).expect( "cannot copy sleep binary" );

  let mut child = Command::new( &disguised )
    .arg( "30" )
    .spawn()
    .expect( "cannot spawn the renamed binary" );
  let pid = child.id();

  let stat = fs::read_to_string( format!( "/proc/{pid}/stat" ) ).expect( "cannot read stat" );
  assert!(
    stat.contains( "(x) Z 9 9)" ),
    "test premise broken: comm was not recorded as expected: {stat:?}",
  );
  assert!(
    pid_alive( pid, None ),
    "a live process was read as a zombie because its name contains ')': {stat:?}",
  );

  child.kill().expect( "cannot kill child" );
  child.wait().expect( "cannot reap child" );
}

/// live08, live09: `proc_starttime` reports only for processes that exist, and
/// does not drift.
#[ test ]
fn live08_proc_starttime_absent_for_dead_pid_and_stable_for_live_one()
{
  assert_eq!( proc_starttime( IMPOSSIBLE_PID ), None, "a start time was invented for a dead pid" );

  let pid = std::process::id();
  let first = proc_starttime( pid ).expect( "cannot read this process's start time" );
  std::thread::sleep( Duration::from_millis( 20 ) );
  let second = proc_starttime( pid ).expect( "cannot read this process's start time" );

  assert_eq!( first, second, "start time changed between reads — it must be fixed for a process's life" );
}
