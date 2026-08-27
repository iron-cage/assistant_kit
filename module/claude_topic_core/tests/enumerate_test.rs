//! Enumeration tests — merging two mechanisms that leave nothing in common.
//!
//! Every test builds a real base directory, a real registry, and real session
//! files under a temp `HOME`. Nothing is faked, because the whole question this
//! module answers is what is actually on disk.
//!
//! The cases worth having are the ones about the *pair*. A dir topic and a fork
//! topic of the same name are two topics (ten04), they sort adjacently rather than
//! collapsing (ten05), and only one of them has a session id to resume (ten07). An
//! implementation that keys on the name alone passes nothing here and looks correct
//! in every single-mechanism case.
//!
//! ## Specification References
//!
//! - `docs/feature/002_topic_enumeration.md` — the merge and the live filter
//! - `docs/invariant/002_mode_travels_with_name.md` — why the pair is the unit
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | ten01 | A `-name/` directory that was never entered | Enumerated, not live |
//! | ten02 | A fork topic with a non-empty session file | Live |
//! | ten03 | A fork topic whose session file is empty | Enumerated, not live |
//! | ten04 | One name in both mechanisms | Two rows, one per mode |
//! | ten05 | Several topics | Sorted by name, then by mode |
//! | ten06 | A base that does not exist | Empty, not an error |
//! | ten07 | `session_id` per mode | Some for fork, None for dir |
//! | ten08 | A registry entry whose file was deleted | Listed with zero sessions |

use std::path::Path;

use claude_topic_core::{ enumerate, enumerate_live, topic_dir, TopicMode };
use tempfile::TempDir;

/// Serializes the tests that mutate process-wide env vars.
///
/// `std::env::set_var` / `remove_var` are not thread-safe across concurrent tests.
/// Every test here redirects `HOME` and `CLR_TOPIC_REGISTRY_DIR`, so every test
/// holds this lock for its whole body.
static ENV_LOCK : std::sync::Mutex< () > = std::sync::Mutex::new( () );

/// A base directory with its own private `HOME` and registry root.
///
/// The temp dirs are fields rather than locals so they are deleted when the test
/// ends, not when the constructor returns.
struct Sandbox
{
  _home : TempDir,
  _registry : TempDir,
  base : TempDir,
}

impl Sandbox
{
  /// Redirect storage and the registry into fresh temp roots.
  fn new() -> Self
  {
    let home = TempDir::new().unwrap();
    let registry = TempDir::new().unwrap();
    std::env::remove_var( "CLAUDE_HOME" );
    std::env::remove_var( "CLAUDE_COWORK_MEMORY_PATH_OVERRIDE" );
    std::env::set_var( "HOME", home.path() );
    std::env::set_var( "CLR_TOPIC_REGISTRY_DIR", registry.path() );
    Self { _home : home, _registry : registry, base : TempDir::new().unwrap() }
  }

  /// The base directory topics are enumerated under.
  fn base( &self ) -> &Path
  {
    self.base.path()
  }

  /// Create a dir-mode topic, giving it `sessions` session files in its own storage.
  fn dir_topic( &self, name : &str, sessions : usize )
  {
    let dir = topic_dir( self.base(), name );
    std::fs::create_dir_all( &dir ).unwrap();
    let storage = claude_storage_core::scope_for( &dir ).claude_session_dir;
    std::fs::create_dir_all( &storage ).unwrap();
    for index in 0 .. sessions
    {
      std::fs::write( storage.join( format!( "0000000{index}-aaaa-4aaa-8aaa-aaaaaaaaaaaa.jsonl" ) ), "{}\n" ).unwrap();
    }
  }

  /// Register a fork-mode topic, writing `body` into its session file. An empty
  /// `body` writes the file but leaves it zero-length; `None` skips the file
  /// entirely, which is the deleted-session case.
  fn fork_topic( &self, name : &str, body : Option< &str > )
  {
    let canonical = claude_storage_core::physical_abs( self.base() );
    claude_topic_core::registry::record( &canonical, name );
    if let Some( body ) = body
    {
      let file = claude_topic_core::fork_session_file( self.base(), name ).unwrap();
      std::fs::create_dir_all( file.parent().unwrap() ).unwrap();
      std::fs::write( &file, body ).unwrap();
    }
  }
}

/// ten01: a topic directory nobody has run in has no conversation to continue, so
/// addressing it would create one rather than continue it.
#[ test ]
fn ten01_unentered_directory_is_listed_but_not_live()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let sandbox = Sandbox::new();
  sandbox.dir_topic( "review", 0 );

  let all = enumerate( sandbox.base() );
  assert_eq!( all.len(), 1 );
  assert_eq!( all[ 0 ].name, "review" );
  assert_eq!( all[ 0 ].mode, TopicMode::Dir );
  assert_eq!( all[ 0 ].sessions, 0 );

  assert!( enumerate_live( sandbox.base() ).is_empty() );
}

/// ten02: a fork topic with a conversation behind it is addressable.
#[ test ]
fn ten02_fork_topic_with_a_session_is_live()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let sandbox = Sandbox::new();
  sandbox.fork_topic( "review", Some( "{\"type\":\"user\"}\n" ) );

  let live = enumerate_live( sandbox.base() );
  assert_eq!( live.len(), 1 );
  assert_eq!( live[ 0 ].name, "review" );
  assert_eq!( live[ 0 ].mode, TopicMode::Fork );
  assert_eq!( live[ 0 ].sessions, 1 );
}

/// ten03: an existing but empty session file is a file, not a conversation.
#[ test ]
fn ten03_fork_topic_with_an_empty_session_is_not_live()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let sandbox = Sandbox::new();
  sandbox.fork_topic( "review", Some( "" ) );

  assert_eq!( enumerate( sandbox.base() ).len(), 1 );
  assert!( enumerate_live( sandbox.base() ).is_empty() );
}

/// ten04: the same name in both mechanisms is two topics, and collapsing them
/// would make the fork one permanently unreachable.
#[ test ]
fn ten04_one_name_in_both_modes_is_two_topics()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let sandbox = Sandbox::new();
  sandbox.dir_topic( "review", 1 );
  sandbox.fork_topic( "review", Some( "{}\n" ) );

  let live = enumerate_live( sandbox.base() );
  assert_eq!( live.len(), 2, "one name, two mechanisms, two topics" );
  assert_eq!( live[ 0 ].mode, TopicMode::Dir );
  assert_eq!( live[ 1 ].mode, TopicMode::Fork );
  assert_eq!( live[ 0 ].name, live[ 1 ].name );
  assert_ne!( live[ 0 ].path, live[ 1 ].path );
}

/// ten05: name first, mode second — a stable order that keeps a name's two
/// mechanisms adjacent.
#[ test ]
fn ten05_sorted_by_name_then_mode()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let sandbox = Sandbox::new();
  sandbox.dir_topic( "beta", 0 );
  sandbox.dir_topic( "alpha", 0 );
  sandbox.fork_topic( "alpha", Some( "{}\n" ) );

  let listed : Vec< _ > = enumerate( sandbox.base() )
    .into_iter()
    .map( | t | format!( "{}:{}", t.name, t.mode ) )
    .collect();
  assert_eq!( listed, vec![ "alpha:dir", "alpha:fork", "beta:dir" ] );
}

/// ten06: the global topic home legitimately does not exist until the first global
/// topic is created, so a missing base is an empty list rather than an error.
#[ test ]
fn ten06_missing_base_enumerates_empty()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let sandbox = Sandbox::new();

  assert!( enumerate( &sandbox.base().join( "nope" ) ).is_empty() );
}

/// ten07: only a fork topic has a name-derived id to resume; a dir topic's
/// sessions carry ordinary Claude-generated ids that no formula predicts.
#[ test ]
fn ten07_session_id_is_present_only_for_fork_topics()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let sandbox = Sandbox::new();
  sandbox.dir_topic( "review", 1 );
  sandbox.fork_topic( "review", Some( "{}\n" ) );

  let live = enumerate_live( sandbox.base() );
  assert_eq!( live[ 0 ].session_id(), None, "dir topic has no derived id" );

  let id = live[ 1 ].session_id().expect( "fork topic has a derived id" );
  let expected = claude_storage_core::topic_session_id
  (
    &claude_storage_core::physical_abs( sandbox.base() ),
    "review",
  ).unwrap();
  assert_eq!( id, expected.as_str() );
}

/// ten08: the registry outlives the sessions it names — the entry stays listed so
/// the name is still visibly taken, with zero sessions saying why it is not live.
#[ test ]
fn ten08_registry_entry_without_a_file_lists_with_zero_sessions()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let sandbox = Sandbox::new();
  sandbox.fork_topic( "review", None );

  let all = enumerate( sandbox.base() );
  assert_eq!( all.len(), 1 );
  assert_eq!( all[ 0 ].sessions, 0 );
  assert!( enumerate_live( sandbox.base() ).is_empty() );
}
