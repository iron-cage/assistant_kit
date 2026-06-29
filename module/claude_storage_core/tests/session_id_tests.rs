//! Unit tests for `SessionId` newtype.
//!
//! Tests the typed wrapper around the UUID stem of a `.jsonl` session filename.

use claude_storage_core::SessionId;

#[ test ]
fn session_id_new_and_as_str()
{
  let id = SessionId::new( "abc-def" );
  assert_eq!( id.as_str(), "abc-def" );
}

#[ test ]
fn session_id_display_matches_inner()
{
  let id = SessionId::new( "abc-def" );
  assert_eq!( format!( "{id}" ), "abc-def" );
}

#[ test ]
fn session_id_clone_equality()
{
  let id = SessionId::new( "abc-def" );
  assert_eq!( id.clone(), id );
}

#[ test ]
fn session_id_from_string_and_str()
{
  let from_str    : SessionId = "xyz".into();
  let from_string : SessionId = String::from( "xyz" ).into();
  assert_eq!( from_str, from_string );
  assert_eq!( from_str, SessionId::new( "xyz" ) );
}
