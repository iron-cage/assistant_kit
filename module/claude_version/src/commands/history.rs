//! `.version.history` — release history from GitHub Releases API with 1-hour cache.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use crate::output::{ OutputFormat, OutputOptions, json_escape };

const RELEASES_API_URL  : &str = "https://api.github.com/repos/anthropics/claude-code/releases?per_page=100";
const CACHE_TTL_SECS    : u64  = 3600;

/// A parsed release entry from the GitHub Releases API.
struct ReleaseInfo
{
  version : String,
  date    : String,
  summary : String,
  body    : String,
}

/// Extract the string value for a given JSON key from raw JSON text.
///
/// # Pitfall
///
/// Never iterate `json.as_bytes()` by index and cast each byte to `char`.
/// `bytes[i] as char` interprets each byte as a Unicode scalar of the same
/// value, silently corrupting every multi-byte UTF-8 sequence (any codepoint
/// above U+007F). Use `str::chars()` instead — it respects character
/// boundaries natively.
fn parse_json_string_value( json : &str, key : &str ) -> Option< String >
{
  let colon_pat = format!( "\"{key}\":" );
  let colon_pos = json.find( &colon_pat )? + colon_pat.len();
  let rest  = &json[ colon_pos.. ];
  let quote = rest.find( '"' )?;
  // Byte offset of the character after the opening quote.
  let value_start = colon_pos + quote + 1;
  let content = &json[ value_start.. ];

  let mut out     = String::new();
  let mut chars   = content.chars();
  let mut escaped = false;

  while let Some( ch ) = chars.next()
  {
    if escaped
    {
      match ch
      {
        'n'  => out.push( '\n' ),
        'r'  => out.push( '\r' ),
        't'  => out.push( '\t' ),
        'b'  => out.push( '\x08' ),  // backspace (JSON \b)
        'f'  => out.push( '\x0C' ),  // form feed  (JSON \f)
        '"'  => out.push( '"'  ),
        '\\' => out.push( '\\' ),
        'u'  =>
        {
          // Consume exactly 4 hex digits that follow \u.
          let hex : String = chars.by_ref().take( 4 ).collect();
          if hex.len() == 4
          {
            if let Ok( cp ) = u32::from_str_radix( &hex, 16 )
            {
              // UTF-16 surrogate pair: high surrogate must be followed by \uLLLL.
              if ( 0xD800..=0xDBFF ).contains( &cp )
              {
                let mut low_hex = String::new();
                if chars.next() == Some( '\\' ) && chars.next() == Some( 'u' )
                {
                  low_hex = chars.by_ref().take( 4 ).collect();
                }
                if low_hex.len() == 4
                {
                  if let Ok( lo ) = u32::from_str_radix( &low_hex, 16 )
                  {
                    if ( 0xDC00..=0xDFFF ).contains( &lo )
                    {
                      let scalar = 0x1_0000 + ( ( cp - 0xD800 ) << 10 ) + ( lo - 0xDC00 );
                      if let Some( c ) = char::from_u32( scalar )
                      {
                        out.push( c );
                      }
                    }
                  }
                }
              }
              else if let Some( c ) = char::from_u32( cp )
              {
                out.push( c );
              }
            }
          }
        }
        other => out.push( other ),
      }
      escaped = false;
    }
    else if ch == '\\'
    {
      escaped = true;
    }
    else if ch == '"'
    {
      return Some( out );
    }
    else
    {
      out.push( ch );
    }
  }

  None
}

/// Parse the full GitHub Releases API JSON response into a `Vec<ReleaseInfo>`.
fn extract_releases( json : &str ) -> Vec< ReleaseInfo >
{
  // Support both spaced ("tag_name": "v) and compact ("tag_name":"v) GitHub API formats.
  let marker_spaced  = "\"tag_name\": \"v";
  let marker_compact = "\"tag_name\":\"v";
  let ( marker, chunks ) : ( &str, Vec< &str > ) = if json.contains( marker_spaced )
  {
    ( marker_spaced,  json.split( marker_spaced  ).collect() )
  }
  else
  {
    ( marker_compact, json.split( marker_compact ).collect() )
  };

  let mut releases = Vec::new();

  for chunk in chunks.iter().skip( 1 )
  {
    let restored = format!( "{marker}{chunk}" );

    let version = parse_json_string_value( &restored, "tag_name" )
    .map( | v | v.strip_prefix( 'v' ).unwrap_or( &v ).to_string() )
    .unwrap_or_default();

    let date = parse_json_string_value( &restored, "published_at" )
    .map( | d | d.chars().take( 10 ).collect() )
    .unwrap_or_default();

    let body_raw = parse_json_string_value( &restored, "body" )
    .unwrap_or_default();

    let summary = body_raw
    .lines()
    .find( | l | l.starts_with( "- " ) )
    .map_or_else( || "(no changelog)".to_string(), | l | l[ 2.. ].trim().to_string() );

    releases.push( ReleaseInfo { version, date, summary, body : body_raw } );
  }

  releases
}

/// Check whether the cache file's mtime is less than 1 hour old.
fn cache_is_fresh( path : &std::path::Path ) -> bool
{
  std::fs::metadata( path )
  .and_then( | m | m.modified() )
  .ok()
  .and_then( | mtime | std::time::SystemTime::now().duration_since( mtime ).ok() )
  .is_some_and( | elapsed | elapsed.as_secs() < CACHE_TTL_SECS )
}

/// Fetch releases JSON, using a 1-hour file cache in `~/.claude/.transient/`.
fn fetch_releases_json( base : &std::path::Path ) -> Result< String, ErrorData >
{
  let cache_dir  = base.join( ".transient" );
  let cache_path = cache_dir.join( "version_history_cache.json" );

  if cache_is_fresh( &cache_path )
  {
    if let Ok( cached ) = std::fs::read_to_string( &cache_path )
    {
      if !cached.is_empty()
      {
        return Ok( cached );
      }
    }
  }

  let output = std::process::Command::new( "curl" )
  .args( [ "-fsSL", RELEASES_API_URL ] )
  .output()
  .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "curl not found or fetch failed: {e}" ) ) )?;

  if !output.status.success()
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "failed to fetch release history".to_string() ) );
  }

  let response = String::from_utf8_lossy( &output.stdout ).to_string();
  if response.trim().is_empty()
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "empty response from GitHub API".to_string() ) );
  }

  let _ = std::fs::create_dir_all( &cache_dir );
  let _ = std::fs::write( &cache_path, &response );

  Ok( response )
}

/// `.version.history` — show release history with changelogs from GitHub.
///
/// # Errors
///
/// Returns `Err(InternalError)` when HOME is missing or the network request fails.
/// Returns `Err(ArgumentTypeMismatch)` when `format::` has an invalid value.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn version_history_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let count = match cmd.arguments.get( "count" )
  {
    Some( Value::Integer( n ) ) => usize::try_from( *n ).unwrap_or( 10 ),
    _                           => 10,
  };

  // count::0 needs no network call — return the appropriate empty response immediately.
  if count == 0
  {
    let content = match opts.format
    {
      OutputFormat::Json => "[]\n".to_string(),
      OutputFormat::Text => String::new(),
    };
    return Ok( OutputData::new( content, "text" ) );
  }

  let paths = super::require_claude_paths()?;
  let json  = fetch_releases_json( paths.base() )?;
  let mut releases = extract_releases( &json );
  releases.truncate( count );

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      if releases.is_empty()
      {
        "[]\n".to_string()
      }
      else
      {
        let entries : Vec< String > = releases.iter().map( | r |
        {
          let v = json_escape( &r.version );
          let d = json_escape( &r.date );
          let s = json_escape( &r.summary );
          format!( "  {{\"version\":\"{v}\",\"date\":\"{d}\",\"summary\":\"{s}\"}}" )
        } ).collect();
        format!( "[\n{}\n]\n", entries.join( ",\n" ) )
      }
    }
    ( OutputFormat::Text, 0 ) =>
    {
      if releases.is_empty()
      {
        String::new()
      }
      else
      {
        let lines : Vec< String > = releases.iter()
        .map( | r | format!( "{}  {}", r.version, r.date ) )
        .collect();
        format!( "{}\n", lines.join( "\n" ) )
      }
    }
    ( OutputFormat::Text, 1 ) =>
    {
      if releases.is_empty()
      {
        String::new()
      }
      else
      {
        let lines : Vec< String > = releases.iter()
        .map( | r | format!( "{}  {}  {}", r.version, r.date, r.summary ) )
        .collect();
        format!( "{}\n", lines.join( "\n" ) )
      }
    }
    ( OutputFormat::Text, _ ) =>
    {
      if releases.is_empty()
      {
        String::new()
      }
      else
      {
        let blocks : Vec< String > = releases.iter()
        .map( | r |
        {
          let header = format!( "## {} ({})", r.version, r.date );
          if r.body.is_empty()
          {
            header
          }
          else
          {
            format!( "{header}\n\n{}", r.body )
          }
        } )
        .collect();
        format!( "{}\n", blocks.join( "\n\n" ) )
      }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}
