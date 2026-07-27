//! Tiered TOML config I/O: read (project + user) and write (user-tier only)
//! flat `key = "value"` entries in a `.toml` file.
//!
//! Mirrors `settings_io`'s JSON precedent's atomicity and type-rejection
//! guarantees, but for TOML's flat scalar-assignment syntax. Only
//! double-quoted basic string values are interpreted; any other value type
//! (number, bool, array, inline table) or malformed line is preserved
//! verbatim through read-modify-write but never returned by the
//! string-typed getter.
//!
//! # Design
//!
//! Hand-rolled, stdlib-only line-oriented parsing — this crate has zero
//! external dependencies. Comments, blank lines, and `[section]` headers are
//! recognized and skipped during parse but NOT preserved on write (the
//! `~/.clr/config.toml` files this module targets are flat scalar-only,
//! with no section headers in practice). Every OTHER key's raw (unparsed)
//! right-hand-side text is captured verbatim during parse and re-emitted
//! unchanged on write — this is what prevents `set_user_tier` from silently
//! deleting sibling settings of a type this module doesn't interpret.

use std::io::{ self, Write };
use std::path::Path;

/// Get a key's string value, checking `project_path` first (if given) then
/// `user_path`. Missing files are treated as absent, not an error. Returns
/// `None` if the key is absent from both tiers, or if its value is present
/// but not a plain double-quoted string.
#[ inline ]
#[ must_use ]
pub fn get_tiered( project_path : Option< &Path >, user_path : &Path, key : &str ) -> Option< String >
{
  if let Some( path ) = project_path
  {
    if let Some( value ) = read_string_key( path, key )
    {
      return Some( value );
    }
  }
  read_string_key( user_path, key )
}

/// Set a key's value in the user-tier TOML file, creating the file if
/// absent. All other keys' raw text is preserved unchanged. Uses atomic
/// write (tmp file + rename).
///
/// # Errors
///
/// Returns `Err` if the file cannot be read (except `NotFound`, treated as
/// empty) or if the write fails.
#[ inline ]
pub fn set_user_tier( user_path : &Path, key : &str, value : &str ) -> Result< (), io::Error >
{
  let mut pairs = read_or_empty( user_path )?;
  upsert_pair( &mut pairs, key, &quote_toml_string( value ) );
  let toml = serialize_flat_table( &pairs );
  atomic_write( user_path, &toml )
}

/// Remove a key from the user-tier TOML file. All other keys' raw text is
/// preserved unchanged. No-op (no write, no file created) if the key or the
/// file itself is absent — mirrors `settings_io::remove_setting`'s
/// idempotent-reset semantics.
///
/// # Errors
///
/// Returns `Err` if the file cannot be read (except `NotFound`, treated as
/// empty) or if the write fails.
#[ inline ]
pub fn remove_user_tier( user_path : &Path, key : &str ) -> Result< (), io::Error >
{
  let mut pairs = read_or_empty( user_path )?;
  let before = pairs.len();
  pairs.retain( |( k, _ )| k != key );
  if pairs.len() == before
  {
    return Ok( () );
  }
  let toml = serialize_flat_table( &pairs );
  atomic_write( user_path, &toml )
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn read_string_key( path : &Path, key : &str ) -> Option< String >
{
  let pairs = read_or_empty( path ).ok()?;
  let raw   = pairs.into_iter().find( |( k, _ )| k == key )?.1;
  unquote_toml_string( &raw )
}

fn read_or_empty( path : &Path ) -> Result< Vec< ( String, String ) >, io::Error >
{
  match std::fs::read_to_string( path )
  {
    Ok( src ) => Ok( parse_flat_table( &src ) ),
    Err( e ) if e.kind() == io::ErrorKind::NotFound => Ok( vec![] ),
    Err( e ) => Err( e ),
  }
}

/// Parse `key = <raw value text>` lines from a flat TOML file. Comments
/// (`#...`), blank lines, and `[section]` headers are skipped. The raw
/// value text (everything after the first `=`, trimmed) is captured
/// verbatim regardless of type — callers decide how to interpret it.
fn parse_flat_table( content : &str ) -> Vec< ( String, String ) >
{
  let mut pairs = vec![];
  for line in content.lines()
  {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with( '#' ) || trimmed.starts_with( '[' )
    {
      continue;
    }
    let Some( eq_pos ) = trimmed.find( '=' ) else { continue };
    let key   = trimmed[ ..eq_pos ].trim();
    let value = trimmed[ eq_pos + 1.. ].trim();
    if key.is_empty()
    {
      continue;
    }
    pairs.push( ( key.to_string(), value.to_string() ) );
  }
  pairs
}

fn upsert_pair( pairs : &mut Vec< ( String, String ) >, key : &str, raw_value : &str )
{
  if let Some( entry ) = pairs.iter_mut().find( |( k, _ )| k == key )
  {
    entry.1 = raw_value.to_string();
  }
  else
  {
    pairs.push( ( key.to_string(), raw_value.to_string() ) );
  }
}

fn serialize_flat_table( pairs : &[ ( String, String ) ] ) -> String
{
  use core::fmt::Write as _;
  pairs.iter().fold( String::new(), | mut acc, ( k, v ) |
  {
    let _ = writeln!( acc, "{k} = {v}" );
    acc
  } )
}

/// Interpret raw value text as a TOML basic string. Returns `None` if the
/// text is not a well-formed double-quoted string (bare tokens, numbers,
/// bools, arrays, and inline tables all return `None` — matching
/// `settings_io::get_string_setting`'s type-rejection semantics).
fn unquote_toml_string( raw : &str ) -> Option< String >
{
  if raw.len() < 2 || !raw.starts_with( '"' ) || !raw.ends_with( '"' )
  {
    return None;
  }
  let inner = &raw[ 1..raw.len() - 1 ];
  let mut out = String::with_capacity( inner.len() );
  let mut chars = inner.chars();
  while let Some( c ) = chars.next()
  {
    if c == '\\'
    {
      match chars.next()
      {
        Some( '"' )   => out.push( '"' ),
        Some( '\\' ) | None => out.push( '\\' ),
        Some( 'n' )   => out.push( '\n' ),
        Some( 't' )   => out.push( '\t' ),
        Some( 'r' )   => out.push( '\r' ),
        Some( other ) => { out.push( '\\' ); out.push( other ); },
      }
    }
    else
    {
      out.push( c );
    }
  }
  Some( out )
}

/// Quote and escape a string value for writing as a TOML basic string.
fn quote_toml_string( s : &str ) -> String
{
  let mut out = String::with_capacity( s.len() + 2 );
  out.push( '"' );
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
  out.push( '"' );
  out
}

fn atomic_write( path : &Path, content : &str ) -> Result< (), io::Error >
{
  let mut tmp_path = path.to_path_buf();
  let filename = tmp_path.file_name()
  .ok_or_else( || io::Error::new( io::ErrorKind::InvalidInput, "path has no filename" ) )?
  .to_string_lossy()
  .into_owned();
  tmp_path.set_file_name( format!( "{filename}.tmp" ) );

  {
    let mut f = std::fs::File::create( &tmp_path )?;
    f.write_all( content.as_bytes() )?;
    f.flush()?;
  }
  std::fs::rename( &tmp_path, path )
}
