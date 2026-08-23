//! Unit tests for `src/cli/tail.rs` — session-id shortening.
//!
//! Relocated out of a `#[ cfg( test ) ]` module in the source file: every test
//! in this crate lives under `tests/`. `claude_storage::cli::tail` is
//! `#[ doc( hidden ) ] pub` for exactly this purpose (see `src/cli/mod.rs`).

use claude_storage::cli::tail::short_session_id;

#[test]
fn test_short_session_id_truncates_to_eight()
{
  assert_eq!( short_session_id( "bff63952-8a23-4794-ad56-3a8e4fc4e9a9" ), "bff63952" );
  assert_eq!( short_session_id( "abc" ), "abc" );
}
