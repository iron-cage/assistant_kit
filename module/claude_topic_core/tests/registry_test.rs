//! Registry tests — the side channel that makes fork-mode names listable.
//!
//! Every test points `CLR_TOPIC_REGISTRY_DIR` at a temp directory, so nothing here
//! reads or writes the real `~/.clr/topics/`. Base paths are literals rather than
//! temp directories: the registry keys on the encoded path and never touches it,
//! so a path that does not exist is a perfectly good key and keeps the assertions
//! exact.
//!
//! The cases worth having are the ones where the failure is silent. Recording is
//! warn-never-fatal by design, which means a bug in it produces a topic that works
//! and cannot be listed — trg03 pins the one input that is genuinely refused, and
//! trg02 pins the append-if-missing rule that keeps a repeat run from growing the
//! file without bound.
//!
//! ## Specification References
//!
//! - `docs/feature/002_topic_enumeration.md` — why the registry exists at all
//! - `docs/invariant/001_registry_non_authoritative.md` — what it is not
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | trg01 | Record then list | The name comes back |
//! | trg02 | Record the same name twice | Listed once |
//! | trg03 | A name containing a newline | Refused, not listed |
//! | trg04 | List a base with nothing recorded | Empty |
//! | trg05 | Blank lines in the file | Skipped |
//! | trg06 | Several names | First-recorded order |
//! | trg07 | Two different bases | Independent files |

use std::path::Path;

use claude_topic_core::registry::{ list, record };
use tempfile::TempDir;

/// Serializes the tests that mutate process-wide env vars.
///
/// `std::env::set_var` / `remove_var` are not thread-safe across concurrent tests.
/// Every test here mutates `CLR_TOPIC_REGISTRY_DIR`, so every test holds this lock.
static ENV_LOCK : std::sync::Mutex< () > = std::sync::Mutex::new( () );

/// Point the registry at a fresh temp root, returning it so it outlives the test.
fn isolated_root() -> TempDir
{
  let root = TempDir::new().unwrap();
  std::env::set_var( "CLR_TOPIC_REGISTRY_DIR", root.path() );
  root
}

/// trg01: a recorded name is a listed name.
#[ test ]
fn trg01_recorded_name_is_listed()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_root();

  record( Path::new( "/base" ), "review" );

  assert_eq!( list( Path::new( "/base" ) ), vec![ "review".to_string() ] );
}

/// trg02: append-if-missing — a repeat run must not grow the file.
#[ test ]
fn trg02_recording_twice_lists_once()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_root();

  record( Path::new( "/base" ), "review" );
  record( Path::new( "/base" ), "review" );

  assert_eq!( list( Path::new( "/base" ) ), vec![ "review".to_string() ] );
}

/// trg03: a newline would corrupt the one-name-per-line format, so it is refused —
/// and refusing it is why such a topic cannot be reached by name later.
#[ test ]
fn trg03_newline_in_a_name_is_refused()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_root();

  record( Path::new( "/base" ), "one\ntwo" );

  assert!( list( Path::new( "/base" ) ).is_empty() );
}

/// trg04: no fork topics recorded is an ordinary state, not an error.
#[ test ]
fn trg04_unknown_base_lists_empty()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_root();

  assert!( list( Path::new( "/never/recorded" ) ).is_empty() );
}

/// trg05: a blank line is not a topic named "".
#[ test ]
fn trg05_blank_lines_are_skipped()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let root = isolated_root();

  record( Path::new( "/base" ), "review" );
  let file = root.path().join( claude_storage_core::encode_path( Path::new( "/base" ) ).unwrap() );
  let body = std::fs::read_to_string( &file ).unwrap();
  std::fs::write( &file, format!( "\n{body}\n\n" ) ).unwrap();

  assert_eq!( list( Path::new( "/base" ) ), vec![ "review".to_string() ] );
}

/// trg06: order is first-recorded-first, so a listing is stable across runs.
#[ test ]
fn trg06_order_is_first_recorded_first()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_root();

  record( Path::new( "/base" ), "zebra" );
  record( Path::new( "/base" ), "apple" );

  assert_eq!( list( Path::new( "/base" ) ), vec![ "zebra".to_string(), "apple".to_string() ] );
}

/// trg07: one file per base — a topic of one base is invisible to another.
#[ test ]
fn trg07_bases_are_independent()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let _root = isolated_root();

  record( Path::new( "/one" ), "review" );
  record( Path::new( "/two" ), "release" );

  assert_eq!( list( Path::new( "/one" ) ), vec![ "review".to_string() ] );
  assert_eq!( list( Path::new( "/two" ) ), vec![ "release".to_string() ] );
}
