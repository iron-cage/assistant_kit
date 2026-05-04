//! Adapter layer: parse raw `argv` tokens into a command name and key-value parameters.
//!
//! Implements the first phase of the unilang pipeline for `claude_profile`.
//! Handles alias expansion (`v::` → `verbosity::`, `dry::true` → `dry::1`)
//! and validates basic syntactic form before handing off to `unilang::Parser`.

use error_tools::{ Error, Result };

/// Param names that only accept boolean values (true/false/1/0).
const BOOL_PARAMS : &[ &str ] = &[
  "dry",
  "active",
  "account", "sub", "tier", "token", "expires", "email", "org", "file", "saved",
];

/// Short alias for verbosity param.
const VERBOSITY_ALIAS : &str = "v";
/// Canonical verbosity key.
const VERBOSITY_KEY   : &str = "verbosity";
/// Maximum accepted verbosity value.
const MAX_VERBOSITY   : u8   = 2;
/// Short alias for format param.
// Fix(issue-fmt-alias):
// Root cause: adapter.rs expanded v:: → verbosity:: but had no corresponding expansion
//   for fmt:: → format::, despite the YAML aliases list declaring it. Because the YAML
//   file is metadata-only (not read at runtime), all alias expansion must be coded here.
// Pitfall: Never rely on unilang.commands.yaml aliases for runtime behavior — they are
//   for documentation/export only. Expand every alias explicitly in argv_to_unilang_tokens.
const FORMAT_ALIAS : &str = "fmt";
/// Canonical format key.
const FORMAT_KEY   : &str = "format";

/// Split `"key::value"` at the first `::`, returning `(key, value)`.
///
/// Returns `None` when `::` is absent.
#[ inline ]
fn split_first_colons( s : &str ) -> Option< ( &str, &str ) >
{
  s.find( "::" ).map( |i| ( &s[ ..i ], &s[ i + 2.. ] ) )
}

/// Convert a raw value for a bool param to its canonical form (`"0"` or `"1"`).
///
/// Accepts `"true"` → `"1"`, `"false"` → `"0"`, and the numeric `"1"` / `"0"` unchanged.
/// Returns an error for any other value.
#[ inline ]
fn normalise_bool_value( key : &str, raw_val : &str ) -> Result< String >
{
  match raw_val
  {
    "true" | "1" => Ok( "1".to_string() ),
    "false" | "0" => Ok( "0".to_string() ),
    other =>
    {
      Err( Error::msg( format!(
        "invalid value for {key}::{other}: expected true, false, 1, or 0"
      ) ) )
    }
  }
}

/// Validate and return a verbosity integer value (must be 0–2).
#[ inline ]
fn parse_verbosity( raw_val : &str ) -> Result< u8 >
{
  let n = raw_val.parse::< u8 >().map_err( |_| Error::msg( format!(
    "verbosity must be 0, 1, or 2, got: '{raw_val}'"
  ) ) )?;
  if n > MAX_VERBOSITY
  {
    return Err( Error::msg( format!(
      "verbosity out of range: {n} (max {MAX_VERBOSITY})"
    ) ) );
  }
  Ok( n )
}

/// Convert raw argv (process args, NOT including argv\[0\]) into unilang token strings.
///
/// Returns `(tokens, needs_help)` where `needs_help=true` signals that help text should
/// be displayed (empty input, `.`, `.help`, or bare `help`).
///
/// # Errors
///
/// Returns an error for:
/// - First arg contains `::` (a param was given instead of a command name)
/// - Param without `::` (not a valid `key::value` token)
/// - Arg starting with `-` (flag syntax rejected — use `.help` instead of `--help`)
/// - `verbosity::` / `v::` value that is not an integer in `[0, 2]`
/// - Bool param (`dry::`) value other than `true`, `false`, `1`, `0`
#[ inline ]
pub fn argv_to_unilang_tokens( argv : &[ String ] ) -> Result< ( Vec< String >, bool ) >
{
  // Step 1: empty → show help
  if argv.is_empty()
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  // Step 1b: `.help` or bare `help` anywhere in argv → show help (FR-02).
  // Must precede all other checks so `.accounts .help` shows help rather than
  // erroring on the missing `::` separator.
  // Bare `help` (without the dot) is treated as a synonym so users following the
  // help footer's "Use '<command> help'" instruction get the expected output
  // rather than a confusing "expected param::value syntax" error.
  if argv.iter().any( |a| a == ".help" || a == "help" )
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  // Step 3a: first arg starting with `-` is a flag — reject
  if argv[ 0 ].starts_with( '-' )
  {
    return Err( Error::msg( format!(
      "unexpected flag '{}': use param::value syntax (e.g., verbosity::2)",
      argv[ 0 ]
    ) ) );
  }

  // Step 3b: first arg must not be a param (must not contain ::)
  if argv[ 0 ].contains( "::" )
  {
    return Err( Error::msg( format!(
      "expected command name as first argument, got '{}'",
      argv[ 0 ]
    ) ) );
  }

  // Step 4: first arg is the command name; bare `.` routes to `.help`
  if argv[ 0 ] == "."
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }
  let command_name = argv[ 0 ].clone();

  // Step 5-6: process remaining args as key::value pairs
  let mut pairs : Vec< ( String, String ) > = vec![];

  for arg in &argv[ 1.. ]
  {
    // Reject --flag / -flag style
    if arg.starts_with( '-' )
    {
      return Err( Error::msg( format!(
        "unexpected flag '{arg}': use param::value syntax (e.g., verbosity::2)"
      ) ) );
    }

    // Require :: separator
    let ( raw_key, raw_val ) = split_first_colons( arg ).ok_or_else( || Error::msg( format!(
      "expected param::value syntax, got: '{arg}'"
    ) ) )?;

    // Expand aliases
    let key : String = if raw_key == VERBOSITY_ALIAS
    {
      VERBOSITY_KEY.to_string()
    }
    else if raw_key == FORMAT_ALIAS
    {
      FORMAT_KEY.to_string()
    }
    else
    {
      raw_key.to_string()
    };

    // Validate verbosity
    if key == VERBOSITY_KEY
    {
      parse_verbosity( raw_val )?;
    }

    // Normalise bool params
    let val : String = if BOOL_PARAMS.contains( &key.as_str() )
    {
      normalise_bool_value( &key, raw_val )?
    }
    else
    {
      raw_val.to_string()
    };

    // Last-occurrence-wins: update existing entry or push
    if let Some( entry ) = pairs.iter_mut().find( |( k, _ )| k == &key )
    {
      entry.1 = val;
    }
    else
    {
      pairs.push( ( key, val ) );
    }
  }

  // Step 7: assemble tokens
  let mut tokens = vec![ command_name ];
  for ( k, v ) in pairs
  {
    tokens.push( format!( "{k}::{v}" ) );
  }

  Ok( ( tokens, false ) )
}
