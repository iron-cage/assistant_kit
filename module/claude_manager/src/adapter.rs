//! Adapter layer: convert raw `argv` tokens to unilang token strings.
//!
//! Implements the first phase of the unilang pipeline for `claude_manager`.
//! Handles alias expansion (`v::` → `verbosity::`), bool normalisation
//! (`dry::true` → `dry::1`), integer range validation (`v::3` → error),
//! and dot-prefix enforcement before handing off to `unilang::Parser`.

use error_tools::{ Error, Result };

/// Bool params that accept only `0`/`1`/`true`/`false`.
const BOOL_PARAMS : &[ &str ] = &[ "dry", "force" ];

/// Integer params that must be non-negative.
const NON_NEG_INT_PARAMS : &[ &str ] = &[ "interval", "count" ];

/// Short alias for verbosity param.
const VERBOSITY_ALIAS : &str = "v";
/// Canonical verbosity key forwarded to unilang.
const VERBOSITY_KEY   : &str = "verbosity";
/// Maximum accepted verbosity value.
const MAX_VERBOSITY   : u8   = 2;

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
/// Accepts only `"1"` → `"1"` and `"0"` → `"0"`.
/// Returns an error for any other value, including `"true"` / `"false"`.
///
/// Note: `"true"` / `"false"` are intentionally rejected — silent boolean
/// coercion (where `dry::true` appears to enable dry-run but silently falls
/// through to real execution on type mismatch) is dangerous.  Only the
/// canonical numeric forms are accepted.
#[ inline ]
fn normalise_bool_value( key : &str, raw_val : &str ) -> Result< String >
{
  match raw_val
  {
    "1" => Ok( "1".to_string() ),
    "0" => Ok( "0".to_string() ),
    other =>
    {
      Err( Error::msg( format!(
        "invalid value for {key}::{other}: expected 0 or 1"
      ) ) )
    }
  }
}

/// Validate and normalise a verbosity value (for both `v::` and `verbosity::` keys).
///
/// Returns the canonical key `"verbosity"` on success, or an error if the value
/// is not a `u8` in `[0, MAX_VERBOSITY]`.
///
/// Fix(issue-verbosity-bypass): `verbosity::` canonical key bypassed 0–2 range check.
/// Root cause: range validation only guarded `v::` alias; `verbosity::` reached unilang unchecked.
/// Pitfall: `verbosity::3` exited 0 (treated as level 2); `verbosity::-1` exited 0 (defaulted to 1).
#[ inline ]
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

/// Validate a non-negative integer parameter value.
///
/// Rejects negative values, non-integers, and values that exceed `i64::MAX` — the
/// upper bound of unilang's internal integer type — so that oversized inputs are
/// caught here with a clear message rather than producing a cryptic type error later.
///
/// Fix(issue-count-overflow): `count::` / `interval::` accepted `u64` values > `i64::MAX`.
/// Root cause: parsed as `u64`; values above `i64::MAX` reached unilang and triggered an opaque overflow error.
/// Pitfall: confusing "number too large to fit in target type" message instead of the adapter's user-friendly text.
#[ inline ]
fn validate_non_neg_int( key : &str, raw_val : &str ) -> Result< String >
{
  let n = raw_val.parse::< u64 >().map_err( |_| Error::msg( format!(
    "{key}:: must be a non-negative integer, got: '{raw_val}'"
  ) ) )?;
  if n > i64::MAX as u64
  {
    return Err( Error::msg( format!(
      "{key}:: value too large: {n} (max {})", i64::MAX
    ) ) );
  }
  Ok( raw_val.to_string() )
}

/// Convert raw argv (process args, NOT including argv\[0\]) into unilang token strings.
///
/// Returns `(tokens, needs_help)` where `needs_help=true` signals that the adapter
/// routed to `.help` (for informational purposes only; caller may ignore it).
///
/// # Normalisation performed
///
/// - Empty argv or `"."` → routes to `".help"`.
/// - `".help"` token anywhere in argv → routes to `".help"` (FR-02).
/// - `--help` / `-h` → routes to `".help"`.
/// - `v::` → expanded to `verbosity::` (value validated as `0`–`2`).
/// - `dry::` / `force::` — bool values normalised to `"0"` or `"1"`.
/// - `interval::` / `count::` — must be non-negative integers.
///
/// # Errors
///
/// Returns an error for:
/// - First arg does not start with `"."` (command prefix required).
/// - First arg contains `"::"` (a param was given as the command name).
/// - First arg starts with `"-"` but is not `"--help"` or `"-h"`.
/// - Any subsequent arg missing `"::"` separator.
/// - `v::` / `verbosity::` value not in range `[0, 2]`.
/// - Bool param value other than `0`, `1`, `true`, `false`.
/// - `interval::` / `count::` value is not a non-negative integer.
#[ inline ]
pub fn argv_to_unilang_tokens( argv : &[ String ] ) -> Result< ( Vec< String >, bool ) >
{
  // Step 1: empty → help.
  if argv.is_empty()
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  // Step 1b: `.help` or bare `help` anywhere in argv → help (FR-02: help flag in any position).
  // Must precede all other checks so `.status .help` shows help rather than
  // erroring on the missing `::` separator.
  // Bare `help` (without the dot) is treated as a synonym so users following the
  // help footer's "Use '<command> help'" instruction get the expected output
  // rather than a confusing "expected param::value syntax" error.
  if argv.iter().any( |a| a == ".help" || a == "help" )
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  // Step 2: --help / -h → help.
  if argv[ 0 ] == "--help" || argv[ 0 ] == "-h"
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  // Step 3a: bare `.` → help.
  if argv[ 0 ] == "."
  {
    return Ok( ( vec![ ".help".to_string() ], true ) );
  }

  // Step 3b: first arg must not be a param (must not contain `::`) — it's the command.
  if argv[ 0 ].contains( "::" )
  {
    return Err( Error::msg( format!(
      "expected command name as first argument, got '{}': commands start with '.'",
      argv[ 0 ]
    ) ) );
  }

  // Step 3c: reject `-flag` style (other than --help/-h already handled).
  if argv[ 0 ].starts_with( '-' )
  {
    return Err( Error::msg( format!(
      "unexpected flag '{}': use param::value syntax",
      argv[ 0 ]
    ) ) );
  }

  // Step 4: enforce dot-prefix for commands.
  if !argv[ 0 ].starts_with( '.' )
  {
    return Err( Error::msg( format!(
      "command must start with '.': got '{}'\nExample: cm .version.show",
      argv[ 0 ]
    ) ) );
  }

  let command_name = argv[ 0 ].clone();

  // Step 5–6: process remaining args as key::value pairs.
  let mut pairs : Vec< ( String, String ) > = vec![];

  for arg in &argv[ 1.. ]
  {
    // Reject `-flag` / `--flag` style.
    if arg.starts_with( '-' )
    {
      return Err( Error::msg( format!(
        "unexpected flag '{arg}': use param::value syntax (e.g., verbosity::2)"
      ) ) );
    }

    // Require `::` separator.
    let ( raw_key, raw_val ) = split_first_colons( arg ).ok_or_else( || Error::msg( format!(
      "expected param::value syntax, got: '{arg}'"
    ) ) )?;

    // Expand `v::` alias → `verbosity::` and validate range for both spellings.
    let key : String = if raw_key == VERBOSITY_ALIAS || raw_key == VERBOSITY_KEY
    {
      normalise_verbosity( raw_key, raw_val )?
    }
    else
    {
      raw_key.to_string()
    };

    // Normalise bool params; validate non-negative integer params.
    let val : String = if BOOL_PARAMS.contains( &key.as_str() )
    {
      normalise_bool_value( &key, raw_val )?
    }
    else if NON_NEG_INT_PARAMS.contains( &key.as_str() )
    {
      validate_non_neg_int( &key, raw_val )?
    }
    else
    {
      raw_val.to_string()
    };

    // Last-occurrence-wins: update existing entry or push.
    if let Some( entry ) = pairs.iter_mut().find( |( k, _ )| k == &key )
    {
      entry.1 = val;
    }
    else
    {
      pairs.push( ( key, val ) );
    }
  }

  // Step 7: assemble tokens.
  let mut tokens = vec![ command_name ];
  for ( k, v ) in pairs
  {
    tokens.push( format!( "{k}::{v}" ) );
  }

  Ok( ( tokens, false ) )
}
