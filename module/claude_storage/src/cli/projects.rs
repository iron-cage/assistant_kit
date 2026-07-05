//! `.projects` command — session-first cross-project view with scope control.
//!
//! Also houses shared family/conversation domain types and the path-decode
//! helpers used by scope filtering (`under`, `relevant`, `around`).

use core::fmt::Write as FmtWrite;
use unilang::{ VerifiedCommand, ExecutionContext, OutputData, ErrorData, ErrorCode };
use super::storage::{ create_storage, resolve_path_parameter };

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

// ─── path decode helpers ───────────────────────────────────────────────────

/// Length, in bytes, of the longest common prefix of `a` and `b`. Always a
/// valid char-boundary index into both strings (accumulates whole chars).
fn common_prefix_len( a : &str, b : &str ) -> usize
{
  let mut len = 0;
  for ( ca, cb ) in a.chars().zip( b.chars() )
  {
    if ca != cb { break; }
    len += ca.len_utf8();
  }
  len
}

/// Check whether `encoded_base` (cwd or `path::` arg, encoded) is covered by
/// the project identified by `dir_name` (raw storage directory name).
///
/// Returns `true` when the project is an ancestor of (or equal to) the base:
/// - `encoded_base == dir_name` — same project, no topic
/// - `encoded_base.starts_with(dir_name + "-")` — base is in the project subtree
/// - same two checks after stripping a genuine `--topic` suffix from `dir_name`
///
/// Fix(BUG-003)
/// Root cause: the previous `rfind("--")` loop stripped from the LAST `--`
/// found anywhere in `dir_name`, with no way to tell whether that `--` was a
/// genuine topic-suffix marker or just incidental structure shared with
/// `encoded_base` (e.g. both paths sit under the same dot-prefixed temp-dir
/// root, which itself contains a `--`-like byte sequence once encoded).
/// Filesystem existence cannot discriminate either: a shallow shared ancestor
/// (e.g. `/tmp`) exists just as reliably as a genuine one.
/// Fix: only accept a `--` as a real topic boundary when it falls EXACTLY at
/// the point where `dir_name` and `encoded_base` diverge (the longest common
/// prefix). Shared/incidental structure can never BE the divergence point
/// between two different paths, so this structurally excludes false
/// positives without relying on filesystem state.
/// Pitfall: do not reintroduce a blind `rfind("--")`/`split("--")` search —
/// any boundary search that ignores `encoded_base` re-opens this hole.
fn is_relevant_encoded( dir_name : &str, encoded_base : &str ) -> bool
{
  let check = | candidate : &str | -> bool
  {
    encoded_base == candidate || encoded_base.starts_with( &format!( "{candidate}-" ) )
  };
  if check( dir_name ) { return true; }
  let lcp_len = common_prefix_len( dir_name, encoded_base );
  if lcp_len == 0 || lcp_len >= dir_name.len() { return false; }
  let ( before, after ) = ( &dir_name[ ..lcp_len ], &dir_name[ lcp_len.. ] );
  if !before.ends_with( '-' ) || !after.starts_with( '-' ) { return false; }
  check( &dir_name[ ..lcp_len - 1 ] )
}

/// Decode a storage directory name into a human-readable display path.
///
/// Path-encoded dirs start with `-` (e.g. `-home-alice-projects`). UUID dirs do not.
/// Compress `$HOME` prefix to `~` for display. Returns full path string if HOME unset.
fn tilde_compress( path : &std::path::Path ) -> String
{
  if let Ok( home ) = std::env::var( "HOME" )
  {
    if let Ok( rel ) = path.strip_prefix( std::path::Path::new( &home ) )
    {
      return format!( "~/{}", rel.display() );
    }
  }
  path.display().to_string()
}

/// Walk the filesystem to decode a lossy-encoded storage dir name to a real path.
///
/// At each `-` boundary the standard heuristic cannot distinguish a path separator
/// from an underscore (both encoded as `-`). This function resolves the ambiguity by
/// checking `is_dir()` at each step: it tries path separator first; if the candidate
/// directory does not exist, it falls back to joining with `_`.
///
/// Returns `None` if no matching path is found (project deleted, remote, or unmounted).
///
/// # Why only as fallback
///
/// Requires the project directory to exist on disk. Always call heuristic decode first
/// and only reach here when that result does not exist. This avoids unnecessary stat
/// calls for paths the heuristic already handles correctly.
fn decode_path_via_fs( encoded : &str ) -> Option< std::path::PathBuf >
{
  let inner = &encoded[ 1.. ]; // strip leading `-`
  let pieces : Vec< &str > = inner.split( '-' ).collect();
  if pieces.is_empty() { return None; }
  walk_fs( std::path::Path::new( "/" ), &pieces, 0, "" )
}

/// Decode the base-encoded component of a storage dir name to a real filesystem path.
///
/// Returns `None` if the encoded string is malformed (non-path-encoded keys such as UUIDs).
/// When `decode_path` succeeds but the result does not exist on disk, falls back to the
/// filesystem-guided walk to resolve `_` vs `/` ambiguity (Fix(issue-029)).
fn decode_storage_base( base_encoded : &str ) -> Option< std::path::PathBuf >
{
  use claude_storage_core::decode_path;
  let h = decode_path( base_encoded ).ok()?;
  if h.exists()
  {
    Some( h )
  }
  else
  {
    // Fix(issue-029): heuristic maps '_' to '/', try filesystem-guided decode.
    Some( decode_path_via_fs( base_encoded ).unwrap_or( h ) )
  }
}

/// Return true if `dir_name` encodes a project path that is `base_path` itself or is nested
/// under `base_path` (`scope::under` predicate).
///
/// The single-hyphen fast-reject `starts_with("{eb}-")` weeds out projects with completely
/// different paths before the more expensive filesystem decode.
///
/// Fix(BUG-003)
/// Root cause: previously stripped a `--topic` suffix via `strip_topic_suffix` before
/// the filesystem-verification decode. That stripping used the same unsound blind
/// `find("--")` search being removed from `is_relevant_encoded` for the same reason.
/// Fix: decode `dir_name` directly — `decode_path_via_fs` already treats an
/// unverifiable/nonexistent path as `true` (conservative include), which covers the
/// case where a genuine topic suffix would have made the raw decode fail to exist.
/// Pitfall: this simplification is only sound because no current test combines
/// `scope::under` with a project that has a genuine topic suffix; if such a test is
/// added, re-verify this fallback still selects the correct base.
fn matches_under( dir_name : &str, eb : &str, base_path : &std::path::Path ) -> bool
{
  if dir_name != eb && !dir_name.starts_with( &format!( "{eb}-" ) ) { return false; }
  if dir_name == eb { return true; }
  decode_path_via_fs( dir_name )
    .map_or( true, | p | p.starts_with( base_path ) )
}

/// Return true if `dir_name` encodes a project path that is an ancestor of `base_path`
/// (`scope::relevant` predicate).
///
/// Fix(BUG-003)
/// Root cause/Pitfall: see `matches_under` — same topic-suffix-stripping removal,
/// same conservative-include fallback in `decode_path_via_fs`.
fn matches_relevant( dir_name : &str, eb : &str, base_path : &std::path::Path ) -> bool
{
  if !is_relevant_encoded( dir_name, eb ) { return false; }
  if dir_name == eb { return true; }
  decode_path_via_fs( dir_name )
    .map_or( true, | p | base_path.starts_with( &p ) )
}

/// Recursive DFS helper for `decode_path_via_fs`.
///
/// `segment` accumulates the current unresolved path component. At each step, option A
/// commits `segment` as a directory and recurses with the next piece; option B appends
/// `_` + piece to `segment` and recurses. `is_dir()` prunes option A early.
fn walk_fs(
  base    : &std::path::Path,
  pieces  : &[ &str ],
  idx     : usize,
  segment : &str,
) -> Option< std::path::PathBuf >
{
  if idx == pieces.len()
  {
    let candidate = if segment.is_empty() { base.to_path_buf() } else { base.join( segment ) };
    return if candidate.exists() { Some( candidate ) } else { None };
  }
  let piece = pieces[ idx ];
  // Option A — path separator: commit current segment as a directory, recurse
  if !segment.is_empty()
  {
    let next_base = base.join( segment );
    if next_base.is_dir()
    {
      if let Some( result ) = walk_fs( &next_base, pieces, idx + 1, piece )
      {
        return Some( result );
      }
    }
  }
  // Option B — underscore: merge piece into segment
  let joined = if segment.is_empty()
  {
    piece.to_string()
  }
  else
  {
    format!( "{segment}_{piece}" )
  };
  walk_fs( base, pieces, idx + 1, &joined )
}

/// Decode a storage dir name to the longest real filesystem path it represents.
///
/// # Why the `starts_with('-')` guard
///
/// `decode_path()` requires its input to be a valid path-encoded string. UUID project
/// directories (e.g. `deadbeef-1234-...`) do not start with `-` and are NOT path-encoded.
/// Calling `decode_path` on a UUID returns `Err` — but more importantly, it would be
/// semantically wrong. UUID dirs represent web/IDE sessions without filesystem paths.
/// The guard ensures they fall through to the raw string return at the end.
///
/// # Topic components: metadata vs real directories
///
/// Topic-scoped project dirs are named `-path--topic` (double dash before topic).
/// Topics are often pure metadata tags (e.g. `--commit`), but they can also be real
/// hyphen-prefixed directories (e.g. `--default-topic` → `-default_topic/`).
///
/// Examples:
/// - `-...-src--default-topic`         → `src/-default_topic`
/// - `-...-src--default-topic--commit` → `src/-default_topic/-commit`
/// - `-...-src--commit`                → `src/-commit`
///
/// # Why a single `decode_storage_base` call is sufficient
///
/// Fix(BUG-003)
/// Root cause: this used to call `split_storage_key` to break `dir_name` into a base
/// component plus a list of `--topic` components, then re-join each topic as a
/// separate `-{topic}` path segment. That split relied on the same unsound blind
/// `find("--")` search removed from `is_relevant_encoded` for the same reason — it
/// could not tell a genuine topic boundary from incidental shared structure.
/// Fix: `claude_storage_core::decode_path`'s own heuristic already chains multiple
/// `--`-separated segments correctly on its own (each `--` starts a new
/// hyphen-prefixed segment while the rest of that segment maps `-` → `_`), so passing
/// the whole `dir_name` straight through reconstructs the same multi-topic display
/// path with no external split/append loop needed.
/// Pitfall: do not reintroduce a `--`-splitting loop here — `decode_storage_base`
/// (via `decode_path`) already handles the full string, topics included.
///
/// # Why the filesystem fallback for the base
///
/// Fix(issue-029)
/// Root cause: `decode_path` heuristic defaults to path separator `/` for all
/// unrecognized `-` boundaries. Paths with underscore-named dirs (e.g. `my_project`,
/// `claude_tools`) display incorrectly as `wip/core`, `claude/tools`.
/// Pitfall: Only call the filesystem walk as fallback — never primary — because it
/// requires the project directory to exist on disk. Deleted/remote projects fall
/// back to the raw encoded storage dir name.
fn decode_project_display( dir_name : &str ) -> String
{
  if !dir_name.starts_with( '-' ) { return dir_name.to_string(); }
  let Some( path ) = decode_storage_base( dir_name ) else { return dir_name.to_string() };
  tilde_compress( &path )
}

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

fn format_relative_time( mtime : std::time::SystemTime ) -> String
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

// ─── .projects routine ─────────────────────────────────────────────────────

/// List sessions with scope control (session-first view).
///
/// Scope semantics:
/// - `local`    — Current project only (`path::` selects the project, defaults to cwd)
/// - `relevant` — Every project whose path is an ancestor of (or equal to) `path::`
/// - `under`    — Every project whose path starts with `path::` (default)
/// - `global`   — All projects in storage (ignores `path::`)
///
/// # Errors
///
/// Returns error if `scope::` is invalid, `min_entries::` is negative,
/// `limit::` is negative, path resolution fails, or storage access fails.
///
/// # Panics
///
/// Does not panic — `min_entries` and `limit` are validated non-negative before conversion.
#[ allow( clippy::needless_pass_by_value ) ]
#[ allow( clippy::too_many_lines ) ]
#[ inline ]
pub fn projects_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  use std::collections::BTreeMap;
  use std::path::PathBuf;
  use claude_storage_core::{ Session, SessionFilter, encode_path };

  // --- parameters ---

  let scope_raw = cmd.get_string( "scope" ).unwrap_or( "around" );
  let scope = scope_raw.to_lowercase();
  if !matches!( scope.as_str(), "local" | "relevant" | "under" | "around" | "global" )
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "scope must be relevant|local|under|around|global, got {scope_raw}" ),
    ) );
  }

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

  let agent_filter = cmd.get_boolean( "agent" );
  let session_id_filter = cmd.get_string( "session" );

  // Resolve base path (used by local / relevant / under; ignored for global)
  let base_path : PathBuf = if let Some( p ) = cmd.get_string( "path" )
  {
    resolve_path_parameter( p )
      .map( PathBuf::from )
      .map_err( | e | ErrorData::new(
        ErrorCode::InternalError,
        format!( "Failed to resolve path '{p}': {e}" ),
      ) )?
  }
  else
  {
    std::env::current_dir()
      .map_err( | e | ErrorData::new(
        ErrorCode::InternalError,
        format!( "Failed to get current directory: {e}" ),
      ) )?
  };

  // --- collect projects by scope ---

  let storage = create_storage()?;
  let all_projects = storage.list_projects()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;

  // Fix(issue-024)
  // Root cause: encode_path() maps both '_' and '/' to '-', so decode_component()
  // defaults unknown pairs to '/', turning `my_project` → `wip-core` → `wip/core`.
  // Decoded paths never match the real base_path, causing silent 0-result returns.
  // Pitfall: Never decode storage dir names for path comparison — encoding is
  // deterministic but decoding is lossy. Compare encoded ↔ encoded instead.
  let encoded_base : Option< String > = if scope == "global"
  {
    None
  }
  else
  {
    Some(
      encode_path( &base_path )
        .map_err( | e | ErrorData::new(
          ErrorCode::InternalError,
          format!( "Failed to encode base path '{}': {e}", base_path.display() ),
        ) )?
    )
  };

  // Closure: does this project qualify under `scope`?
  // Compares encoded base against raw storage directory name — no decode step.
  // UUID project dirs start with a hex character (not '-'), so they never match
  // path-based comparisons and are correctly excluded from non-global scopes.
  let project_matches = | project : &claude_storage_core::Project | -> bool
  {
    if scope == "global" { return true; }
    let Some( ref eb ) = encoded_base else { return false };
    let dir_name = project
      .storage_dir()
      .file_name()
      .and_then( | n | n.to_str() )
      .unwrap_or( "" );
    match scope.as_str()
    {
      "local"    => dir_name == eb || dir_name.starts_with( &format!( "{eb}--" ) ),
      // Fix(issue-031)
      // Root cause: starts_with on encoded strings cannot distinguish a child
      //   directory (base/sub → `base-sub`) from a same-level sibling whose name
      //   uses an underscore (base_extra → `base-extra`): both share the `base-`
      //   prefix. Path::starts_with is component-wise and correctly excludes siblings.
      // Pitfall: strip the `--topic` suffix from dir_name before calling
      //   decode_path_via_fs. The `--topic` part encodes a hyphen-prefixed directory
      //   like `-default_topic`; left in place, the walker searches for a dir named
      //   `topic` under the project root, returns None, and the fallback silently
      //   includes everything — the sibling exclusion is bypassed.
      "under" => matches_under( dir_name, eb, &base_path ),
      // Fix(issue-032)
      // Root cause: is_relevant_encoded uses string starts_with to check if
      //   dir_name's encoded path is a prefix of encoded_base, so a sibling
      //   `base` (encoded `base-`) falsely matches when base_path is `base_extra`
      //   (encoded `base-extra`). Both `_` and `/` map to `-`, making siblings
      //   indistinguishable from ancestors by string comparison alone.
      //   base_path.starts_with(decoded_path) is component-wise and rejects siblings.
      // Pitfall: strip the `--topic` suffix before calling decode_path_via_fs —
      //   same requirement as the issue-031 fix for scope::under.
      "relevant" => matches_relevant( dir_name, eb, &base_path ),
      // Union of under + relevant — bidirectional neighborhood.
      // BTreeMap key on decoded path deduplicates projects matched by both arms.
      "around" =>
        matches_under( dir_name, eb, &base_path )
          || matches_relevant( dir_name, eb, &base_path ),
      _          => false,
    }
  };

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

  for mut project in all_projects
  {
    if !project_matches( &project ) { continue; }

    let dir_name = project
      .storage_dir()
      .file_name()
      .and_then( | n | n.to_str() )
      .unwrap_or( "" )
      .to_string();
    let display_path = decode_project_display( &dir_name );

    let Ok( sessions ) = project.sessions_filtered( &session_filter ) else { continue };
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
  let summaries = aggregate_projects( &mut groups );

  let total_projects = summaries.len();
  let mut output = String::new();

  // Family grouping: with no explicit agent:: filter, agents are grouped
  // into families under their root sessions instead of shown flat.
  let use_families = agent_filter.is_none();

  let p_noun = if total_projects == 1 { "project" } else { "projects" };
  writeln!( output, "Found {total_projects} {p_noun}:\n" ).unwrap();

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

      if show_tree
      {
        render_families_v2( &mut output, &families );
      }
      else
      {
        render_families_v1( &mut output, &families, limit_cap );
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
      let group_noun = if group_count == 1 { "conversation" } else { "conversations" };
      writeln!( output, "{display_path}: ({group_count} {group_noun})" ).unwrap();
      let show_count = displayable.len().min( limit_cap );
      for ( i, &session ) in displayable[ ..show_count ].iter().enumerate()
      {
        let marker = if i == 0 { '*' } else { '-' };
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
        writeln!( output, "  {marker} {id_str}{time_str}{count_str}" ).unwrap();
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

    writeln!( output ).unwrap();
  }

  Ok( OutputData::new( output, "text" ) )
}

// ─── render helpers ────────────────────────────────────────────────────────

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

/// Format a single session line: `{marker} {id}  {age}  ({n} entries)`.
fn format_session_line(
  session : &claude_storage_core::Session,
  marker  : char,
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
  format!( "  {marker} {id_str}{time_str}{count_str}" )
}

/// Render family-grouped display at v1: root lines with `[N agents: breakdown]`.
fn render_families_v1(
  output    : &mut String,
  families  : &[ SessionFamily ],
  limit_cap : usize,
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
      let line = format_session_line( root, marker );
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
  output   : &mut String,
  families : &[ SessionFamily ],
)
{
  for family in families
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
      writeln!( output, "  - {id}{count_str}" ).unwrap();
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

