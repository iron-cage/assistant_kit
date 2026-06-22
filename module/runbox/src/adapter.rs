//! Adapter layer: convert raw argv tokens to unilang token strings.
//!
//! Handles dot-prefix enforcement and `::` separator validation
//! before handing off to `unilang::Parser`.

use error_tools::{ Error, Result };

fn split_first_colons( s : &str ) -> Option< ( &str, &str ) >
{
  s.find( "::" ).map( |i| ( &s[ ..i ], &s[ i + 2.. ] ) )
}

/// Convert raw argv into unilang token strings.
///
/// Returns `(tokens, needs_help)`.
///
/// # Errors
///
/// - First arg does not start with `.`
/// - First arg contains `::` (looks like a param, not a command)
/// - Any subsequent arg missing `::` separator
/// - `-`-prefixed flags in any position
#[ inline ]
pub fn argv_to_unilang_tokens( argv : &[ String ] ) -> Result< ( Vec< String >, bool ) >
{
  if argv.is_empty()
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  if argv.iter().any( |a| a == ".help" || a == "help" )
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  if argv[ 0 ] == "--help" || argv[ 0 ] == "-h"
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  if argv[ 0 ] == "."
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  if argv[ 0 ].contains( "::" )
  {
    return Err( Error::msg( format!(
      "expected command name as first argument, got '{}': commands start with '.'",
      argv[ 0 ]
    ) ) );
  }

  if argv[ 0 ].starts_with( '-' )
  {
    return Err( Error::msg( format!(
      "unexpected flag '{}': use param::value syntax",
      argv[ 0 ]
    ) ) );
  }

  if !argv[ 0 ].starts_with( '.' )
  {
    return Err( Error::msg( format!(
      "command must start with '.': got '{}'",
      argv[ 0 ]
    ) ) );
  }

  let command_name = argv[ 0 ].clone();
  let mut pairs : Vec< ( String, String ) > = vec![];

  for arg in &argv[ 1.. ]
  {
    if arg.starts_with( '-' )
    {
      return Err( Error::msg( format!(
        "unexpected flag '{arg}': use param::value syntax (e.g., image::my_project)"
      ) ) );
    }

    let ( raw_key, raw_val ) = split_first_colons( arg ).ok_or_else( || Error::msg( format!(
      "expected param::value syntax, got: '{arg}'"
    ) ) )?;

    let key = raw_key.to_string();
    let val = raw_val.to_string();

    // Duplicate keys: last value wins (overwrite-in-place).
    if let Some( entry ) = pairs.iter_mut().find( |( k, _ )| k == &key )
    {
      entry.1 = val;
    }
    else
    {
      pairs.push( ( key, val ) );
    }
  }

  let mut tokens = vec![ command_name ];
  for ( k, v ) in pairs
  {
    tokens.push( format!( "{k}::{v}" ) );
  }

  Ok( ( tokens, false ) )
}
