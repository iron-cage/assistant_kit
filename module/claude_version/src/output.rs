//! Output formatting: text/json selection and JSON string escaping.
//!
//! Provides the `OutputOptions` struct used by command handlers to determine
//! output format and verbosity. All format decisions are centralised here
//! rather than scattered across command modules.

use unilang::data::{ ErrorCode, ErrorData };
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use core::fmt::Write as _;

/// Available output formats for command results.
#[ derive( Debug, Clone, Copy, PartialEq ) ]
pub enum OutputFormat
{
  /// Human-readable, newline-separated text (default).
  Text,
  /// Machine-readable JSON object or array.
  Json,
}

/// Parsed output options extracted from a `VerifiedCommand`.
#[ derive( Debug ) ]
pub struct OutputOptions
{
  /// Verbosity level: 0 = minimal, 1 = normal (default), 2 = verbose.
  pub verbosity : u8,
  /// Output format.
  pub format    : OutputFormat,
}

impl OutputOptions
{
  /// Extract `OutputOptions` from a `VerifiedCommand`'s argument map.
  ///
  /// # Errors
  ///
  /// Returns `Err(ErrorData)` with `ErrorCode::ArgumentTypeMismatch` if
  /// `format::` has an unrecognised value.
  #[ inline ]
  pub fn from_cmd( cmd : &VerifiedCommand ) -> Result< Self, ErrorData >
  {
    // Parse verbosity: Integer argument, default 1.
    let verbosity = match cmd.arguments.get( "verbosity" )
    {
      Some( Value::Integer( n ) ) => u8::try_from( *n ).unwrap_or( 1 ),
      _                           => 1,
    };

    // Parse format: String argument, default "text".
    let format = match cmd.arguments.get( "format" )
    {
      Some( Value::String( s ) ) =>
      {
        match s.as_str()
        {
          "text" => OutputFormat::Text,
          "json" => OutputFormat::Json,
          other  =>
          {
            return Err( ErrorData::new(
              ErrorCode::ArgumentTypeMismatch,
              format!( "unknown format '{other}': expected text or json" ),
            ) );
          }
        }
      }
      _ => OutputFormat::Text,
    };

    Ok( OutputOptions { verbosity, format } )
  }
}

/// Escape a string for safe embedding inside a JSON string value.
///
/// Handles: `"`, `\`, newline, carriage return, tab, and all C0 control
/// characters (U+0000–U+001F) per RFC 8259 § 7.
#[ inline ]
#[ must_use ]
pub fn json_escape( s : &str ) -> String
{
  let mut out = String::with_capacity( s.len() );
  for ch in s.chars()
  {
    match ch
    {
      '"'  => out.push_str( "\\\"" ),
      '\\' => out.push_str( "\\\\" ),
      '\n' => out.push_str( "\\n"  ),
      '\r' => out.push_str( "\\r"  ),
      '\t' => out.push_str( "\\t"  ),
      // RFC 8259 requires all other C0 control chars to be escaped as \uXXXX.
      c if ( c as u32 ) < 0x20 => write!( out, "\\u{:04x}", c as u32 ).unwrap(),
      c    => out.push( c ),
    }
  }
  out
}
