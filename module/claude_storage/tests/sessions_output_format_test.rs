//! Tests for `.sessions` command — output format.
//!
//! ## Coverage
//!
//! IT-17 through IT-29: output format behaviour — `verbosity::1` grouping,
//! agent session collapse, entry counts, filter interaction with collapse,
//! v1 entry count, limit truncation, and zero-byte session exclusion.
//!
//! ## Test Case Index
//!
//! | ID | Test Name | Category |
//! |----|-----------|----------|
//! | IT-17 | v1 output groups sessions under project path headers | Output Format |
//! | IT-18 | path header always present at v1 for scope::local single project | Output Format |
//! | IT-19 | agent sessions collapsed to count line at v1 without agent:: filter | Output Format |
//! | IT-20 | agent sessions shown individually at v2+ | Output Format |
//! | IT-21 | entry count shown per session at v2+ | Output Format |
//! | IT-22 | agent::1 explicit filter disables collapse at v1 | Output Format |
//! | IT-27 | entry count shown per session at v1 | Output Format |
//! | IT-28 | limit::N truncates main sessions shown at v1 | Output Format |
//! | IT-29 | zero-byte sessions excluded from v1 display | Output Format |

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

// ─────────────────────────────────────────────────────────────────────────────
// Output Format Redesign (plan-004)
//
// Root Cause: sessions_routine collected sessions into a flat Vec<(label, id)>
// and formatted project labels as format!("{:?}", project.id()) — opaque encoded
// strings. At scope::global with 60+ sessions there was no grouping, no readable
// paths, and no way to tell which sessions belonged to which project.
//
// Why Not Caught: All existing format tests only checked for presence of session
// IDs or "Found N" header. No test asserted path-group headers or agent collapse.
//
// Fix Applied: sessions_routine redesigned to collect into BTreeMap<String,
// Vec<Session>> keyed by decoded project path. Output loop emits path headers at
// v1+, collapses agent sessions at v1 when no agent:: filter, shows entry counts
// at v2+.
//
// Prevention: Always add format assertions (path header, agent collapse, entry
// count) when testing commands with structured grouped output.
//
// Pitfall: decode_path() requires input starting with '-'. UUID project dirs
// don't start with '-' — guard with starts_with('-') before calling decode.
// ─────────────────────────────────────────────────────────────────────────────

// IT-17: v1 output groups sessions under project path headers
#[test]
fn it_17_v1_groups_sessions_by_project_path()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project_a = root.path().join( "proj_alpha" );
  let project_b = root.path().join( "proj_beta" );
  common::write_path_project_session( &storage_root, &project_a, "session-alpha-001", 2 );
  common::write_path_project_session( &storage_root, &project_b, "session-beta-001", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // Must have at least one path header line: ends with ':' and contains '/' or '~'
  assert!(
    s.lines().any( | l | l.contains( ':' ) && ( l.contains( '/' ) || l.contains( '~' ) ) ),
    "v1 must show project path headers ending with ':'; got:\n{s}"
  );
  assert!( s.contains( "session-alpha-001" ), "must contain session-alpha-001; got:\n{s}" );
  assert!( s.contains( "session-beta-001" ), "must contain session-beta-001; got:\n{s}" );
}

// IT-18: path header always present at v1 for scope::local single project
#[test]
fn it_18_path_header_present_at_v1_single_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "my_proj" );
  common::write_path_project_session( &storage_root, &project, "session-path-test", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.lines().any( | l | l.contains( ':' ) && ( l.contains( '/' ) || l.contains( '~' ) ) ),
    "path header must be shown at v1 even for single-project local scope; got:\n{s}"
  );
}

// IT-19: agent sessions grouped in family display at v1 without agent:: filter
//
// Updated for family display (TSK-002): agents are shown as family brackets
// `[N agents: breakdown]` per root, not as a flat `+ N agent sessions` collapse.
// Flat agents without valid parent linkage become orphan families with `?` marker.
#[test]
fn it_19_agent_sessions_collapsed_at_v1_no_filter()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "mixed_project" );
  // 2 main sessions
  common::write_path_project_session( &storage_root, &project, "session-main-a", 2 );
  common::write_path_project_session( &storage_root, &project, "session-main-b", 2 );
  // 3 agent sessions (IDs start with "agent-" for is_agent_session() to return true)
  common::write_path_project_session( &storage_root, &project, "agent-task-001", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-002", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-003", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // Agent IDs must NOT appear individually at v1
  assert!( !s.contains( "agent-task-001" ), "agent-task-001 must NOT appear individually at v1; got:\n{s}" );
  assert!( !s.contains( "agent-task-002" ), "agent-task-002 must NOT appear individually at v1; got:\n{s}" );
  assert!( !s.contains( "agent-task-003" ), "agent-task-003 must NOT appear individually at v1; got:\n{s}" );
  // Family display: agents must appear as count in brackets or as orphan marker
  assert!(
    s.contains( "agent" ),
    "must show agent info in family display at v1; got:\n{s}"
  );
  // Old collapse format must NOT appear
  assert!(
    !s.contains( "+ " ),
    "must NOT show old '+ N agent' collapse line; got:\n{s}"
  );
  // Main sessions must still appear individually
  assert!( s.contains( "session-main-a" ), "session-main-a must appear; got:\n{s}" );
  assert!( s.contains( "session-main-b" ), "session-main-b must appear; got:\n{s}" );
}

// IT-20: agent sessions shown individually at v2+
#[test]
fn it_20_agent_sessions_shown_individually_at_v2()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "mixed_project_v2" );
  common::write_path_project_session( &storage_root, &project, "session-main-a", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-001", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-002", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-003", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // All agent sessions must appear individually (tree-indented at v2+)
  assert!( s.contains( "agent-task-001" ), "agent-task-001 must appear individually at v2; got:\n{s}" );
  assert!( s.contains( "agent-task-002" ), "agent-task-002 must appear individually at v2; got:\n{s}" );
  assert!( s.contains( "agent-task-003" ), "agent-task-003 must appear individually at v2; got:\n{s}" );
  // No old-format collapse line
  assert!(
    !s.contains( "+ " ),
    "must NOT show old '+ N agent' collapse at v2; got:\n{s}"
  );
  // Must have path header (grouped output)
  assert!(
    s.lines().any( | l | l.contains( ':' ) && ( l.contains( '/' ) || l.contains( '~' ) ) ),
    "v2 must show project path header ending with ':'; got:\n{s}"
  );
}

// IT-21: entry count shown per session at v2+
#[test]
fn it_21_entry_count_shown_at_v2()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "count_proj" );
  // Exactly 4 entries
  common::write_path_project_session( &storage_root, &project, "session-count-test", 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "(4 entries)" ),
    "v2 must show '(4 entries)' for a 4-entry session; got:\n{s}"
  );
}

// IT-22: agent::1 explicit filter disables collapse at v1
#[test]
fn it_22_agent_filter_disables_collapse_at_v1()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "agent_filter_proj" );
  common::write_path_project_session( &storage_root, &project, "session-main-z", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-001", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-002", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-003", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .arg( "agent::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // agent::1 shows only agent sessions individually, no collapse
  assert!( s.contains( "agent-task-001" ), "agent-task-001 must appear individually with agent::1; got:\n{s}" );
  assert!( s.contains( "agent-task-002" ), "agent-task-002 must appear individually with agent::1; got:\n{s}" );
  assert!( s.contains( "agent-task-003" ), "agent-task-003 must appear individually with agent::1; got:\n{s}" );
  assert!(
    !s.contains( "3 agent" ),
    "must NOT show collapse line when agent::1 is set; got:\n{s}"
  );
  // Must have path header (grouped output)
  assert!(
    s.lines().any( | l | l.contains( ':' ) && ( l.contains( '/' ) || l.contains( '~' ) ) ),
    "agent::1 v1 must show project path header ending with ':'; got:\n{s}"
  );
}

// IT-27: entry count shown per session at v1
//
// Root Cause: Entry counts were only shown at v2+; v1 used bare session IDs
// with no metadata. Users needed v2 just to see how many entries a session had.
//
// Why Not Caught: No test verified v1 output contained entry counts.
//
// Fix Applied: v1 now shows `  - {short_id}  {mtime}  ({n} entries)` for each
// main session (mirrors v2 info density at the default verbosity).
//
// Prevention: Always add a v1 entry-count assertion when adding entry count to
// lower verbosity levels.
//
// Pitfall: Synthetic test IDs are not UUID-format (len != 36), so short_id
// returns them intact — assertions against full IDs still pass at v1.
#[test]
fn it_27_entry_count_shown_at_v1()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "count_v1_proj" );
  // Exactly 4 entries
  common::write_path_project_session( &storage_root, &project, "session-v1-count", 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "(4 entries)" ),
    "v1 must show '(4 entries)' for a 4-entry session; got:\n{s}"
  );
}

// IT-28: limit::N truncates main sessions shown at v1
//
// Root Cause: No per-project display limit existed; large projects (100+ sessions)
// flooded output at default verbosity.
//
// Why Not Caught: No test exercised the limit:: parameter before it was added.
//
// Fix Applied: limit:: parameter caps the number of main sessions displayed per
// project at v1; a trailing "... and N more" hint points to verbosity::0.
//
// Prevention: Always add a truncation assertion when adding a limit parameter
// to any list command.
//
// Pitfall: limit::0 means unlimited (not zero sessions). Only positive values
// activate truncation.
#[test]
fn it_28_limit_truncates_display()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "limit_proj" );
  // 5 main sessions
  common::write_path_project_session( &storage_root, &project, "session-limit-a", 2 );
  common::write_path_project_session( &storage_root, &project, "session-limit-b", 2 );
  common::write_path_project_session( &storage_root, &project, "session-limit-c", 2 );
  common::write_path_project_session( &storage_root, &project, "session-limit-d", 2 );
  common::write_path_project_session( &storage_root, &project, "session-limit-e", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .arg( "limit::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // Truncation hint must appear
  assert!(
    s.contains( "and 3 more" ),
    "limit::2 with 5 sessions must show '... and 3 more'; got:\n{s}"
  );
}

// IT-29: zero-byte sessions excluded from v1 display
//
// Root Cause: Claude Code creates zero-byte JSONL files as startup placeholders
// (B8). These aren't real sessions but appeared in v1 output as empty entries.
//
// Why Not Caught: No test created zero-byte session files to verify exclusion.
//
// Fix Applied: v1 loop filters out zero-byte main sessions before display.
// Zero-byte files remain visible at v0 (pipe-safe) and v2+ (where showing
// "(0 entries)" can be informative).
//
// Prevention: Any test that cares about displayed session count must account for
// whether zero-byte sessions exist in the storage fixture.
//
// Pitfall: fs::metadata().len() == 0 is the check — do not rely on entry count
// because count_entries() reads the file and returns Ok(0) for a valid empty
// file, same as a zero-byte file.
#[test]
fn it_29_zero_byte_sessions_excluded_at_v1()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project = root.path().join( "zero_byte_proj" );
  common::write_path_project_session( &storage_root, &project, "session-real", 2 );

  // Create a zero-byte placeholder session (B8 behaviour)
  {
    let encoded = claude_storage_core::encode_path( &project ).unwrap();
    let dir = storage_root.join( "projects" ).join( &encoded );
    std::fs::create_dir_all( &dir ).unwrap();
    // File::create with no writes leaves the file at zero bytes
    let _ = std::fs::File::create( dir.join( "session-placeholder.jsonl" ) ).unwrap();
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // Zero-byte placeholder must NOT appear at v1
  assert!(
    !s.contains( "session-placeholder" ),
    "zero-byte placeholder must NOT appear at v1; got:\n{s}"
  );
  // Real session must still appear
  assert!(
    s.contains( "session-real" ),
    "real session must still appear when zero-byte is excluded; got:\n{s}"
  );
}
