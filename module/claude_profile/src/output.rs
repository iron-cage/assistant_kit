//! Output formatting: text/json selection, JSON string escaping, duration display.
//!
//! Provides the `OutputOptions` struct used by command handlers to determine
//! output format and verbosity. All format decisions are centralised here
//! rather than scattered across command modules.

use unilang::data::{ ErrorCode, ErrorData };
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

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

/// Format a duration in seconds as a compact human-readable string.
///
/// Output form: `Nd Nh Nm` — only non-zero components are shown, with minutes
/// always present as the most-granular unit (sub-minute precision is dropped).
///
/// - `0` → `"0m"` (special case: the only time minutes appear as zero)
/// - `60` → `"1m"`, `3600` → `"1h"`, `86400` → `"1d"`
/// - `3660` → `"1h 1m"`, `86460` → `"1d 1m"`, `90000` → `"1d 1h"`
///
/// Used to display rate-limit reset times (`resets in …`).
#[ inline ]
#[ must_use ]
pub fn format_duration_secs( secs : u64 ) -> String
{
  if secs == 0 { return "0m".to_string(); }
  let days  = secs / 86400;
  let hours = ( secs % 86400 ) / 3600;
  let mins  = ( secs % 3600 ) / 60;
  let mut parts = Vec::new();
  if days  > 0 { parts.push( format!( "{days}d" ) ); }
  if hours > 0 { parts.push( format!( "{hours}h" ) ); }
  if mins  > 0 || parts.is_empty() { parts.push( format!( "{mins}m" ) ); }
  parts.join( " " )
}

/// Escape a string for safe embedding inside a JSON string value.
///
/// Handles: `"`, `\`, newline, carriage return, tab.
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
      c    => out.push( c ),
    }
  }
  out
}
