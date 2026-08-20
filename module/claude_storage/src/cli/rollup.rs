//! `.rollup` command — flexible grouped/filtered/sorted/projected token-usage
//! table.
//!
//! See `docs/cli/command/14_rollup.md` for the full CLI contract this
//! implements. Aggregation itself (grouping, filtering, sorting, percent
//! computation) is delegated entirely to `claude_storage_core::rollup` — this
//! file only walks scope-resolved sessions into `RollupInput`s, parses the 5
//! new CLI parameters, and renders the chosen column projection. Never
//! duplicates the core engine's own grouping/sort/filter logic (see
//! `claude_storage_core/src/rollup.rs`'s own doc comment for that split).

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use claude_storage_core::
{
  GroupKey, SortKey, SortOrder, StringMatcher, Project, RollupInput, RollupParams, RollupRow,
  build_rollup,
};
use super::storage::create_storage;
use super::scope::{ validate_scope, resolve_scoped_projects, resolve_base_path };

/// Column selectable via `columns::` — a CLI-only projection concern,
/// mirroring `.usage`'s own `render_row`/`format_tokens` split: the core
/// engine always computes every [`RollupRow`] field, this enum only picks
/// which subset to print and in what order.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
enum ColumnKey
{
  /// Group label (session id / project cwd / model / day).
  Group,
  /// Number of distinct contributing sessions.
  Sessions,
  /// Number of deduplicated assistant turns.
  Calls,
  /// Fresh (non-cached) input tokens.
  Input,
  /// Generated output tokens.
  Output,
  /// Combined `cache_read + cache_creation` tokens.
  Cache,
  /// Largest single call's context size (the "window size" metric).
  MaxContext,
  /// `input + output + cache`.
  Total,
  /// Share of the full filtered grand total, as a percentage.
  Percent,
  /// Earliest contributing timestamp.
  First,
  /// Latest contributing timestamp.
  Last,
}

/// Default column set shown when `columns::` is not given. Omits `First`/
/// `Last` (verbose, niche) but keeps every count/token metric visible,
/// including `MaxContext` — the "window size" metric this command exists
/// partly to surface (see `docs/cli/command/14_rollup.md`'s Notes).
const DEFAULT_COLUMNS : &[ ColumnKey ] =
&[
  ColumnKey::Group, ColumnKey::Sessions, ColumnKey::Calls,
  ColumnKey::Input, ColumnKey::Output, ColumnKey::Cache,
  ColumnKey::MaxContext, ColumnKey::Total, ColumnKey::Percent,
];

/// Flexible token-usage table: group by session/project/model/day, filter by
/// model substring, sort by any computed column, and project only the
/// columns you want.
///
/// Parameters (see `docs/cli/command/14_rollup.md`):
/// - `group::` — grouping dimension (default `session`)
/// - `sort::` — sort column (default `total`)
/// - `order::` — sort direction (default `desc`)
/// - `model::` — model substring filter, applied before grouping
/// - `columns::` — comma-separated column projection (default: see
///   [`DEFAULT_COLUMNS`])
/// - `scope::`, `path::`, `depth::`, `limit::` — reused unchanged from
///   `.usage` (see `docs/cli/command/13_usage.md`)
///
/// # Errors
///
/// Returns error (exit 1) when `depth`/`limit` is negative, `scope` is not
/// one of the five documented values, `group`/`sort`/`order` is not one of
/// their documented values, or `columns::` names an unknown column.
///
/// # Exit Codes
///
/// Exits directly with code 2 when `scope::local` resolves no project —
/// matches the `.usage`/`.tail`/`.status` "not found = usage error"
/// convention.
///
/// # Panics
///
/// Does not panic — both `usize` conversions below are only reached after
/// the negative-value branches already returned.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn rollup_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  // Validate arguments before any storage access, mirroring `.usage`
  // (docs/cli/command/13_usage.md's Algorithm step 1).
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
  let group_by = parse_group( cmd.get_string( "group" ) )?;
  let sort_by = parse_sort( cmd.get_string( "sort" ) )?;
  let order = parse_order( cmd.get_string( "order" ) )?;
  let columns = parse_columns( cmd.get_string( "columns" ) )?;
  let model_filter = cmd.get_string( "model" ).map( StringMatcher::new );
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

  let inputs = collect_inputs( &projects, depth_filter.as_ref() );
  let params = RollupParams
  {
    group_by,
    sort_by,
    order,
    model_filter,
    limit : usize::try_from( limit ).expect( "limit < 0 rejected above" ),
  };
  let rows = build_rollup( &inputs, &params );

  let mut output = render_header( &columns );
  for row in &rows
  {
    output.push( '\n' );
    output.push_str( &render_row( row, &columns ) );
  }
  Ok( OutputData::new( output, "text" ) )
}

/// Report the missing local-scope project on stderr and exit 2.
///
/// Deliberately a local copy of `.usage`'s own `exit_no_project` — see
/// `usage.rs`'s `session_mtime`/`short_id` precedent comments for the same
/// established rationale: small, command-specific glue stays local rather
/// than coupling two independent commands through a shared private helper.
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

/// Build unsorted [`RollupInput`]s for every non-agent session across
/// `projects`, dropping sessions beyond the depth cap when one is set.
///
/// `project_label` is each session's own recorded `stats.cwd` (falling back
/// to `"unknown"`, matching `GroupKey::Model`/`GroupKey::Day`'s own fallback
/// convention in the core engine) rather than the lossy-encoded storage
/// directory name — the same field `.usage`'s `Dir` column already shows,
/// and simpler/more transparent than decoding `cli/scope.rs`'s internal
/// storage-path representation for a second, unrelated purpose.
///
/// Deliberately a local copy of `.usage`'s own `collect_rows` shape (same
/// precedent as `exit_no_project` above).
fn collect_inputs(
  projects     : &[ Project ],
  depth_filter : Option< &( usize, std::path::PathBuf ) >,
) -> Vec< RollupInput >
{
  let mut inputs = Vec::new();
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
      let project_label = stats.cwd.clone().unwrap_or_else( || "unknown".to_string() );
      inputs.push( RollupInput { session_id : session.id().to_string(), project_label, stats } );
    }
  }
  inputs
}

/// Is the session's recorded cwd more than `cap` path components from `base`?
///
/// A session with no recorded cwd is kept — mirrors the scope resolver's own
/// conservative-include fallback for unverifiable paths. Deliberately a
/// local copy of `.usage`'s own `beyond_depth` (same precedent as
/// `exit_no_project` above).
fn beyond_depth( stats : &claude_storage_core::SessionStats, cap : usize, base : &std::path::Path ) -> bool
{
  let Some( cwd ) = stats.cwd.as_deref() else { return false };
  component_distance( std::path::Path::new( cwd ), base ) > cap
}

/// Absolute difference in path component count between `a` and `b`.
/// Deliberately a local copy of `.usage`'s own `component_distance`.
fn component_distance( a : &std::path::Path, b : &std::path::Path ) -> usize
{
  a.components().count().abs_diff( b.components().count() )
}

/// Parse `group::` (default `session`) to a [`GroupKey`].
fn parse_group( raw : Option< &str > ) -> core::result::Result< GroupKey, ErrorData >
{
  match raw.unwrap_or( "session" ).to_lowercase().as_str()
  {
    "session" => Ok( GroupKey::Session ),
    "project" => Ok( GroupKey::Project ),
    "model" => Ok( GroupKey::Model ),
    "day" => Ok( GroupKey::Day ),
    other => Err( ErrorData::new( ErrorCode::InternalError, format!( "group must be session|project|model|day, got {other}" ) ) ),
  }
}

/// Parse `sort::` (default `total`) to a [`SortKey`].
fn parse_sort( raw : Option< &str > ) -> core::result::Result< SortKey, ErrorData >
{
  match raw.unwrap_or( "total" ).to_lowercase().as_str()
  {
    "total" => Ok( SortKey::Total ),
    "input" => Ok( SortKey::Input ),
    "output" => Ok( SortKey::Output ),
    "cache" => Ok( SortKey::Cache ),
    "max_context" => Ok( SortKey::MaxContext ),
    "calls" => Ok( SortKey::Calls ),
    "sessions" => Ok( SortKey::Sessions ),
    "group" => Ok( SortKey::Group ),
    other => Err( ErrorData::new( ErrorCode::InternalError,
      format!( "sort must be total|input|output|cache|max_context|calls|sessions|group, got {other}" ) ) ),
  }
}

/// Parse `order::` (default `desc`) to a [`SortOrder`].
fn parse_order( raw : Option< &str > ) -> core::result::Result< SortOrder, ErrorData >
{
  match raw.unwrap_or( "desc" ).to_lowercase().as_str()
  {
    "desc" => Ok( SortOrder::Desc ),
    "asc" => Ok( SortOrder::Asc ),
    other => Err( ErrorData::new( ErrorCode::InternalError, format!( "order must be asc|desc, got {other}" ) ) ),
  }
}

/// Parse `columns::` (default: [`DEFAULT_COLUMNS`]) to an ordered column
/// list — a bare comma-separated list, e.g. `columns::group,total,calls`.
fn parse_columns( raw : Option< &str > ) -> core::result::Result< Vec< ColumnKey >, ErrorData >
{
  let Some( raw ) = raw else { return Ok( DEFAULT_COLUMNS.to_vec() ) };
  raw.split( ',' ).map( str::trim ).map( parse_column ).collect()
}

/// Parse one `columns::` entry (case-insensitive) to a [`ColumnKey`].
fn parse_column( name : &str ) -> core::result::Result< ColumnKey, ErrorData >
{
  match name.to_lowercase().as_str()
  {
    "group" => Ok( ColumnKey::Group ),
    "sessions" => Ok( ColumnKey::Sessions ),
    "calls" => Ok( ColumnKey::Calls ),
    "input" => Ok( ColumnKey::Input ),
    "output" => Ok( ColumnKey::Output ),
    "cache" => Ok( ColumnKey::Cache ),
    "max_context" => Ok( ColumnKey::MaxContext ),
    "total" => Ok( ColumnKey::Total ),
    "percent" => Ok( ColumnKey::Percent ),
    "first" => Ok( ColumnKey::First ),
    "last" => Ok( ColumnKey::Last ),
    other => Err( ErrorData::new( ErrorCode::InternalError, format!(
      "unknown column '{other}' — valid: group|sessions|calls|input|output|cache|max_context|total|percent|first|last"
    ) ) ),
  }
}

/// Fixed print width for one column, regardless of which columns are shown
/// alongside it — keeps any `columns::` combination aligned predictably.
fn column_width( col : ColumnKey ) -> usize
{
  match col
  {
    ColumnKey::Group => 24,
    ColumnKey::Sessions | ColumnKey::Input | ColumnKey::Output | ColumnKey::Cache | ColumnKey::MaxContext | ColumnKey::Total => 8,
    ColumnKey::Calls | ColumnKey::Percent => 6,
    ColumnKey::First | ColumnKey::Last => 20,
  }
}

/// Left-aligned columns are text (group label, timestamps); every numeric
/// column is right-aligned.
fn is_left_aligned( col : ColumnKey ) -> bool
{
  matches!( col, ColumnKey::Group | ColumnKey::First | ColumnKey::Last )
}

/// Header text for one column.
fn column_header( col : ColumnKey ) -> &'static str
{
  match col
  {
    ColumnKey::Group => "Group",
    ColumnKey::Sessions => "Sessions",
    ColumnKey::Calls => "Calls",
    ColumnKey::Input => "Input",
    ColumnKey::Output => "Output",
    ColumnKey::Cache => "Cache",
    ColumnKey::MaxContext => "MaxCtx",
    ColumnKey::Total => "Total",
    ColumnKey::Percent => "Pct",
    ColumnKey::First => "First",
    ColumnKey::Last => "Last",
  }
}

/// Render the full header line for the chosen `columns::` projection.
fn render_header( columns : &[ ColumnKey ] ) -> String
{
  columns.iter().map( | &col |
  {
    let width = column_width( col );
    let label = column_header( col );
    if is_left_aligned( col ) { format!( "{label:<width$}" ) } else { format!( "{label:>width$}" ) }
  } ).collect::< Vec< _ > >().join( "  " )
}

/// Render one data row for the chosen `columns::` projection.
fn render_row( row : &RollupRow, columns : &[ ColumnKey ] ) -> String
{
  columns.iter().map( | &col | render_cell( row, col ) ).collect::< Vec< _ > >().join( "  " )
}

/// Render one cell, padded/truncated to its column's fixed width.
fn render_cell( row : &RollupRow, col : ColumnKey ) -> String
{
  let width = column_width( col );
  match col
  {
    ColumnKey::Group =>
    {
      let text = truncate_str( short_id( &row.group ), width );
      format!( "{text:<width$}" )
    }
    ColumnKey::Sessions => format!( "{:>width$}", row.sessions ),
    ColumnKey::Calls => format!( "{:>width$}", row.calls ),
    ColumnKey::Input => format!( "{:>width$}", format_tokens( row.input ) ),
    ColumnKey::Output => format!( "{:>width$}", format_tokens( row.output ) ),
    ColumnKey::Cache => format!( "{:>width$}", format_tokens( row.cache() ) ),
    ColumnKey::MaxContext => format!( "{:>width$}", format_tokens( row.max_context ) ),
    ColumnKey::Total => format!( "{:>width$}", format_tokens( row.total() ) ),
    ColumnKey::Percent => format!( "{:>width$}", format!( "{:.1}%", row.percent ) ),
    ColumnKey::First => { let text = row.first.as_deref().unwrap_or( "-" ); format!( "{text:<width$}" ) }
    ColumnKey::Last => { let text = row.last.as_deref().unwrap_or( "-" ); format!( "{text:<width$}" ) }
  }
}

/// Format a token count: bare below 1000, `N.Nk` in the thousands, `N.NM`
/// from a million up. Deliberately a local copy of `.usage`'s own
/// `format_tokens` (same precedent as `exit_no_project` above).
fn format_tokens( n : u64 ) -> String
{
  if n < 1_000
  {
    format!( "{n}" )
  }
  else if n < 1_000_000
  {
    #[ allow( clippy::cast_precision_loss ) ]
    let k = n as f64 / 1_000.0;
    format!( "{k:.1}k" )
  }
  else
  {
    #[ allow( clippy::cast_precision_loss ) ]
    let m = n as f64 / 1_000_000.0;
    format!( "{m:.1}M" )
  }
}

/// First 8 characters of a UUID-shaped group label; other labels (project
/// cwd, model name, day) pass through unchanged. Deliberately a local copy
/// of `.usage`'s own `short_id` (same precedent as `exit_no_project` above).
fn short_id( id : &str ) -> &str
{
  const UUID_LEN : usize = 36;
  const UUID_SHORT_LEN : usize = 8;
  if id.len() == UUID_LEN && id.as_bytes().get( 8 ) == Some( &b'-' )
  {
    &id[ ..UUID_SHORT_LEN ]
  }
  else
  {
    id
  }
}

/// Cut `text` at `max_chars` characters, marking the cut with a trailing
/// `…`. Generalizes `.usage`'s own `truncate_command` (35-char, Command-
/// column-specific) to any width/column.
fn truncate_str( text : &str, max_chars : usize ) -> String
{
  if text.chars().count() <= max_chars
  {
    return text.to_string();
  }
  let keep = max_chars.saturating_sub( 1 );
  let mut truncated : String = text.chars().take( keep ).collect();
  truncated.push( '…' );
  truncated
}
