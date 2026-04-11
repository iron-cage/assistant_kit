//! Adapter layer: convert raw argv tokens to unilang token strings.
//!
//! Handles alias expansion (`v::` → `verbosity::`), bool normalisation
//! (`installed::true` → error; only `0`/`1` accepted), and dot-prefix
//! enforcement before handing off to `unilang::Parser`.

use error_tools::{ Error, Result };

/// Bool params that accept only `0`/`1`.
const BOOL_PARAMS : &[ &str ] = &[ "installed" ];

/// Short alias for verbosity param.
const VERBOSITY_ALIAS : &str = "v";
/// Canonical verbosity key forwarded to unilang.
const VERBOSITY_KEY   : &str = "verbosity";
/// Maximum accepted verbosity value.
const MAX_VERBOSITY   : u8   = 2;

fn split_first_colons( s : &str ) -> Option< ( &str, &str ) >
{
  s.find( "::" ).map( |i| ( &s[ ..i ], &s[ i + 2.. ] ) )
}

fn normalise_bool_value( key : &str, raw_val : &str ) -> Result< String >
{
  match raw_val
  {
    "1" => Ok( "1".to_string() ),
    "0" => Ok( "0".to_string() ),
    other => Err( Error::msg( format!(
      "invalid value for {key}::{other}: expected 0 or 1"
    ) ) ),
  }
}

fn normalise_verbosity( key : &str, raw_val : &str ) -> Result< String >
{
  let n : u8 = raw_val.parse().map_err( |_| Error::msg( format!(
    "{key}:: must be 0, 1, or 2, got: '{raw_val}'"
  ) ) )?;
  if n > MAX_VERBOSITY
  {
    return Err( Error::msg( format!(
      "{key}:: out of range: {n} (max {MAX_VERBOSITY})"
    ) ) );
  }
  Ok( VERBOSITY_KEY.to_string() )
}

/// Convert raw argv into unilang token strings.
///
/// Returns `(tokens, needs_help)`.
///
/// # Errors
///
/// - First arg does not start with `.`
/// - Any subsequent arg missing `::` separator
/// - `v::` / `verbosity::` value not in `[0, 2]`
/// - Bool param value other than `0` or `1`
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
        "unexpected flag '{arg}': use param::value syntax (e.g., kind::rule)"
      ) ) );
    }

    let ( raw_key, raw_val ) = split_first_colons( arg ).ok_or_else( || Error::msg( format!(
      "expected param::value syntax, got: '{arg}'"
    ) ) )?;

    let key : String = if raw_key == VERBOSITY_ALIAS || raw_key == VERBOSITY_KEY
    {
      normalise_verbosity( raw_key, raw_val )?
    }
    else
    {
      raw_key.to_string()
    };

    let val : String = if BOOL_PARAMS.contains( &key.as_str() )
    {
      normalise_bool_value( &key, raw_val )?
    }
    else
    {
      raw_val.to_string()
    };

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
