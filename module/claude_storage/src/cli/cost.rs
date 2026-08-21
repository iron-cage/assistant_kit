//! `.cost` command — per-conversation token/cost accounting table.
//!
//! See `docs/cli/command/15_cost.md` for the full CLI contract this
//! implements. Scanning and aggregation are delegated entirely to
//! `claude_storage_core::cost` (per-model usage, cache-TTL split,
//! compactions, max context) and `claude_storage_core::family` (agent
//! fold-in) — this file only resolves which conversations to report on,
//! applies pricing, and renders the table. Pricing lives HERE, not in the
//! core engine: prices change over time and carry an as-of date, while
//! token counts are facts of the transcript (see `PRICES_AS_OF`).

use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use claude_storage_core::
{
  ConversationUsage, Session, Storage, aggregate_reports, cost_report, find_family,
  most_recent_session_in_dir,
};
use super::storage::{ create_storage, resolve_path_parameter };
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Date the [`MODEL_RATES`] table was last synchronized with the published
/// API price list — surfaced verbatim in the output's trailing note so a
/// reader always knows how stale the estimate might be.
const PRICES_AS_OF : &str = "2026-08-21";

/// USD per million tokens for one model family, by token kind.
struct ModelRates
{
  /// Fresh input tokens.
  input : f64,
  /// Cache writes with 5-minute TTL (also used for unknown-TTL writes —
  /// 5m is the API default TTL, so unbrokendown writes were 5m writes).
  write_5m : f64,
  /// Cache writes with 1-hour TTL.
  write_1h : f64,
  /// Cache reads.
  read : f64,
  /// Output tokens.
  output : f64,
}

/// Model pricing, matched by case-insensitive substring against the
/// recorded model ID; FIRST match wins, so more specific needles must
/// precede the generic ones they contain (`opus-4-1` before `opus-4`).
/// USD per MTok, from the published API price list as of [`PRICES_AS_OF`].
const MODEL_RATES : &[ ( &str, ModelRates ) ] =
&[
  ( "fable",      ModelRates { input : 10.0, write_5m : 12.5,   write_1h : 20.0, read : 1.0,  output : 50.0 } ),
  ( "mythos",     ModelRates { input : 10.0, write_5m : 12.5,   write_1h : 20.0, read : 1.0,  output : 50.0 } ),
  ( "opus-4-1",   ModelRates { input : 15.0, write_5m : 18.75,  write_1h : 30.0, read : 1.5,  output : 75.0 } ),
  ( "opus-4",     ModelRates { input : 5.0,  write_5m : 6.25,   write_1h : 10.0, read : 0.5,  output : 25.0 } ),
  ( "opus-5",     ModelRates { input : 5.0,  write_5m : 6.25,   write_1h : 10.0, read : 0.5,  output : 25.0 } ),
  ( "3-opus",     ModelRates { input : 15.0, write_5m : 18.75,  write_1h : 30.0, read : 1.5,  output : 75.0 } ),
  ( "sonnet-5",   ModelRates { input : 2.0,  write_5m : 2.5,    write_1h : 4.0,  read : 0.2,  output : 10.0 } ),
  ( "sonnet-4",   ModelRates { input : 3.0,  write_5m : 3.75,   write_1h : 6.0,  read : 0.3,  output : 15.0 } ),
  ( "3-7-sonnet", ModelRates { input : 3.0,  write_5m : 3.75,   write_1h : 6.0,  read : 0.3,  output : 15.0 } ),
  ( "3-5-sonnet", ModelRates { input : 3.0,  write_5m : 3.75,   write_1h : 6.0,  read : 0.3,  output : 15.0 } ),
  ( "haiku-4-5",  ModelRates { input : 1.0,  write_5m : 1.25,   write_1h : 2.0,  read : 0.1,  output : 5.0 } ),
  ( "3-5-haiku",  ModelRates { input : 0.8,  write_5m : 1.0,    write_1h : 1.6,  read : 0.08, output : 4.0 } ),
  ( "3-haiku",    ModelRates { input : 0.25, write_5m : 0.3125, write_1h : 0.5,  read : 0.03, output : 1.25 } ),
];

/// One conversation selected for the report: the project directory holding
/// its root session file, plus the full root session ID.
struct SelectedConversation
{
  project_dir : PathBuf,
  root_id : String,
}

/// Per-conversation cost table: exact token counts (input, output, cache
/// read/write), deduplicated request count, compactions, max context, and
/// estimated USD cost — with each conversation's agent (subagent) sessions
/// folded into its row.
///
/// Parameters (see `docs/cli/command/15_cost.md`):
/// - `session_ids::` — comma-separated session IDs or unique ID prefixes,
///   searched across ALL projects (default: most recent session of the
///   current directory's project)
/// - `path::` — directory whose project anchors the default resolution when
///   `session_ids::` is omitted (default: current directory)
/// - `agents::` — `1` (default) folds each conversation's agent sessions
///   into its row; `0` reports the root session alone
///
/// # Errors
///
/// Returns error (exit 1) when `agents::` is not `0`/`1`, `session_ids::`
/// is empty, an ID matches nothing, or a prefix is ambiguous.
///
/// # Exit Codes
///
/// Exits directly with code 2 when `session_ids::` is omitted and the
/// current directory (or `path::`) has no project or the project has no
/// session — matches the `.usage`/`.rollup` "not found = usage error"
/// convention.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn cost_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  // Validate arguments before any storage access, mirroring `.rollup`
  // (docs/cli/command/14_rollup.md's Algorithm step 1).
  let agents = cmd.get_integer( "agents" ).unwrap_or( 1 );
  if agents != 0 && agents != 1
  {
    return Err( ErrorData::new( ErrorCode::InternalError, "agents must be 0 or 1".to_string() ) );
  }
  let include_agents = agents == 1;
  let session_ids_raw = cmd.get_string( "session_ids" );
  let path_raw = cmd.get_string( "path" );

  // `session_ids::` emptiness is argument validation too, so it precedes
  // storage access: an all-empty list (e.g. `session_ids::,`) is rejected
  // whether or not storage is even readable.
  let requested : Option< Vec< &str > > = session_ids_raw
    .map( | raw | raw.split( ',' ).map( str::trim ).filter( | s | !s.is_empty() ).collect() );
  if requested.as_ref().is_some_and( Vec::is_empty )
  {
    return Err( ErrorData::new( ErrorCode::InternalError,
      "session_ids must contain at least one session ID".to_string() ) );
  }

  let storage = create_storage()?;

  let selected = match requested
  {
    Some( ids ) => resolve_requested_ids( &storage, &ids )?,
    None => vec![ resolve_default_conversation( &storage, path_raw ) ],
  };

  let mut rows = Vec::with_capacity( selected.len() );
  for conversation in &selected
  {
    rows.push( collect_conversation( conversation, include_agents )? );
  }

  Ok( OutputData::new( render_table( &rows ), "text" ) )
}

/// Resolve each requested ID (already split and trimmed by the caller —
/// guaranteed non-empty) against every non-agent session across ALL
/// projects. Exact match wins over prefix match; a prefix must be unique;
/// an ID duplicated across projects resolves to the richest copy (greatest
/// entry count — the `Fix(BUG-528)` convention). Duplicate requests for the
/// same conversation are collapsed to the first occurrence; output order
/// follows request order.
fn resolve_requested_ids( storage : &Storage, requested : &[ &str ] )
  -> core::result::Result< Vec< SelectedConversation >, ErrorData >
{
  // One walk over all projects' non-agent sessions; per-project read
  // failures are skipped gracefully, mirroring `.rollup`'s collect_inputs.
  let mut index : Vec< ( PathBuf, String ) > = Vec::new();
  let projects = storage.list_projects()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;
  for project in &projects
  {
    let Ok( sessions ) = project.sessions() else { continue };
    for session in sessions
    {
      if session.is_agent_session() { continue; }
      index.push( ( project.storage_dir().to_path_buf(), session.id().to_string() ) );
    }
  }

  let mut selected : Vec< SelectedConversation > = Vec::new();
  for &request in requested
  {
    let exact : Vec< &( PathBuf, String ) > = index.iter().filter( | ( _, id ) | id == request ).collect();
    let resolved = if exact.is_empty()
    {
      let prefixed : Vec< &( PathBuf, String ) > =
        index.iter().filter( | ( _, id ) | id.starts_with( request ) ).collect();
      let mut distinct : Vec< &str > = prefixed.iter().map( | ( _, id ) | id.as_str() ).collect();
      distinct.sort_unstable();
      distinct.dedup();
      match distinct.len()
      {
        0 => return Err( ErrorData::new( ErrorCode::InternalError, format!( "Session not found: {request}" ) ) ),
        1 => richest_copy( &prefixed ),
        _ => return Err( ErrorData::new( ErrorCode::InternalError,
          format!( "ambiguous session ID prefix '{}': matches {}", request, distinct.join( ", " ) ) ) ),
      }
    }
    else
    {
      richest_copy( &exact )
    };

    if !selected.iter().any( | s | s.root_id == resolved.root_id )
    {
      selected.push( resolved );
    }
  }

  Ok( selected )
}

/// Among physical copies of one session ID (same ID in several project
/// directories — git-worktree-style forked history), pick the copy with the
/// greatest entry count, the same tie-break `.rollup`'s `collect_inputs`
/// established (`Fix(BUG-528)`). Entry counts are only computed when more
/// than one copy exists.
fn richest_copy( copies : &[ &( PathBuf, String ) ] ) -> SelectedConversation
{
  let ( project_dir, root_id ) = if copies.len() == 1
  {
    ( copies[ 0 ].0.clone(), copies[ 0 ].1.clone() )
  }
  else
  {
    let best = copies.iter().max_by_key( | ( dir, id ) |
    {
      let path = dir.join( format!( "{id}.jsonl" ) );
      Session::load( &path ).ok().and_then( | s | s.count_entries().ok() ).unwrap_or( 0 )
    } ).expect( "copies is non-empty at both call sites" );
    ( best.0.clone(), best.1.clone() )
  };
  SelectedConversation { project_dir, root_id }
}

/// Default when `session_ids::` is omitted: the most recent non-agent
/// session of the project owning `path::` (or the current directory).
/// No project or no session exits 2 directly — the `.usage`/`.rollup`
/// "not found = usage error" convention, deliberately a local copy of
/// `.rollup`'s own `exit_no_project` shape.
fn resolve_default_conversation( storage : &Storage, path_raw : Option< &str > ) -> SelectedConversation
{
  let project = match path_raw
  {
    Some( raw ) => match resolve_path_parameter( raw )
    {
      Ok( resolved ) => storage.load_project_for_path( &resolved ),
      Err( e ) =>
      {
        eprintln!( "No project found for path: {raw} ({e})" );
        std::process::exit( 2 );
      }
    },
    None => storage.load_project_for_cwd(),
  };
  let Ok( project ) = project else
  {
    match path_raw
    {
      Some( raw ) => eprintln!( "No project found for path: {raw}" ),
      None => eprintln!( "No project found for current directory" ),
    }
    std::process::exit( 2 );
  };
  let Some( root_id ) = most_recent_session_in_dir( project.storage_dir() ) else
  {
    eprintln!( "No session found in project" );
    std::process::exit( 2 );
  };
  SelectedConversation
  {
    project_dir : project.storage_dir().to_path_buf(),
    root_id : root_id.as_str().to_string(),
  }
}

/// Scan one selected conversation: the root session's report, plus — when
/// `include_agents` — every agent session in its family. An unreadable
/// agent file is skipped with a stderr warning (per-session graceful
/// degradation, the `Fix(BUG-506)` convention); an unreadable ROOT is a
/// hard error, since the user asked for exactly that conversation.
fn collect_conversation( conversation : &SelectedConversation, include_agents : bool )
  -> core::result::Result< ConversationUsage, ErrorData >
{
  let family = find_family( &conversation.project_dir, &conversation.root_id )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError,
      format!( "Failed to resolve session family for {}: {e}", conversation.root_id ) ) )?;

  let root_session = Session::load( &family.root_path )
    .map_err( | e | ErrorData::new( ErrorCode::InternalError,
      format!( "Failed to load session {}: {e}", conversation.root_id ) ) )?;
  let mut reports = vec!
  [
    cost_report( &root_session )
      .map_err( | e | ErrorData::new( ErrorCode::InternalError,
        format!( "Failed to read session {}: {e}", conversation.root_id ) ) )?,
  ];

  if include_agents
  {
    for agent_path in &family.agent_paths
    {
      match Session::load( agent_path ).and_then( | s | cost_report( &s ) )
      {
        Ok( report ) => reports.push( report ),
        Err( e ) => eprintln!( "Warning: skipping agent session {}: {e}", agent_path.display() ),
      }
    }
  }

  Ok( aggregate_reports( &conversation.root_id, &reports ) )
}

/// Estimated USD cost of `usage` at [`MODEL_RATES`]. A model with no rate
/// entry contributes nothing and is recorded in `unpriced` for the
/// footnote — silence would misread as "cost fully covered".
fn cost_of( usage : &ConversationUsage, unpriced : &mut BTreeSet< String > ) -> f64
{
  let mut total = 0.0;
  for model in &usage.models
  {
    let Some( rates ) = rates_for( &model.model ) else
    {
      if model.total_tokens() > 0
      {
        unpriced.insert( model.model.clone() );
      }
      continue;
    };
    // Unknown-TTL cache writes price at the 5m rate: 5 minutes is the API
    // default TTL, so a write recorded before the TTL breakdown existed
    // was a 5m write.
    #[ allow( clippy::cast_precision_loss ) ]
    {
      total += ( model.input_tokens as f64 ).mul_add( rates.input,
        ( model.output_tokens as f64 ).mul_add( rates.output,
        ( model.cache_read_tokens as f64 ).mul_add( rates.read,
        ( model.cache_1h_write_tokens as f64 ).mul_add( rates.write_1h,
        ( model.cache_5m_write_tokens + model.cache_unknown_ttl_write_tokens ) as f64 * rates.write_5m ) ) ) )
        / 1_000_000.0;
    }
  }
  total
}

/// First [`MODEL_RATES`] entry whose needle the lowercased model ID
/// contains, or `None` for an unknown model.
fn rates_for( model : &str ) -> Option< &'static ModelRates >
{
  let lower = model.to_lowercase();
  MODEL_RATES.iter().find( | ( needle, _ ) | lower.contains( needle ) ).map( | ( _, rates ) | rates )
}

const COL_CONVERSATION : usize = 12;
const COL_AGENTS : usize = 6;
const COL_REQ : usize = 7;
const COL_TOKENS : usize = 14;
const COL_MAX_CTX : usize = 11;
const COL_COMPACT : usize = 7;
const COL_COST : usize = 10;

/// Render the full table: header, one row per conversation, a TOTAL row
/// when more than one conversation is shown, unpriced-model footnotes, and
/// the price-date note. No trailing newline (`execute_oneshot` output
/// contract).
fn render_table( rows : &[ ConversationUsage ] ) -> String
{
  let mut lines = vec![ format!(
    "{:<cw$}  {:>aw$}  {:>rw$}  {:>tw$}  {:>tw$}  {:>tw$}  {:>tw$}  {:>tw$}  {:>mw$}  {:>pw$}  {:>dw$}",
    "Conversation", "Agents", "Req", "Input", "Output", "CacheR", "CacheW", "Total", "MaxCtx", "Compact", "Cost",
    cw = COL_CONVERSATION, aw = COL_AGENTS, rw = COL_REQ, tw = COL_TOKENS,
    mw = COL_MAX_CTX, pw = COL_COMPACT, dw = COL_COST,
  ) ];

  let mut unpriced : BTreeSet< String > = BTreeSet::new();
  let mut total_cost = 0.0;
  for usage in rows
  {
    let cost = cost_of( usage, &mut unpriced );
    total_cost += cost;
    lines.push( render_row( truncate_str( short_id( &usage.root_id ), COL_CONVERSATION ).as_str(),
      usage, &group_thousands( usage.max_context_tokens ), cost ) );
  }

  if rows.len() > 1
  {
    // MaxCtx stays per-call and is not additive across conversations —
    // the TOTAL row deliberately shows a dash there. Its Cost sums the
    // per-row costs before rounding, so it can differ from the sum of the
    // displayed (independently rounded) row costs by a cent.
    let sum = | f : fn( &ConversationUsage ) -> u64 | rows.iter().map( f ).sum::< u64 >();
    let agents_total : usize = rows.iter().map( | u | u.agent_count ).sum();
    let compactions_total : usize = rows.iter().map( | u | u.compactions ).sum();
    let calls_total : usize = rows.iter().map( ConversationUsage::total_calls ).sum();
    let line = format!(
      "{:<cw$}  {:>aw$}  {:>rw$}  {:>tw$}  {:>tw$}  {:>tw$}  {:>tw$}  {:>tw$}  {:>mw$}  {:>pw$}  {:>dw$}",
      "TOTAL",
      group_thousands( agents_total as u64 ),
      group_thousands( calls_total as u64 ),
      group_thousands( sum( ConversationUsage::total_input_tokens ) ),
      group_thousands( sum( ConversationUsage::total_output_tokens ) ),
      group_thousands( sum( ConversationUsage::total_cache_read_tokens ) ),
      group_thousands( sum( ConversationUsage::total_cache_write_tokens ) ),
      group_thousands( sum( ConversationUsage::total_tokens ) ),
      "—",
      group_thousands( compactions_total as u64 ),
      format!( "${total_cost:.2}" ),
      cw = COL_CONVERSATION, aw = COL_AGENTS, rw = COL_REQ, tw = COL_TOKENS,
      mw = COL_MAX_CTX, pw = COL_COMPACT, dw = COL_COST,
    );
    lines.push( line );
  }

  for model in &unpriced
  {
    lines.push( format!( "note: no pricing for model '{model}' — its tokens are excluded from Cost" ) );
  }
  lines.push( format!( "Cost: estimated at API list prices ({PRICES_AS_OF}); tokens are exact." ) );

  lines.join( "\n" )
}

/// Render one conversation row.
fn render_row( label : &str, usage : &ConversationUsage, max_ctx : &str, cost : f64 ) -> String
{
  format!(
    "{:<cw$}  {:>aw$}  {:>rw$}  {:>tw$}  {:>tw$}  {:>tw$}  {:>tw$}  {:>tw$}  {:>mw$}  {:>pw$}  {:>dw$}",
    label,
    group_thousands( usage.agent_count as u64 ),
    group_thousands( usage.total_calls() as u64 ),
    group_thousands( usage.total_input_tokens() ),
    group_thousands( usage.total_output_tokens() ),
    group_thousands( usage.total_cache_read_tokens() ),
    group_thousands( usage.total_cache_write_tokens() ),
    group_thousands( usage.total_tokens() ),
    max_ctx,
    group_thousands( usage.compactions as u64 ),
    format!( "${cost:.2}" ),
    cw = COL_CONVERSATION, aw = COL_AGENTS, rw = COL_REQ, tw = COL_TOKENS,
    mw = COL_MAX_CTX, pw = COL_COMPACT, dw = COL_COST,
  )
}

/// Exact integer with thousands separators (`1234567` → `1,234,567`) — a
/// deliberate divergence from `.usage`/`.rollup`'s rounded `N.Nk`/`N.NM`
/// `format_tokens`: this is a billing-audit table, and rounded token counts
/// cannot be cross-checked against an invoice.
fn group_thousands( n : u64 ) -> String
{
  let digits = n.to_string();
  let mut out = String::with_capacity( digits.len() + digits.len() / 3 );
  for ( i, ch ) in digits.chars().enumerate()
  {
    if i > 0 && ( digits.len() - i ) % 3 == 0
    {
      out.push( ',' );
    }
    out.push( ch );
  }
  out
}

/// First 8 characters of a UUID-shaped session ID; other labels pass
/// through unchanged. Deliberately a local copy of `.rollup`'s own
/// `short_id` (see that file's `exit_no_project` precedent comment).
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
/// `…`. Deliberately a local copy of `.rollup`'s own `truncate_str`.
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
