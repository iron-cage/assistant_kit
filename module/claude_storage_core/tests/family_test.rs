//! Unit tests for `family::find_family()` — session family discovery across
//! the hierarchical (`{root}/subagents/*.jsonl`) and flat
//! (`agent-*.jsonl` + first-entry `sessionId`) layouts.
//!
//! Association contract under test:
//! `claude_storage/docs/invariant/002_session_family.md` — hierarchical
//! membership is decided by directory structure alone; flat membership by
//! the `sessionId` field of the FIRST entry.

use claude_storage_core::find_family;
use std::path::{ Path, PathBuf };

/// Write `lines` as a file under `dir`, creating parent directories.
fn write_file( dir : &Path, name : &str, lines : &[ &str ] ) -> PathBuf
{
  let path = dir.join( name );
  if let Some( parent ) = path.parent()
  {
    std::fs::create_dir_all( parent ).expect( "create parent dirs" );
  }
  std::fs::write( &path, lines.join( "\n" ) ).expect( "write file" );
  path
}

/// Minimal user entry carrying a `sessionId` field.
fn user_line( session_id : &str ) -> String
{
  format!( r#"{{"type":"user","sessionId":"{session_id}","message":{{"role":"user","content":"hi"}}}}"# )
}

/// Test `find_family` hierarchical layout discovery
///
/// ## Purpose
/// Validates that every `*.jsonl` inside `{project}/{root}/subagents/` is
/// associated with the root purely by directory structure.
///
/// ## Coverage
/// Two hierarchical agents plus the root; `root_path`/`root_id` fields;
/// deterministic (sorted) agent order.
///
/// ## Validation Strategy
/// Builds the layout on a tempdir, asserts exact `agent_paths` content and
/// order.
///
/// ## Related Requirements
/// `docs/invariant/002_session_family.md` — hierarchical association
#[ test ]
fn find_family_hierarchical_layout()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let dir = tmp.path();
  let root_path = write_file( dir, "root-1111.jsonl", &[ &user_line( "root-1111" ) ] );
  let a2 = write_file( dir, "root-1111/subagents/agent-bb.jsonl", &[ &user_line( "root-1111" ) ] );
  let a1 = write_file( dir, "root-1111/subagents/agent-aa.jsonl", &[ &user_line( "root-1111" ) ] );

  let family = find_family( dir, "root-1111" ).expect( "family found" );
  assert_eq!( family.root_id, "root-1111" );
  assert_eq!( family.root_path, root_path );
  assert_eq!( family.agent_paths, vec![ a1, a2 ], "sorted by path" );
}

/// Test `find_family` hierarchical membership ignores the sessionId field
///
/// ## Purpose
/// Locks in that hierarchical agents belong to the root whose directory
/// they live in, even when their own `sessionId` field names a DIFFERENT
/// session — the directory structure is the mandated association authority.
///
/// ## Coverage
/// One hierarchical agent whose first entry carries a foreign `sessionId`.
///
/// ## Validation Strategy
/// Asserts the agent is still included in the directory-owning root's
/// family.
///
/// ## Related Requirements
/// `docs/invariant/002_session_family.md` — "never use sessionId in
/// hierarchical layout"
#[ test ]
fn find_family_hierarchical_ignores_session_id_field()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let dir = tmp.path();
  write_file( dir, "root-2222.jsonl", &[ &user_line( "root-2222" ) ] );
  let agent = write_file( dir, "root-2222/subagents/agent-x.jsonl", &[ &user_line( "some-other-session" ) ] );

  let family = find_family( dir, "root-2222" ).expect( "family found" );
  assert_eq!( family.agent_paths, vec![ agent ] );
}

/// Test `find_family` flat layout association via first-entry sessionId
///
/// ## Purpose
/// Validates that a top-level `agent-*.jsonl` belongs to the family exactly
/// when its FIRST entry's `sessionId` names the root.
///
/// ## Coverage
/// One matching flat agent, one flat agent bound to a different root, and
/// one non-agent sibling session — only the first is included.
///
/// ## Validation Strategy
/// Asserts `agent_paths` contains exactly the matching flat agent.
///
/// ## Related Requirements
/// `docs/invariant/002_session_family.md` — flat association
#[ test ]
fn find_family_flat_layout_first_entry_session_id()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let dir = tmp.path();
  write_file( dir, "root-3333.jsonl", &[ &user_line( "root-3333" ) ] );
  write_file( dir, "root-9999.jsonl", &[ &user_line( "root-9999" ) ] );
  let mine = write_file( dir, "agent-mine.jsonl", &[ &user_line( "root-3333" ) ] );
  write_file( dir, "agent-other.jsonl", &[ &user_line( "root-9999" ) ] );

  let family = find_family( dir, "root-3333" ).expect( "family found" );
  assert_eq!( family.agent_paths, vec![ mine ] );
}

/// Test `find_family` missing root file is an error
///
/// ## Purpose
/// Validates that a family is anchored on its root — agent files alone
/// never constitute one.
///
/// ## Coverage
/// Project directory containing only an agent file for the requested root.
///
/// ## Validation Strategy
/// Asserts `find_family` returns `Err` when `{root}.jsonl` is absent.
///
/// ## Related Requirements
/// `docs/invariant/002_session_family.md` — root anchoring
#[ test ]
fn find_family_missing_root_errors()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let dir = tmp.path();
  write_file( dir, "agent-orphan.jsonl", &[ &user_line( "root-4444" ) ] );

  assert!( find_family( dir, "root-4444" ).is_err() );
}

/// Test `find_family` root with no agents yields an empty family
///
/// ## Purpose
/// Validates the common case — a conversation that never spawned agents.
///
/// ## Coverage
/// Root file alone, no subagents directory, no flat agents.
///
/// ## Validation Strategy
/// Asserts `agent_paths` is empty and root fields are populated.
///
/// ## Related Requirements
/// `docs/invariant/002_session_family.md`
#[ test ]
fn find_family_no_agents()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let dir = tmp.path();
  write_file( dir, "root-5555.jsonl", &[ &user_line( "root-5555" ) ] );

  let family = find_family( dir, "root-5555" ).expect( "family found" );
  assert!( family.agent_paths.is_empty() );
  assert_eq!( family.root_id, "root-5555" );
}

/// Test `find_family` ignores non-jsonl noise in subagents
///
/// ## Purpose
/// Validates noise tolerance: `.meta.json` sidecars and stray files inside
/// `subagents/` never surface as agent sessions.
///
/// ## Coverage
/// `agent-a.meta.json` (0-byte, as real storage produces) and a `.txt` file
/// alongside one real agent `.jsonl`.
///
/// ## Validation Strategy
/// Asserts only the `.jsonl` file is returned.
///
/// ## Related Requirements
/// `docs/invariant/002_session_family.md` — meta.json sidecars may be empty
#[ test ]
fn find_family_ignores_non_jsonl_in_subagents()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let dir = tmp.path();
  write_file( dir, "root-6666.jsonl", &[ &user_line( "root-6666" ) ] );
  let agent = write_file( dir, "root-6666/subagents/agent-a.jsonl", &[ &user_line( "root-6666" ) ] );
  write_file( dir, "root-6666/subagents/agent-a.meta.json", &[] );
  write_file( dir, "root-6666/subagents/notes.txt", &[ "not a session" ] );

  let family = find_family( dir, "root-6666" ).expect( "family found" );
  assert_eq!( family.agent_paths, vec![ agent ] );
}

/// Test `find_family` skips unreadable flat agent candidates gracefully
///
/// ## Purpose
/// Validates per-file graceful degradation: an empty or malformed-first-line
/// flat agent file is silently excluded, never a hard failure.
///
/// ## Coverage
/// An empty `agent-*.jsonl` and one whose first line is not JSON, alongside
/// one valid matching flat agent.
///
/// ## Validation Strategy
/// Asserts the call succeeds and returns only the valid agent.
///
/// ## Related Requirements
/// Graceful degradation convention (`Fix(BUG-489)`/`Fix(BUG-506)` lineage)
#[ test ]
fn find_family_flat_agent_empty_or_malformed_skipped()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let dir = tmp.path();
  write_file( dir, "root-7777.jsonl", &[ &user_line( "root-7777" ) ] );
  let good = write_file( dir, "agent-good.jsonl", &[ &user_line( "root-7777" ) ] );
  write_file( dir, "agent-empty.jsonl", &[] );
  write_file( dir, "agent-garbage.jsonl", &[ "not json at all {" ] );

  let family = find_family( dir, "root-7777" ).expect( "family found" );
  assert_eq!( family.agent_paths, vec![ good ] );
}

/// Test `find_family` combines hierarchical and flat agents
///
/// ## Purpose
/// Validates that both layouts contribute to one family when both exist,
/// with deterministic combined ordering.
///
/// ## Coverage
/// One hierarchical agent plus one matching flat agent for the same root.
///
/// ## Validation Strategy
/// Asserts both paths are present and the list is sorted.
///
/// ## Related Requirements
/// `docs/invariant/002_session_family.md` — both layouts
#[ test ]
fn find_family_combined_layouts()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let dir = tmp.path();
  write_file( dir, "root-8888.jsonl", &[ &user_line( "root-8888" ) ] );
  let flat = write_file( dir, "agent-flat.jsonl", &[ &user_line( "root-8888" ) ] );
  let hier = write_file( dir, "root-8888/subagents/agent-hier.jsonl", &[ &user_line( "root-8888" ) ] );

  let family = find_family( dir, "root-8888" ).expect( "family found" );
  let mut expected = vec![ flat, hier ];
  expected.sort();
  assert_eq!( family.agent_paths, expected );
}
