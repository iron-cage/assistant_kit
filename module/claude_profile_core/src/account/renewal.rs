//! Billing renewal override (`_renewal_at`) and its timestamp helpers.

use std::path::Path;
use claude_core::file_io::atomic_write;

/// The operation to apply to `_renewal_at` in `{name}.json`.
#[ derive( Debug ) ]
pub enum RenewalOperation
{
  /// Set `_renewal_at` to the given ISO-8601 UTC string (stored verbatim).
  At( String ),
  /// Remove `_renewal_at` from the file.
  Clear,
}

/// Write or clear a billing renewal timestamp override in `{name}.json`.
///
/// Reads the existing `{name}.json` (or starts with `{}` if absent), applies `op`,
/// and writes back. All other top-level keys (e.g. `oauthAccount`) are preserved.
///
/// When `dry` is `true`, no file is written; returns a `[dry-run]` status line.
///
/// # Errors
///
/// Returns `NotFound` if `{name}.credentials.json` does not exist.
/// Returns I/O errors on file read/write failure.
#[ inline ]
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
pub fn account_renewal(
  name             : &str,
  credential_store : &Path,
  op               : &RenewalOperation,
  dry              : bool,
) -> Result< String, std::io::Error >
{
  let cred_path = credential_store.join( format!( "{name}.credentials.json" ) );
  if !cred_path.exists()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::NotFound,
      format!( "account '{name}' not found in {}", credential_store.display() ),
    ) );
  }

  let meta_path    = credential_store.join( format!( "{name}.json" ) );
  let existing_str = std::fs::read_to_string( &meta_path )
    .unwrap_or_else( |_| "{}".to_string() );
  let mut val = serde_json::from_str::< serde_json::Value >( &existing_str )
    .unwrap_or_else( |_| serde_json::json!( {} ) );
  let obj = val.as_object_mut()
    .ok_or_else( || std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      format!( "{name}.json is not a JSON object" ),
    ) )?;

  let status_str = match op
  {
    RenewalOperation::At( ts ) =>
    {
      obj.insert( "_renewal_at".to_string(), serde_json::Value::String( ts.clone() ) );
      format!( "set _renewal_at = {ts}" )
    }
    RenewalOperation::Clear =>
    {
      obj.remove( "_renewal_at" );
      "cleared _renewal_at".to_string()
    }
  };

  if dry
  {
    return Ok( format!( "[dry-run] {name}: would {status_str}\n" ) );
  }

  let new_json = serde_json::to_string_pretty( &val )
    .map( | s | s + "\n" )
    .map_err( |e| std::io::Error::new( std::io::ErrorKind::InvalidData, e.to_string() ) )?;
  atomic_write( &meta_path, &new_json )?;
  Ok( format!( "{name}: {status_str}\n" ) )
}

/// Format a Unix timestamp (seconds since epoch) as an ISO-8601 UTC string.
///
/// Output format: `YYYY-MM-DDTHH:MM:SSZ`. Used by `from_now::` delta computation.
/// Does not depend on chrono.
#[ doc( hidden ) ]
#[ inline ]
#[ must_use ]
pub fn secs_to_iso8601( secs : u64 ) -> String
{
  let sec  = secs % 60;
  let min  = ( secs / 60 ) % 60;
  let hour = ( secs / 3600 ) % 24;
  let days = secs / 86400;

  let mut year  = 1970_u64;
  let mut d_rem = days;
  loop
  {
    let dy = if is_leap( year ) { 366 } else { 365 };
    if d_rem < dy { break; }
    d_rem -= dy;
    year  += 1;
  }

  let month_days : [ u64; 12 ] =
    [ 31, if is_leap( year ) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ];
  let mut month = 0_usize;
  while month < 12 && d_rem >= month_days[ month ]
  {
    d_rem -= month_days[ month ];
    month += 1;
  }

  format!( "{year:04}-{:02}-{:02}T{hour:02}:{min:02}:{sec:02}Z", month + 1, d_rem + 1 )
}

/// Parse a signed duration string into a signed second count.
///
/// Format: `±Xd Xh Xm` with optional spaces between unit-suffixed numbers.
/// Prefix sign is required: `+` for future, `-` for past.
/// Units: `d` (86400s), `h` (3600s), `m` (60s).
/// Examples: `+1h30m`, `-30m`, `+1d12h`, `+0m`.
///
/// # Errors
///
/// Returns a descriptive `String` on malformed input.
#[ doc( hidden ) ]
#[ inline ]
pub fn parse_from_now_delta( s : &str ) -> Result< i64, String >
{
  let s = s.trim();
  if s.is_empty() { return Err( "from_now:: value is empty".to_string() ); }
  let ( sign, rest ) = match s.chars().next()
  {
    Some( '+' ) => ( 1_i64,  &s[ 1.. ] ),
    Some( '-' ) => ( -1_i64, &s[ 1.. ] ),
    _           => return Err( format!( "from_now:: must start with '+' or '-', got: '{s}'" ) ),
  };
  if rest.trim().is_empty()
  {
    return Err( format!(
      "from_now:: '{s}' has no duration components; expected e.g. +1h, +30m, +1d"
    ) );
  }
  let mut total_secs = 0_i64;
  let mut pos        = 0_usize;
  let bytes          = rest.as_bytes();
  while pos < bytes.len()
  {
    while pos < bytes.len() && bytes[ pos ] == b' ' { pos += 1; }
    if pos >= bytes.len() { break; }
    let num_start = pos;
    while pos < bytes.len() && bytes[ pos ].is_ascii_digit() { pos += 1; }
    if pos == num_start
    {
      return Err( format!( "from_now:: unexpected character '{}' at position {pos}", bytes[ num_start ] as char ) );
    }
    let num : i64 = rest[ num_start..pos ].parse()
      .map_err( |_| "from_now:: numeric overflow".to_string() )?;
    if pos >= bytes.len()
    {
      return Err( format!( "from_now:: missing unit after number {num} (use d, h, or m)" ) );
    }
    match bytes[ pos ]
    {
      b'd' => { total_secs += num * 86400; pos += 1; }
      b'h' => { total_secs += num * 3600;  pos += 1; }
      b'm' => { total_secs += num * 60;    pos += 1; }
      c    => return Err( format!( "from_now:: unknown unit '{}' (supported: d, h, m)", c as char ) ),
    }
  }
  Ok( sign * total_secs )
}

fn is_leap( y : u64 ) -> bool
{
  ( y % 4 == 0 && y % 100 != 0 ) || y % 400 == 0
}
