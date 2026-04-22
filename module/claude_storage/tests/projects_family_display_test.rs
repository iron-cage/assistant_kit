//! Tests for `.projects` family and agent session display.
//!
//! ## Coverage
//!
//! IT-1, IT-33: Default list mode and zero-session header (post-summary-removal).
//! IT-36..IT-48: Family/agent tree display at verbosity 1 and 2.
//!
//! | ID    | What it covers                                                   |
//! |-------|------------------------------------------------------------------|
//! | IT-1  | Default (no args) uses list mode, not summary                    |
//! | IT-33 | Empty storage shows "Found 0 projects:" header                   |
//! | IT-36 | Family header format — `conversation` + `agent`                  |
//! | IT-37 | Per-root agent type breakdown bracket                            |
//! | IT-38 | Hierarchical format detection (subagents/ layout)                |
//! | IT-39 | Flat format detection (sessionId-based parent link)              |
//! | IT-40 | Orphan agent display with `?` marker                             |
//! | IT-41 | Childless root has no bracket suffix                             |
//! | IT-42 | meta.json agent type propagation                                 |
//! | IT-43 | Empty/missing meta.json falls back to `unknown`                  |
//! | IT-44 | v1 orphan line shows `? (orphan)` label                          |
//! | IT-45 | v2 root entry count singular — `(1 entry)` not `(1 entries)`    |
//! | IT-46 | v2 agent entry count singular — `1 entry` not `1 entries`       |
//! | IT-47 | Empty-string `agentType` (`""`) falls back to `unknown`          |
//! | IT-48 | Whitespace-only `agentType` (`"  "`) falls back to `unknown`     |

mod common;

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

// ────────────────────────────────────────────────────────────────────────────
// IT-1 / T01: Default (no args) shows list output, not a single-project summary
// ────────────────────────────────────────────────────────────────────────────

/// IT-1: Default (no args) outputs list-mode `Found N project(s):` format.
///
/// ## Root Cause (tested behaviour)
/// After summary-mode removal (task-019), bare `clg .projects` uses the default
/// `scope::around` in list mode. There is no longer a single-project summary path.
///
/// ## Verification
/// - stdout contains `Found` (list-mode header)
/// - stdout does NOT contain `Active project` (summary block absent)
#[test]
fn it1_default_shows_list_output()
{
  let root = tempfile::TempDir::new().unwrap();
  let project_path = root.path().join( "myproject" );
  std::fs::create_dir_all( &project_path ).unwrap();

  common::write_path_project_session_with_last_message(
    root.path(), &project_path, "session-it1", 2, "Hello from last entry"
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", root.path().to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ),          "bare .projects must output 'Found N project(s):' format; got:\n{s}" );
  assert!( !s.contains( "Active project" ), "bare .projects must not output summary block; got:\n{s}" );
}


// ────────────────────────────────────────────────────────────────────────────
// IT-33: Empty scope → "Found 0 projects:"
// ────────────────────────────────────────────────────────────────────────────

/// IT-33: Empty storage → list-mode zero-result header.
///
/// ## Root Cause (tested behaviour)
/// When no sessions exist under the cwd, list mode must emit `Found 0 projects:`
/// rather than empty output or an error. Summary mode was removed in task-019;
/// the zero-result path now always uses the list-mode header.
///
/// ## Verification
/// - exit code is 0
/// - stdout contains "Found 0 projects:"
/// - stderr is empty
#[test]
fn it33_no_sessions_shows_zero_result_header()
{
  let root = tempfile::TempDir::new().unwrap();
  let project_path = root.path().join( "empty_proj" );
  std::fs::create_dir_all( &project_path ).unwrap();
  // No session files written — storage is empty.

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", root.path().to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Found 0 projects:" ),
    "empty storage must produce list-mode zero-result header; got:\n{s}"
  );
  assert!( stderr( &out ).is_empty(), "stderr must be empty; got: {}", stderr( &out ) );
}


// ────────────────────────────────────────────────────────────────────────────
// Family Display Tests (IT-36 through IT-43)
// ────────────────────────────────────────────────────────────────────────────

/// IT-36: Family header format — `(N conversations, M agents)` replaces `(N sessions)`
///
/// ## Root Cause (tested behaviour)
/// When a project contains both root sessions and agent sessions, the v1 project
/// header must show `(N conversations, M agents)` instead of `(N sessions)`.
///
/// ## Verification
/// - stdout contains `conversations`
/// - stdout contains `agents`
/// - stdout does NOT contain `+ ` agent collapse line
#[test]
fn it36_family_header_format()
{
  let root = tempfile::TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "family_proj_36" );
  std::fs::create_dir_all( &project ).unwrap();

  // 1 root + 3 hierarchical agents
  common::write_hierarchical_path_session(
    &storage_root, &project, "root-session-36",
    &[ ( "a1", "Explore" ), ( "a2", "Explore" ), ( "a3", "general-purpose" ) ],
    2,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "conversation" ),
    "header must contain 'conversation(s)'; got:\n{s}"
  );
  assert!(
    s.contains( "agent" ),
    "header must contain 'agent(s)'; got:\n{s}"
  );
  assert!(
    !s.contains( "+ " ),
    "must NOT contain old '+ N agent' collapse line; got:\n{s}"
  );
}

/// IT-37: Per-root agent breakdown `[N agents: N×Type, …]`
///
/// ## Root Cause (tested behaviour)
/// Each root session line at v1 includes an inline bracket suffix showing agent
/// count and type distribution for that family.
///
/// ## Verification
/// - stdout contains `[3 agents:`
/// - stdout contains `Explore`
/// - stdout contains `general-purpose`
#[test]
fn it37_per_root_agent_breakdown()
{
  let root = tempfile::TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "family_proj_37" );
  std::fs::create_dir_all( &project ).unwrap();

  common::write_hierarchical_path_session(
    &storage_root, &project, "root-session-37",
    &[ ( "b1", "Explore" ), ( "b2", "Explore" ), ( "b3", "general-purpose" ) ],
    2,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "[3 agents:" ),
    "must show '[3 agents:' bracket breakdown; got:\n{s}"
  );
  assert!(
    s.contains( "Explore" ),
    "must show 'Explore' type in breakdown; got:\n{s}"
  );
  assert!(
    s.contains( "general-purpose" ),
    "must show 'general-purpose' type in breakdown; got:\n{s}"
  );
}

/// IT-38: Hierarchical format detection — agents attributed to correct parent
///
/// ## Root Cause (tested behaviour)
/// With two root sessions each having distinct agents in their own
/// `{uuid}/subagents/` directory, each root line shows only its own agent count.
///
/// ## Verification
/// - Each root session line has a distinct `[N agents:` count
#[test]
fn it38_hierarchical_format_detection()
{
  let root = tempfile::TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "family_proj_38" );
  std::fs::create_dir_all( &project ).unwrap();

  let encoded = common::write_hierarchical_path_session(
    &storage_root, &project, "root-38-alpha",
    &[ ( "c1", "Explore" ), ( "c2", "Explore" ) ],
    2,
  );

  // Second root with 1 agent
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-38-beta",
    &[ ( "c3", "Plan" ) ],
    2,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // One root should show [2 agents:, the other [1 agent:
  assert!(
    s.contains( "[2 agents:" ),
    "root-38-alpha must show '[2 agents:'; got:\n{s}"
  );
  assert!(
    s.contains( "[1 agent:" ),
    "root-38-beta must show '[1 agent:'; got:\n{s}"
  );
}

/// IT-39: Flat format detection — agents grouped by `sessionId` linkage
///
/// ## Root Cause (tested behaviour)
/// Flat-format agents (`agent-*.jsonl` at project root) are grouped by their
/// `sessionId` field to the correct parent session.
///
/// ## Verification
/// - Root session line contains `[2 agents:`
#[test]
fn it39_flat_format_detection()
{
  let root = tempfile::TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "family_proj_39" );
  std::fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project )
    .expect( "encode project path" );

  // Root session
  common::write_test_session( &storage_root, &encoded, "root-session-39", 2 );

  // 2 flat agents linked to root
  common::write_flat_agent_session(
    &storage_root, &encoded, "d1", "root-session-39", 2,
  );
  common::write_flat_agent_session(
    &storage_root, &encoded, "d2", "root-session-39", 2,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "[2 agents:" ),
    "flat agents must be attributed to parent via sessionId; got:\n{s}"
  );
}

/// IT-40: Orphan family display — root missing, `?` marker present
///
/// ## Root Cause (tested behaviour)
/// When agent sessions exist in `{uuid}/subagents/` but no matching root
/// `.jsonl` file exists, the display shows a `?` marker.
///
/// ## Verification
/// - stdout contains `?`
#[test]
fn it40_orphan_family_display()
{
  let root = tempfile::TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "family_proj_40" );
  std::fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project )
    .expect( "encode project path" );

  // Create subagents directory for a non-existent root
  let orphan_uuid = "orphan-root-40";
  let subagents_dir = storage_root
    .join( "projects" )
    .join( &encoded )
    .join( orphan_uuid )
    .join( "subagents" );
  std::fs::create_dir_all( &subagents_dir ).unwrap();

  // Write agent session manually in the subagents dir
  {
    use std::io::Write as _;
    let agent_path = subagents_dir.join( "agent-e1.jsonl" );
    let mut f = std::fs::File::create( &agent_path ).unwrap();
    writeln!(
      f,
      r#"{{"type":"user","uuid":"orphan-u1","parentUuid":null,"timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"{orphan_uuid}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"orphan agent"}}}}"#
    ).unwrap();
  }
  common::write_agent_meta_json( &subagents_dir, "e1", "Explore" );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( '?' ),
    "orphan family must show '?' marker; got:\n{s}"
  );
}

/// IT-41: Childless root — no bracket suffix on v1 line
///
/// ## Root Cause (tested behaviour)
/// A root session with zero agents should NOT display a `[` bracket suffix.
///
/// ## Verification
/// - The root session line does NOT contain `[`
#[test]
fn it41_childless_root_no_bracket()
{
  let root = tempfile::TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "family_proj_41" );
  std::fs::create_dir_all( &project ).unwrap();

  // Root with 0 agents
  common::write_path_project_session( &storage_root, &project, "root-session-41", 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // Find the session line and check it has no bracket
  let session_line = s.lines().find( | l | l.contains( "root-session-41" ) );
  assert!(
    session_line.is_some(),
    "root-session-41 must appear in output; got:\n{s}"
  );
  assert!(
    !session_line.unwrap().contains( '[' ),
    "childless root must NOT have '[' bracket suffix; got:\n{s}"
  );
}

/// IT-42: Meta.json `agentType` in breakdown
///
/// ## Root Cause (tested behaviour)
/// The agent type from `meta.json` appears in the family breakdown string.
///
/// ## Verification
/// - stdout contains `Plan`
#[test]
fn it42_meta_json_agent_type()
{
  let root = tempfile::TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "family_proj_42" );
  std::fs::create_dir_all( &project ).unwrap();

  common::write_hierarchical_path_session(
    &storage_root, &project, "root-session-42",
    &[ ( "f1", "Plan" ) ],
    2,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Plan" ),
    "meta.json agentType 'Plan' must appear in breakdown; got:\n{s}"
  );
}

/// IT-43: Empty/malformed meta.json fallback to "unknown"
///
/// ## Root Cause (tested behaviour)
/// When `meta.json` exists but is empty (0 bytes), the agent type falls
/// back to "unknown" in the breakdown.
///
/// ## Verification
/// - stdout contains `unknown`
#[test]
fn it43_empty_meta_json_fallback()
{
  let root = tempfile::TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "family_proj_43" );
  std::fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project )
    .expect( "encode project path" );

  // Root session
  common::write_test_session( &storage_root, &encoded, "root-session-43", 2 );

  // Create subagents dir + agent with empty meta.json
  let subagents_dir = storage_root
    .join( "projects" )
    .join( &encoded )
    .join( "root-session-43" )
    .join( "subagents" );
  std::fs::create_dir_all( &subagents_dir ).unwrap();

  {
    use std::io::Write as _;
    let agent_path = subagents_dir.join( "agent-g1.jsonl" );
    let mut f = std::fs::File::create( &agent_path ).unwrap();
    writeln!(
      f,
      r#"{{"type":"user","uuid":"empty-meta-u1","parentUuid":null,"timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"root-session-43","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"empty meta agent"}}}}"#
    ).unwrap();
  }

  // Create empty (0-byte) meta.json
  std::fs::File::create( subagents_dir.join( "agent-g1.meta.json" ) ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "unknown" ),
    "empty meta.json must show 'unknown' type; got:\n{s}"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// Bug reproducer tests — discovered via manual testing (CC-C1, CC-E1 singular)
// ────────────────────────────────────────────────────────────────────────────

/// IT-44: v1 orphan line shows `? (orphan)` label
///
/// ## Root Cause (tested behaviour)
/// Spec (`docs/cli/commands.md`): `  ? (orphan)  [N agents: breakdown]`
/// Actual (before fix): `  ?  [N agents: breakdown]` — the `(orphan)` label was omitted.
///
/// ## Why Not Caught
/// IT-40 asserted `s.contains('?')` — the `?` marker was present. The test
/// didn't assert the presence of the `(orphan)` label text specifically.
///
/// ## Fix Applied
/// `render_families_v1`: orphan else-branch changed from `"  ?{bracket}"` to
/// `"  ? (orphan){bracket}"`.
///
/// ## Prevention
/// Assert the exact label token `(orphan)` whenever an orphan family exists.
///
/// ## Pitfall
/// Testing only `?` presence passes even when the descriptive label is absent.
// test_kind: bug_reproducer(issue-cc-c1)
#[test]
fn it44_v1_orphan_shows_orphan_label()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "orphan_label_proj_44" );
  std::fs::create_dir_all( &project ).unwrap();

  // Flat agent with non-existent parent — becomes orphan
  common::write_flat_agent_session( &storage_root, &{
    claude_storage_core::encode_path( &project ).expect( "encode" )
  }, "orphan-001", "no-such-root-xyz", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
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

/// IT-45: v2 root entry count singular — `(1 entry)` not `(1 entries)`
///
/// ## Root Cause (tested behaviour)
/// `render_families_v2` used `format!("  ({n} entries)")` unconditionally.
/// When n=1, English grammar requires `"(1 entry)"` not `"(1 entries)"`.
///
/// ## Why Not Caught
/// Existing v2 tests used sessions with multiple entries (≥2) where plural is correct.
///
/// ## Fix Applied
/// `render_families_v2`: entry count string uses `if n == 1 { "entry" } else { "entries" }`.
///
/// ## Prevention
/// Always include a 1-entry case when testing count-based display strings.
///
/// ## Pitfall
/// Plural-only tests pass even when singular is grammatically wrong.
// test_kind: bug_reproducer(issue-cc-singular-v2-root)
#[test]
fn it45_v2_root_entry_count_singular()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "singular_v2_root_45" );
  common::write_path_project_session( &storage_root, &project, "root-singular-45", 1 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "(1 entry)" ),
    "v2 root with 1 entry must show '(1 entry)' not '(1 entries)'; got:\n{s}"
  );
  assert!(
    !s.contains( "(1 entries)" ),
    "v2 root must NOT show '(1 entries)'; got:\n{s}"
  );
}

/// IT-46: v2 agent entry count singular — `1 entry` not `1 entries`
///
/// ## Root Cause (tested behaviour)
/// `render_families_v2` agent loop used `format!("  {n} entries")` unconditionally.
/// When n=1, English grammar requires `"1 entry"` not `"1 entries"`.
///
/// ## Why Not Caught
/// Existing v2 agent tests used agents with multiple entries (≥2).
///
/// ## Fix Applied
/// `render_families_v2`: agent entry count string uses singular noun when n=1.
///
/// ## Prevention
/// Test agent display with exactly 1 entry when count is shown.
///
/// ## Pitfall
/// Multi-entry tests pass even when singular-entry display is grammatically wrong.
// test_kind: bug_reproducer(issue-cc-singular-v2-agent)
#[test]
fn it46_v2_agent_entry_count_singular()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "singular_v2_agent_46" );
  std::fs::create_dir_all( &project ).unwrap();

  // Hierarchical agent with 1 entry
  common::write_hierarchical_path_session(
    &storage_root,
    &project,
    "root-singular-46",
    &[ ( "agent-s46a", "Explore" ) ],
    1,
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "1 entry" ),
    "v2 agent with 1 entry must show '1 entry' not '1 entries'; got:\n{s}"
  );
  assert!(
    !s.contains( "1 entries" ),
    "v2 agent must NOT show '1 entries'; got:\n{s}"
  );
}

/// IT-47: Empty-string `agentType` in meta.json falls back to "unknown"
///
/// ## Root Cause (tested behaviour)
/// `parse_agent_meta` used `.unwrap_or("unknown")` after `.as_str()`.
/// When `agentType` is `""`, `as_str()` returns `Some("")` — not `None` —
/// so `unwrap_or` never triggers. The breakdown displayed `1×` (empty label).
///
/// ## Why Not Caught
/// IT-43 tested a 0-byte meta.json file (parse failure → unknown).
/// No test used `{"agentType":""}` where the key exists with an empty value.
///
/// ## Fix Applied
/// Added `.filter( | s | !s.is_empty() )` before `.unwrap_or( "unknown" )`
/// so both missing keys and empty-string values resolve to "unknown".
///
/// ## Prevention
/// Test the empty-string variant explicitly, separately from the missing-key
/// and missing-file variants.
///
/// ## Pitfall
/// `unwrap_or` only catches `None`; it cannot replace an empty `Some("")`.
// test_kind: bug_reproducer(issue-mt-empty-agenttype)
#[test]
fn it47_empty_string_agent_type_falls_back_to_unknown()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "meta_empty_str_47" );
  std::fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project )
    .expect( "encode project path" );

  // Root session
  common::write_test_session( &storage_root, &encoded, "root-empty-47", 2 );

  // Flat agent file: sessionId points to root
  common::write_flat_agent_session(
    &storage_root,
    &encoded,
    "empty-type-47",
    "root-empty-47",
    2,
  );

  // Overwrite meta.json with empty-string agentType
  let meta_path = storage_root
    .join( "projects" )
    .join( &encoded )
    .join( "agent-empty-type-47.meta.json" );
  std::fs::write( &meta_path, r#"{"agentType":""}"# ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "unknown" ),
    "agentType empty string must display as 'unknown'; got:\n{s}"
  );
  assert!(
    !s.contains( "1×]" ),
    "agentType empty string must NOT render as bare '1×]'; got:\n{s}"
  );
}

/// IT-48: Whitespace-only `agentType` in meta.json falls back to "unknown"
///
/// ## Root Cause (tested behaviour)
/// Same root cause as IT-47: `as_str()` returns `Some("  ")` for whitespace-only
/// values. Without a `.trim()` check, `unwrap_or("unknown")` never fires and
/// the breakdown displays `1×  ` (whitespace label), which is invisible noise.
///
/// ## Why Not Caught
/// IT-47 only covered the empty-string case `""`. Whitespace `"  "` is a
/// distinct input that also needs explicit coverage.
///
/// ## Fix Applied
/// Filter checks `!s.trim().is_empty()` — catches both `""` and `"  "`.
///
/// ## Prevention
/// Include whitespace-only inputs in meta.json edge-case tests.
///
/// ## Pitfall
/// Visual inspection misses whitespace labels because they render as blank space.
// test_kind: bug_reproducer(issue-mt-whitespace-agenttype)
#[test]
fn it48_whitespace_agent_type_falls_back_to_unknown()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "meta_ws_str_48" );
  std::fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project )
    .expect( "encode project path" );

  common::write_test_session( &storage_root, &encoded, "root-ws-48", 2 );

  common::write_flat_agent_session(
    &storage_root,
    &encoded,
    "ws-type-48",
    "root-ws-48",
    2,
  );

  // Overwrite meta.json with whitespace-only agentType
  let meta_path = storage_root
    .join( "projects" )
    .join( &encoded )
    .join( "agent-ws-type-48.meta.json" );
  std::fs::write( &meta_path, r#"{"agentType":"  "}"# ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "unknown" ),
    "whitespace agentType must display as 'unknown'; got:\n{s}"
  );
  assert!(
    !s.contains( "×  " ),
    "whitespace agentType must NOT render as '×  ' (whitespace label); got:\n{s}"
  );
}
