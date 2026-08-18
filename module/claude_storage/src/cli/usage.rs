//! `.usage` command — per-session usage table (turns, tokens, duration, dir).
//!
//! See `docs/cli/command/13_usage.md` for the full contract this implements.

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use claude_storage_core::{ EntryType, Project, Session, SessionStats };
use super::storage::create_storage;
use super::scope::{ validate_scope, resolve_scoped_projects, resolve_base_path };

/// Byte length of a canonical UUID session ID.
const UUID_LEN : usize = 36;
/// Characters shown of a UUID session ID.
const UUID_SHORT_LEN : usize = 8;
/// Command column truncation threshold, in characters.
const COMMAND_MAX_CHARS : usize = 35;

/// Reproduced byte-for-byte from `docs/cli/command/13_usage.md`'s worked
/// example. NOT derivable from `render_row`'s format string: the example's own
/// `Cache` heading sits one column right of the data rows' values — the spec's
/// quirk is kept verbatim so the rendered table matches the doc exactly.
const TABLE_HEADER : &str = "Session   Command                            Turns      In     Out   Cache      Dur  Dir";

/// One rendered table row plus the sort key (session-file mtime).
struct UsageRow
{
  session_id : String,
  command : String,
  turns : usize,
  input : u64,
  output : u64,
  cache : u64,
  dur_secs : i64,
  dir : String,
  mtime : std::time::SystemTime,
}

/// Per-session usage table: turns, token totals, wall-clock duration, and
/// working directory, most recently modified session first.
///
/// Parameters (see `docs/cli/command/13_usage.md`):
/// - `scope::` — project selection (default `local`)
/// - `path::` — anchor directory overriding cwd
/// - `depth::` — component-distance cap for `under`/`relevant`/`around`
///   (default 3, `0` = unbounded)
/// - `limit::` — flat row cap across the whole result set (default 0 = all)
///
/// # Errors
///
/// Returns error (exit 1) when `depth`/`limit` is negative or `scope` is not
/// one of the five documented values.
///
/// # Exit Codes
///
/// Exits directly with code 2 when `scope::local` resolves no project —
/// matches the `.tail`/`.status` "not found = usage error" convention.
///
/// # Panics
///
/// Does not panic — both `usize` conversions below are only reached after the
/// negative-value branches already returned.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn usage_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  // Validate arguments before any storage access, per INT-19/20/21.
  let depth = cmd.get_integer( "depth" ).unwrap_or( 3 );
  if depth < 0
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "depth must be non-negative".to_string() ) );
  }
  let limit = cmd.get_integer( "limit" ).unwrap_or( 0 );
  if limit < 0
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "limit must be non-negative".to_string() ) );
  }
  let scope = validate_scope( cmd.get_string( "scope" ), "local" )?;
  let path_raw = cmd.get_string( "path" );

  let storage = create_storage()?;
  let projects = resolve_scoped_projects( &storage, &scope, path_raw )?;
  if scope == "local" && projects.is_empty()
  {
    exit_no_project( path_raw );
  }

  // depth:: caps candidate distance only for the walking scopes; `local` and
  // `global` ignore it (docs/cli/param/26_depth.md).
  let depth_filter = if depth > 0 && matches!( scope.as_str(), "under" | "relevant" | "around" )
  {
    let cap = usize::try_from( depth ).expect( "depth < 0 rejected above" );
    Some( ( cap, resolve_base_path( path_raw )? ) )
  }
  else
  {
    None
  };

  let mut rows = collect_rows( &projects, depth_filter.as_ref() );
  rows.sort_by_key( | row | core::cmp::Reverse( row.mtime ) );
  if limit > 0
  {
    rows.truncate( usize::try_from( limit ).expect( "limit < 0 rejected above" ) );
  }

  let mut output = String::from( TABLE_HEADER );
  for row in &rows
  {
    output.push( '\n' );
    output.push_str( &render_row( row ) );
  }
  Ok( OutputData::new( output, "text" ) )
}

/// Report the missing local-scope project on stderr and exit 2.
fn exit_no_project( path_raw : Option< &str > ) -> !
{
  if let Some( raw ) = path_raw
  {
    match resolve_base_path( Some( raw ) )
    {
      Ok( resolved ) => eprintln!( "No project found for path: {}", resolved.display() ),
      Err( _ ) => eprintln!( "No project found for path: {raw}" ),
    }
  }
  else
  {
    eprintln!( "No project found for current directory" );
  }
  std::process::exit( 2 );
}

/// Build unsorted rows for every non-agent session across `projects`,
/// dropping sessions beyond the depth cap when one is set.
fn collect_rows(
  projects     : &[ Project ],
  depth_filter : Option< &( usize, std::path::PathBuf ) >,
) -> Vec< UsageRow >
{
  let mut rows = Vec::new();
  for project in projects
  {
    let Ok( sessions ) = project.sessions() else { continue };
    for mut session in sessions
    {
      // Belt and braces: `Project::sessions()` already excludes `agent-*`
      // sidecar files; the guard keeps the exclusion explicit and local.
      if session.is_agent_session() { continue; }
      let Ok( stats ) = session.stats() else { continue };
      if let Some( ( cap, base ) ) = depth_filter
      {
        if beyond_depth( &stats, *cap, base ) { continue; }
      }
      rows.push( build_row( &mut session, &stats ) );
    }
  }
  rows
}

/// Is the session's recorded cwd more than `cap` path components from `base`?
///
/// A session with no recorded cwd is kept — mirrors the scope resolver's own
/// conservative-include fallback for unverifiable paths.
fn beyond_depth( stats : &SessionStats, cap : usize, base : &std::path::Path ) -> bool
{
  let Some( cwd ) = stats.cwd.as_deref() else { return false };
  component_distance( std::path::Path::new( cwd ), base ) > cap
}

/// Absolute difference in path component count between `a` and `b`.
fn component_distance( a : &std::path::Path, b : &std::path::Path ) -> usize
{
  a.components().count().abs_diff( b.components().count() )
}

/// Assemble one row from a session's stats plus its first-command text.
fn build_row( session : &mut Session, stats : &SessionStats ) -> UsageRow
{
  UsageRow
  {
    session_id : session.id().to_string(),
    command : session_command( session ),
    turns : stats.assistant_entries,
    input : stats.total_input_tokens,
    output : stats.total_output_tokens,
    cache : stats.total_cache_read_tokens,
    dur_secs : duration_secs( stats ),
    dir : stats.cwd.clone().unwrap_or_default(),
    mtime : session_mtime( session ),
  }
}

/// Session file modification time (`UNIX_EPOCH` when unreadable).
///
/// Deliberately a local copy of `.projects`' private helper — Task 511 keeps
/// `projects.rs` untouched (C8).
fn session_mtime( session : &Session ) -> std::time::SystemTime
{
  std::fs::metadata( session.storage_path() )
    .and_then( | m | m.modified() )
    .unwrap_or( std::time::SystemTime::UNIX_EPOCH )
}

/// Wall-clock span in seconds between the session's first and last entry
/// timestamps; 0 when either timestamp is absent or unparseable.
fn duration_secs( stats : &SessionStats ) -> i64
{
  let Some( first ) = stats.first_timestamp.as_deref().and_then( parse_iso_seconds ) else { return 0 };
  let Some( last ) = stats.last_timestamp.as_deref().and_then( parse_iso_seconds ) else { return 0 };
  ( last - first ).max( 0 )
}

/// Parse an ISO-8601 UTC timestamp (`YYYY-MM-DDTHH:MM:SS[.fff]Z`) to seconds
/// since the UNIX epoch. Hand-rolled — the crate deliberately has no date
/// dependency; only same-format subtraction is ever performed on the result.
fn parse_iso_seconds( ts : &str ) -> Option< i64 >
{
  let ( date, time ) = ts.split_once( 'T' )?;
  let mut dp = date.split( '-' );
  let year : i64 = dp.next()?.parse().ok()?;
  let month : i64 = dp.next()?.parse().ok()?;
  let day : i64 = dp.next()?.parse().ok()?;
  if dp.next().is_some() { return None; }
  let time = time.strip_suffix( 'Z' ).unwrap_or( time );
  let time = time.split_once( '.' ).map_or( time, | ( whole, _ ) | whole );
  let mut tp = time.split( ':' );
  let hour : i64 = tp.next()?.parse().ok()?;
  let minute : i64 = tp.next()?.parse().ok()?;
  let second : i64 = tp.next()?.parse().ok()?;
  if tp.next().is_some() { return None; }
  Some( days_from_civil( year, month, day ) * 86_400 + hour * 3600 + minute * 60 + second )
}

/// Days from the civil epoch (1970-01-01) for a proleptic Gregorian date —
/// Howard Hinnant's `days_from_civil` algorithm.
fn days_from_civil( year : i64, month : i64, day : i64 ) -> i64
{
  let year = if month <= 2 { year - 1 } else { year };
  let era = ( if year >= 0 { year } else { year - 399 } ) / 400;
  let yoe = year - era * 400;
  let mp = if month > 2 { month - 3 } else { month + 9 };
  let doy = ( 153 * mp + 2 ) / 5 + day - 1;
  let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
  era * 146_097 + doe - 719_468
}

/// Format a token count: bare below 1000, `N.Nk` in the thousands, `N.NM`
/// from a million up (`docs/cli/command/13_usage.md` column rules).
fn format_tokens( n : u64 ) -> String
{
  if n < 1_000
  {
    format!( "{n}" )
  }
  else if n < 1_000_000
  {
    format!( "{:.1}k", n as f64 / 1_000.0 )
  }
  else
  {
    format!( "{:.1}M", n as f64 / 1_000_000.0 )
  }
}

/// Format a duration: `Ns` below a minute, `NmNNs` below an hour, `NhNNm`
/// from an hour up (`docs/cli/command/13_usage.md` column rules).
fn format_duration( secs : i64 ) -> String
{
  let secs = secs.max( 0 );
  if secs < 60
  {
    format!( "{secs}s" )
  }
  else if secs < 3600
  {
    format!( "{}m{:02}s", secs / 60, secs % 60 )
  }
  else
  {
    format!( "{}h{:02}m", secs / 3600, ( secs % 3600 ) / 60 )
  }
}

/// First main-chain user message of the session, `<command-name>` unwrapped,
/// newline-flattened, and truncated for the Command column. Empty when the
/// session has no usable user entry.
fn session_command( session : &mut Session ) -> String
{
  let Ok( entries ) = session.entries() else { return String::new() };
  for entry in entries
  {
    if !matches!( entry.entry_type, EntryType::User ) || entry.is_sidechain { continue; }
    let raw = entry.content_text();
    let text = unwrap_command_name( &raw ).replace( [ '\r', '\n' ], " " );
    let text = text.trim();
    if text.is_empty() { continue; }
    return truncate_command( text );
  }
  String::new()
}

/// Extract the slash-command name from a `<command-name>…</command-name>`
/// wrapper; plain text passes through unchanged.
fn unwrap_command_name( text : &str ) -> &str
{
  let Some( start ) = text.find( "<command-name>" ) else { return text };
  let inner = &text[ start + "<command-name>".len().. ];
  let Some( end ) = inner.find( "</command-name>" ) else { return text };
  inner[ ..end ].trim()
}

/// Cut command text at 35 characters, marking the cut with a trailing `…`.
fn truncate_command( text : &str ) -> String
{
  if text.chars().count() <= COMMAND_MAX_CHARS
  {
    return text.to_string();
  }
  let mut truncated : String = text.chars().take( COMMAND_MAX_CHARS ).collect();
  truncated.push( '…' );
  truncated
}

/// First 8 characters of a UUID-shaped session ID; other IDs pass through.
///
/// Deliberately a local copy of `.projects`' private helper — Task 511 keeps
/// `projects.rs` untouched (C8).
fn short_id( id : &str ) -> &str
{
  if id.len() == UUID_LEN && id.as_bytes().get( 8 ) == Some( &b'-' )
  {
    &id[ ..UUID_SHORT_LEN ]
  }
  else
  {
    id
  }
}

/// Render one data row. Column widths measured from the worked example in
/// `docs/cli/command/13_usage.md` — they reproduce both example rows exactly.
fn render_row( row : &UsageRow ) -> String
{
  format!(
    "{:<8}  {:<35}{:>5}{:>8}{:>8}{:>7}{:>9}  {}",
    short_id( &row.session_id ),
    row.command,
    row.turns,
    format_tokens( row.input ),
    format_tokens( row.output ),
    format_tokens( row.cache ),
    format_duration( row.dur_secs ),
    row.dir,
  )
}
