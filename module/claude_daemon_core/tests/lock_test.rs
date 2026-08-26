//! Single-instance lock tests.
//!
//! Every lock is taken against a real file in a `TempDir` and released by real
//! descriptor closure. `flock` is per open file description, not per process, so
//! a second `acquire` in this same process contends exactly as a second daemon
//! would — which is what makes contention testable without spawning one.
//!
//! ## Specification References
//!
//! - `docs/feature/001_single_instance.md` — why `flock` and not a PID file
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | lock01 | Acquire on a fresh path | `Ok`, and the file exists |
//! | lock02 | Acquire while held | `Err( AlreadyRunning )` naming the path |
//! | lock03 | Acquire after the holder drops | `Ok` |
//! | lock04 | Missing parent directory | Created, not an error |
//! | lock05 | Existing lock file with content | Content preserved |
//! | lock06 | `path()` | The path that was acquired |
//! | lock07 | Two different lock paths | Both held at once |

use std::fs;

use claude_daemon_core::{ acquire, Error };

/// lock01: taking a lock creates the file and reports the path.
#[ test ]
fn lock01_acquire_creates_the_lock_file()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let path = dir.path().join( "instance.lock" );

  let lock = acquire( &path ).expect( "acquire failed on a fresh path" );

  assert!( path.exists(), "lock file was not created" );
  assert_eq!( lock.path(), path, "lock reports a different path than it took" );
}

/// lock02: a held lock refuses a second acquisition immediately.
///
/// Non-blocking is the point. A second daemon's correct response is to talk to
/// the running one, so queueing behind the lock would mean waiting for an event
/// — the first daemon exiting — that the caller does not actually want.
#[ test ]
fn lock02_second_acquire_is_refused()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let path = dir.path().join( "instance.lock" );

  let _held = acquire( &path ).expect( "first acquire failed" );

  match acquire( &path )
  {
    Err( Error::AlreadyRunning { lock_path } ) =>
      assert_eq!( lock_path, path, "AlreadyRunning names the wrong path" ),
    other => panic!( "expected AlreadyRunning, got {other:?}" ),
  }
}

/// lock03: dropping the holder releases the lock.
///
/// This is what a PID file cannot do. The kernel releases an advisory lock when
/// the last descriptor closes — including when the holder is `SIGKILL`ed — so a
/// stale lock cannot outlive its owner.
#[ test ]
fn lock03_lock_is_released_on_drop()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let path = dir.path().join( "instance.lock" );

  let held = acquire( &path ).expect( "first acquire failed" );
  assert!( acquire( &path ).is_err(), "lock was not held" );

  drop( held );

  let _reacquired = acquire( &path ).expect( "acquire after release failed" );
}

/// lock04: the parent directory is created on demand.
///
/// The runtime directory does not exist before the first daemon starts, and
/// failing at that moment would make a clean install look like a fault.
#[ test ]
fn lock04_missing_parent_directory_is_created()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let path = dir.path().join( "-daemon" ).join( "nested" ).join( "instance.lock" );

  let _lock = acquire( &path ).expect( "acquire failed for a missing parent directory" );

  assert!( path.exists(), "lock file was not created under the new directory" );
}

/// lock05: an existing lock file's content survives acquisition.
///
/// The file is opened without truncation. Truncating would discard whatever a
/// holder had written into it at the exact moment a *contending* process tried
/// and failed to take the lock — the one moment that content might be read.
#[ test ]
fn lock05_existing_content_is_not_truncated()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let path = dir.path().join( "instance.lock" );
  fs::write( &path, "pre-existing marker" ).expect( "cannot seed lock file" );

  let _lock = acquire( &path ).expect( "acquire failed on an existing file" );

  let content = fs::read_to_string( &path ).expect( "cannot read lock file" );
  assert_eq!( content, "pre-existing marker", "lock file was truncated" );
}

/// lock06: the lock reports the path it holds.
#[ test ]
fn lock06_path_reports_the_lock_file()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let path = dir.path().join( "named.lock" );

  let lock = acquire( &path ).expect( "acquire failed" );

  assert_eq!( lock.path(), path );
}

/// lock07: the lock is per path, not global.
///
/// Two daemons rooted at different Claude homes are two legitimate instances;
/// the single-instance rule is per home, and a process-global flag would
/// conflate them.
#[ test ]
fn lock07_distinct_paths_lock_independently()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let first = dir.path().join( "one.lock" );
  let second = dir.path().join( "two.lock" );

  let _a = acquire( &first ).expect( "first acquire failed" );
  let _b = acquire( &second ).expect( "second path should lock independently" );
}
