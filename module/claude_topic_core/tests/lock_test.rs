//! Lock tests — keeping two writers off one conversation.
//!
//! Every test points `CLR_TOPIC_LOCK_DIR` at a temp directory, so nothing here
//! touches a lock any other process might be holding. Topic paths are literals:
//! the lock keys on the encoded path and never opens it, so the path need not
//! exist.
//!
//! The cases worth having are the reclaim ones. Exclusion is easy to get right and
//! easy to test; what makes an advisory lock usable in practice is that a crashed
//! owner does not wedge the topic forever (tlk03) — and what makes reclaim
//! dangerous is doing it to an owner that is merely quiet rather than gone
//! (tlk07). Both directions are asserted, because getting one right by breaking
//! the other is the obvious way to fail here.
//!
//! ## Specification References
//!
//! - `docs/feature/005_topic_lock.md` — scope, reclaim, and the window it leaves
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | tlk01 | Lock a topic, then lock it again | Second is denied |
//! | tlk02 | Drop the guard, then lock again | Granted |
//! | tlk03 | A lock file naming a dead pid | Reclaimed |
//! | tlk04 | A lock file with unparseable content | Reclaimed |
//! | tlk05 | Two topics | Distinct lock files |
//! | tlk06 | `CLR_TOPIC_LOCK` | Read as the run-path opt-in |
//! | tlk07 | A lock file naming a live pid | Denied, naming that pid |

use std::path::PathBuf;

use claude_topic_core::{ lock_file, try_lock, LockDenied, Topic, TopicMode };
use tempfile::TempDir;

/// Serializes the tests that mutate process-wide env vars.
///
/// `std::env::set_var` / `remove_var` are not thread-safe across concurrent tests.
/// Every test here redirects `CLR_TOPIC_LOCK_DIR`, so every test holds this lock.
static ENV_LOCK : std::sync::Mutex< () > = std::sync::Mutex::new( () );

/// Point locking at a fresh temp root, returning it so it outlives the test.
fn isolated_lock_dir() -> TempDir
{
  let root = TempDir::new().unwrap();
  std::env::set_var( "CLR_TOPIC_LOCK_DIR", root.path() );
  root
}

/// A fork topic whose session file is `/no-such-storage/<name>.jsonl`.
fn topic( name : &str ) -> Topic
{
  Topic
  {
    name : name.to_string(),
    mode : TopicMode::Fork,
    path : PathBuf::from( format!( "/no-such-storage/{name}.jsonl" ) ),
    sessions : 1,
  }
}

/// tlk01: one holder at a time — this is the whole point.
#[ test ]
fn tlk01_second_lock_on_a_held_topic_is_denied()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_lock_dir();
  let subject = topic( "review" );

  let held = try_lock( &subject ).expect( "first lock" );
  assert!( held.path().exists() );

  match try_lock( &subject )
  {
    Err( LockDenied::Held( pid ) ) => assert_eq!( pid, std::process::id() ),
    other => panic!( "expected Held, got {other:?}" ),
  }
}

/// tlk02: releasing on drop is what makes the guard usable in ordinary control
/// flow rather than something a caller has to remember to undo.
#[ test ]
fn tlk02_dropping_the_guard_releases_the_topic()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_lock_dir();
  let subject = topic( "review" );

  let path =
  {
    let held = try_lock( &subject ).expect( "first lock" );
    held.path().to_path_buf()
  };
  assert!( !path.exists(), "drop must remove the lock file" );

  try_lock( &subject ).expect( "the topic is free again" );
}

/// tlk03: drop does not run on `SIGKILL`, so a lock left by a process that is gone
/// must not wedge the topic forever.
#[ test ]
fn tlk03_a_dead_owners_lock_is_reclaimed()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_lock_dir();
  let subject = topic( "review" );

  // PID 0 never appears in /proc, so this owner is unambiguously not running.
  let path = lock_file( &subject ).unwrap();
  std::fs::write( &path, "0 12345" ).unwrap();

  let held = try_lock( &subject ).expect( "a dead owner's lock is reclaimable" );
  assert_eq!( held.path(), path );
  assert_eq!( std::fs::read_to_string( &path ).unwrap().split( ' ' ).next().unwrap(), std::process::id().to_string() );
}

/// tlk04: content that cannot be attributed to any process cannot be respected
/// either, so it is reclaimed on the same terms as a dead owner's.
#[ test ]
fn tlk04_an_unparseable_lock_is_reclaimed()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_lock_dir();
  let subject = topic( "review" );

  let path = lock_file( &subject ).unwrap();
  std::fs::write( &path, "not a pid at all" ).unwrap();

  try_lock( &subject ).expect( "unattributable content is reclaimable" );
}

/// tlk05: the lock keys on the topic's own resolved path, so two topics never
/// share one.
#[ test ]
fn tlk05_distinct_topics_get_distinct_lock_files()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_lock_dir();

  let one = lock_file( &topic( "review" ) ).unwrap();
  let two = lock_file( &topic( "release" ) ).unwrap();
  assert_ne!( one, two );

  let _held = try_lock( &topic( "review" ) ).expect( "first" );
  try_lock( &topic( "release" ) ).expect( "a different topic is unaffected" );
}

/// tlk06: the ordinary run path stays unlocked unless explicitly opted in — see
/// the module docs for why that default is deliberate.
#[ test ]
fn tlk06_run_path_opt_in_reads_the_environment_switch()
{
  let _guard = ENV_LOCK.lock().unwrap();

  std::env::remove_var( "CLR_TOPIC_LOCK" );
  assert!( !claude_topic_core::enabled_for_run_path() );

  std::env::set_var( "CLR_TOPIC_LOCK", "1" );
  assert!( claude_topic_core::enabled_for_run_path() );

  std::env::set_var( "CLR_TOPIC_LOCK", "true" );
  assert!( claude_topic_core::enabled_for_run_path() );

  std::env::set_var( "CLR_TOPIC_LOCK", "0" );
  assert!( !claude_topic_core::enabled_for_run_path() );

  std::env::remove_var( "CLR_TOPIC_LOCK" );
}

/// tlk07: the other half of tlk03 — a live owner keeps its lock, and is named so
/// the caller can say who has it. A record with no start time is the
/// starttime-unavailable form, which must still read as alive.
#[ test ]
fn tlk07_a_live_owners_lock_is_respected()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_lock_dir();
  let subject = topic( "review" );

  let path = lock_file( &subject ).unwrap();
  std::fs::write( &path, std::process::id().to_string() ).unwrap();

  match try_lock( &subject )
  {
    Err( LockDenied::Held( pid ) ) => assert_eq!( pid, std::process::id() ),
    other => panic!( "a live owner must keep its lock, got {other:?}" ),
  }
  assert!( path.exists(), "the live owner's lock file must survive" );
}
