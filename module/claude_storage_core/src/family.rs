//! Session family discovery — a root session plus the agent sessions it
//! spawned.
//!
//! Implements the association contract of
//! `claude_storage/docs/invariant/002_session_family.md`: hierarchical-layout
//! agents (`{project_dir}/{root_id}/subagents/*.jsonl`) belong to the family
//! by directory structure alone (the mandated authority — never their
//! `sessionId` field); flat-layout agents (`{project_dir}/agent-*.jsonl`)
//! belong when the `sessionId` field of their FIRST entry names the root.
//! Powers the `claude_storage` CLI's `.cost` command's fold-in of agent
//! sessions; see `claude_storage/docs/cli/command/15_cost.md`.

use std::
{
  fs,
  io::{ BufRead, BufReader },
  path::{ Path, PathBuf },
};

use crate::{ Error, Result, json::parse_json };

/// A root session plus every agent session file associated with it.
///
/// Paths only — no session content is loaded. Callers decide what to do with
/// each member (e.g. `cost::cost_report` per file for the `.cost` command).
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub struct SessionFamily
{
  /// Root (non-agent) session ID — filename stem of `root_path`.
  pub root_id : String,
  /// Path of the root session's JSONL file.
  pub root_path : PathBuf,
  /// Paths of every agent session file belonging to this root, across both
  /// layouts, sorted by path for deterministic iteration order.
  pub agent_paths : Vec< PathBuf >,
}

/// Discover the session family of `root_id` inside `project_dir`.
///
/// Membership rules (see `docs/invariant/002_session_family.md` in the
/// `claude_storage` crate):
/// - **Hierarchical layout**: every `*.jsonl` file directly inside
///   `{project_dir}/{root_id}/subagents/` — the directory structure is the
///   association authority, matching `Project::iter_session_files()`'s own
///   discovery (which likewise takes any `.jsonl` there, not only
///   `agent-*`-prefixed ones). The `sessionId` field of hierarchical agents
///   is deliberately never consulted.
/// - **Flat layout**: every top-level `{project_dir}/agent-*.jsonl` file
///   whose FIRST parseable non-empty line carries `"sessionId"` equal to
///   `root_id`. Only the first line is read — the invariant pins the
///   association to the first entry, and later entries may carry other IDs.
///
/// A flat agent file that is empty, unreadable, or whose first line does not
/// parse is silently skipped (per-file graceful degradation, mirroring
/// `Session::stats()`'s per-line convention) — a corrupted sidecar must not
/// fail family discovery for the root.
///
/// # Errors
///
/// Returns `Error::SessionNotFound` when `{project_dir}/{root_id}.jsonl`
/// does not exist — a family is anchored on its root; agent files alone
/// never constitute one. Returns `Error::Io` when the project directory
/// itself cannot be read.
#[ inline ]
pub fn find_family( project_dir : &Path, root_id : &str ) -> Result< SessionFamily >
{
  let root_path = project_dir.join( format!( "{root_id}.jsonl" ) );
  if !root_path.is_file()
  {
    return Err( Error::session_not_found( root_path.to_string_lossy().to_string() ) );
  }

  let mut agent_paths = Vec::new();

  // Hierarchical layout: {project_dir}/{root_id}/subagents/*.jsonl —
  // directory structure alone decides membership. An unreadable subagents
  // directory is silently skipped, mirroring `Project::iter_session_files()`'s
  // own `let Ok(...) else { continue }` on the same directory.
  let subagents_dir = project_dir.join( root_id ).join( "subagents" );
  if let Ok( entries ) = fs::read_dir( &subagents_dir )
  {
    for entry in entries.flatten()
    {
      let path = entry.path();
      if path.extension().and_then( | s | s.to_str() ) == Some( "jsonl" ) && path.is_file()
      {
        agent_paths.push( path );
      }
    }
  }

  // Flat layout: {project_dir}/agent-*.jsonl associated via the sessionId
  // field of the FIRST entry.
  let entries = fs::read_dir( project_dir )
    .map_err( | e | Error::io( e, format!( "reading project directory: {}", project_dir.display() ) ) )?;
  for entry in entries.flatten()
  {
    let path = entry.path();
    let Some( filename ) = path.file_name().and_then( | s | s.to_str() ) else { continue };
    if !filename.starts_with( "agent-" ) { continue; }
    if path.extension().and_then( | s | s.to_str() ) != Some( "jsonl" ) || !path.is_file() { continue; }
    if first_entry_session_id( &path ).as_deref() == Some( root_id )
    {
      agent_paths.push( path );
    }
  }

  agent_paths.sort();

  Ok( SessionFamily
  {
    root_id : root_id.to_string(),
    root_path,
    agent_paths,
  })
}

/// `sessionId` of the first parseable non-empty line of `path`, or `None`
/// when the file is unreadable/empty or its first non-empty line fails to
/// parse. Reads only as far as that single line — never the whole file.
fn first_entry_session_id( path : &Path ) -> Option< String >
{
  let file = fs::File::open( path ).ok()?;
  let reader = BufReader::new( file );
  for line in reader.lines()
  {
    let Ok( line ) = line else { return None };
    if line.trim().is_empty() { continue; }
    let json = parse_json( &line ).ok()?;
    return json.get_str( "sessionId" ).map( std::string::ToString::to_string );
  }
  None
}
