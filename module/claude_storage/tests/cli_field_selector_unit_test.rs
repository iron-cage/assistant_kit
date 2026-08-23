//! Unit tests for `src/cli/field_selector.rs` — the `fields::` selector.
//!
//! Relocated out of a `#[ cfg( test ) ]` module in the source file: every test
//! in this crate lives under `tests/`. `claude_storage::cli::field_selector` is
//! `#[ doc( hidden ) ] pub` for exactly this purpose (see `src/cli/mod.rs`).
//!
//! Cases mirror `docs/cli/type/15_field_selector.md` TC-1 … TC-9.

use claude_storage::cli::field_selector::{ CANONICAL_FIELDS, FieldSelector };

#[ test ]
fn tc1_single_valid_token_accepted()
{
  let sel = FieldSelector::parse( "timestamp" ).unwrap();
  assert_eq!( sel.fields(), vec![ "timestamp" ] );
}

#[ test ]
fn tc2_multi_token_request_order_preserved()
{
  let sel = FieldSelector::parse( "uuid,model" ).unwrap();
  assert_eq!( sel.fields(), vec![ "uuid", "model" ] );
}

#[ test ]
fn tc3_all_expands_to_full_canonical_vocabulary()
{
  let sel = FieldSelector::parse( "all" ).unwrap();
  assert_eq!( sel.fields(), CANONICAL_FIELDS.to_vec() );
}

#[ test ]
fn tc4_invalid_token_rejected_lists_all_18_names()
{
  let err = FieldSelector::parse( "bogus" ).unwrap_err();
  assert_eq!(
    err,
    "unknown field 'bogus' — valid fields: uuid, parent_uuid, timestamp, entry_type, cwd, session_id, version, git_branch, user_type, is_sidechain, content, thinking_level, thinking_disabled, model, message_id, stop_reason, stop_sequence, request_id, or all"
  );
}

#[ test ]
fn tc5_all_combined_with_other_token_rejected()
{
  let err = FieldSelector::parse( "all,uuid" ).unwrap_err();
  assert_eq!( err, "'all' cannot be combined with other fields" );
}

#[ test ]
fn tc6_case_insensitive_per_token_parsing()
{
  let sel = FieldSelector::parse( "UUID,Timestamp" ).unwrap();
  assert_eq!( sel.fields(), vec![ "uuid", "timestamp" ] );
}

#[ test ]
fn tc7_whitespace_trimmed_around_tokens_and_commas()
{
  let sel = FieldSelector::parse( " uuid , timestamp " ).unwrap();
  assert_eq!( sel.fields(), vec![ "uuid", "timestamp" ] );
}

#[ test ]
fn tc8_duplicate_token_collapses_to_one_occurrence()
{
  let sel = FieldSelector::parse( "uuid,uuid" ).unwrap();
  assert_eq!( sel.fields(), vec![ "uuid" ] );
}

#[ test ]
fn tc9_empty_string_rejected()
{
  let err = FieldSelector::parse( "" ).unwrap_err();
  assert_eq!( err, "fields must be non-empty" );
}
