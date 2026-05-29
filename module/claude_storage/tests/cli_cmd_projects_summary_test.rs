//! Integration tests for the `clg .projects` command — display and summary group.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/07_projects.md`
//!
//! ## Coverage
//!
//! - INT-26: zero-byte sessions excluded from v1 display
//! - INT-27: Summary header format — LEGACY (task-019 removed summary mode; tests list format)
//! - INT-28: Truncation gate ≤ 50 chars — LEGACY (truncation helpers removed with summary mode)
//! - INT-29: Truncation formula > 50 chars — LEGACY (truncation helpers removed with summary mode)
//! - INT-30: No sessions → "No active project found." — now "Found 0 projects:" (task-019)
//! - INT-31: Explicit `scope::local` produces list output
//! - INT-32: Explicit `limit::N` produces list output
//! - INT-33: Family header format (conversations + agents)
//! - INT-34: Per-root agent breakdown [N agents: type summary]
//! - INT-35: Hierarchical format detection (subagents/ path)
//! - INT-36: Flat format detection (sessionId linkage)
//! - INT-37: Orphan family display (root missing)
//! - INT-38: Childless root (no bracket suffix)
//! - INT-39: Meta.json agentType in breakdown
//! - INT-40: Empty/malformed meta.json fallback to "unknown"
//! - INT-41: v1 orphan shows `? (orphan)` label (bug-cc-c1)
//! - INT-42: v2 root entry count singular `(1 entry)`
//! - INT-43: v2 agent entry count singular `1 entry`
//! - INT-41b: `verbosity::1` alone — LEGACY (was: stays summary; now: shows list output)
//! - INT-42b: "Active project" header — LEGACY (task-019: summary removed; shows "Found N projects:")
//! - INT-43b: Session count aggregate — LEGACY (task-019: now shown in list-mode header)
//! - INT-44: List mode shows projects sorted by recency
//! - INT-45: `verbosity::0` outputs project paths only
//! - INT-46: Topic path shown even when topic dir absent from disk
//! - INT-47: Topic path shown when topic dir present on disk
//! - INT-48: Default-topic path shown when topic dir absent from disk
//! - INT-49: Base path shown correctly with no topic suffix
//! - INT-50: Double-topic key shows both topic components unconditionally
//!
//! Tests INT-1..INT-25: → `cli_cmd_projects_test.rs`

mod common;

use std::fs;
use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

// ─── INT-26 ───────────────────────────────────────────────────────────────────

/// INT-26: Zero-byte sessions excluded from v1 display.
#[ test ]
fn int_26_v1_zero_byte_sessions_excluded()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-real", 2 );

  // Create a zero-byte placeholder (B8 behaviour — Claude Code startup artifact).
  let encoded = claude_storage_core::encode_path( &project ).unwrap();
  let dir = storage_root.join( "projects" ).join( &encoded );
  fs::create_dir_all( &dir ).unwrap();
  let _ = fs::File::create( dir.join( "session-placeholder.jsonl" ) ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-real" ),        "real session must appear; got:\n{s}" );
  assert!( !s.contains( "session-placeholder" ), "zero-byte placeholder must be absent at v1; got:\n{s}" );
}

// ─── INT-27 ───────────────────────────────────────────────────────────────────

/// INT-27: Summary header format — LEGACY spec entry.
///
/// The spec described a single-project summary block:
///   `Active project  {path}  (N sessions, last active Xd ago)`
/// Task-019 removed summary mode. Bare `clg .projects` now always uses list
/// mode (`scope::around`), emitting `Found N projects:` and grouped path headers.
/// This test verifies current list-mode output contains project and session info.
#[ test ]
fn int_27_list_mode_shows_project_and_session_info()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int27", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // List mode header must be present (summary mode was removed in task-019).
  assert!( s.contains( "Found" ),          "list-mode header must be present; got:\n{s}" );
  assert!( !s.contains( "Active project" ), "summary block must be absent (task-019); got:\n{s}" );
}

// ─── INT-28 ───────────────────────────────────────────────────────────────────

/// INT-28: Truncation gate (message ≤ 50 chars shown in full) — LEGACY spec entry.
///
/// The spec described a `Last message:` section in summary mode with a
/// truncation gate at 50 characters. Task-019 deleted the summary mode along
/// with `truncate_message`, `last_text_entry`, `TRUNCATE_THRESHOLD`, and
/// `TRUNCATE_PREVIEW`. There is no `Last message:` section in current output.
/// This test verifies the command handles sessions with short messages without
/// errors, producing list-mode output.
#[ test ]
fn int_28_short_last_message_produces_valid_list_output()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj28" );
  fs::create_dir_all( &project ).unwrap();
  // 40-character message (≤ 50 — was "shown in full" in legacy summary mode).
  common::write_path_project_session_with_last_message(
    &storage_root, &project, "session-int28", 1,
    "Fix typo in the readme file near line 10",
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ),     "list-mode header must be present; got:\n{s}" );
  assert!( !s.contains( "Last message" ), "summary 'Last message:' section must be absent; got:\n{s}" );
}

// ─── INT-29 ───────────────────────────────────────────────────────────────────

/// INT-29: Truncation formula (message > 50 chars as first30…last30) — LEGACY spec entry.
///
/// Same as INT-28: truncation helpers were deleted with summary mode (task-019).
/// Test verifies the command handles sessions with long messages without errors.
#[ test ]
fn int_29_long_last_message_produces_valid_list_output()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj29" );
  fs::create_dir_all( &project ).unwrap();
  // 60-character message (> 50 — was "first30…last30" in legacy summary mode).
  common::write_path_project_session_with_last_message(
    &storage_root, &project, "session-int29", 1,
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA_BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ),     "list-mode header must be present; got:\n{s}" );
  assert!( !s.contains( "Last message" ), "summary 'Last message:' section must be absent; got:\n{s}" );
}

// ─── INT-30 ───────────────────────────────────────────────────────────────────

/// INT-30: No sessions in scope — current behavior is "Found 0 projects:".
///
/// The spec said `"No active project found."` (summary-mode sentinel). Task-019
/// replaced all zero-result paths with the list-mode zero-result header
/// `"Found 0 projects:"`.
#[ test ]
fn int_30_no_sessions_shows_zero_result_header()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  fs::create_dir_all( &storage_root ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // After task-019: list-mode zero-result header replaces the summary sentinel.
  assert!(
    s.contains( "Found 0" ) || s.is_empty(),
    "empty storage must show 'Found 0' header or empty output; got:\n{s}"
  );
  assert!(
    !s.contains( "No active project found" ),
    "legacy summary sentinel must be absent; got:\n{s}"
  );
  assert!( stderr( &out ).is_empty(), "stderr must be empty; got: {}", stderr( &out ) );
}

// ─── INT-31 ───────────────────────────────────────────────────────────────────

/// INT-31: Explicit `scope::local` produces list output.
///
/// With summary mode removed (task-019), ANY scope produces list output.
/// Explicit `scope::local` must emit the `Found N project(s):` header.
#[ test ]
fn int_31_explicit_scope_local_shows_list_output()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj31" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int31", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ),          "scope::local must emit list-mode header; got:\n{s}" );
  assert!( !s.contains( "Active project" ), "summary block must be absent; got:\n{s}" );
}

// ─── INT-32 ───────────────────────────────────────────────────────────────────

/// INT-32: Explicit `limit::N` produces list output.
///
/// With summary mode removed (task-019), `limit::N` keeps list mode (it was
/// already a list-mode-only parameter). The `Found N project(s):` header must
/// be present.
#[ test ]
fn int_32_explicit_limit_shows_list_output()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj32" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int32", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "limit::5" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ),          "limit::N must emit list-mode header; got:\n{s}" );
  assert!( !s.contains( "Active project" ), "summary block must be absent; got:\n{s}" );
}

// ─── INT-33 ───────────────────────────────────────────────────────────────────

/// INT-33: Family header format (conversations + agents).
#[ test ]
fn int_33_family_header_includes_conversations_and_agents()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj33" );
  fs::create_dir_all( &project ).unwrap();
  common::write_hierarchical_path_session(
    &storage_root, &project, "root-int33",
    &[ ( "a1", "Explore" ), ( "a2", "Explore" ), ( "a3", "general-purpose" ) ],
    2,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "conversation" ), "header must contain 'conversation(s)'; got:\n{s}" );
  assert!( s.contains( "agent" ),        "header must contain 'agent(s)'; got:\n{s}" );
  assert!( !s.contains( "+ " ),          "old collapse format must be absent; got:\n{s}" );
}

// ─── INT-34 ───────────────────────────────────────────────────────────────────

/// INT-34: Per-root agent breakdown [N agents: type summary].
#[ test ]
fn int_34_per_root_agent_breakdown_shows_type_summary()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj34" );
  fs::create_dir_all( &project ).unwrap();
  // 2×Explore + 1×general-purpose → [3 agents: 2×Explore, 1×general-purpose]
  common::write_hierarchical_path_session(
    &storage_root, &project, "root-int34",
    &[ ( "b1", "Explore" ), ( "b2", "Explore" ), ( "b3", "general-purpose" ) ],
    2,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "[3 agents:" ),      "must show '[3 agents:' bracket; got:\n{s}" );
  assert!( s.contains( "Explore" ),          "must show 'Explore' type; got:\n{s}" );
  assert!( s.contains( "general-purpose" ),  "must show 'general-purpose' type; got:\n{s}" );
}

// ─── INT-35 ───────────────────────────────────────────────────────────────────

/// INT-35: Hierarchical format detection — each root shows only its own agent count.
#[ test ]
fn int_35_hierarchical_format_each_root_shows_own_agent_count()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj35" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = common::write_hierarchical_path_session(
    &storage_root, &project, "root-35-alpha",
    &[ ( "c1", "Explore" ), ( "c2", "Explore" ) ],
    2,
  );
  // Second root with 1 agent under the same project
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-35-beta",
    &[ ( "c3", "Plan" ) ],
    2,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "[2 agents:" ), "root-35-alpha must show '[2 agents:'; got:\n{s}" );
  assert!( s.contains( "[1 agent:" ),  "root-35-beta must show '[1 agent:'; got:\n{s}" );
}

// ─── INT-36 ───────────────────────────────────────────────────────────────────

/// INT-36: Flat format detection — agents attributed to parent via `sessionId`.
#[ test ]
fn int_36_flat_format_agents_attributed_by_session_id()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj36" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_test_session( &storage_root, &encoded, "root-session-36", 2 );
  common::write_flat_agent_session( &storage_root, &encoded, "d1", "root-session-36", 2 );
  common::write_flat_agent_session( &storage_root, &encoded, "d2", "root-session-36", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "[2 agents:" ), "flat agents must be attributed to parent; got:\n{s}" );
}

// ─── INT-37 ───────────────────────────────────────────────────────────────────

/// INT-37: Orphan family display — root missing, `?` marker present.
#[ test ]
fn int_37_orphan_family_shows_question_mark()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj37" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  let orphan_id = "orphan-root-37";
  let subagents_dir = storage_root
    .join( "projects" )
    .join( &encoded )
    .join( orphan_id )
    .join( "subagents" );
  fs::create_dir_all( &subagents_dir ).unwrap();

  {
    use std::io::Write as _;
    let mut f = fs::File::create( subagents_dir.join( "agent-e37.jsonl" ) ).unwrap();
    writeln!(
      f,
      r#"{{"type":"user","uuid":"orphan-u37","parentUuid":null,"timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"{orphan_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"orphan"}}}}"#
    ).unwrap();
  }
  common::write_agent_meta_json( &subagents_dir, "e37", "Explore" );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( '?' ), "orphan family must show '?' marker; got:\n{s}" );
}

// ─── INT-38 ───────────────────────────────────────────────────────────────────

/// INT-38: Childless root — no bracket suffix on v1 line.
#[ test ]
fn int_38_childless_root_has_no_bracket_suffix()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj38" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "root-38", 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let session_line = s.lines().find( | l | l.contains( "root-38" ) );
  assert!( session_line.is_some(), "root-38 must appear in output; got:\n{s}" );
  assert!(
    !session_line.unwrap().contains( '[' ),
    "childless root must NOT have '[' bracket suffix; got:\n{s}"
  );
}

// ─── INT-39 ───────────────────────────────────────────────────────────────────

/// INT-39: `meta.json` `agentType` appears in breakdown.
#[ test ]
fn int_39_meta_json_agent_type_appears_in_breakdown()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj39" );
  fs::create_dir_all( &project ).unwrap();
  common::write_hierarchical_path_session(
    &storage_root, &project, "root-int39",
    &[ ( "f39", "Plan" ) ],
    2,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Plan" ), "meta.json agentType 'Plan' must appear in breakdown; got:\n{s}" );
}

// ─── INT-40 ───────────────────────────────────────────────────────────────────

/// INT-40: Empty/malformed `meta.json` fallback to "unknown".
#[ test ]
fn int_40_empty_meta_json_falls_back_to_unknown()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj40" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_test_session( &storage_root, &encoded, "root-int40", 2 );

  let subagents_dir = storage_root
    .join( "projects" ).join( &encoded ).join( "root-int40" ).join( "subagents" );
  fs::create_dir_all( &subagents_dir ).unwrap();

  {
    use std::io::Write as _;
    let mut f = fs::File::create( subagents_dir.join( "agent-g40.jsonl" ) ).unwrap();
    writeln!(
      f,
      r#"{{"type":"user","uuid":"g40-u1","parentUuid":null,"timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"root-int40","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"agent"}}}}"#
    ).unwrap();
  }
  // Empty (0-byte) meta.json → parse failure → falls back to "unknown"
  fs::File::create( subagents_dir.join( "agent-g40.meta.json" ) ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "unknown" ), "empty meta.json must show 'unknown' type; got:\n{s}" );
}

// ─── INT-41 ───────────────────────────────────────────────────────────────────

/// INT-41: v1 orphan shows `? (orphan)` label (bug-cc-c1).
#[ test ]
fn int_41_v1_orphan_shows_orphan_label()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj41" );
  fs::create_dir_all( &project ).unwrap();

  // Flat agent with a non-existent parent → orphan family.
  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_flat_agent_session( &storage_root, &encoded, "orphan-41", "no-such-root", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "? (orphan)" ),
    "v1 orphan line must show '? (orphan)' label per spec; got:\n{s}"
  );
}

// ─── INT-42 ───────────────────────────────────────────────────────────────────

/// INT-42: v2 root entry count singular — `(1 entry)` not `(1 entries)`.
#[ test ]
fn int_42_v2_root_entry_count_singular()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj42" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "root-singular-42", 1 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "(1 entry)" ),  "v2 root with 1 entry must show '(1 entry)'; got:\n{s}" );
  assert!( !s.contains( "(1 entries)" ), "must NOT show '(1 entries)'; got:\n{s}" );
}

// ─── INT-43 ───────────────────────────────────────────────────────────────────

/// INT-43: v2 agent entry count singular — `1 entry` not `1 entries`.
#[ test ]
fn int_43_v2_agent_entry_count_singular()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj43" );
  fs::create_dir_all( &project ).unwrap();
  common::write_hierarchical_path_session(
    &storage_root, &project, "root-int43",
    &[ ( "agent-s43", "Explore" ) ],
    1,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "1 entry" ),   "v2 agent with 1 entry must show '1 entry'; got:\n{s}" );
  assert!( !s.contains( "1 entries" ), "must NOT show '1 entries'; got:\n{s}" );
}

// ─── INT-41b ──────────────────────────────────────────────────────────────────

/// INT-41b: `verbosity::1` alone — LEGACY spec entry.
///
/// The spec said `verbosity::1` should stay in summary mode (fix for
/// bug-is-default-verbosity). Task-019 removed summary mode entirely; the
/// `is_default` gate that caused the bug was deleted. `verbosity::1` now
/// produces list output like any other explicit parameter.
#[ test ]
fn int_41b_verbosity_1_shows_list_output()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj41b" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-41b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // After task-019: verbosity::1 yields list output; no summary mode exists.
  assert!( s.contains( "Found" ),          "verbosity::1 must emit list-mode header; got:\n{s}" );
  assert!( !s.contains( "Active project" ), "summary block must be absent; got:\n{s}" );
}

// ─── INT-42b ──────────────────────────────────────────────────────────────────

/// INT-42b: Summary mode shows "Active project" header — LEGACY spec entry.
///
/// After task-019 the summary mode and its `Active project` header were removed.
/// Bare `clg .projects` emits `Found N project(s):` (list mode). This test
/// verifies the list-mode header is present and the legacy header is absent.
#[ test ]
fn int_42b_bare_projects_shows_found_header_not_active_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj42b" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-42b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ),          "bare .projects must emit list-mode header; got:\n{s}" );
  assert!( !s.contains( "Active project" ), "legacy 'Active project' header must be absent; got:\n{s}" );
}

// ─── INT-43b ──────────────────────────────────────────────────────────────────

/// INT-43b: Summary mode shows session count aggregate — LEGACY spec entry.
///
/// The spec said summary mode shows `(N sessions,` in the `Active project` line.
/// After task-019, list mode aggregates sessions across projects via
/// `Found N project(s):` plus grouped session listings. This test verifies
/// the list output is non-empty and contains session info when sessions exist.
#[ test ]
fn int_43b_list_output_contains_session_info()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj43b" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-43b-a", 2 );
  common::write_path_project_session( &storage_root, &project, "session-43b-b", 2 );
  common::write_path_project_session( &storage_root, &project, "session-43b-c", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ), "must emit list-mode header; got:\n{s}" );
  assert!(
    !s.is_empty(),
    "output must contain session info when 3 sessions exist; got empty stdout"
  );
}

// ─── INT-44 ───────────────────────────────────────────────────────────────────

/// INT-44: List mode shows projects sorted by recency (most recently active first).
#[ test ]
fn int_44_list_mode_projects_sorted_by_recency()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let proj_alpha = root.path().join( "proj_alpha" ); // alphabetically first, older
  let proj_beta  = root.path().join( "proj_beta" );  // alphabetically second, newer
  fs::create_dir_all( &proj_alpha ).unwrap();
  fs::create_dir_all( &proj_beta ).unwrap();

  let enc_alpha = common::write_path_project_session( &storage_root, &proj_alpha, "session-alpha", 2 );
  let enc_beta  = common::write_path_project_session( &storage_root, &proj_beta,  "session-beta",  2 );

  let old_t = std::time::SystemTime::UNIX_EPOCH + core::time::Duration::from_secs( 1_000 );
  let new_t = std::time::SystemTime::UNIX_EPOCH + core::time::Duration::from_secs( 2_000 );
  {
    let p = storage_root.join( "projects" ).join( &enc_alpha ).join( "session-alpha.jsonl" );
    let f = std::fs::OpenOptions::new().write( true ).open( &p ).unwrap();
    f.set_times( std::fs::FileTimes::new().set_modified( old_t ) ).unwrap();
  }
  {
    let p = storage_root.join( "projects" ).join( &enc_beta ).join( "session-beta.jsonl" );
    let f = std::fs::OpenOptions::new().write( true ).open( &p ).unwrap();
    f.set_times( std::fs::FileTimes::new().set_modified( new_t ) ).unwrap();
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let pos_beta  = s.find( "proj_beta" ).expect( "proj_beta must appear" );
  let pos_alpha = s.find( "proj_alpha" ).expect( "proj_alpha must appear" );
  assert!(
    pos_beta < pos_alpha,
    "proj_beta (newer) must appear before proj_alpha (older); got:\n{s}"
  );
}

// ─── INT-45 ───────────────────────────────────────────────────────────────────

/// INT-45: `verbosity::0` outputs project paths only — no session IDs.
#[ test ]
fn int_45_verbosity_0_shows_project_paths_only()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj_v0_45" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-v0-45", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "proj_v0_45" ),    "v0 must output project path; got:\n{s}" );
  assert!( !s.contains( "session-v0-45" ), "v0 must NOT output session IDs; got:\n{s}" );
}

// ─── INT-46 ───────────────────────────────────────────────────────────────────

/// INT-46: Topic path shown even when topic dir absent from disk.
#[ test ]
fn int_46_topic_path_shown_when_topic_dir_absent()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  fs::create_dir_all( &project ).unwrap();
  // Build storage key with `--commit` suffix. Do NOT create `-commit` dir on disk.
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode" );
  let topic_key = format!( "{encoded_base}--commit" );
  common::write_test_session( &storage_root, &topic_key, "session-int46-absent", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "/-commit" ),
    "topic '/-commit' must appear even when dir is absent from disk; got:\n{s}"
  );
}

// ─── INT-47 ───────────────────────────────────────────────────────────────────

/// INT-47: Topic path shown when topic dir present on disk.
#[ test ]
fn int_47_topic_path_shown_when_topic_dir_present()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project   = root.path().join( "myproject" );
  let topic_dir = project.join( "-commit" );
  fs::create_dir_all( &topic_dir ).unwrap();
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode" );
  let topic_key = format!( "{encoded_base}--commit" );
  common::write_test_session( &storage_root, &topic_key, "session-int47-present", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "/-commit" ),
    "topic '/-commit' must appear when dir is present on disk; got:\n{s}"
  );
}

// ─── INT-48 ───────────────────────────────────────────────────────────────────

/// INT-48: Default-topic path shown when topic dir absent from disk.
#[ test ]
fn int_48_default_topic_path_shown_when_absent()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  fs::create_dir_all( &project ).unwrap();
  // `--default-topic` suffix. Do NOT create `-default_topic` dir on disk.
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode" );
  let topic_key = format!( "{encoded_base}--default-topic" );
  common::write_test_session( &storage_root, &topic_key, "session-int48-absent-dt", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "/-default_topic" ),
    "topic '/-default_topic' must appear even when dir is absent; got:\n{s}"
  );
}

// ─── INT-49 ───────────────────────────────────────────────────────────────────

/// INT-49: Base path shown correctly with no topic suffix.
#[ test ]
fn int_49_base_path_shown_with_no_topic()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int49-no-topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int49-no-topic" ), "session must appear; got:\n{s}" );
  assert!( !s.contains( "/-commit" ),        "no topic — must not show /-commit; got:\n{s}" );
  assert!( !s.contains( "/-default_topic" ), "no topic — must not show /-default_topic; got:\n{s}" );
}

// ─── INT-50 ───────────────────────────────────────────────────────────────────

/// INT-50: Double-topic key shows both topic components unconditionally.
#[ test ]
fn int_50_double_topic_key_shows_both_components()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  fs::create_dir_all( &project ).unwrap();
  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode" );
  // Two `--` separators: both topic dirs absent from disk.
  let topic_key = format!( "{encoded_base}--default-topic--commit" );
  common::write_test_session( &storage_root, &topic_key, "session-int50-double", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "/-default_topic" ),
    "double-topic key must include first topic '/-default_topic'; got:\n{s}"
  );
  assert!(
    s.contains( "/-commit" ),
    "double-topic key must include second topic '/-commit'; got:\n{s}"
  );
}
