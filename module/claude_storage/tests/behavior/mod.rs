//! Behavior hypothesis invalidation tests.
//!
//! Each file covers exactly one behavior from `docs/claude_code/behavior.md` (B1..B15).
//! Tests inspect real `~/.claude/` storage to verify Claude Code's actual output.
//! If Claude Code changes behavior, the tests go RED.
//!
//! ## File Index
//!
//! | File | Behavior | Category |
//! |------|----------|----------|
//! | `b01_default_continues.rs` | B1 — default invocation continues most recent session | Continuation |
//! | `b02_new_session.rs` | B2 — `--new-session` creates separate `.jsonl` | Continuation |
//! | `b03_print_flag.rs` | B3 — `-p` is output mode, not session flag | Flags |
//! | `b04_continue_flag.rs` | B4 — `-c` aliases default continuation | Flags |
//! | `b05_mtime_selection.rs` | B5 — current session selected by mtime | Selection |
//! | `b06_session_accumulation.rs` | B6 — sessions accumulate as separate files | Storage |
//! | `b07_agent_sessions.rs` | B7 — agent sessions are `agent-*.jsonl` siblings | Storage |
//! | `b08_zero_byte_init.rs` | B8 — 0-byte `.jsonl` created as placeholder on startup | Storage |
//! | `b09_storage_path.rs` | B9 — project path uses `/`→`-` encoding | Storage |
//! | `b10_entry_threading.rs` | B10 — entries linked via `parentUuid` | Entries |
//! | `b11_auto_continue.rs` | B11 — `CLAUDE_CODE_AUTO_CONTINUE` env var | Flags |
//! | `b12_agent_session_id_is_parent.rs` | B12 — agent `sessionId` = parent UUID | Families |
//! | `b13_subagent_directory_structure.rs` | B13 — `{uuid}/subagents/` hierarchy | Families |
//! | `b14_agent_meta_json.rs` | B14 — `.meta.json` sidecars with `agentType` | Families |
//! | `b15_agent_slug_field.rs` | B15 — agents carry shared `slug` field | Families |
//! | `b16_tools_disable.rs` | B16 — `--tools ""` disables tool invocation; definitions may or may not be stripped (H1 vs H2 ❓) | Flags |
//! | `b17_parentuuid_self_contained.rs` | B17 — `parentUuid` orphaned-link rate < 1%; compaction-boundary exception documented | Entries |
//! | `b18_no_cross_session_links.rs` | B18 — first conversation entry of every session has `parentUuid: null` | Entries |

#[ allow( dead_code ) ]
#[ path = "../common/mod.rs" ]
mod common;

mod b01_default_continues;
mod b02_new_session;
mod b03_print_flag;
mod b04_continue_flag;
mod b05_mtime_selection;
mod b06_session_accumulation;
mod b07_agent_sessions;
mod b08_zero_byte_init;
mod b09_storage_path;
mod b10_entry_threading;
mod b11_auto_continue;
mod b12_agent_session_id_is_parent;
mod b13_subagent_directory_structure;
mod b14_agent_meta_json;
mod b15_agent_slug_field;
mod b16_tools_disable;
mod b17_parentuuid_self_contained;
mod b18_no_cross_session_links;

// ---------------------------------------------------------------------------
// Shared helpers for behavior tests
// ---------------------------------------------------------------------------

/// Resolve `~/.claude/` directory. Returns `None` if it does not exist.
#[ must_use ]
#[ inline ]
pub fn claude_home() -> Option< std::path::PathBuf >
{
  let home = std::env::var( "HOME" ).ok()?;
  let claude = std::path::PathBuf::from( home ).join( ".claude" );
  if claude.is_dir() { Some( claude ) } else { None }
}

/// Resolve `~/.claude/projects/` directory. Returns `None` if missing.
#[ must_use ]
#[ inline ]
pub fn claude_projects_dir() -> Option< std::path::PathBuf >
{
  let projects = claude_home()?.join( "projects" );
  if projects.is_dir() { Some( projects ) } else { None }
}

/// List all project directories under `~/.claude/projects/`.
#[ must_use ]
#[ inline ]
pub fn find_projects() -> Vec< std::path::PathBuf >
{
  let Some( projects_dir ) = claude_projects_dir() else { return vec![] };
  std::fs::read_dir( projects_dir )
    .ok()
    .into_iter()
    .flatten()
    .filter_map( Result::ok )
    .filter( | e | e.file_type().map( | t | t.is_dir() ).unwrap_or( false ) )
    .map( | e | e.path() )
    .collect()
}

/// List `.jsonl` files in a project directory (non-agent, non-zero-byte).
#[ must_use ]
#[ inline ]
pub fn find_sessions( project : &std::path::Path ) -> Vec< std::path::PathBuf >
{
  std::fs::read_dir( project )
    .ok()
    .into_iter()
    .flatten()
    .filter_map( Result::ok )
    .filter( | e |
    {
      let name = e.file_name();
      let name = name.to_string_lossy();
      name.ends_with( ".jsonl" )
        && !name.starts_with( "agent-" )
        && e.metadata().map( | m | m.len() > 0 ).unwrap_or( false )
    })
    .map( | e | e.path() )
    .collect()
}

/// List all `.jsonl` files in a project (including agent and zero-byte).
#[ must_use ]
#[ inline ]
pub fn find_all_jsonl( project : &std::path::Path ) -> Vec< std::path::PathBuf >
{
  std::fs::read_dir( project )
    .ok()
    .into_iter()
    .flatten()
    .filter_map( Result::ok )
    .filter( | e | e.file_name().to_string_lossy().ends_with( ".jsonl" ) )
    .map( | e | e.path() )
    .collect()
}

/// List `agent-*.jsonl` files in a project directory.
#[ must_use ]
#[ inline ]
pub fn find_agent_sessions( project : &std::path::Path ) -> Vec< std::path::PathBuf >
{
  std::fs::read_dir( project )
    .ok()
    .into_iter()
    .flatten()
    .filter_map( Result::ok )
    .filter( | e |
    {
      let name = e.file_name();
      let name = name.to_string_lossy();
      name.starts_with( "agent-" ) && name.ends_with( ".jsonl" )
    })
    .map( | e | e.path() )
    .collect()
}

/// Find the `claude` binary on PATH. Returns `None` if not found.
#[ must_use ]
#[ inline ]
pub fn find_claude_binary() -> Option< std::path::PathBuf >
{
  let path_var = std::env::var( "PATH" ).ok()?;
  for dir in path_var.split( ':' )
  {
    let candidate = std::path::PathBuf::from( dir ).join( "claude" );
    if candidate.is_file() { return Some( candidate ); }
  }
  None
}

/// Capture stdout from a process output as a UTF-8 string.
#[ must_use ]
#[ inline ]
pub fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

/// Capture stderr from a process output as a UTF-8 string.
#[ must_use ]
#[ inline ]
pub fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

/// List `{uuid}/subagents/` directories under a project directory.
///
/// Returns pairs of `(parent_uuid, subagents_dir_path)` where `parent_uuid` is the
/// directory name (expected to be a UUID) and the path points to the `subagents/` dir.
#[ must_use ]
#[ inline ]
pub fn find_subagent_dirs( project : &std::path::Path ) -> Vec< ( String, std::path::PathBuf ) >
{
  std::fs::read_dir( project )
    .ok()
    .into_iter()
    .flatten()
    .filter_map( Result::ok )
    .filter( | e | e.file_type().map( | t | t.is_dir() ).unwrap_or( false ) )
    .filter_map( | e |
    {
      let name = e.file_name().to_string_lossy().into_owned();
      let subagents = e.path().join( "subagents" );
      if subagents.is_dir() { Some( ( name, subagents ) ) } else { None }
    })
    .collect()
}

/// List `.meta.json` files in a subagents directory.
#[ must_use ]
#[ inline ]
pub fn find_meta_json_files( subagents_dir : &std::path::Path ) -> Vec< std::path::PathBuf >
{
  std::fs::read_dir( subagents_dir )
    .ok()
    .into_iter()
    .flatten()
    .filter_map( Result::ok )
    .filter( | e | e.file_name().to_string_lossy().ends_with( ".meta.json" ) )
    .map( | e | e.path() )
    .collect()
}

/// List `agent-*.jsonl` files in a subagents directory (non-zero-byte).
#[ must_use ]
#[ inline ]
pub fn find_subagent_sessions( subagents_dir : &std::path::Path ) -> Vec< std::path::PathBuf >
{
  std::fs::read_dir( subagents_dir )
    .ok()
    .into_iter()
    .flatten()
    .filter_map( Result::ok )
    .filter( | e |
    {
      let name = e.file_name();
      let name = name.to_string_lossy();
      name.starts_with( "agent-" )
        && name.ends_with( ".jsonl" )
        && e.metadata().map( | m | m.len() > 0 ).unwrap_or( false )
    })
    .map( | e | e.path() )
    .collect()
}
