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
  #[ inline ]
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
  #[ inline ]
  #[ must_use ]
  pub fn fields( &self ) -> Vec< &str >
  {
    self.names.iter().map( std::string::String::as_str ).collect()
  }
}
