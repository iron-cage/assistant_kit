//! `.projects` command — session-first cross-project view with scope control.
//!
//! Also houses shared family/conversation domain types. Scope/path
//! resolution itself lives in `super::scope`.

use core::fmt::Write as FmtWrite;
use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::{ create_storage, load_project_for_param };
use super::scope::{ validate_scope, resolve_scoped_projects, decode_project_display };
use super::projects_overview::{ OverviewRow, render_flat, render_tree };
use super::liveness::{ Liveness, LivenessMap };

// ─── constants ─────────────────────────────────────────────────────────────

/// UUID string length (8-4-4-4-12 = 36 chars).
const UUID_LEN : usize = 36;

/// Characters to display from each end when short-displaying a UUID.
const UUID_SHORT_LEN : usize = 8;

/// Fallback agent type when `meta.json` is absent or missing `agentType`.
const AGENT_TYPE_UNKNOWN : &str = "unknown";

/// Seconds-per-unit thresholds for relative time formatting.
const SECS_PER_MIN   : u64 = 60;
const SECS_PER_HOUR  : u64 = 3_600;
const SECS_PER_DAY   : u64 = 86_400;
const SECS_PER_MONTH : u64 = 2_592_000;

// ─── sessions output helpers ───────────────────────────────────────────────

fn session_mtime( session : &claude_storage_core::Session ) -> Option< std::time::SystemTime >
{
  std::fs::metadata( session.storage_path() )
    .ok()
    .and_then( | m | m.modified().ok() )
}

fn is_zero_byte_session( session : &claude_storage_core::Session ) -> bool
{
  std::fs::metadata( session.storage_path() )
    .is_ok_and( | m | m.len() == 0 )
}

// Shorten real UUID-format IDs to first `UUID_SHORT_LEN` chars.
// Non-UUID IDs (e.g. synthetic test IDs) are returned intact.
fn short_id( id : &str ) -> &str
{
  if id.len() == UUID_LEN && id.as_bytes().get( UUID_SHORT_LEN ) == Some( &b'-' ) { &id[ ..UUID_SHORT_LEN ] }
  else { id }
}

pub( super ) fn format_relative_time( mtime : std::time::SystemTime ) -> String
{
  let elapsed = std::time::SystemTime::now()
    .duration_since( mtime )
    .unwrap_or_default();
  let secs = elapsed.as_secs();
  if secs < SECS_PER_MIN        { format!( "{secs}s ago" ) }
  else if secs < SECS_PER_HOUR  { format!( "{}m ago", secs / SECS_PER_MIN ) }
  else if secs < SECS_PER_DAY   { format!( "{}h ago", secs / SECS_PER_HOUR ) }
  else if secs < SECS_PER_MONTH { format!( "{}d ago", secs / SECS_PER_DAY ) }
  else                          { format!( "{}mo ago", secs / SECS_PER_MONTH ) }
}

// ─── family detection ──────────────────────────────────────────────────────

struct AgentMeta { agent_type : String }

struct AgentInfo
{
  session    : claude_storage_core::Session,
  agent_type : String,
}

pub( super ) struct SessionFamily
{
  root   : Option< claude_storage_core::Session >,
  agents : Vec< AgentInfo >,
}

/// A Conversation is the user-facing unit of interaction — one logical chat.
///
/// # Current implementation (1:1 mapping)
///
/// Each `SessionFamily` maps to exactly one `Conversation` via
/// `group_into_conversations`. The identity mapping is a placeholder
/// until cross-session chain detection is implemented.
///
/// # Future: Chain Detection contract
///
/// When implemented, one `Conversation` may span multiple `SessionFamily`
/// values representing work continued across `--new-session` invocations.
/// No explicit storage links exist (B17, B18 invariants); detection uses
/// temporal proximity and content heuristics.
pub struct Conversation
{
  families : Vec< SessionFamily >,
}

impl core::fmt::Debug for Conversation
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.debug_struct( "Conversation" )
      .field( "family_count", &self.conversation_count() )
      .finish()
  }
}

impl Conversation
{
  pub( super ) fn root_session( &self ) -> Option< &claude_storage_core::Session >
  {
    self.families.first().and_then( | f | f.root.as_ref() )
  }

  fn all_agents( &self ) -> impl Iterator< Item = &AgentInfo >
  {
    self.families.iter().flat_map( | f | f.agents.iter() )
  }

  fn conversation_count( &self ) -> usize
  {
    self.families.len()
  }
}

// Group session families into conversations (currently 1:1 identity mapping).
//
// Each `SessionFamily` maps to exactly one `Conversation`. Placeholder for
// future cross-session chain detection (B17/B18 invariants rule out storage links).
pub( super ) fn group_into_conversations( families : Vec< SessionFamily > ) -> Vec< Conversation >
{
  families
    .into_iter()
    .map( | family | Conversation { families : vec![ family ] } )
    .collect()
}

struct ProjectSummary
{
  display_path : String,
  last_mtime   : std::time::SystemTime,
}

/// Read `meta.json` sidecar for an agent session.
///
/// Derives the meta path by replacing the `.jsonl` extension with `.meta.json`.
/// Uses `claude_storage_core::parse_json` (not `serde_json`) because the core
/// crate already provides a JSON parser and `serde_json` is not a dependency.
/// Returns `AgentMeta { agent_type: "unknown" }` on any error (missing file,
/// empty file, malformed JSON, missing `agentType` key, or blank `agentType`).
///
/// Fix(issue-mt-empty-agenttype)
/// Root cause: `.unwrap_or("unknown")` only catches `None`; `Some("")` and
/// `Some("  ")` slipped through, rendering as empty or whitespace labels.
/// Pitfall: `unwrap_or` cannot replace a non-None but semantically empty value —
/// always pair it with `.filter(|s| !s.trim().is_empty())`.
fn parse_agent_meta( agent_path : &std::path::Path ) -> AgentMeta
{
  let meta_path = agent_path.with_extension( "meta.json" );
  let content = match std::fs::read_to_string( &meta_path )
  {
    Ok( c ) if !c.is_empty() => c,
    _ => return AgentMeta { agent_type : AGENT_TYPE_UNKNOWN.into() },
  };
  let Ok( val ) = claude_storage_core::parse_json( &content ) else
  {
    return AgentMeta { agent_type : AGENT_TYPE_UNKNOWN.into() };
  };
  let agent_type = val.as_object()
    .and_then( | obj | obj.get( "agentType" ) )
    .and_then( claude_storage_core::JsonValue::as_str )
    .filter( | s | !s.trim().is_empty() )
    .unwrap_or( AGENT_TYPE_UNKNOWN )
    .to_string();
  AgentMeta { agent_type }
}

/// Extract parent UUID from hierarchical agent path.
///
/// Layout: `{project_dir}/{parent_uuid}/subagents/agent-{id}.jsonl`
/// Returns `parent_uuid` by navigating `parent/parent/file_name`.
fn extract_parent_hierarchical( agent_path : &std::path::Path ) -> Option< String >
{
  agent_path
    .parent()?  // subagents/
    .parent()?  // {parent_uuid}/
    .file_name()?
    .to_str()
    .map( String::from )
}

/// Extract parent session ID from first JSONL line of a flat agent file.
///
/// Reads only the first line and parses the `sessionId` field.
fn extract_parent_flat( agent_path : &std::path::Path ) -> Option< String >
{
  use std::io::BufRead;
  let file = std::fs::File::open( agent_path ).ok()?;
  let mut reader = std::io::BufReader::new( file );
  let mut line = String::new();
  reader.read_line( &mut line ).ok()?;
  let val = claude_storage_core::parse_json( &line ).ok()?;
  val.as_object()?
    .get( "sessionId" )?
    .as_str()
    .map( String::from )
}

/// Detect whether this project uses hierarchical agent storage.
///
/// Returns `true` if any agent path contains a "subagents" component.
fn is_hierarchical_format( agents : &[ &claude_storage_core::Session ] ) -> bool
{
  agents.iter().any( | s |
    s.storage_path().components().any( | c | c.as_os_str() == "subagents" )
  )
}

/// Resolve parent links for a list of agent sessions.
///
/// Detects hierarchical vs flat format, extracts parent IDs, and partitions
/// agents into a parent-keyed map and an orphan list.
fn resolve_agent_parents(
  agents : Vec< claude_storage_core::Session >,
) -> ( std::collections::HashMap< String, Vec< AgentInfo > >, Vec< AgentInfo > )
{
  use std::collections::HashMap;

  let agent_refs : Vec< &claude_storage_core::Session > = agents.iter().collect();
  let hierarchical = is_hierarchical_format( &agent_refs );

  let mut parent_map : HashMap< String, Vec< AgentInfo > > = HashMap::new();
  let mut orphans : Vec< AgentInfo > = Vec::new();

  for agent in agents
  {
    let meta = parse_agent_meta( agent.storage_path() );
    let parent_id = if hierarchical
    {
      extract_parent_hierarchical( agent.storage_path() )
    }
    else
    {
      extract_parent_flat( agent.storage_path() )
    };

    let info = AgentInfo { session : agent, agent_type : meta.agent_type };
    match parent_id
    {
      Some( pid ) => parent_map.entry( pid ).or_default().push( info ),
      None => orphans.push( info ),
    }
  }

  ( parent_map, orphans )
}

/// Build session families from a flat list of sessions.
///
/// Groups agent sessions under their parent root sessions. Handles both
/// hierarchical (path-based) and flat (`sessionId`-based) parent detection.
/// Agents without a matching root become orphan families.
pub( super ) fn build_families(
  sessions : Vec< claude_storage_core::Session >,
) -> Vec< SessionFamily >
{
  let mut roots  : Vec< claude_storage_core::Session > = Vec::new();
  let mut agents : Vec< claude_storage_core::Session > = Vec::new();
  for s in sessions
  {
    if s.is_agent_session() { agents.push( s ); }
    else { roots.push( s ); }
  }

  if agents.is_empty()
  {
    return roots.into_iter()
      .map( | r | SessionFamily { root : Some( r ), agents : Vec::new() } )
      .collect();
  }

  let ( mut parent_map, mut orphan_agents ) = resolve_agent_parents( agents );

  let mut families : Vec< SessionFamily > = Vec::new();
  for root in roots
  {
    let children = parent_map.remove( root.id() ).unwrap_or_default();
    families.push( SessionFamily { root : Some( root ), agents : children } );
  }

  for ( _pid, agents_vec ) in parent_map
  {
    orphan_agents.extend( agents_vec );
  }
  if !orphan_agents.is_empty()
  {
    families.push( SessionFamily { root : None, agents : orphan_agents } );
  }

  families.sort_by( | a, b |
  {
    let ta = a.root.as_ref().and_then( session_mtime )
      .unwrap_or( std::time::UNIX_EPOCH );
    let tb = b.root.as_ref().and_then( session_mtime )
      .unwrap_or( std::time::UNIX_EPOCH );
    tb.cmp( &ta )
  } );

  families
}

/// Format agent type breakdown as `"N×Type, M×Type"` sorted by count desc.
fn format_type_breakdown( agents : &[ AgentInfo ] ) -> String
{
  use std::collections::HashMap;
  let mut counts : HashMap< &str, usize > = HashMap::new();
  for a in agents
  {
    *counts.entry( a.agent_type.as_str() ).or_default() += 1;
  }
  let mut pairs : Vec< ( &str, usize ) > = counts.into_iter().collect();
  pairs.sort_by( | a, b | b.1.cmp( &a.1 ).then_with( || a.0.cmp( b.0 ) ) );
  pairs.iter()
    .map( | ( t, n ) | format!( "{n}\u{00d7}{t}" ) )
    .collect::< Vec< _ > >()
    .join( ", " )
}

/// Aggregate sessions by project, returning projects sorted by last mtime descending.
///
/// For each project in `groups`, finds the most-recently-modified non-zero-byte session.
/// Projects where no session has a readable mtime are excluded.
///
/// # Pitfalls
///
/// - (P4) Finds the most-active PROJECT by max(mtime) per project — not the
///   globally most-active session. A project with 3 old sessions and 1 new
///   session has `last_mtime` = that new session's mtime.
/// - (P5) Returns a Vec sorted by mtime descending; never iterate `groups`
///   directly for time-sorted output — `BTreeMap` order is alphabetical.
fn aggregate_projects(
  groups : &mut std::collections::BTreeMap< String, Vec< claude_storage_core::Session > >,
) -> Vec< ProjectSummary >
{
  let mut summaries : Vec< ProjectSummary > = Vec::new();

  for ( display_path, sessions ) in groups.iter_mut()
  {
    // Fix(issue-034): Exclude zero-byte placeholder sessions from best-session
    // selection in aggregate_projects.
    //
    // Root cause: `best` selection iterated all sessions including zero-byte
    // placeholders. When a zero-byte file had a more recent mtime than any real
    // session, it became the "best" session with a stale timestamp.
    //
    // Pitfall: `is_zero_byte_session()` must be applied at every aggregation
    // site — not only in the render layer.
    let best = sessions
      .iter()
      .enumerate()
      .filter( | ( _, s ) | !is_zero_byte_session( s ) )
      .filter_map( | ( i, s ) | session_mtime( s ).map( | t | ( i, t ) ) )
      .max_by_key( | &( _, t ) | t );

    let Some( ( _, best_time ) ) = best else { continue };

    summaries.push( ProjectSummary
    {
      display_path : display_path.clone(),
      last_mtime   : best_time,
    } );
  }

  // Most recently active project first.
  summaries.sort_by_key( | b | core::cmp::Reverse( b.last_mtime ) );
  summaries
}

/// Validate `type::` — narrows scoped projects by naming scheme.
///
/// # Errors
///
/// Returns error when the value is not one of `uuid`, `path`, `all`.
fn validate_project_type( type_raw : Option< &str > ) -> core::result::Result< String, ErrorData >
{
  let raw = type_raw.unwrap_or( "all" );
  let value = raw.to_lowercase();
  if !matches!( value.as_str(), "uuid" | "path" | "all" )
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "type must be uuid|path|all, got {raw}" ),
    ) );
  }
  Ok( value )
}

/// Validate `detail::` — selects terse overview vs full session-detail rendering.
///
/// Defaults to `projects` (terse). This fallback must stay in step with the
/// `detail` argument's `default:` in `unilang.commands.yaml` — the YAML default
/// applies to CLI dispatch, this one to any call that omits the argument
/// entirely (REPL paths, direct routine invocation in tests).
///
/// # Errors
///
/// Returns error when the value is not one of `projects`, `sessions`.
fn validate_detail_level( detail_raw : Option< &str > ) -> core::result::Result< String, ErrorData >
{
  let raw = detail_raw.unwrap_or( "projects" );
  let value = raw.to_lowercase();
  if !matches!( value.as_str(), "projects" | "sessions" )
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "detail must be projects|sessions, got {raw}" ),
    ) );
  }
  Ok( value )
}

// ─── .projects routine ─────────────────────────────────────────────────────

/// List sessions with scope control (session-first view).
///
/// Scope semantics (full definitions: `super::scope::resolve_scoped_projects`):
/// - `local`    — Current project only (`path::` selects the project, defaults to cwd)
/// - `relevant` — Every project whose path is an ancestor of (or equal to) `path::`
/// - `under`    — Every project whose path starts with `path::`
/// - `around`   — Union of `under` + `relevant` (default)
/// - `global`   — All projects in storage (ignores `path::`)
///
/// # Errors
///
/// Returns error if `scope::` is invalid, `min_entries::` is negative,
/// `limit::` is negative, `since_days::` is negative, path resolution fails,
/// or storage access fails.
///
/// # Panics
///
/// Does not panic — `min_entries`, `limit`, and `since_days` are validated
/// non-negative before conversion.
// Two dispatch branches (ids:: scripting mode and scope-based listing) plus their shared
// filter/sort/limit chain — splitting them would duplicate the filter chain per branch.
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
pub fn projects_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  use std::collections::BTreeMap;
  use claude_storage_core::{ Session, SessionFilter, ProjectId };

  // --- ids:: scripting-mode dispatch (bypasses scope-based listing entirely) ---

  if cmd.get_boolean( "ids" ).unwrap_or( false )
  {
    let proj_id = cmd.get_string( "project" )
      .ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "project parameter required for ids:: listing".to_string(),
      ) )?;
    let storage = create_storage()?;
    let project = load_project_for_param( &storage, proj_id )?;
    let sessions = project.all_sessions()
      .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to load sessions: {e}" ) ) )?;

    // `live::` narrows this branch to the same yes/no it applies to a listing —
    // "these ids, but only if the project is running" — instead of being
    // silently dropped because this branch answers before the listing path
    // reaches its filter. Probed only when asked: `ids::` is the hot scripting
    // path and an unconditional process-table walk would tax every caller.
    let ids_suppressed = match cmd.get_boolean( "live" )
    {
      None => false,
      Some( want_live ) =>
      {
        let liveness = LivenessMap::detect();
        // Detection reports only positives, so an empty `live::1` result cannot
        // be distinguished from an unreadable process table. A listing says so
        // in prose; a scripting mode must fail loudly instead, or the caller
        // consumes "nothing is running" as fact.
        if want_live && !liveness.any_attached()
        {
          return Err( ErrorData::new(
            ErrorCode::InternalError,
            "live::1 requires a readable /proc on the same host as the sessions; \
             no attached Claude Code processes are visible".to_string(),
          ) );
        }
        let display_path = project
          .storage_dir()
          .file_name()
          .and_then( | n | n.to_str() )
          .map( decode_project_display )
          .unwrap_or_default();
        let live_now = sessions
          .iter()
          .filter( | s | !is_zero_byte_session( s ) )
          .filter_map( session_mtime )
          .max()
          .and_then( | last | liveness.project_state( &display_path, last ) )
          .is_some();
        live_now != want_live
      }
    };

    let families = build_families( sessions );
    let conversations = group_into_conversations( families );
    let count_mode = cmd.get_boolean( "count" ).unwrap_or( false );
    let conversations = if ids_suppressed { Vec::new() } else { conversations };
    if count_mode
    {
      return Ok( OutputData::new( format!( "{}", conversations.len() ), "text" ) );
    }
    let mut out = String::new();
    for conv in &conversations
    {
      if let Some( s ) = conv.root_session()
      {
        writeln!( out, "{}", s.id() ).unwrap();
      }
    }
    return Ok( OutputData::new( out, "text" ) );
  }

  // --- parameters ---

  let scope = validate_scope( cmd.get_string( "scope" ), "around" )?;
  let project_type = validate_project_type( cmd.get_string( "type" ) )?;
  let detail_level = validate_detail_level( cmd.get_string( "detail" ) )?;

  let show_tree = cmd.get_boolean( "show_tree" ).unwrap_or( false );

  let min_entries_filter = if let Some( n ) = cmd.get_integer( "min_entries" )
  {
    if n < 0
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "Invalid min_entries: {n}. Must be non-negative" ),
      ) );
    }
    Some( usize::try_from( n ).expect( "min_entries < 0 rejected above" ) )
  }
  else { None };

  let limit_cap = if let Some( n ) = cmd.get_integer( "limit" )
  {
    if n < 0
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "Invalid limit: {n}. Must be non-negative" ),
      ) );
    }
    let v = usize::try_from( n ).expect( "limit < 0 rejected above" );
    // 0 means unlimited — map to usize::MAX so comparisons work without special-casing
    if v == 0 { usize::MAX } else { v }
  }
  else { usize::MAX };

  let since_cutoff = if let Some( n ) = cmd.get_integer( "since_days" )
  {
    if n < 0
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "Invalid since_days: {n}. Must be non-negative" ),
      ) );
    }
    // 0 and 1 both mean the most recent 24 hours: a session touched today must stay
    // inside a 0-day window, while N >= 1 keeps the strict `now - N*24h` cutoff of
    // the manual jq algorithm this parameter productizes.
    let days = u64::try_from( n ).expect( "since_days < 0 rejected above" ).max( 1 );
    Some( std::time::SystemTime::now() - core::time::Duration::from_secs( days * 86_400 ) )
  }
  else { None };

  let show_topic = cmd.get_boolean( "show_topic" ).unwrap_or( false );

  // Probed once per invocation: every row consults the same process-table and
  // history snapshot, so no row can disagree with another about what is running.
  let liveness = LivenessMap::detect();
  let live_filter = cmd.get_boolean( "live" );

  let agent_filter = cmd.get_boolean( "agent" );
  let session_id_filter = cmd.get_string( "session" );

  // --- collect projects by scope ---

  let storage = create_storage()?;
  let scoped_projects = resolve_scoped_projects( &storage, &scope, cmd.get_string( "path" ) )?;

  // --- narrow by type:: / filter::, decoding each surviving project's display path once ---

  let path_filter = cmd.get_string( "filter" ).map( str::to_lowercase );
  let scoped_projects = scoped_projects
    .into_iter()
    .filter_map( | project |
    {
      let type_ok = match project_type.as_str()
      {
        "uuid" => matches!( project.id(), ProjectId::Uuid( _ ) ),
        "path" => matches!( project.id(), ProjectId::Path( _ ) ),
        _ => true,
      };
      if !type_ok { return None; }
      let dir_name = project
        .storage_dir()
        .file_name()
        .and_then( | n | n.to_str() )
        .unwrap_or( "" )
        .to_string();
      let display_path = decode_project_display( &dir_name );
      if let Some( ref substr ) = path_filter
      {
        if !display_path.to_lowercase().contains( substr.as_str() ) { return None; }
      }
      Some( ( project, display_path ) )
    } )
    .collect::< Vec< _ > >();

  // --- build session filter ---

  let session_filter = SessionFilter
  {
    agent_only                : agent_filter,
    min_entries               : min_entries_filter,
    session_id_substring      : session_id_filter.map( std::string::ToString::to_string ),
  };

  // --- collect sessions grouped by decoded project path (Algorithm B) ---

  // BTreeMap gives deterministic, alphabetically sorted project order.
  let mut groups : BTreeMap< String, Vec< Session > > = BTreeMap::new();

  for ( mut project, display_path ) in scoped_projects
  {
    let Ok( mut sessions ) = project.sessions_filtered( &session_filter ) else { continue };
    if let Some( cutoff ) = since_cutoff
    {
      // Day-window filter on the same mtime the recency sort below already uses;
      // sessions with unreadable mtime are excluded (cannot prove they are recent).
      sessions.retain( | s | session_mtime( s ).is_some_and( | t | t >= cutoff ) );
    }
    if sessions.is_empty() { continue; }

    groups
      .entry( display_path )
      .or_default()
      .extend( sessions );
  }

  // --- sort each project's sessions by mtime descending (most recent first) ---

  for sessions in groups.values_mut()
  {
    sessions.sort_by( | a, b |
    {
      let ta = session_mtime( a ).unwrap_or( std::time::UNIX_EPOCH );
      let tb = session_mtime( b ).unwrap_or( std::time::UNIX_EPOCH );
      tb.cmp( &ta )
    } );
  }

  // --- format output (Algorithm C) ---

  // Aggregate into time-sorted project summaries (P5: never iterate groups directly).
  // aggregate_projects borrows groups mutably then releases; groups used below for
  // session lookup by display_path key.
  let mut summaries = aggregate_projects( &mut groups );

  // `live::` narrows to projects with (or without) an attached Claude Code
  // process. Deliberately project-scoped rather than session-scoped: filtering
  // sessions here would desynchronize every per-project count computed below
  // from what actually renders (the issue-034 class of defect). Which
  // conversation is being driven is answered by marking it, not by hiding
  // its siblings.
  if let Some( want_live ) = live_filter
  {
    summaries.retain( | s | liveness.project_state( &s.display_path, s.last_mtime ).is_some() == want_live );
  }

  // Detection reports only positives (`super::liveness`, "Detection never claims
  // a negative"), so an empty `live::1` result is ambiguous between "nothing is
  // running" and "this host cannot see the processes" — say so instead of
  // presenting an empty list as an answer.
  if live_filter == Some( true ) && !liveness.any_attached()
  {
    return Ok( OutputData::new(
      "No attached Claude Code processes found.\n\
       Liveness is read from the process table and requires a readable /proc on the same host \
       as the sessions — inside a container, or on a platform without /proc, nothing is visible \
       even while sessions are running.\n".to_string(),
      "text",
    ) );
  }

  let total_projects = summaries.len();
  let mut output = String::new();

  // Family grouping: with no explicit agent:: filter, agents are grouped
  // into families under their root sessions instead of shown flat.
  let use_families = agent_filter.is_none();

  // `detail::projects` renders through `super::projects_overview`, which needs
  // every row's counts up front to size its columns and build its tree — so the
  // loop below collects rows instead of streaming lines. `detail::sessions`
  // keeps streaming, one project block at a time.
  let terse = detail_level == "projects";
  let mut overview_rows : Vec< OverviewRow > = Vec::new();

  if !terse
  {
    let p_noun = if total_projects == 1 { "project" } else { "projects" };
    writeln!( output, "Found {total_projects} {p_noun}:\n" ).unwrap();
  }

  for summary in summaries
  {
    // Retrieve (and remove) sessions for this project from groups.
    let sessions = groups.remove( &summary.display_path ).unwrap_or_default();
    let display_path = &summary.display_path;

    if use_families
    {
      // Build families from sessions and group into conversations (1:1 now)
      let families = build_families( sessions );
      let conversations = group_into_conversations( families );

      // Fix(issue-034): Count only displayable (non-zero-byte) root sessions in header.
      //
      // Root cause: families.iter().filter(|f| f.root.is_some()).count() counted ALL
      // root families including those whose root is a zero-byte placeholder. render_families_v1
      // excludes zero-byte roots from display, so the header showed "(2 sessions)" while
      // zero lines were rendered below it.
      //
      // Pitfall: The render layer and the count must apply identical zero-byte filters.
      // If render changes to show/hide zero-byte sessions, update this count expression too.
      let root_count = conversations
        .iter()
        .filter( | c | c.root_session().is_some_and( | s | !is_zero_byte_session( s ) ) )
        .count();
      let agent_count : usize = conversations.iter().map( | c | c.all_agents().count() ).sum();
      // Unpack back to families for rendering (Phase 4 will use Conversation directly)
      let families : Vec< SessionFamily > = conversations
        .into_iter()
        .flat_map( | c | c.families )
        .collect();

      if terse
      {
        overview_rows.push( OverviewRow
        {
          display_path  : display_path.clone(),
          conversations : root_count,
          agents        : agent_count,
          last_mtime    : summary.last_mtime,
        } );
      }
      else
      {
        let r_noun = if root_count == 1 { "conversation" } else { "conversations" };
        if agent_count > 0
        {
          let a_noun = if agent_count == 1 { "agent" } else { "agents" };
          writeln!( output, "{display_path}: ({root_count} {r_noun}, {agent_count} {a_noun})" ).unwrap();
        }
        else
        {
          writeln!( output, "{display_path}: ({root_count} {r_noun})" ).unwrap();
        }
      }

      if detail_level == "sessions"
      {
        if show_tree
        {
          render_families_v2( &mut output, &families, display_path, &liveness );
        }
        else
        {
          render_families_v1( &mut output, &families, limit_cap, show_topic, display_path, &liveness );
        }
      }
    }
    else
    {
      // Fix(issue-034): Flat branch — compute displayable before group_count so
      // the header count matches what is actually rendered.
      //
      // Root cause: `group_count = sessions.len()` was computed before the
      // `displayable` filter that excludes zero-byte non-agent sessions.
      // The header showed "(2 sessions)" when `displayable` produced 0 lines.
      //
      // Pitfall: Never count from the unfiltered source after a render filter
      // has been defined. Move the filter computation above the count so both
      // the header and the render loop use the same source of truth.
      let displayable : Vec< &Session > = sessions
        .iter()
        .filter( | &s | s.is_agent_session() || !is_zero_byte_session( s ) )
        .collect();
      let group_count = displayable.len();
      if terse
      {
        // Family grouping is off here (agent:: was set explicitly), so the
        // conversation/agent split does not apply — attribute the count to
        // whichever column the filter actually selected.
        let is_agent_view = agent_filter == Some( true );
        overview_rows.push( OverviewRow
        {
          display_path  : display_path.clone(),
          conversations : if is_agent_view { 0 } else { group_count },
          agents        : if is_agent_view { group_count } else { 0 },
          last_mtime    : summary.last_mtime,
        } );
      }
      else
      {
        let group_noun = if group_count == 1 { "conversation" } else { "conversations" };
        writeln!( output, "{display_path}: ({group_count} {group_noun})" ).unwrap();
      }
      if detail_level == "sessions"
      {
        let show_count = displayable.len().min( limit_cap );
        for ( i, &session ) in displayable[ ..show_count ].iter().enumerate()
        {
          let marker = if i == 0 { '*' } else { '-' };
          let state = session_liveness( session, i, display_path, &liveness );
          let line = format_session_line( session, marker, show_topic, state );
          writeln!( output, "{line}" ).unwrap();
        }
        if displayable.len() > limit_cap
        {
          let hidden = displayable.len() - limit_cap;
          // "conversation" is the user-facing taxonomy noun; "session" is the internal storage term.
          let hidden_noun = if hidden == 1 { "conversation" } else { "conversations" };
          writeln!(
            output,
            "  ... and {hidden} more {hidden_noun}  (use limit::0 to list all)"
          ).unwrap();
        }
      }
    }

    // Block separator between per-project session listings. Terse rows are one
    // line each, so a separator there would only double the output height.
    if !terse
    {
      writeln!( output ).unwrap();
    }
  }

  if terse
  {
    output = if show_tree { render_tree( &overview_rows, &liveness ) }
             else { render_flat( &overview_rows, &liveness ) };
  }

  Ok( OutputData::new( output, "text" ) )
}

// ─── render helpers ────────────────────────────────────────────────────────

// Topic display cap, matching the manual jq algorithm's 90-char truncation
// that `show_topic::` productizes.
const TOPIC_MAX_CHARS : usize = 90;

/// Extract the display text of a transcript entry's `message.content`.
///
/// Handles both content shapes Claude Code writes: a plain string, or an
/// array of content blocks where the first `{"type":"text"}` block carries
/// the text.
fn extract_message_text( entry : &std::collections::HashMap< String, claude_storage_core::JsonValue > ) -> Option< String >
{
  let message = entry.get( "message" )?;
  let content = message.get( "content" )?;
  if let Some( s ) = content.as_str()
  {
    return Some( s.to_string() );
  }
  content
    .as_array()?
    .iter()
    .find( | block | block.get_str( "type" ) == Some( "text" ) )
    .and_then( | block | block.get_str( "text" ) )
    .map( String::from )
}

/// Extract a session's topic: its first `"type":"user"` entry's message text.
///
/// Streams the transcript line by line (the same file already read for entry
/// counting) and returns the first user entry whose text is non-empty after
/// trimming. Newlines are flattened to spaces and the result is truncated to
/// `TOPIC_MAX_CHARS` characters.
fn session_topic( session : &claude_storage_core::Session ) -> Option< String >
{
  use std::io::BufRead;
  let file = std::fs::File::open( session.storage_path() ).ok()?;
  let reader = std::io::BufReader::new( file );
  for line in reader.lines()
  {
    let Ok( line ) = line else { break };
    if line.trim().is_empty() { continue; }
    let Ok( val ) = claude_storage_core::parse_json( &line ) else { continue };
    let Some( obj ) = val.as_object() else { continue };
    if obj.get( "type" ).and_then( claude_storage_core::JsonValue::as_str ) != Some( "user" )
    {
      continue;
    }
    let Some( text ) = extract_message_text( obj ) else { continue };
    let flat = text.replace( [ '\r', '\n' ], " " );
    let trimmed = flat.trim();
    if trimmed.is_empty() { continue; }
    return Some( trimmed.chars().take( TOPIC_MAX_CHARS ).collect() );
  }
  None
}

/// Format `[N agents: breakdown]` bracket suffix for a family with agents.
///
/// Returns empty string when the agent list is empty.
fn format_agent_bracket( agents : &[ AgentInfo ] ) -> String
{
  if agents.is_empty() { return String::new(); }
  let n = agents.len();
  let noun = if n == 1 { "agent" } else { "agents" };
  let breakdown = format_type_breakdown( agents );
  format!( "  [{n} {noun}: {breakdown}]" )
}

/// Attachment state of one session, or `None` when nothing is driving it.
///
/// Agent sidecars are never marked: liveness is tracked per conversation, and
/// `history.jsonl` only ever names root session ids — so an agent could only be
/// marked through the mtime-rank fallback, where it would be a coincidence of
/// ordering rather than evidence.
fn session_liveness(
  session      : &claude_storage_core::Session,
  rank         : usize,
  display_path : &str,
  liveness     : &LivenessMap,
) -> Option< Liveness >
{
  if session.is_agent_session() { return None; }
  let mtime = session_mtime( session )?;
  liveness.session_state( display_path, session.id(), rank, mtime )
}

/// Format a single session line: `{marker} {id}  {age}  ({n} entries)[  state][  topic]`.
fn format_session_line(
  session    : &claude_storage_core::Session,
  marker     : char,
  with_topic : bool,
  state      : Option< Liveness >,
) -> String
{
  let id_str = short_id( session.id() );
  let time_str = session_mtime( session )
    .map( | t | format!( "  {}", format_relative_time( t ) ) )
    .unwrap_or_default();
  let count_str = session
    .count_entries()
    .map( | n |
    {
      let noun = if n == 1 { "entry" } else { "entries" };
      format!( "  ({n} {noun})" )
    } )
    .unwrap_or_default();
  let topic_str = if with_topic
  {
    session_topic( session )
      .map( | t | format!( "  {t}" ) )
      .unwrap_or_default()
  }
  else { String::new() };
  // Ahead of the topic, which is free text long enough to push a trailing tag
  // off the edge of a terminal.
  let state_str = state.map( | s | format!( "  {}", s.label() ) ).unwrap_or_default();
  format!( "  {marker} {id_str}{time_str}{count_str}{state_str}{topic_str}" )
}

/// Render family-grouped display at v1: root lines with `[N agents: breakdown]`.
fn render_families_v1(
  output       : &mut String,
  families     : &[ SessionFamily ],
  limit_cap    : usize,
  with_topic   : bool,
  display_path : &str,
  liveness     : &LivenessMap,
)
{
  let displayable : Vec< &SessionFamily > = families.iter()
    .filter( | f | !f.root.as_ref().is_some_and( is_zero_byte_session ) )
    .collect();
  let show_count = displayable.len().min( limit_cap );

  for ( i, family ) in displayable[ ..show_count ].iter().enumerate()
  {
    if let Some( root ) = &family.root
    {
      let marker = if i == 0 { '*' } else { '-' };
      // `i` is the root's rank in this project's mtime-descending order, which
      // is the ordering `LivenessMap`'s headless fallback is defined against.
      let state = session_liveness( root, i, display_path, liveness );
      let line = format_session_line( root, marker, with_topic, state );
      let bracket = format_agent_bracket( &family.agents );
      writeln!( output, "{line}{bracket}" ).unwrap();
    }
    else
    {
      let bracket = format_agent_bracket( &family.agents );
      writeln!( output, "  ? (orphan){bracket}" ).unwrap();
    }
  }

  if displayable.len() > limit_cap
  {
    let hidden = displayable.len() - limit_cap;
    // "conversation" is the user-facing taxonomy noun; "session" is the internal storage term.
    let noun = if hidden == 1 { "conversation" } else { "conversations" };
    writeln!( output, "  ... and {hidden} more {noun}  (use limit::0 to list all)" ).unwrap();
  }
}

/// Render family-grouped display at v2+: tree-indented agents under each root.
fn render_families_v2(
  output       : &mut String,
  families     : &[ SessionFamily ],
  display_path : &str,
  liveness     : &LivenessMap,
)
{
  for ( i, family ) in families.iter().enumerate()
  {
    if let Some( root ) = &family.root
    {
      let id = root.id();
      let count_str = root
        .count_entries()
        .map( | n | {
          let noun = if n == 1 { "entry" } else { "entries" };
          format!( "  ({n} {noun})" )
        } )
        .unwrap_or_default();
      // Same tag the flat family view carries: the layout chosen to inspect a
      // conversation must not decide whether "is this one running" is answered.
      let state_str = session_liveness( root, i, display_path, liveness )
        .map( | s | format!( "  {}", s.label() ) )
        .unwrap_or_default();
      writeln!( output, "  - {id}{count_str}{state_str}" ).unwrap();
    }
    else
    {
      writeln!( output, "  ? (orphan agents)" ).unwrap();
    }

    for ( j, agent ) in family.agents.iter().enumerate()
    {
      let connector = if j + 1 < family.agents.len() { "\u{251c}\u{2500}" } else { "\u{2514}\u{2500}" };
      let aid = agent.session.id();
      let atype = &agent.agent_type;
      let acount = agent.session
        .count_entries()
        .map( | n | {
          let noun = if n == 1 { "entry" } else { "entries" };
          format!( "  {n} {noun}" )
        } )
        .unwrap_or_default();
      writeln!( output, "    {connector} {aid}  {atype}{acount}" ).unwrap();
    }
  }
}

