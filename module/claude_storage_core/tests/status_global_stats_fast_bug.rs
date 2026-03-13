//! Bug Reproducers for `global_stats()` and agent session discovery issues
//!
//! ## issue-015: `global_stats()` blocks on JSONL parsing for large storage
//!
//! ## Root Cause
//!
//! `Storage::global_stats()` calls `project.project_stats()` for every project,
//! which calls `all_sessions()` and then `session.stats()` (reads + parses the JSONL
//! file) for every session. With 1903 projects / 2449 sessions / ~7 GB of JSONL data
//! this takes >2 minutes at the default `.status` verbosity, making the command
//! completely unusable.
//!
//! ## Why Not Caught (issue-015)
//!
//! Development and tests used small synthetic storage (few projects, few sessions).
//! The `O(total_JSONL_bytes)` complexity only shows at real-world storage sizes (thousands
//! of projects). No test existed that measured or bounded execution time.
//!
//! ## Fix Applied (issue-015)
//!
//! Added `Storage::global_stats_fast()` which counts projects and sessions using only
//! filesystem directory listings (no JSONL parsing). Uses `Project::count_sessions_split()`
//! which checks filename prefix (`agent-`) to classify session types without loading content.
//! `status_routine` now uses `global_stats_fast()` for verbosity 0-1 (the common case)
//! and `global_stats()` only at verbosity 2+ where the user explicitly requests deep stats.
//!
//! ## Prevention (issue-015)
//!
//! When computing counts of filesystem objects (files, directories), use metadata-only
//! operations (`readdir`, filename checks). Never parse file content just to count entries
//! unless the count itself is entry-level (e.g., "how many JSON objects in this JSONL").
//!
//! ## Pitfall (issue-015)
//!
//! `global_stats()` complexity is `O(total_JSONL_bytes)`, not `O(project_count)`. Any command
//! that calls it at the default verbosity will block indefinitely on large storage.
//!
//! ---
//!
//! ## issue-018: Agent sessions in new Claude Code v2.x format were invisible
//!
//! ## Root Cause (issue-018)
//!
//! Claude Code v2.x changed agent session storage from `{project_dir}/agent-{id}.jsonl`
//! (old) to `{project_dir}/{main_uuid}/subagents/agent-{id}.jsonl` (new). The
//! `iter_session_files()` scanner only examined the top-level project directory,
//! so new-format agent sessions were never found. With 11,757 agent session files
//! in production storage, `global_stats_fast()` still reported `Agent: 0`.
//!
//! ## Why Not Caught (issue-018)
//!
//! All tests used synthetic data in the old format. No test verified the new
//! `{uuid}/subagents/` directory structure introduced in Claude Code v2.x.
//!
//! ## Fix Applied (issue-018)
//!
//! Extended `iter_session_files()` to also traverse `{project_dir}/{uuid}/subagents/`
//! subdirectories when `include_agents = true`. Both old-format (direct `agent-*.jsonl`)
//! and new-format (`{uuid}/subagents/agent-*.jsonl`) are now discovered.
//!
//! ## Prevention (issue-018)
//!
//! When implementing storage iterators, verify the format against the actual filesystem
//! structure using real production data, not just synthetic test data.
//!
//! ## Pitfall (issue-018)
//!
//! Storage formats change between Claude Code versions. Never hardcode a directory
//! depth assumption — always inspect actual paths before writing an iterator.

use claude_storage_core::{ Storage, Project };
use std::fs;
use tempfile::TempDir;

/// Helper: create a minimal JSONL session file (empty, just needs to exist)
fn create_session_file( dir : &std::path::Path, name : &str )
{
  fs::write( dir.join( format!( "{name}.jsonl" ) ), b"" ).expect( "create session" );
}

/// Test `global_stats_fast()` returns correct project and session counts without JSONL parsing.
///
/// ## Purpose
///
/// Verifies the fast path (`global_stats_fast`) counts projects and sessions correctly
/// using only filesystem metadata. Session files are left empty — the test proves
/// we never attempt to parse their content.
///
/// ## Coverage
///
/// - Projects with only main sessions
/// - Projects with only agent sessions
/// - Projects with mixed main and agent sessions
/// - UUID-named projects (32-char hex without dashes: detected as UUID by storage)
/// - Path-named projects (hyphen-prefixed: detected as path by storage)
///
/// ## Validation Strategy
///
/// Create synthetic storage structure, call `global_stats_fast()`, assert counts.
/// Session files are intentionally left empty — if the implementation tries to parse
/// them it would return errors (empty JSONL is invalid).
#[test]
fn global_stats_fast_counts_correctly()
{
  let temp = TempDir::new().expect( "create temp dir" );
  let projects_dir = temp.path().join( "projects" );
  fs::create_dir_all( &projects_dir ).expect( "create projects dir" );

  // Project 1: UUID-type (hex UUID), 2 main sessions
  let p1 = projects_dir.join( "a1b2c3d4-e5f6-7890-abcd-ef1234567890" );
  fs::create_dir_all( &p1 ).expect( "create p1" );
  create_session_file( &p1, "11111111-1111-1111-1111-111111111111" );
  create_session_file( &p1, "22222222-2222-2222-2222-222222222222" );

  // Project 2: UUID-type, 1 main + 1 agent session
  let p2 = projects_dir.join( "b2c3d4e5-f6a7-8901-bcde-f12345678901" );
  fs::create_dir_all( &p2 ).expect( "create p2" );
  create_session_file( &p2, "33333333-3333-3333-3333-333333333333" );
  create_session_file( &p2, "agent-aabbccdd" );

  // Project 3: path-type (hyphen-prefixed), 3 agent sessions
  let p3 = projects_dir.join( "-home-user-myproject" );
  fs::create_dir_all( &p3 ).expect( "create p3" );
  create_session_file( &p3, "agent-11223344" );
  create_session_file( &p3, "agent-55667788" );
  create_session_file( &p3, "agent-99aabbcc" );

  let storage = Storage::with_root( temp.path() );
  let stats = storage.global_stats_fast().expect( "global_stats_fast" );

  // p1: 2 main, p2: 1 main + 1 agent, p3: 3 agents → totals: 3 main + 4 agent = 7
  assert_eq!( stats.total_projects, 3, "total projects" );
  assert_eq!( stats.uuid_projects, 2, "uuid projects" );
  assert_eq!( stats.path_projects, 1, "path projects" );
  assert_eq!( stats.total_sessions, 7, "total sessions: p1=2 + p2=2 + p3=3" );
  assert_eq!( stats.main_sessions, 3, "main sessions: p1=2 + p2=1 + p3=0" );
  assert_eq!( stats.agent_sessions, 4, "agent sessions: p1=0 + p2=1 + p3=3" );

  // Entries and tokens are NOT counted by the fast path
  assert_eq!( stats.total_entries, 0, "entries must be 0 in fast path" );
  assert_eq!( stats.total_input_tokens, 0, "input tokens must be 0 in fast path" );
}

/// Test `global_stats_fast()` handles empty storage (no projects) without error.
#[test]
fn global_stats_fast_empty_storage()
{
  let temp = TempDir::new().expect( "create temp dir" );
  let projects_dir = temp.path().join( "projects" );
  fs::create_dir_all( &projects_dir ).expect( "create projects dir" );

  let storage = Storage::with_root( temp.path() );
  let stats = storage.global_stats_fast().expect( "global_stats_fast" );

  assert_eq!( stats.total_projects, 0 );
  assert_eq!( stats.total_sessions, 0 );
  assert_eq!( stats.main_sessions, 0 );
  assert_eq!( stats.agent_sessions, 0 );
}

/// Test `Storage::with_root()` accepts paths and uses `global_stats_fast()` correctly.
#[test]
fn storage_with_root_uses_fast_stats()
{
  let temp = TempDir::new().expect( "create temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p1 = projects_dir.join( "-home-test" );
  fs::create_dir_all( &p1 ).expect( "create p1" );
  create_session_file( &p1, "session-aaaabbbb" );

  let storage = Storage::with_root( temp.path() );
  let stats = storage.global_stats_fast().expect( "global_stats_fast" );

  assert_eq!( stats.total_projects, 1 );
  assert_eq!( stats.path_projects, 1 );
  assert_eq!( stats.total_sessions, 1 );
  assert_eq!( stats.main_sessions, 1 );
  assert_eq!( stats.agent_sessions, 0 );
}

/// Test `global_stats_fast()` finds agent sessions in new Claude Code v2.x format.
///
/// ## Purpose
///
/// Verifies issue-018 fix: agent sessions stored in `{project_dir}/{uuid}/subagents/`
/// subdirectories (new format) are correctly discovered and counted.
///
/// ## Coverage
///
/// - New-format agent sessions: `{project_dir}/{uuid}/subagents/agent-*.jsonl`
/// - Old-format agent sessions: `{project_dir}/agent-*.jsonl` (backward compat)
/// - Mixed: project with both old-format and new-format agents
///
/// ## Validation Strategy
///
/// Create the exact directory structure used by Claude Code v2.x, call `global_stats_fast()`,
/// assert `agent_sessions` equals the number of new-format agent files created.
/// Before fix: `agent_sessions = 0` (subdirectories were invisible).
/// After fix: `agent_sessions` matches the actual count.
///
/// ## Related Requirements
///
/// issue-018: `iter_session_files()` must discover `{project_dir}/{uuid}/subagents/*.jsonl`
// test_kind: bug_reproducer(issue-018)
#[test]
fn global_stats_fast_finds_new_format_agent_sessions()
{
  let temp = TempDir::new().expect( "create temp dir" );
  let projects_dir = temp.path().join( "projects" );
  fs::create_dir_all( &projects_dir ).expect( "create projects dir" );

  // New-format structure: {project_dir}/{main_uuid}/subagents/agent-*.jsonl
  let p1 = projects_dir.join( "-home-user-pr-review" );
  fs::create_dir_all( &p1 ).expect( "create p1" );

  // Main session file at top level
  create_session_file( &p1, "43860c56-f828-44bd-953a-432920676b63" );

  // New-format agent sessions inside {main_uuid}/subagents/
  let subagents_dir = p1.join( "43860c56-f828-44bd-953a-432920676b63" ).join( "subagents" );
  fs::create_dir_all( &subagents_dir ).expect( "create subagents dir" );
  create_session_file( &subagents_dir, "agent-a60f26fee00bf6e3e" );
  create_session_file( &subagents_dir, "agent-b288d40921ed01252" );
  create_session_file( &subagents_dir, "agent-c9afcb57867dd64e" );

  let storage = Storage::with_root( temp.path() );
  let stats = storage.global_stats_fast().expect( "global_stats_fast" );

  assert_eq!( stats.total_projects, 1, "total projects" );
  // Before fix: agent_sessions = 0 (subagents/ was invisible)
  // After fix: agent_sessions = 3 (subagents traversed correctly)
  assert_eq!( stats.main_sessions, 1, "1 main session at top level" );
  assert_eq!( stats.agent_sessions, 3, "3 agent sessions in subagents/ subdir" );
  assert_eq!( stats.total_sessions, 4, "total: 1 main + 3 agent" );
}

/// Test `global_stats_fast()` handles both old and new agent session formats together.
///
/// ## Coverage
///
/// Old format: `agent-*.jsonl` directly in project dir (Claude Code v1.x)
/// New format: `{uuid}/subagents/agent-*.jsonl` (Claude Code v2.x)
/// Both should be counted as agent sessions.
// test_kind: bug_reproducer(issue-018)
#[test]
fn global_stats_fast_mixed_old_and_new_format_agents()
{
  let temp = TempDir::new().expect( "create temp dir" );
  let projects_dir = temp.path().join( "projects" );
  fs::create_dir_all( &projects_dir ).expect( "create projects dir" );

  let p1 = projects_dir.join( "-home-user-mixed" );
  fs::create_dir_all( &p1 ).expect( "create p1" );

  // Main session
  create_session_file( &p1, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" );

  // OLD format: agent session directly in project dir
  create_session_file( &p1, "agent-old-format" );

  // NEW format: agent sessions in subagents/ subdir
  let sub = p1.join( "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" ).join( "subagents" );
  fs::create_dir_all( &sub ).expect( "create subagents" );
  create_session_file( &sub, "agent-new-format-1" );
  create_session_file( &sub, "agent-new-format-2" );

  let storage = Storage::with_root( temp.path() );
  let stats = storage.global_stats_fast().expect( "global_stats_fast" );

  assert_eq!( stats.main_sessions, 1, "1 main session" );
  assert_eq!( stats.agent_sessions, 3, "1 old-format + 2 new-format agents" );
  assert_eq!( stats.total_sessions, 4, "1 + 3 = 4" );
}

/// Test `Project::sessions()` (`include_agents=false`) does NOT traverse subagents/ subdirs.
///
/// ## Purpose
///
/// Verifies the invariant that `sessions()` returns ONLY main sessions and excludes
/// new-format agent sessions stored in `{uuid}/subagents/`. The `include_agents=false`
/// guard must prevent subdirectory traversal entirely.
///
/// ## Coverage
///
/// - Main session at top level is returned
/// - New-format agents in `{uuid}/subagents/` are NOT returned
/// - Old-format agents (`agent-*.jsonl`) are also NOT returned
///
/// ## Validation Strategy
///
/// Create a project with both main and new-format agents, call `sessions()`, assert
/// only main sessions are returned. This guards against a regression where removing
/// the `include_agents` check would expose agents through the main-session API.
///
/// ## Related Requirements
///
/// issue-018: `iter_session_files()` must not expose new-format agents through the
/// main-only `sessions()` accessor.
#[ test ]
fn sessions_main_only_excludes_new_format_agents()
{
  let temp = TempDir::new().expect( "create temp dir" );
  let p_dir = temp.path().join( "-home-user-project" );
  fs::create_dir_all( &p_dir ).expect( "create project dir" );

  // Main session at top level
  create_session_file( &p_dir, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" );

  // Old-format agent at top level
  create_session_file( &p_dir, "agent-old-format" );

  // New-format agents in subagents/ subdir
  let sub = p_dir.join( "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" ).join( "subagents" );
  fs::create_dir_all( &sub ).expect( "create subagents dir" );
  create_session_file( &sub, "agent-new-1" );
  create_session_file( &sub, "agent-new-2" );

  let project = Project::load( &p_dir ).expect( "load project" );
  let sessions = project.sessions().expect( "sessions" );

  // sessions() must return only the 1 main session; no agents
  assert_eq!( sessions.len(), 1, "sessions() must exclude all agent sessions" );
  assert_eq!( sessions[ 0 ].id(), "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee", "main session id" );
}

/// Test `iter_session_files` ignores non-JSONL files inside subagents/ dirs.
///
/// ## Purpose
///
/// Verifies that non-JSONL files placed inside `{uuid}/subagents/` are silently
/// ignored. This prevents crashes from unexpected files (e.g., `.DS_Store`,
/// `readme.md`, or `.tmp` files).
///
/// ## Coverage
///
/// - `.txt` file in subagents/ is ignored
/// - Directory inside subagents/ is ignored
/// - JSONL files with `agent-` prefix are still counted correctly
///
/// ## Validation Strategy
///
/// Place non-JSONL files alongside valid agent sessions in subagents/, assert
/// counts match only the JSONL files.
#[ test ]
fn global_stats_fast_ignores_non_jsonl_in_subagents()
{
  let temp = TempDir::new().expect( "create temp dir" );
  let projects_dir = temp.path().join( "projects" );
  fs::create_dir_all( &projects_dir ).expect( "create projects dir" );

  let p1 = projects_dir.join( "-home-user-noisy" );
  fs::create_dir_all( &p1 ).expect( "create p1" );

  create_session_file( &p1, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" );

  let sub = p1.join( "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" ).join( "subagents" );
  fs::create_dir_all( &sub ).expect( "create subagents dir" );

  // Valid agent session
  create_session_file( &sub, "agent-valid" );

  // Non-JSONL files that must be ignored
  fs::write( sub.join( "readme.txt" ), b"not a session" ).expect( "create txt" );
  fs::write( sub.join( ".DS_Store" ), b"mac metadata" ).expect( "create DS_Store" );
  fs::create_dir_all( sub.join( "nested_dir" ) ).expect( "create nested dir" );

  let storage = Storage::with_root( temp.path() );
  let stats = storage.global_stats_fast().expect( "global_stats_fast" );

  assert_eq!( stats.main_sessions, 1, "1 main session" );
  assert_eq!( stats.agent_sessions, 1, "only the .jsonl agent; txt/DS_Store/dir ignored" );
  assert_eq!( stats.total_sessions, 2 );
}

/// Test `global_stats_fast()` handles empty subagents/ dirs without crashing.
///
/// ## Purpose
///
/// Verifies that an empty `{uuid}/subagents/` directory does not cause an error
/// or incorrect count. Empty subagents dirs arise when a main session is created
/// but no agents have been spawned yet.
///
/// ## Validation Strategy
///
/// Create project with main session + empty subagents dir, assert counts are correct.
#[ test ]
fn global_stats_fast_empty_subagents_dir()
{
  let temp = TempDir::new().expect( "create temp dir" );
  let projects_dir = temp.path().join( "projects" );
  fs::create_dir_all( &projects_dir ).expect( "create projects dir" );

  let p1 = projects_dir.join( "-home-user-empty-sub" );
  fs::create_dir_all( &p1 ).expect( "create p1" );

  create_session_file( &p1, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" );

  // Empty subagents dir — no agent sessions
  let sub = p1.join( "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" ).join( "subagents" );
  fs::create_dir_all( &sub ).expect( "create empty subagents dir" );

  let storage = Storage::with_root( temp.path() );
  let stats = storage.global_stats_fast().expect( "global_stats_fast" );

  assert_eq!( stats.main_sessions, 1, "1 main session" );
  assert_eq!( stats.agent_sessions, 0, "empty subagents dir = 0 agents" );
  assert_eq!( stats.total_sessions, 1 );
}
