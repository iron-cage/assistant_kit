//! Dependency-free flat-JSON field extraction helpers.

/// Extract a quoted string field from a JSON blob without external dependencies.
///
/// Handles optional whitespace after the colon: both `"key":"val"` and
/// `"key": "val"` forms.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_string_field( json : &str, key : &str ) -> Option< String >
{
  let search = format!( "\"{key}\":" );
  let colon_end = json.find( &search )? + search.len();
  let rest = json[ colon_end.. ].trim_start();
  if !rest.starts_with( '"' ) { return None; }
  let inner = &rest[ 1.. ];
  let end = inner.find( '"' )?;
  Some( inner[ ..end ].to_string() )
}

/// Extract an unsigned integer field from a JSON blob without external dependencies.
///
/// Handles optional whitespace after the colon.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_u64_field( json : &str, key : &str ) -> Option< u64 >
{
  let search = format!( "\"{key}\":" );
  let colon_end = json.find( &search )? + search.len();
  let rest = json[ colon_end.. ].trim_start();
  let end = rest
    .find( | c : char | !c.is_ascii_digit() )
    .unwrap_or( rest.len() );
  if end == 0 { return None; }
  rest[ ..end ].parse().ok()
}

/// Extract a boolean field from a JSON blob without external dependencies.
///
/// Handles optional whitespace after the colon. Returns `None` when the key is
/// absent or the value is not literally `true` or `false`.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_bool_field( json : &str, key : &str ) -> Option< bool >
{
  let search = format!( "\"{key}\":" );
  let colon_end = json.find( &search )? + search.len();
  let rest = json[ colon_end.. ].trim_start();
  if rest.starts_with( "true" ) { return Some( true ); }
  if rest.starts_with( "false" ) { return Some( false ); }
  None
}

/// Extract a string array field from a JSON blob without external dependencies.
///
/// Handles optional whitespace after the colon. Returns an empty `Vec` when
/// the key is absent, the value is not an array, or no quoted strings are found.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn parse_string_array_field( json : &str, key : &str ) -> Vec< String >
{
  let search    = format!( "\"{key}\":" );
  let colon_end = match json.find( &search )
  {
    Some( p ) => p + search.len(),
    None      => return Vec::new(),
  };
  let rest = json[ colon_end.. ].trim_start();
  if !rest.starts_with( '[' ) { return Vec::new(); }
  let end = match rest[ 1.. ].find( ']' )
  {
    Some( p ) => 1 + p,
    None      => return Vec::new(),
  };
  let inner = &rest[ 1..end ];
  let mut values = Vec::new();
  let mut pos    = 0_usize;
  while pos < inner.len()
  {
    let Some( q_start ) = inner[ pos.. ].find( '"' ) else { break };
    let start_val = pos + q_start + 1;
    let Some( q_end ) = inner[ start_val.. ].find( '"' ) else { break };
    let end_val = start_val + q_end;
    values.push( inner[ start_val..end_val ].to_string() );
    pos = end_val + 1;
  }
  values
}

/// Bound a JSON substring to its first top-level `{...}` object block via brace-depth counting.
///
/// Fix(BUG-002)
/// Root cause: `parse_string_field()`/`parse_u64_field()`/`parse_bool_field()`/
/// `parse_string_array_field()` search for `"key":` unboundedly across the entire
/// input, with no awareness of enclosing object boundaries. On multi-entry JSON
/// (e.g. a `roles` array with several membership objects), a caller intending to
/// scope a search to one entry silently matches the first occurrence of `key`
/// anywhere in the string, including in a later or earlier sibling entry.
/// Pitfall: callers needing per-entry field extraction from multi-entry JSON must
/// first bound the entry via `extract_object_block()` and pass only that slice to
/// the `parse_*_field()` helpers — passing the full multi-entry JSON directly
/// always risks silently returning a sibling entry's value instead of the intended
/// entry's.
///
/// Returns `None` when `s` does not start with `{`, or the block is unclosed.
#[ doc( hidden ) ]
#[ must_use ]
#[ inline ]
pub fn extract_object_block( s : &str ) -> Option< &str >
{
  if !s.starts_with( '{' ) { return None; }
  let mut depth = 0_i32;
  for ( i, c ) in s.char_indices()
  {
    match c
    {
      '{' => depth += 1,
      '}' =>
      {
        depth -= 1;
        if depth == 0 { return Some( &s[ ..=i ] ); }
      }
      _ => {}
    }
  }
  None
}
