//! `.version.list mode::history` — release history from GitHub Releases API with 1-hour cache.

use claude_version_core::version::VERSION_HISTORY;
use unilang::data::{ ErrorCode, ErrorData, OutputData };

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

/// Parse the full GitHub Releases API JSON response into a `Vec<ReleaseInfo>`.
///
/// Returns an empty `Vec` on malformed/non-JSON input or when the top-level
/// value is not an array — matches the pre-existing fallback-on-error contract
/// (see `render_history_mode()`), no panic.
fn extract_releases( json : &str ) -> Vec< ReleaseInfo >
{
  let Ok( parsed ) = claude_storage_core::parse_json( json ) else { return Vec::new(); };
  let Some( array ) = parsed.as_array() else { return Vec::new(); };

  array.iter()
  .map( | entry |
  {
    let version = entry.get_str( "tag_name" )
    .map( | v | v.strip_prefix( 'v' ).unwrap_or( v ).to_string() )
    .unwrap_or_default();

    let date = entry.get_str( "published_at" )
    .map( | d | d.chars().take( 10 ).collect() )
    .unwrap_or_default();

    let body_raw = entry.get_str( "body" ).unwrap_or_default().to_string();

    let summary = body_raw
    .lines()
    .find( | l | l.starts_with( "- " ) )
    .map_or_else( || "(no changelog)".to_string(), | l | l[ 2.. ].trim().to_string() );

    ReleaseInfo { version, date, summary, body : body_raw }
  } )
  .collect()
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

/// Build release entries from the compiled-in `VERSION_HISTORY` snapshot.
///
/// Used when the live GitHub Releases API fetch and the local cache both fail
/// (e.g. no network). `body` is reconstructed as a single bullet from `summary`
/// — the compiled-in snapshot carries no full changelog text — so `v::2` output
/// still renders a structurally valid, if abbreviated, changelog block.
fn fallback_releases() -> Vec< ReleaseInfo >
{
  VERSION_HISTORY.iter()
  .map( | r | ReleaseInfo
  {
    version : r.version.to_string(),
    date    : r.date.to_string(),
    summary : r.summary.to_string(),
    body    : format!( "- {}", r.summary ),
  } )
  .collect()
}

/// `.version.list mode::history` — show release history with changelogs from GitHub.
///
/// Falls back to the compiled-in `VERSION_HISTORY` snapshot (a stderr warning is
/// printed) when the live fetch and the local cache both fail; only a missing
/// `HOME` still surfaces as an error, since the fallback needs no filesystem access.
///
/// `count` and `opts` are parsed by the caller (`version::version_list_routine`'s
/// `mode::` dispatch), shared with `mode::aliases`'s own parameter parsing.
///
/// # Errors
///
/// Returns `Err(InternalError)` when HOME is missing.
#[ allow( clippy::missing_inline_in_public_items, clippy::too_many_lines ) ]
pub( super ) fn render_history_mode( count : usize, opts : &OutputOptions ) -> Result< OutputData, ErrorData >
{
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
  let mut releases = if let Ok( json ) = fetch_releases_json( paths.base() )
  {
    extract_releases( &json )
  }
  else
  {
    eprintln!( "warning: .version.list mode::history could not reach the GitHub Releases API; showing compiled-in offline snapshot (versions 2.1.74-2.1.220)" );
    fallback_releases()
  };
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

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn t01_well_formed_two_elements_ascii_only()
  {
    let json = r#"[{"tag_name": "v1.2.3", "published_at": "2024-01-15T10:00:00Z", "body": "- Fixed bug A\n- Added feature B"},{"tag_name": "v1.2.2", "published_at": "2024-01-01T09:00:00Z", "body": "- Initial release"}]"#;
    let releases = extract_releases( json );
    assert_eq!( releases.len(), 2 );
    assert_eq!( releases[ 0 ].version, "1.2.3" );
    assert_eq!( releases[ 0 ].date, "2024-01-15" );
    assert_eq!( releases[ 0 ].summary, "Fixed bug A" );
    assert_eq!( releases[ 0 ].body, "- Fixed bug A\n- Added feature B" );
    assert_eq!( releases[ 1 ].version, "1.2.2" );
    assert_eq!( releases[ 1 ].date, "2024-01-01" );
    assert_eq!( releases[ 1 ].summary, "Initial release" );
  }

  #[ test ]
  fn t02_standard_escapes_in_body()
  {
    let json = r#"[{"tag_name": "v2.0.0", "published_at": "2024-02-01T00:00:00Z", "body": "- Say \"quoted\" text\nSecond line with backslash: \\"}]"#;
    let releases = extract_releases( json );
    assert_eq!( releases.len(), 1 );
    assert_eq!( releases[ 0 ].body, "- Say \"quoted\" text\nSecond line with backslash: \\" );
  }

  #[ test ]
  fn t03_non_bmp_codepoint_surrogate_pair_in_body()
  {
    let json = r#"[{"tag_name": "v3.0.0", "published_at": "2024-03-01T00:00:00Z", "body": "- Celebration \uD83D\uDE00 release"}]"#;
    let releases = extract_releases( json );
    assert_eq!( releases.len(), 1 );
    assert_eq!( releases[ 0 ].body, "- Celebration \u{1F600} release" );
  }

  #[ test ]
  fn t04_missing_body_field_defaults_empty()
  {
    let json = r#"[{"tag_name": "v4.0.0", "published_at": "2024-04-01T00:00:00Z"}]"#;
    let releases = extract_releases( json );
    assert_eq!( releases.len(), 1 );
    assert_eq!( releases[ 0 ].body, "" );
    assert_eq!( releases[ 0 ].summary, "(no changelog)" );
  }

  #[ test ]
  fn t05_malformed_input_returns_empty_vec_no_panic()
  {
    let json = "this is not valid json at all {{{";
    let releases = extract_releases( json );
    assert!( releases.is_empty() );
  }

  #[ test ]
  fn t06_tag_without_v_prefix_is_not_silently_dropped()
  {
    let json = r#"[{"tag_name": "1.0.0", "published_at": "2024-01-01", "body": "no-v-prefix"},{"tag_name": "v2.0.0", "published_at": "2024-02-01", "body": "has-v-prefix"}]"#;
    let releases = extract_releases( json );
    assert_eq!( releases.len(), 2, "a release tagged without a leading 'v' must not be dropped" );
    assert_eq!( releases[ 0 ].version, "1.0.0" );
    assert_eq!( releases[ 1 ].version, "2.0.0" );
  }

  #[ test ]
  fn t07_unpaired_high_surrogate_does_not_corrupt_body()
  {
    let json = "[{\"tag_name\": \"v3.0.0\", \"published_at\": \"2024-03-01\", \"body\": \"before \\uD83D after\"}]";
    let releases = extract_releases( json );
    assert_eq!( releases.len(), 1 );
    assert_eq!( releases[ 0 ].body, "before \u{FFFD} after", "unpaired high surrogate must become U+FFFD, not corrupt or truncate the surrounding text" );
  }
}
