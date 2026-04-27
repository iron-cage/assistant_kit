//! Behavioral scope semantics tests for `.projects` command.
//!
//! ## Coverage
//!
//! Tests that verify which sessions are included or excluded based on the
//! `scope::` parameter value, including:
//! - Basic scope semantics: `local`, `under`, `relevant`, `global`
//! - Underscore path encoding: IT-9..IT-13 (`bug_reproducer(issue-024)`)
//! - UUID project scope exclusion
//! - Topic-scoped directory matching (`scope::local` + `--{topic}` suffix)
//!
//! ## Scope Semantics
//!
//! | Value     | Project qualifies when…                              |
//! |-----------|------------------------------------------------------|
//! | `local`   | project path == base path (exact)                    |
//! | `relevant`| base path `starts_with` project path (ancestors)     |
//! | `under`   | project path `starts_with` base path (subtree)       |
//! | `global`  | always (entire storage)                              |
//!
//! ## Related Files
//!
//! - `projects_edge_case_test.rs` — EC-1..EC-9 parameter acceptance/rejection
//! - `projects_command_test.rs` — filter/validation/output formatting tests
//! - `projects_path_encoding_test.rs` — path decode/display bug reproducers (IT-23..IT-26)
//! - `projects_scope_around_test.rs` — `scope::around` neighborhood semantics (IT-57..IT-59)

mod common;

use tempfile::TempDir;

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

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
// Behavioural: scope::local returns only sessions for the exact matching project
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn local_scope_returns_only_matching_project_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project_a = root.path().join( "proja" );
  let project_b = root.path().join( "projb" );
  common::write_path_project_session( &storage_root, &project_a, "session-local-a", 2 );
  common::write_path_project_session( &storage_root, &project_b, "session-local-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project_a.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-local-a" ),  "must contain session-local-a; got:\n{s}" );
  assert!( !s.contains( "session-local-b" ), "must NOT contain session-local-b; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Behavioural: scope::under returns sessions from all projects in subtree
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn under_scope_returns_all_projects_in_subtree()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let workspace   = root.path().join( "workspace" );
  let project_a   = workspace.join( "a" );
  let project_b   = workspace.join( "b" );
  let outside     = root.path().join( "outside" );

  common::write_path_project_session( &storage_root, &project_a, "session-under-a",   2 );
  common::write_path_project_session( &storage_root, &project_b, "session-under-b",   2 );
  common::write_path_project_session( &storage_root, &outside,   "session-under-out", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", workspace.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-under-a" ),    "must contain session-under-a; got:\n{s}" );
  assert!( s.contains( "session-under-b" ),    "must contain session-under-b; got:\n{s}" );
  assert!( !s.contains( "session-under-out" ), "must NOT contain session-under-out; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Behavioural: scope::relevant returns sessions from ancestor projects
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn relevant_scope_includes_ancestor_projects()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let cwd         = root.path().join( "workspace" ).join( "proj" ).join( "src" );
  let parent      = root.path().join( "workspace" ).join( "proj" );
  let grandparent = root.path().join( "workspace" );
  let unrelated   = root.path().join( "other" );

  common::write_path_project_session( &storage_root, &cwd,         "session-rel-cwd",    2 );
  common::write_path_project_session( &storage_root, &parent,      "session-rel-parent", 2 );
  common::write_path_project_session( &storage_root, &grandparent, "session-rel-grand",  2 );
  common::write_path_project_session( &storage_root, &unrelated,   "session-rel-other",  2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", cwd.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-rel-cwd" ),    "must contain session-rel-cwd; got:\n{s}" );
  assert!( s.contains( "session-rel-parent" ), "must contain session-rel-parent; got:\n{s}" );
  assert!( s.contains( "session-rel-grand" ),  "must contain session-rel-grand; got:\n{s}" );
  assert!( !s.contains( "session-rel-other" ),  "must NOT contain session-rel-other; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Behavioural: scope::global returns sessions from all projects
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn global_scope_returns_all_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let proj_a = root.path().join( "a" );
  let proj_b = root.path().join( "b" ).join( "deep" );

  common::write_path_project_session( &storage_root, &proj_a, "session-glob-a", 2 );
  common::write_path_project_session( &storage_root, &proj_b, "session-glob-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-glob-a" ), "must contain session-glob-a; got:\n{s}" );
  assert!( s.contains( "session-glob-b" ), "must contain session-glob-b; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// issue-024 regression: scope::local/relevant/under return 0 results when the
// base path contains underscores.
//
// Root Cause: encode_path() maps both '_' and '/' to '-', so `wip_core` →
// `wip-core`. decode_component() defaults unknown pairs to '/', so `wip-core`
// → `wip/core`. The old code compared decoded project paths against the real
// base_path, and `wip/core ≠ wip_core`, so all projects silently fell through.
//
// Why Not Caught: All existing tests used alphanumeric path components only
// (e.g. "proj", "proja", "workspace/a"). No test exercised underscore
// components before this fix.
//
// Fix Applied: Encode base_path once with encode_path() and compare the
// resulting encoded string directly against each project's raw storage
// directory name. Encoding is deterministic: no decode step needed.
//
// Prevention: Always include at least one test with underscore components when
// testing path-scope logic.
//
// Pitfall: Paths with underscores and paths with an extra directory component
// encode identically (e.g. `wip_core` → `wip-core`, `wip/core` → `wip-core`).
// For scope::under this was resolved in issue-031 (TSK-060): a two-stage predicate
// uses decode_path_via_fs + Path::starts_with (component-wise) to correctly
// exclude sibling `wip_core_extra` when base is `wip_core`. See it_25.
// ─────────────────────────────────────────────────────────────────────────────

// IT-9: scope::local finds project when path contains underscores
//
// bug_reproducer(issue-024)
#[test]
fn scope_local_finds_project_with_underscore_path()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project_path = root.path().join( "wip_core" );
  let unrelated    = root.path().join( "other" );
  common::write_path_project_session( &storage_root, &project_path, "session-local-underscore", 2 );
  common::write_path_project_session( &storage_root, &unrelated,    "session-unrelated",         2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project_path.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-local-underscore" ),
    "scope::local must find underscore-path project; got:\n{s}"
  );
  assert!(
    !s.contains( "session-unrelated" ),
    "scope::local must exclude unrelated project; got:\n{s}"
  );
}

// IT-10: scope::under finds projects in subtree when base path has underscores
//
// bug_reproducer(issue-024)
#[test]
fn scope_under_finds_project_with_underscore_path()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let base          = root.path().join( "wip_core" );
  let child         = base.join( "child" );
  let outside       = root.path().join( "other" );
  common::write_path_project_session( &storage_root, &base,    "session-under-base",    2 );
  common::write_path_project_session( &storage_root, &child,   "session-under-child",   2 );
  common::write_path_project_session( &storage_root, &outside, "session-under-outside", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-under-base" ),
    "scope::under must include base project itself; got:\n{s}"
  );
  assert!(
    s.contains( "session-under-child" ),
    "scope::under must include child of underscore-path base; got:\n{s}"
  );
  assert!(
    !s.contains( "session-under-outside" ),
    "scope::under must exclude projects outside base subtree; got:\n{s}"
  );
}

// IT-11: scope::relevant finds ancestor when ancestor path has underscores
//
// bug_reproducer(issue-024)
#[test]
fn scope_relevant_finds_ancestor_with_underscore_path()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let ancestor  = root.path().join( "wip_core" );
  let cwd       = ancestor.join( "sub" ).join( "child" );
  let unrelated = root.path().join( "other" );
  common::write_path_project_session( &storage_root, &ancestor,  "session-rel-ancestor",  2 );
  common::write_path_project_session( &storage_root, &cwd,       "session-rel-cwd",        2 );
  common::write_path_project_session( &storage_root, &unrelated, "session-rel-unrelated",  2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", cwd.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-rel-ancestor" ),
    "scope::relevant must find ancestor with underscore path; got:\n{s}"
  );
  assert!(
    s.contains( "session-rel-cwd" ),
    "scope::relevant must include current project; got:\n{s}"
  );
  assert!(
    !s.contains( "session-rel-unrelated" ),
    "scope::relevant must exclude unrelated project; got:\n{s}"
  );
}

// IT-12: scope::relevant finds topic-scoped ancestor when path has underscores
//
// The ancestor project directory has both underscore encoding in the path AND
// a topic suffix (stored as `--topic-name`). The is_relevant_encoded helper
// must strip the topic suffix via rfind("--") before comparing.
//
// bug_reproducer(issue-024)
#[test]
fn scope_relevant_finds_topic_scoped_ancestor()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let ancestor_path = root.path().join( "wip_core" );
  let cwd           = ancestor_path.join( "child" );

  // Child project (no topic)
  common::write_path_project_session( &storage_root, &cwd, "session-topic-cwd", 2 );

  // Ancestor project stored with topic suffix `--default-topic`
  let ancestor_encoded = claude_storage_core::encode_path( &ancestor_path )
    .expect( "encode ancestor path" );
  let ancestor_topic_dir = format!( "{ancestor_encoded}--default-topic" );
  common::write_test_session( &storage_root, &ancestor_topic_dir, "session-topic-ancestor", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", cwd.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-topic-ancestor" ),
    "scope::relevant must find topic-scoped ancestor with underscore path; got:\n{s}"
  );
  assert!(
    s.contains( "session-topic-cwd" ),
    "scope::relevant must include current project; got:\n{s}"
  );
}

// IT-13: scope::under with multiple underscore components in base finds nested projects
//
// Tests the more complex case where the base path contains multiple underscore
// components (`my_project/sub_module`). Each component encodes `_` → `-`, and
// the comparison must work correctly across all of them.
//
// bug_reproducer(issue-024)
#[test]
fn scope_under_finds_deeply_nested_with_multiple_underscore_components()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let base      = root.path().join( "my_project" ).join( "sub_module" );
  let child     = base.join( "feature_x" );
  let unrelated = root.path().join( "other_project" );

  common::write_path_project_session( &storage_root, &base,      "session-multi-base",      2 );
  common::write_path_project_session( &storage_root, &child,     "session-multi-child",     2 );
  common::write_path_project_session( &storage_root, &unrelated, "session-multi-unrelated", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-multi-base" ),
    "scope::under must include multi-underscore base itself; got:\n{s}"
  );
  assert!(
    s.contains( "session-multi-child" ),
    "scope::under must include child of multi-underscore base; got:\n{s}"
  );
  assert!(
    !s.contains( "session-multi-unrelated" ),
    "scope::under must exclude unrelated project; got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Coverage: UUID projects only match scope::global, excluded from local/under
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn uuid_project_only_matches_global_scope()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Path-based project
  let project = root.path().join( "proj" );
  common::write_path_project_session( &storage_root, &project, "session-path", 2 );

  // UUID project: write directly with a UUID-like directory name
  let uuid_dir = storage_root.join( "projects" ).join( "deadbeef-1234-5678-abcd-ef0123456789" );
  std::fs::create_dir_all( &uuid_dir ).unwrap();
  {
    use std::io::Write as _;
    let mut f = std::fs::File::create( uuid_dir.join( "session-uuid-test.jsonl" ) ).unwrap();
    writeln!( f, r#"{{"type":"user","uuid":"x01","parentUuid":null,"timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"session-uuid-test","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"hi"}}}}"# ).unwrap();
    writeln!( f, r#"{{"type":"assistant","uuid":"x02","parentUuid":"x01","timestamp":"2025-01-01T00:00:02Z","cwd":"/tmp","sessionId":"session-uuid-test","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"rq1","message":{{"role":"assistant","model":"claude-test","id":"m1","content":[{{"type":"text","text":"hello"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"# ).unwrap();
  }

  // scope::global → includes UUID session
  let out_global = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" ).arg( "scope::global" ).arg( "verbosity::0" )
    .output().unwrap();
  assert_exit( &out_global, 0 );
  let s_global = stdout( &out_global );
  // v0 outputs project paths: UUID project decoded as the UUID string itself
  assert!( s_global.contains( "deadbeef-1234-5678" ), "global must include UUID project path; got:\n{s_global}" );
  assert!( s_global.contains( "proj" ), "global must include path project; got:\n{s_global}" );

  // scope::local → excludes UUID session (only path project visible)
  let out_local = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" ).arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::0" )
    .output().unwrap();
  assert_exit( &out_local, 0 );
  let s_local = stdout( &out_local );
  assert!( !s_local.contains( "deadbeef" ), "local must exclude UUID project; got:\n{s_local}" );
  assert!( s_local.contains( "proj" ), "local must include path project; got:\n{s_local}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// Behavioral: scope::local includes sessions from topic-scoped subdirectories
//
// ## Purpose
// Confirms that scope::local returns sessions from both the plain project
// directory (`{encoded}`) and any topic-scoped variant (`{encoded}--topic`).
// Claude Code stores sessions in `-default_topic`-suffixed directories, so
// `scope::local` must include those to return all sessions for a project.
//
// ## Coverage
// Topic suffix matching via `dir_name.starts_with("{eb}--")`. Complements
// IT-12 which tests this for scope::relevant with an underscore path; this
// test uses scope::local with a plain alphanumeric path.
//
// ## Validation Strategy
// Create two session files: one in the plain project dir, one in the topic-
// scoped dir. Assert scope::local returns both. Assert a session from an
// unrelated project is excluded.
//
// ## Related Requirements
// `scope::local` semantic: project path == base path (exact), including
// topic suffix variants of that project.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn scope_local_matches_topic_scoped_directory()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let project    = root.path().join( "myproject" );
  let unrelated  = root.path().join( "other" );

  // Session in the plain project dir
  common::write_path_project_session( &storage_root, &project, "session-plain-dir",  2 );

  // Session in a topic-scoped project dir (`{encoded}--default-topic`)
  let encoded = claude_storage_core::encode_path( &project )
    .expect( "encode project path" );
  let topic_dir = format!( "{encoded}--default-topic" );
  common::write_test_session( &storage_root, &topic_dir, "session-topic-dir", 2 );

  // Unrelated project
  common::write_path_project_session( &storage_root, &unrelated, "session-other", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-plain-dir" ), "scope::local must include session in plain dir; got:\n{s}" );
  assert!( s.contains( "session-topic-dir" ), "scope::local must include session in topic-scoped dir; got:\n{s}" );
  assert!( !s.contains( "session-other" ),    "scope::local must exclude unrelated project; got:\n{s}" );
}
