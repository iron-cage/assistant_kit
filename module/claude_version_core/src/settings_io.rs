//! Settings I/O: read and write Claude Code's `settings.json` file.
//!
//! Provides typed access to Claude's JSON configuration.  Writes are atomic
//! (write to `{path}.tmp` then rename) to prevent partial-write corruption.
//! Implements type inference so plain string values are promoted to bool or
//! numeric JSON types where unambiguous.
//!
//! # Design
//!
//! Hand-rolled JSON parsing is used to avoid extra dependencies.  Nested
//! objects and arrays are preserved as raw JSON strings during round-trip —
//! the parser does not interpret their contents, it only captures them
//! verbatim so they survive read→modify→write cycles intact.

use std::io::{ self, Write };
use std::path::Path;

/// How a raw string value is represented in the JSON file.
#[ derive( Debug, PartialEq ) ]
pub enum StoredAs
{
  /// `"true"` or `"false"` → JSON boolean (`true` / `false`).
  Bool,
  /// Parseable as `i64` or `f64` → JSON number.
  Number,
  /// Anything else → JSON string.
  Str,
  /// Nested JSON object or array — output verbatim, never quoted.
  Raw,
}

/// Infer how a raw string value should be stored in `settings.json`.
///
/// `"0"` and `"1"` are intentionally `Number`, not `Bool`
/// (spec Algorithm 2: parse as integer before comparing to "true"/"false").
/// Values starting with `{` or `[` are treated as raw JSON (nested structures).
///
/// # Non-finite floats
///
/// Rust's `f64::from_str` accepts `NaN`, `inf`, `infinity` (and case
/// variants), but these are NOT valid JSON number literals.  Writing them
/// as bare values corrupts the file.
///
/// Fix(issue-infer-nan): Guard with `is_finite()`.
/// Root cause: `f64::from_str` silently parses non-JSON-compatible tokens.
/// Pitfall: Always gate float classification with `is_finite()` in
/// serialization contexts.
#[ inline ]
#[ must_use ]
pub fn infer_type( raw : &str ) -> StoredAs
{
  let trimmed = raw.trim_start();
  if trimmed.starts_with( '{' ) || trimmed.starts_with( '[' )
  {
    return StoredAs::Raw;
  }
  match raw
  {
    "true" | "false" => StoredAs::Bool,
    "null"           => StoredAs::Raw,
    other =>
    {
      if other.parse::< i64 >().is_ok()
      || other.parse::< f64 >().is_ok_and( f64::is_finite )
      {
        StoredAs::Number
      }
      else
      {
        StoredAs::Str
      }
    }
  }
}

/// Read all key-value settings from a JSON file.
///
/// Returns a vector of `(key, value_as_string)` pairs. Booleans are returned
/// as `"true"` / `"false"`, numbers as their decimal string form, strings
/// unquoted.  Nested objects and arrays are stored as raw JSON strings.
///
/// # Errors
///
/// Returns `Err(NotFound)` if the file does not exist.
/// Returns `Err(InvalidData)` if the file is not valid JSON.
#[ inline ]
pub fn read_all_settings( path : &Path ) -> Result< Vec< ( String, String ) >, io::Error >
{
  let src = std::fs::read_to_string( path )?;
  json_parse_flat_object( &src )
}

/// Return the value for a single key from a JSON settings file, or `None` if
/// the key is absent.
///
/// # Errors
///
/// Returns `Err(NotFound)` if the file does not exist.
/// Returns `Err(InvalidData)` if the file is not valid JSON.
#[ inline ]
pub fn get_setting( path : &Path, key : &str ) -> Result< Option< String >, io::Error >
{
  let pairs = read_all_settings( path )?;
  Ok( pairs.into_iter().find( |( k, _ )| k == key ).map( |( _, v )| v ) )
}

/// Write (or update) a single key in a JSON settings file, creating the file
/// if absent.  Uses atomic write (tmp file + rename).
///
/// Returns the `StoredAs` variant that was chosen for the value.
///
/// # Errors
///
/// Returns `Err` if the file cannot be read (except `NotFound`, which is
/// treated as an empty file) or if the write fails.
#[ inline ]
pub fn set_setting( path : &Path, key : &str, raw_value : &str ) -> Result< StoredAs, io::Error >
{
  let mut pairs = read_or_empty( path )?;
  upsert_pair( &mut pairs, key, raw_value );
  let stored_as = infer_type( raw_value );
  let json = json_serialize_flat_object( &pairs );
  atomic_write( path, &json )?;
  Ok( stored_as )
}

/// Set a key inside the `"env"` sub-object of a JSON settings file.
///
/// Creates the `"env"` key if absent.  Environment variable values are
/// always stored as JSON strings.  Uses atomic write.
///
/// # Errors
///
/// Returns `Err` if the file cannot be read or written.
#[ inline ]
pub fn set_env_var( path : &Path, key : &str, value : &str ) -> Result< (), io::Error >
{
  let mut pairs = read_or_empty( path )?;
  let env_idx   = pairs.iter().position( |( k, _ )| k == "env" );

  let mut env_pairs = match env_idx
  {
    Some( idx ) => json_parse_flat_object( &pairs[ idx ].1 )?,
    None        => vec![],
  };

  upsert_pair( &mut env_pairs, key, value );
  let env_json = json_serialize_env_compact( &env_pairs );

  match env_idx
  {
    Some( idx ) => pairs[ idx ].1 = env_json,
    None        => pairs.push( ( "env".to_string(), env_json ) ),
  }

  let json = json_serialize_flat_object( &pairs );
  atomic_write( path, &json )
}

/// Remove a key from the `"env"` sub-object.  No-op if the key or
/// `"env"` block is absent.  Uses atomic write.
///
/// # Errors
///
/// Returns `Err` if the file cannot be read or written.
#[ inline ]
pub fn remove_env_var( path : &Path, key : &str ) -> Result< (), io::Error >
{
  let mut pairs = match read_all_settings( path )
  {
    Ok( p )  => p,
    Err( e ) if e.kind() == io::ErrorKind::NotFound => return Ok( () ),
    Err( e ) => return Err( e ),
  };

  let Some( idx ) = pairs.iter().position( |( k, _ )| k == "env" ) else { return Ok( () ) };

  let mut env_pairs = json_parse_flat_object( &pairs[ idx ].1 )?;
  let before = env_pairs.len();
  env_pairs.retain( |( k, _ )| k != key );
  if env_pairs.len() == before { return Ok( () ); }

  pairs[ idx ].1 = json_serialize_env_compact( &env_pairs );

  let json = json_serialize_flat_object( &pairs );
  atomic_write( path, &json )
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn read_or_empty( path : &Path ) -> Result< Vec< ( String, String ) >, io::Error >
{
  match read_all_settings( path )
  {
    Ok( p )  => Ok( p ),
    Err( e ) if e.kind() == io::ErrorKind::NotFound => Ok( vec![] ),
    Err( e ) => Err( e ),
  }
}

fn upsert_pair( pairs : &mut Vec< ( String, String ) >, key : &str, value : &str )
{
  if let Some( entry ) = pairs.iter_mut().find( |( k, _ )| k == key )
  {
    entry.1 = value.to_string();
  }
  else
  {
    pairs.push( ( key.to_string(), value.to_string() ) );
  }
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

// ─── Hand-rolled JSON parser ─────────────────────────────────────────────────

/// Parse a JSON object `{key: value, ...}` into `Vec<(String, String)>`.
///
/// Scalar values are returned as their unquoted/parsed string form.
/// Nested objects and arrays are captured as raw JSON strings (verbatim).
fn json_parse_flat_object( src : &str ) -> Result< Vec< ( String, String ) >, io::Error >
{
  let s = src.trim();
  if !s.starts_with( '{' ) || !s.ends_with( '}' )
  {
    return Err( io::Error::new( io::ErrorKind::InvalidData, "expected JSON object" ) );
  }
  let inner = s[ 1 .. s.len() - 1 ].trim();
  if inner.is_empty()
  {
    return Ok( vec![] );
  }

  let chars : Vec< char > = inner.chars().collect();
  let mut pos    = 0;
  let mut result = vec![];

  while pos < chars.len()
  {
    skip_ws( &chars, &mut pos );
    if pos >= chars.len() { break; }

    // Parse key
    if chars[ pos ] != '"'
    {
      return Err( io::Error::new( io::ErrorKind::InvalidData, "expected quoted key" ) );
    }
    let ( key, next ) = parse_json_string( &chars, pos )?;
    pos = next;

    // Expect ':'
    skip_ws( &chars, &mut pos );
    if pos >= chars.len() || chars[ pos ] != ':'
    {
      return Err( io::Error::new( io::ErrorKind::InvalidData, "expected ':' after key" ) );
    }
    pos += 1;

    // Parse value
    skip_ws( &chars, &mut pos );
    let ( value, next ) = parse_json_value( &chars, pos )?;
    pos = next;

    result.push( ( key, value ) );

    skip_ws( &chars, &mut pos );
    if pos < chars.len() && chars[ pos ] == ','
    {
      pos += 1;
    }
  }

  Ok( result )
}

fn skip_ws( chars : &[ char ], pos : &mut usize )
{
  while *pos < chars.len() && chars[ *pos ].is_whitespace()
  {
    *pos += 1;
  }
}

fn parse_json_string( chars : &[ char ], start : usize ) -> Result< ( String, usize ), io::Error >
{
  // start is index of opening '"'
  let mut result = String::new();
  let mut i = start + 1;
  while i < chars.len()
  {
    match chars[ i ]
    {
      '"' => return Ok( ( result, i + 1 ) ),
      '\\' =>
      {
        i += 1;
        if i >= chars.len()
        {
          return Err( io::Error::new( io::ErrorKind::InvalidData, "unterminated escape" ) );
        }
        match chars[ i ]
        {
          '"'  => result.push( '"' ),
          '\\' => result.push( '\\' ),
          '/'  => result.push( '/' ),
          'n'  => result.push( '\n' ),
          'r'  => result.push( '\r' ),
          't'  => result.push( '\t' ),
          'u'  =>
          {
            // Unicode escape: \uXXXX — pass through as literal characters
            if i + 4 < chars.len()
            {
              let hex : String = chars[ i + 1 ..= i + 4 ].iter().collect();
              if let Ok( cp ) = u32::from_str_radix( &hex, 16 )
              {
                if let Some( c ) = char::from_u32( cp )
                {
                  result.push( c );
                }
              }
              i += 4;
            }
          }
          other =>
          {
            // Tolerate unknown escapes by passing them through
            result.push( '\\' );
            result.push( other );
          }
        }
      }
      c => result.push( c ),
    }
    i += 1;
  }
  Err( io::Error::new( io::ErrorKind::InvalidData, "unterminated string" ) )
}

fn parse_json_value( chars : &[ char ], start : usize ) -> Result< ( String, usize ), io::Error >
{
  if start >= chars.len()
  {
    return Err( io::Error::new( io::ErrorKind::InvalidData, "expected value" ) );
  }

  match chars[ start ]
  {
    '"' =>
    {
      let ( s, end ) = parse_json_string( chars, start )?;
      Ok( ( s, end ) )
    }
    '{' | '[' => consume_balanced( chars, start ),
    't' =>
    {
      check_literal( chars, start, &[ 't', 'r', 'u', 'e' ], "true" )
    }
    'f' =>
    {
      check_literal( chars, start, &[ 'f', 'a', 'l', 's', 'e' ], "false" )
    }
    'n' =>
    {
      check_literal( chars, start, &[ 'n', 'u', 'l', 'l' ], "null" )
    }
    c if c == '-' || c.is_ascii_digit() =>
    {
      let end = start + chars[ start .. ]
      .iter()
      .take_while( |&&ch| ch == '-' || ch == '.' || ch.is_ascii_digit() || ch == 'e' || ch == 'E' || ch == '+' )
      .count();
      let num_str : String = chars[ start .. end ].iter().collect();
      Ok( ( num_str, end ) )
    }
    other =>
    {
      Err( io::Error::new(
        io::ErrorKind::InvalidData,
        format!( "unexpected value start '{other}'" ),
      ) )
    }
  }
}

/// Consume a balanced `{ ... }` or `[ ... ]` structure, returning the raw
/// text including the outer delimiters.  Handles nested pairs and strings
/// (so `"}"` inside a string does not close the block).
fn consume_balanced( chars : &[ char ], start : usize ) -> Result< ( String, usize ), io::Error >
{
  let open  = chars[ start ];
  let close = if open == '{' { '}' } else { ']' };
  let mut depth     = 1_u32;
  let mut i         = start + 1;
  let mut in_string = false;
  let mut escape    = false;

  while i < chars.len() && depth > 0
  {
    if escape
    {
      escape = false;
      i += 1;
      continue;
    }
    match chars[ i ]
    {
      '\\' if in_string => { escape = true; }
      '"'               => { in_string = !in_string; }
      c if c == open  && !in_string => { depth += 1; }
      c if c == close && !in_string => { depth -= 1; }
      _ => {}
    }
    i += 1;
  }

  if depth != 0
  {
    return Err( io::Error::new( io::ErrorKind::InvalidData, "unbalanced brackets" ) );
  }

  let raw : String = chars[ start .. i ].iter().collect();
  Ok( ( raw, i ) )
}

fn check_literal(
  chars   : &[ char ],
  start   : usize,
  literal : &[ char ],
  name    : &str,
) -> Result< ( String, usize ), io::Error >
{
  if chars[ start .. ].starts_with( literal )
  {
    Ok( ( name.to_string(), start + literal.len() ) )
  }
  else
  {
    Err( io::Error::new(
      io::ErrorKind::InvalidData,
      format!( "expected '{name}'" ),
    ) )
  }
}

// ─── JSON serializer ─────────────────────────────────────────────────────────

/// Serialize a key-value map to a JSON object string.
///
/// Values are inferred: `Bool` → bare `true`/`false`, `Number` → bare number,
/// `Raw` → verbatim (nested object/array), `Str` → quoted and JSON-escaped.
fn json_serialize_flat_object( pairs : &[ ( String, String ) ] ) -> String
{
  if pairs.is_empty()
  {
    return "{}".to_string();
  }

  let entries : Vec< String > = pairs.iter().map( |( k, v ) |
  {
    let json_key = format!( "\"{}\"", json_escape( k ) );
    let json_val = match infer_type( v )
    {
      StoredAs::Bool | StoredAs::Number | StoredAs::Raw => v.clone(),
      StoredAs::Str  => format!( "\"{}\"", json_escape( v ) ),
    };
    format!( "{json_key}: {json_val}" )
  } ).collect();

  format!( "{{\n  {}\n}}", entries.join( ",\n  " ) )
}

/// Serialize env-var pairs as a compact single-line JSON object.
///
/// All values are stored as JSON strings (environment variables are strings).
fn json_serialize_env_compact( pairs : &[ ( String, String ) ] ) -> String
{
  if pairs.is_empty()
  {
    return "{}".to_string();
  }

  let entries : Vec< String > = pairs.iter().map( |( k, v ) |
    format!( "\"{}\": \"{}\"", json_escape( k ), json_escape( v ) )
  ).collect();

  format!( "{{{}}}", entries.join( ", " ) )
}

/// Escape a string for use inside a JSON quoted string.
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
