//! `FieldSelector` — attribute-projection selector for `.show`'s `fields::` parameter.
//!
//! See `docs/cli/type/15_field_selector.md` for the canonical spec this
//! implementation follows byte-exact (constants, constraints, error strings).

/// Canonical field vocabulary, in canonical order (18 names).
pub const CANONICAL_FIELDS : [ &str; 18 ] =
[
  "uuid", "parent_uuid", "timestamp", "entry_type", "cwd", "session_id",
  "version", "git_branch", "user_type", "is_sidechain", "content",
  "thinking_level", "thinking_disabled",
  "model", "message_id", "stop_reason", "stop_sequence", "request_id",
];

/// Attribute-projection selector — a validated list of field names, or the
/// special `all` token expanding to the full canonical vocabulary.
#[ derive( Debug, Clone ) ]
pub struct FieldSelector
{
  names : Vec< String >,
}

impl FieldSelector
{
  /// Parse a comma-separated `fields::` value into a validated selector.
  ///
  /// # Errors
  ///
  /// Returns an error string for: empty input, an unknown field token, or
  /// `all` combined with any other token.
  pub fn parse( value : &str ) -> core::result::Result< Self, String >
  {
    if value.trim().is_empty()
    {
      return Err( "fields must be non-empty".to_string() );
    }

    let tokens : Vec< String > = value.split( ',' ).map( | t | t.trim().to_lowercase() ).collect();

    if tokens.iter().any( | t | t == "all" )
    {
      if tokens.len() > 1
      {
        return Err( "'all' cannot be combined with other fields".to_string() );
      }
      return Ok( Self
      {
        names : CANONICAL_FIELDS.iter().map( | s | ( *s ).to_string() ).collect(),
      });
    }

    let mut names : Vec< String > = Vec::new();
    for token in tokens
    {
      if !CANONICAL_FIELDS.contains( &token.as_str() )
      {
        return Err( format!( "unknown field '{token}' — valid fields: {}, or all", CANONICAL_FIELDS.join( ", " ) ) );
      }
      if !names.contains( &token )
      {
        names.push( token );
      }
    }

    Ok( Self { names } )
  }

  /// Canonical field names, in request order (or full-vocabulary order when `all` was given).
  #[ must_use ]
  pub fn fields( &self ) -> Vec< &str >
  {
    self.names.iter().map( std::string::String::as_str ).collect()
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

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
}
