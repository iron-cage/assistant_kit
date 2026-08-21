//! Unit tests for the deterministic topic-session UUID rule (`topic_session_id`,
//! `topic_session_file`).
//!
//! ## Purpose
//!
//! The topic→UUID rule is a cross-binary contract: `clr` (fork-based topics) and
//! `claude_storage` (`.session.path topic::`) must resolve the same
//! `(canonical base, topic name)` pair to the same session UUID with zero
//! coordination. These tests pin the rule to golden vectors computed by two
//! INDEPENDENT external implementations (`uuidgen --sha1` and Python's
//! `uuid.uuid5`), so any drift in the hand-written SHA-1/UUIDv5 code — or any
//! accidental change to the namespace constant or the NUL-separated name
//! layout — fails loudly against externally-reproducible expectations rather
//! than against the implementation's own output.
//!
//! ## Reproducing the vectors
//!
//! ```sh
//! uuidgen --sha1 --namespace @dns --name clr.topic
//! python3 -c 'import uuid; ns = uuid.uuid5( uuid.NAMESPACE_DNS, "clr.topic" ); \
//!   print( uuid.uuid5( ns, "/home/user1/pro\0review" ) )'
//! ```

use std::path::Path;
use claude_storage_core::{ topic_session_id, topic_session_file };

/// Golden vector 1: the doc-comment example pair, cross-checked with uuidgen + Python.
#[ test ]
fn golden_vector_home_pro_review()
{
  let id = topic_session_id( Path::new( "/home/user1/pro" ), "review" ).unwrap();
  assert_eq!( id.as_str(), "e36d752a-341e-5db1-94c5-c8b91cccbfff" );
}

/// Golden vector 2: short path and single-character topic.
#[ test ]
fn golden_vector_tmp_x_a()
{
  let id = topic_session_id( Path::new( "/tmp/x" ), "a" ).unwrap();
  assert_eq!( id.as_str(), "41299c24-a8f5-589f-9fce-8474fc855532" );
}

/// Golden vector 3: a disambiguation-suffixed topic name hashes to an unrelated UUID.
#[ test ]
fn golden_vector_suffixed_topic()
{
  let id = topic_session_id( Path::new( "/home/user1/pro" ), "review-2" ).unwrap();
  assert_eq!( id.as_str(), "ec8f75ea-cf5e-5302-9327-1b5f15644864" );
}

/// The rule is a pure function: same inputs always yield the same UUID.
#[ test ]
fn deterministic_across_calls()
{
  let first  = topic_session_id( Path::new( "/tmp/x" ), "a" ).unwrap();
  let second = topic_session_id( Path::new( "/tmp/x" ), "a" ).unwrap();
  assert_eq!( first, second );
}

/// Distinct bases and distinct topics each produce distinct UUIDs.
#[ test ]
fn distinct_inputs_distinct_uuids()
{
  let base_a = topic_session_id( Path::new( "/tmp/x" ), "a" ).unwrap();
  let base_b = topic_session_id( Path::new( "/tmp/y" ), "a" ).unwrap();
  let topic_b = topic_session_id( Path::new( "/tmp/x" ), "b" ).unwrap();
  assert_ne!( base_a, base_b );
  assert_ne!( base_a, topic_b );
}

/// The NUL separator makes the (path, name) concatenation unambiguous: moving a
/// character across the boundary changes the hash input, so `("/a", "bc")` and
/// `("/a/b", "c")` — identical when naively concatenated with `/` — differ.
#[ test ]
fn nul_separator_prevents_boundary_collisions()
{
  let split_one = topic_session_id( Path::new( "/a" ), "bc" ).unwrap();
  let split_two = topic_session_id( Path::new( "/a/b" ), "c" ).unwrap();
  assert_ne!( split_one, split_two );
}

/// The emitted format is a canonical lowercase hyphenated UUID with the v5
/// version nibble and RFC 4122 variant bits.
#[ test ]
fn output_is_canonical_v5_uuid()
{
  let id = topic_session_id( Path::new( "/tmp/x" ), "anything" ).unwrap();
  let s = id.as_str();
  assert_eq!( s.len(), 36 );
  let dash_positions : Vec< usize > =
    s.char_indices().filter( | ( _, c ) | *c == '-' ).map( | ( i, _ ) | i ).collect();
  assert_eq!( dash_positions, vec![ 8, 13, 18, 23 ] );
  assert!( s.chars().all( | c | c == '-' || c.is_ascii_lowercase() || c.is_ascii_digit() ) );
  assert_eq!( &s[ 14 ..15 ], "5", "version nibble must be 5" );
  let variant = s.as_bytes()[ 19 ];
  assert!(
    matches!( variant, b'8' | b'9' | b'a' | b'b' ),
    "variant nibble must be RFC 4122 (8/9/a/b), got {}",
    variant as char
  );
}

/// `topic_session_file` composes the storage dir with `<uuid>.jsonl` — pin the
/// file-name half against golden vector 1 (the directory half is
/// `to_storage_path_for`'s contract, covered by `continuation_tests.rs`).
#[ test ]
fn session_file_ends_with_uuid_jsonl()
{
  // HOME is always set in the test environment; the storage prefix itself is
  // environment-dependent, so assert only the composed file name.
  let file = topic_session_file( Path::new( "/home/user1/pro" ), "review" ).unwrap();
  assert!(
    file.to_str().unwrap().ends_with( "/e36d752a-341e-5db1-94c5-c8b91cccbfff.jsonl" ),
    "unexpected session file path: {}",
    file.display()
  );
  assert!( file.is_absolute() );
}
