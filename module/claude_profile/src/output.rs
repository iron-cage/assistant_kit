//! Output formatting: text/json selection, JSON string escaping, duration display.
//!
//! Provides the `OutputOptions` struct used by command handlers to determine
//! output format. All format decisions are centralised here rather than
//! scattered across command modules.

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
  /// ASCII table via `data_fmt` (`.accounts` only).
  Table,
}

/// Parsed output options extracted from a `VerifiedCommand`.
#[ derive( Debug ) ]
pub struct OutputOptions
{
  /// Output format.
  pub format : OutputFormat,
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
    // Parse format: String argument, default "text".
    let format = match cmd.arguments.get( "format" )
    {
      Some( Value::String( s ) ) =>
      {
        match s.to_ascii_lowercase().as_str()
        {
          "text"  => OutputFormat::Text,
          "json"  => OutputFormat::Json,
          "table" => OutputFormat::Table,
          _other  =>
          {
            return Err( ErrorData::new(
              ErrorCode::ArgumentTypeMismatch,
              format!( "unknown format '{s}': expected text, json, or table" ),
            ) );
          }
        }
      }
      _ => OutputFormat::Text,
    };

    Ok( OutputOptions { format } )
  }

  /// Return `true` when the selected format is `Table`.
  #[ inline ]
  #[ must_use ]
  pub fn is_table( &self ) -> bool { matches!( self.format, OutputFormat::Table ) }
}

/// Format a duration in seconds as a compact human-readable string.
///
/// At most 2 non-zero components from `Nd Nh Nm`; the least-significant component
/// is dropped when 3 non-zero components would otherwise appear. Sub-minute
/// precision is dropped; `"0m"` is emitted when the duration is zero.
///
/// - `0` → `"0m"` (special case: the only time minutes appear as zero)
/// - `60` → `"1m"`, `3600` → `"1h"`, `86400` → `"1d"`
/// - `3660` → `"1h 1m"`, `86460` → `"1d 1m"`, `90000` → `"1d 1h"`
/// - `90300` (1d 1h 5m) → `"1d 1h"` (minutes dropped — 2-unit cap)
///
/// Used to display rate-limit reset times (`resets in …`).
#[ inline ]
#[ must_use ]
pub fn format_duration_secs( secs : u64 ) -> String
{
  if secs == 0
  {
    return "0m".to_string();
  }
  let days  = secs / 86400;
  let hours = ( secs % 86400 ) / 3600;
  let mins  = ( secs % 3600 ) / 60;
  let mut parts = Vec::new();
  if days > 0
  {
    parts.push( format!( "{days}d" ) );
  }
  if hours > 0
  {
    parts.push( format!( "{hours}h" ) );
  }
  if mins > 0 || parts.is_empty()
  {
    parts.push( format!( "{mins}m" ) );
  }
  parts.truncate( 2 );
  parts.join( " " )
}

/// Parse an integer `0`-or-`1` flag from `cmd.arguments` with a configurable default.
///
/// Returns `default` when absent; rejects non-`Value::Integer` values or integers outside
/// `{0, 1}` with `ArgumentTypeMismatch`.
///
/// Pitfall: params registered as `Kind::String` (e.g. `touch::`, `only_next::`) deliver
/// all values — including `"0"`, `"1"`, `"true"`, and `"false"` — as `Value::String`, so
/// the `"true"`/`"false"` alias arms (lines 130-131) are only reachable for `Kind::String`
/// params. For `Kind::Integer` params (e.g. `who::`, `rotate::`) the unilang routing layer
/// calls `"true".parse::<i64>()` at framework level, which fails before this function is
/// invoked — `"true"` is rejected with exit 1. Use `Kind::String` if you want "true"/"false"
/// as accepted aliases; use `Kind::Integer` if you want hard integer-only enforcement.
#[ inline ]
pub(crate) fn parse_int_flag( cmd : &VerifiedCommand, name : &str, default : i64 ) -> Result< i64, ErrorData >
{
  match cmd.arguments.get( name )
  {
    None                                       => Ok( default ),
    Some( Value::Integer( 0 ) )                => Ok( 0 ),
    Some( Value::Integer( 1 ) )                => Ok( 1 ),
    Some( Value::String( s ) ) if s == "0"     => Ok( 0 ),
    Some( Value::String( s ) ) if s == "1"     => Ok( 1 ),
    Some( Value::String( s ) ) if s == "true"  => Ok( 1 ),
    Some( Value::String( s ) ) if s == "false" => Ok( 0 ),
    _ => Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "{name}:: must be 0, 1, false, or true" ),
    ) ),
  }
}

fn base64url_decode( s : &str ) -> Option< Vec< u8 > >
{
  // Translate URL-safe alphabet to standard and add padding.
  let pad = match s.len() % 4 { 0 => 0, 2 => 2, 3 => 1, _ => return None };
  let b64 : String = s.chars()
    .map( |c| match c { '-' => '+', '_' => '/', c => c } )
    .chain( core::iter::repeat( '=' ).take( pad ) )
    .collect();
  // Decode groups of 4 base64 characters → 3 bytes.
  const ALPHA : &[ u8 ] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  // ALPHA has 64 entries (positions 0–63), so the position always fits in u32.
  let val = |c : u8| ALPHA.iter().position( |&a| a == c )
    .and_then( |v| u32::try_from( v ).ok() );
  let bytes = b64.as_bytes();
  let mut out = Vec::with_capacity( b64.len() / 4 * 3 );
  let mut i = 0;
  while i + 3 < bytes.len()
  {
    let v0 = val( bytes[ i ] )?;
    let v1 = val( bytes[ i + 1 ] )?;
    // `& 0xFF` makes the narrowing cast lossless — the upper bits are always zero.
    out.push( ( ( ( v0 << 2 ) | ( v1 >> 4 ) ) & 0xFF ) as u8 );
    if bytes[ i + 2 ] != b'='
    {
      let v2 = val( bytes[ i + 2 ] )?;
      out.push( ( ( ( v1 << 4 ) | ( v2 >> 2 ) ) & 0xFF ) as u8 );
    }
    if bytes[ i + 3 ] != b'='
    {
      let v2 = val( bytes[ i + 2 ] )?;
      let v3 = val( bytes[ i + 3 ] )?;
      out.push( ( ( ( v2 << 6 ) | v3 ) & 0xFF ) as u8 );
    }
    i += 4;
  }
  Some( out )
}

/// Extracts the `exp` claim from the `accessToken` JWT inside a credentials JSON string.
///
/// Returns `Some(exp_ms)` where `exp_ms = exp_secs * 1000`, or `None` if the token is
/// absent, malformed, or missing the `exp` field.  No signature verification is performed —
/// the claim is used only for display purposes.
#[ must_use ]
#[ inline ]
pub fn jwt_exp_ms( creds_json : &str ) -> Option< u64 >
{
  // Locate the accessToken string value.
  let key   = "\"accessToken\":\"";
  let start = creds_json.find( key )? + key.len();
  let rest  = &creds_json[ start.. ];
  let end   = rest.find( '"' )?;
  let token = &rest[ ..end ];
  // Split JWT into header.payload.signature — take payload (second segment).
  let mut parts   = token.splitn( 3, '.' );
  let _header     = parts.next()?;
  let payload_b64 = parts.next()?;
  // Base64url-decode and UTF-8-decode the payload.
  let payload_bytes = base64url_decode( payload_b64 )?;
  let payload       = core::str::from_utf8( &payload_bytes ).ok()?;
  // Extract the numeric `exp` field.
  let needle    = "\"exp\":";
  let after     = &payload[ payload.find( needle )? + needle.len().. ];
  let digits_end = after.find( |c : char| !c.is_ascii_digit() ).unwrap_or( after.len() );
  let exp_secs : u64 = after[ ..digits_end ].parse().ok()?;
  Some( exp_secs * 1000 )
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
