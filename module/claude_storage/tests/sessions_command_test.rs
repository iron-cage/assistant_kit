//! Tests for `.sessions` command — scope-aware session listing.
//!
//! ## Coverage
//!
//! Edge-case tests EC-1..EC-8 from `docs/cli/testing/param/scope.md` plus
//! behavioural tests for all four scopes with synthetic storage fixtures.
//!
//! ### Family Display (IT-36..IT-48)
//!
//! | ID    | What it covers                                       |
//! |-------|------------------------------------------------------|
//! | IT-36 | Family header format — `conversation` + `agent`      |
//! | IT-37 | Per-root agent type breakdown bracket                |
//! | IT-38 | Hierarchical format detection (subagents/ layout)    |
//! | IT-39 | Flat format detection (sessionId-based parent link)  |
//! | IT-40 | Orphan agent display with `?` marker                 |
//! | IT-41 | Childless root has no bracket suffix                 |
//! | IT-42 | meta.json agent type propagation                     |
//! | IT-43 | Empty/missing meta.json falls back to `unknown`      |
//! | IT-44 | v1 orphan line shows `? (orphan)` label              |
//! | IT-45 | v2 root entry count singular — `(1 entry)` not `(1 entries)` |
//! | IT-46 | v2 agent entry count singular — `1 entry` not `1 entries` |
//! | IT-47 | Empty-string `agentType` (`""`) falls back to `unknown` |
//! | IT-48 | Whitespace-only `agentType` (`"  "`) falls back to `unknown` |
//!
//! ## Scope Semantics
//!
//! | Value     | Project qualifies when…                              |
//! |-----------|------------------------------------------------------|
//! | `local`   | project path == base path (exact)                    |
//! | `relevant`| base path starts_with project path (ancestors)       |
//! | `under`   | project path starts_with base path (subtree)         |
//! | `global`  | always (entire storage)                              |

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
// EC-1: scope::local accepted — exit 0
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec1_scope_local_accepted()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::local" )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-2: scope::relevant accepted — exit 0
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec2_scope_relevant_accepted()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::relevant" )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-3: scope::under accepted — exit 0 with path::
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec3_scope_under_accepted()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", root.path().display() ) )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-4: scope::global accepted — exit 0
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec4_scope_global_accepted()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-5: scope::RELEVANT (uppercase) is case-insensitive — same output as lowercase
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec5_scope_case_insensitive()
{
  let root = TempDir::new().unwrap();
  let home = root.path().to_str().unwrap();

  let lower = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".sessions" ).arg( "scope::relevant" )
    .output().unwrap();

  let upper = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".sessions" ).arg( "scope::RELEVANT" )
    .output().unwrap();

  assert_exit( &lower, 0 );
  assert_exit( &upper, 0 );
  assert_eq!(
    lower.stdout, upper.stdout,
    "scope::relevant and scope::RELEVANT must produce identical stdout"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-6: scope::all (invalid) → exit 1 with exact error message
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec6_invalid_scope_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::all" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "scope must be relevant|local|under|global, got all" ),
    "error must contain exact message; got: {err}"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-7: scope:: omitted → defaults to under (same output as explicit scope::under)
//
// Fixture: parent project + child project (under parent path) so that scope::local
// and scope::under produce different results. Under includes the child; local doesn't.
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec7_omitted_scope_defaults_to_under()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let parent = root.path().join( "parent_proj" );
  let child  = parent.join( "child_sub" );
  common::write_path_project_session( &storage_root, &parent, "session-parent", 1 );
  common::write_path_project_session( &storage_root, &child,  "session-child",  1 );

  let implicit = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( format!( "path::{}", parent.display() ) )
    .output().unwrap();

  let explicit = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", parent.display() ) )
    .output().unwrap();

  assert_exit( &implicit, 0 );
  let s = core::str::from_utf8( &implicit.stdout ).unwrap();
  // default scope must include sub-project sessions (under behavior)
  assert!(
    s.contains( "session-child" ),
    "default scope must include sub-project sessions (under behavior); got:\n{s}"
  );
  assert_eq!(
    implicit.stdout, explicit.stdout,
    "omitting scope:: must produce same output as scope::under"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-8: scope::global ignores path:: — output identical with or without path::
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec8_global_ignores_path()
{
  let root = TempDir::new().unwrap();
  let home = root.path().to_str().unwrap();

  let without_path = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".sessions" ).arg( "scope::global" )
    .output().unwrap();

  let with_path = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".sessions" ).arg( "scope::global" ).arg( "path::/nonexistent-subpath" )
    .output().unwrap();

  assert_exit( &without_path, 0 );
  assert_exit( &with_path, 0 );
  assert_eq!(
    without_path.stdout, with_path.stdout,
    "scope::global must produce identical output regardless of path::"
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-glob-a" ), "must contain session-glob-a; got:\n{s}" );
  assert!( s.contains( "session-glob-b" ), "must contain session-glob-b; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Behavioural: verbosity::0 → no header, just session IDs
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn verbosity_zero_no_header()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  common::write_path_project_session( &storage_root, &project, "session-v0-test", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( !s.contains( "Found" ),      "verbosity::0 must not emit 'Found N sessions' header; got:\n{s}" );
  assert!( s.contains( "session-v0-test" ), "must still list session ID; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Behavioural: session:: filter narrows results
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn session_filter_narrows_results()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  // two sessions in the same project
  common::write_path_project_session( &storage_root, &project, "session-keep-001", 2 );
  common::write_path_project_session( &storage_root, &project, "session-drop-002", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "session::keep" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-keep-001" ),  "must contain matching session; got:\n{s}" );
  assert!( !s.contains( "session-drop-002" ), "must exclude non-matching session; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Validation: verbosity out of range → exit 1
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn invalid_verbosity_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" ).arg( "verbosity::99" )
    .output().unwrap();
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "verbosity" ),
    "error must mention verbosity; got: {}",
    stderr( &out )
  );
}

// ────────────────────────────────────────────────────────────────────────────
// Validation: min_entries negative → exit 1
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn invalid_min_entries_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" ).arg( "min_entries::-1" )
    .output().unwrap();
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "min_entries" ),
    "error must mention min_entries; got: {}",
    stderr( &out )
  );
}

// ────────���───────────────────────────────────────────────────────────────────
// Coverage: agent::1 returns only agent sessions, agent::0 excludes them
// ─────────────────────────��──────────────────────────────────────────────────
#[test]
fn agent_filter_includes_only_agent_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  common::write_path_project_session( &storage_root, &project, "session-main", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-001", 2 );

  // agent::1 → only agent session
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "agent::1" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "agent-task-001" ), "agent::1 must include agent session; got:\n{s}" );
  assert!( !s.contains( "session-main" ), "agent::1 must exclude main session; got:\n{s}" );
}

#[test]
fn agent_filter_excludes_agent_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  common::write_path_project_session( &storage_root, &project, "session-main", 2 );
  common::write_path_project_session( &storage_root, &project, "agent-task-002", 2 );

  // agent::0 → only main sessions
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "agent::0" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-main" ), "agent::0 must include main session; got:\n{s}" );
  assert!( !s.contains( "agent-task-002" ), "agent::0 must exclude agent session; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Coverage: min_entries:: filters by actual entry count
// ─────────────────────────────────────────────────��──────────────────────────
#[test]
fn min_entries_filters_by_entry_count()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  // 2-entry session and 6-entry session
  common::write_path_project_session( &storage_root, &project, "session-short", 2 );
  common::write_path_project_session( &storage_root, &project, "session-long", 6 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "min_entries::3" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-long" ), "min_entries::3 must include 6-entry session; got:\n{s}" );
  assert!( !s.contains( "session-short" ), "min_entries::3 must exclude 2-entry session; got:\n{s}" );
}

// ───────────��────────────────────────────────────────────────────────────────
// Coverage: verbosity::2 shows project path header (grouped format)
// ─���──────────────────────────────────────────────────────���───────────────────
#[test]
fn verbosity_two_includes_project_label()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  common::write_path_project_session( &storage_root, &project, "session-v2-test", 2 );

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
  assert!( s.contains( "Found" ), "verbosity::2 must emit 'Found N sessions' header; got:\n{s}" );
  assert!(
    s.lines().any( | l | l.contains( ':' ) && ( l.contains( '/' ) || l.contains( '~' ) ) ),
    "verbosity::2 must show project path header; got:\n{s}"
  );
  assert!( s.contains( "session-v2-test" ), "must list session ID; got:\n{s}" );
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" ).arg( "scope::global" ).arg( "verbosity::0" )
    .output().unwrap();
  assert_exit( &out_global, 0 );
  let s_global = stdout( &out_global );
  assert!( s_global.contains( "session-uuid-test" ), "global must include UUID session; got:\n{s_global}" );

  // scope::local → excludes UUID session
  let out_local = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" ).arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::0" )
    .output().unwrap();
  assert_exit( &out_local, 0 );
  let s_local = stdout( &out_local );
  assert!( !s_local.contains( "session-uuid-test" ), "local must exclude UUID session; got:\n{s_local}" );
  assert!( s_local.contains( "session-path" ), "local must include path session; got:\n{s_local}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// issue-025 regression: "Found 1 sessions:" uses wrong plural — must be
// "Found 1 session:" (singular).
//
// Root Cause: sessions_routine always formats the count noun as "sessions"
// regardless of count. English grammar requires singular ("session") when
// count == 1.
//
// Why Not Caught: No existing test asserted the exact singular/plural form of
// the "Found N sessions:" header — only that the word "Found" was present.
//
// Fix Applied: Derive the noun ("session" vs "sessions") based on `rows.len()`
// before formatting the header, and use the derived noun in the format string.
//
// Prevention: Always add an exact-string assertion for count-bearing output
// when writing tests, not just a contains("Found") check.
//
// Pitfall: "Found 0 sessions:" should remain plural ("sessions"), consistent
// with English grammar where zero takes plural form.
// ─────────────────────────────────────────────────────────────────────────────

// IT-14: singular noun when exactly 1 session found
//
// bug_reproducer(issue-025)
#[test]
fn output_uses_singular_noun_when_exactly_one_session_found()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  // Exactly one session
  common::write_path_project_session( &storage_root, &project, "session-singular-test", 2 );

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
    s.contains( "Found 1 session:" ),
    "with 1 result, header must use singular 'session' (not 'sessions'); got:\n{s}"
  );
  assert!(
    !s.contains( "Found 1 sessions:" ),
    "with 1 result, header must NOT use plural 'sessions'; got:\n{s}"
  );
  assert!( s.contains( "session-singular-test" ), "must list the session ID; got:\n{s}" );
}

// IT-15: plural noun when 2 or more sessions found
//
// bug_reproducer(issue-025)
#[test]
fn output_uses_plural_noun_when_multiple_sessions_found()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  // Two sessions
  common::write_path_project_session( &storage_root, &project, "session-plural-a", 2 );
  common::write_path_project_session( &storage_root, &project, "session-plural-b", 2 );

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
    s.contains( "Found 2 sessions:" ),
    "with 2 results, header must use plural 'sessions'; got:\n{s}"
  );
}

// IT-16: zero sessions header still uses plural ("Found 0 sessions:")
//
// bug_reproducer(issue-025)
#[test]
fn output_uses_plural_noun_when_zero_sessions_found()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // No sessions at all (empty storage)
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
    s.contains( "Found 0 sessions:" ),
    "with 0 results, header must use plural 'sessions' (zero takes plural in English); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation: verbosity::-1 (negative) → exit 1
//
// ## Purpose
// Validates that the verbosity parameter lower bound is enforced. The valid
// range is 0–5; negative values must be rejected with a clear error message.
//
// ## Coverage
// Boundary: verbosity below minimum (< 0). Complements the existing
// `invalid_verbosity_rejected` test which only checks the upper bound (99).
//
// ## Validation Strategy
// Assert exit code 1 and that stderr mentions "verbosity" so the user knows
// which parameter caused the error.
//
// ## Related Requirements
// Same validation contract as `status_routine`, `search_routine`, etc.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn verbosity_negative_one_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" ).arg( "verbosity::-1" )
    .output().unwrap();
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "verbosity" ),
    "error must mention verbosity; got: {}",
    stderr( &out )
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation: agent::2 (out of range) → exit 1
//
// ## Purpose
// Validates that the agent parameter only accepts boolean values (0 or 1).
// Values outside that range must be rejected with a descriptive error.
//
// ## Coverage
// Boolean validation: value > 1. Complements EC-6 (scope validation) and
// `invalid_min_entries_rejected` (numeric validation).
//
// ## Validation Strategy
// Assert exit code 1. The error is produced by the unilang boolean parser
// before sessions_routine is entered.
//
// ## Related Requirements
// `agent::` is documented as accepting 0 or 1 only.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn agent_value_out_of_range_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" ).arg( "agent::2" )
    .output().unwrap();
  assert_exit( &out, 1 );
}

// ─────────────────────────────────────────────────────────────────────────────
// Behavioral: min_entries::0 includes all sessions (no lower bound)
//
// ## Purpose
// Confirms that min_entries::0 is treated as "no minimum" and returns all
// sessions regardless of entry count. This is the zero-value boundary case.
//
// ## Coverage
// Boundary: min_entries == 0 includes sessions with any entry count, including
// 1-entry sessions. Complements `min_entries_filters_by_entry_count` which
// tests min_entries::3 with sessions of 2 and 6 entries.
//
// ## Validation Strategy
// Create two sessions (1-entry and 4-entry). Assert both appear in output
// when min_entries::0 is used, since 1 >= 0 and 4 >= 0.
//
// ## Related Requirements
// Consistent with standard "minimum N means N or more" semantics.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn min_entries_zero_includes_all_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );

  common::write_path_project_session( &storage_root, &project, "session-one-entry",  1 );
  common::write_path_project_session( &storage_root, &project, "session-four-entry", 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "min_entries::0" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-one-entry" ),  "min_entries::0 must include 1-entry session; got:\n{s}" );
  assert!( s.contains( "session-four-entry" ), "min_entries::0 must include 4-entry session; got:\n{s}" );
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
    .arg( ".sessions" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-plain-dir" ), "scope::local must include session in plain dir; got:\n{s}" );
  assert!( s.contains( "session-topic-dir" ), "scope::local must include session in topic-scoped dir; got:\n{s}" );
  assert!( !s.contains( "session-other" ),    "scope::local must exclude unrelated project; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge case: scope::local/under/relevant with path::/ → exit 1 (unencodable)
//
// ## Purpose
// Confirms that passing path::/ to a scope that needs to encode the base path
// fails with exit 1 and a clear error.  `encode_path()` cannot represent the
// filesystem root because path components are empty after stripping the leading
// '/'.  scope::global is exempt because it never encodes the path.
//
// ## Coverage
// Boundary: unencodable path (filesystem root).  Complements EC-8 which tests
// that scope::global ignores path:: entirely (including /nonexistent paths).
//
// ## Validation Strategy
// Assert exit code 1 for scope::under (representative of the three non-global
// scopes), and exit code 0 for scope::global with the same path::/  to confirm
// the distinction.
//
// ## Related Requirements
// encode_path() contract: returns error when path is empty after normalization.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn root_path_rejected_for_non_global_scope()
{
  let root = TempDir::new().unwrap();

  // scope::under with path::/ must fail (unencodable base path)
  let out_under = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::under" )
    .arg( "path::/" )
    .output()
    .unwrap();

  assert_ne!(
    out_under.status.code().unwrap_or( -1 ), 0,
    "scope::under path::/ must exit non-zero; got exit 0"
  );
  assert!(
    stderr( &out_under ).contains( "path is empty after normalization" )
      || stderr( &out_under ).contains( "Failed to encode" ),
    "error must mention path encoding failure; got: {}",
    stderr( &out_under )
  );

  // scope::global with path::/ must still succeed (global ignores path)
  let out_global = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::global" )
    .arg( "path::/" )
    .output()
    .unwrap();

  assert_exit( &out_global, 0 );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Hyphen-Prefixed Topic Directory (issue-030)
//
// Root Cause: decode_project_display stripped the `--topic` suffix before
// decoding, so `-...-src--default-topic` displayed as `src` even when
// `-default_topic` is a real filesystem directory (the actual working directory).
//
// Why Not Caught: All prior tests used simple session paths with no
// hyphen-prefixed directory components. No test path ended in `/-default_topic`
// or any other `-name` component that the topic strip discarded.
//
// Fix Applied: decode_project_display now tries to extend the decoded base path
// by each `--topic` component as a real filesystem directory. The display uses
// the longest existing path prefix. So `-...-src--default-topic` displays as
// `src/-default_topic` when that directory exists on disk.
//
// Prevention: Test that sessions created from a hyphen-prefixed working
// directory (e.g. `src/-default_topic`) display the full path in the header.
//
// Pitfall: The extension step calls `candidate.exists()`, so the project
// directory must exist on disk at display time. Deleted projects fall back to
// the base path (old behaviour), which is acceptable.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-030)
fn it_24_decode_display_includes_hyphen_prefixed_topic_dir()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Project path ending in a hyphen-prefixed directory (the real CWD pattern)
  let project = root.path().join( "src" ).join( "-default_topic" );
  // Create the actual directories so the existence check passes
  std::fs::create_dir_all( &project ).expect( "create src/-default_topic dir" );
  common::write_path_project_session( &storage_root, &project, "session-topic-dir-test", 2 );

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
    s.contains( "-default_topic" ),
    "display path must include hyphen-prefixed topic dir '-default_topic'; got:\n{s}"
  );
  assert!(
    !s.lines().any( | l | l.trim_end().ends_with( "src:" ) ),
    "display path must NOT be truncated to 'src' when '-default_topic' exists; got:\n{s}"
  );
  assert!( s.contains( "session-topic-dir-test" ), "session ID must appear; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// Decode Display — Underscore Directory Names (issue-029)
//
// Root Cause: encode_path converts `_` → `-` (lossy). The heuristic decoder
// defaults to path separator (`/`) for all unrecognized `-` boundaries, so
// underscore-named directories like `wip_core` decode to `wip/core` in the
// display path.
//
// Why Not Caught: All prior tests used simple single-word project dir names
// (e.g., "proj", "agent_filter_proj"). No test path had underscore-named
// intermediate components like `wip_core/project`.
//
// Fix Applied: decode_project_display now checks whether the heuristic-decoded
// path exists on the filesystem. If not, it falls back to decode_path_via_fs
// which walks the real directory tree, choosing `/` vs `_` at each `-` boundary
// by calling is_dir() on the candidate path prefix.
//
// Prevention: Test project paths that contain underscore-named intermediate
// directories. The test must also create those directories on disk so the
// filesystem walk can verify existence.
//
// Pitfall: decode_path_via_fs requires the project directory to exist at display
// time. Deleted or remote projects fall back to the raw encoded storage dir name.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-029)
fn it_23_decode_display_preserves_underscore_named_dirs()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Project path with underscore-named directory component
  let project = root.path().join( "wip_core" ).join( "myproject" );
  // Create the actual directories so filesystem-guided decode can verify existence
  std::fs::create_dir_all( &project ).expect( "create project dir with underscore component" );
  common::write_path_project_session( &storage_root, &project, "session-underscore-test", 2 );

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
    s.contains( "wip_core" ),
    "display path must preserve underscore: 'wip_core' not 'wip/core'; got:\n{s}"
  );
  assert!(
    !s.lines().any( | l | l.contains( "wip/core" ) ),
    "display path must NOT split wip_core into wip/core; got:\n{s}"
  );
  assert!( s.contains( "session-underscore-test" ), "session ID must appear; got:\n{s}" );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::under — Sibling Module Exclusion (issue-031)
//
// Root Cause: encode_path maps both `_` and `/` to `-`. The `under` predicate
// used string starts_with on encoded forms, so a sibling `base_extra/` passed
// the same prefix check as a child `base/sub/`: both encoded forms start with
// the `base-` prefix. String comparison cannot distinguish path-separator `/`
// from underscore `_` in encoded form.
//
// Why Not Caught: All prior scope::under tests used simple single-word base dirs
// (e.g., "workspace"). No test had a sibling whose name was the base name plus
// an underscore suffix, simulating real module naming like `claude_storage_core`
// next to `claude_storage`.
//
// Fix Applied: Two-stage predicate. String prefix is fast-reject only. Candidates
// passing string check (not exact) are verified via decode_path_via_fs +
// Path::starts_with. Path::starts_with is component-wise: Path("/x/base_extra")
// does NOT start_with Path("/x/base") even though string "/x/base_extra"
// starts_with "/x/base".
//
// Prevention: Always test scope::under with a sibling whose encoded form shares the
// base encoded prefix (underscore-suffix sibling). Create all directories on disk
// so decode_path_via_fs can resolve them correctly.
//
// Pitfall: decode_path_via_fs returns None for deleted/remote paths. The fixed
// predicate uses unwrap_or(true) (conservative include) to avoid silently dropping
// sessions from projects that existed when the session was created.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-031)
fn it_25_scope_under_excludes_underscore_named_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Simulate: base = module/claude_storage
  //           child = module/claude_storage/sub  (under base → must appear)
  //           sibling = module/base_extra         (NOT under base → must not appear)
  let base    = root.path().join( "base" );
  let child   = base.join( "sub" );
  let sibling = root.path().join( "base_extra" );

  // Directories must exist on disk: decode_path_via_fs uses is_dir() to walk.
  // Without real dirs the walker returns None → unwrap_or(true) includes all.
  std::fs::create_dir_all( &child ).expect( "create child dir" );
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );

  common::write_path_project_session( &storage_root, &child,   "session-it25-child",   2 );
  common::write_path_project_session( &storage_root, &sibling, "session-it25-sibling", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it25-child" ),
    "must contain session-it25-child (child base/sub is under base); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it25-sibling" ),
    "must NOT contain session-it25-sibling (sibling base_extra is NOT under base); got:\n{s}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// scope::relevant — Sibling Module Exclusion (issue-032)
//
// Root Cause: encode_path maps both `_` and `/` to `-`. The `relevant` scope
// predicate (is_relevant_encoded) uses string starts_with: encoded_base
// starts_with(dir_name + "-"). A sibling `base/` passed the same prefix check
// as a real ancestor: if base_path is `/tmp/base_extra`, the project at `/tmp/base`
// (encoded `-tmp-base`) matched because `-tmp-base-extra` starts with `-tmp-base-`.
// String comparison cannot distinguish `/` from `_` in encoded form.
//
// Why Not Caught: All prior scope::relevant tests used simple ancestor chains
// (e.g., /a, /a/b, /a/b/c). No test had a sibling whose encoded name was a
// prefix of the current path's encoded form — the `base` vs `base_extra` pattern.
//
// Fix Applied: Two-stage predicate in the `"relevant"` arm of project_matches.
// is_relevant_encoded is fast-reject only. Exact encoded match returns true.
// Prefix-match candidates are verified via decode_path_via_fs +
// base_path.starts_with(decoded_path). Path::starts_with is component-wise:
// Path("/x/base_extra").starts_with(Path("/x/base")) → false.
//
// Prevention: Always test scope::relevant with a project whose name is a
// string prefix of the current path's name (underscore-suffix sibling).
// Create all directories on disk so decode_path_via_fs can resolve them.
//
// Pitfall: Same as issue-031 fix for scope::under — decode_path_via_fs returns
// None for deleted/remote paths; is_none_or provides conservative include.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
// bug_reproducer(issue-032)
fn it_26_scope_relevant_excludes_underscore_named_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Simulate: sibling = base      (NOT an ancestor of base_extra despite prefix match)
  //           target  = base_extra (current path; encoded -...-base-extra)
  // /base encoded to `-...-base`; `/base_extra` encoded to `-...-base-extra`.
  // Without fix: is_relevant_encoded returns true because encoded_base starts
  // with (dir_name + "-"), making scope::relevant include /base as a false ancestor.
  let sibling = root.path().join( "base" );
  let target  = root.path().join( "base_extra" );

  // Directories must exist on disk: decode_path_via_fs uses is_dir() to walk.
  // Without real dirs the walker returns None → is_none_or(true) includes all.
  std::fs::create_dir_all( &sibling ).expect( "create sibling dir" );
  std::fs::create_dir_all( &target ).expect( "create target dir" );

  common::write_path_project_session( &storage_root, &sibling, "session-it26-sibling", 2 );
  common::write_path_project_session( &storage_root, &target,  "session-it26-target",  2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".sessions" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", target.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-it26-target" ),
    "must contain session-it26-target (current project at base_extra); got:\n{s}"
  );
  assert!(
    !s.contains( "session-it26-sibling" ),
    "must NOT contain session-it26-sibling (/base is NOT an ancestor of /base_extra); got:\n{s}"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-1 / T01: Default (no args) shows active-session summary, not a list
// ────────────────────────────────────────────────────────────────────────────

/// IT-1: Default (no args) shows active-session summary
///
/// ## Root Cause (tested behaviour)
/// When `clg .sessions` is invoked with no arguments, it activates summary mode:
/// prints a single-session block for the most-recent session instead of the
/// full project-grouped session list.
///
/// ## Verification
/// - stdout contains "Active session" header line
/// - stdout contains "Project" line
/// - stdout contains "Last message:" section
/// - stdout does NOT contain "Found" (list-mode header)
#[test]
fn it1_default_shows_active_session_summary()
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
    .arg( ".sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Active session" ), "expected summary header; got:\n{s}" );
  assert!( s.contains( "Project" ), "expected Project line; got:\n{s}" );
  assert!( s.contains( "Last message:" ), "expected Last message section; got:\n{s}" );
  assert!( !s.contains( "Found" ), "expected no list header; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-30 / T01: Summary header format (id, age, count, project path)
// ────────────────────────────────────────────────────────────────────────────

/// IT-30: Summary header format — id, age, count, path
///
/// ## Root Cause (tested behaviour)
/// The first line must be `Active session  {8-char-id}  {age}  {N} entries`.
/// The second line must be `Project  {path}`.
///
/// ## Verification
/// - First line starts with "Active session"
/// - First line contains the session ID (or its first 8 chars)
/// - First line contains the entry count and "entries"
/// - Second line starts with "Project"
#[test]
fn it30_summary_header_format()
{
  let root = tempfile::TempDir::new().unwrap();
  let project_path = root.path().join( "proj30" );
  std::fs::create_dir_all( &project_path ).unwrap();

  // Write 2 standard + 1 last-message entry → total 3 entries
  common::write_path_project_session_with_last_message(
    root.path(), &project_path, "session-it30", 2, "Last msg for it30"
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", root.path().to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().collect();

  assert!( !lines.is_empty(), "output should not be empty" );
  assert!(
    lines[ 0 ].starts_with( "Active session" ),
    "first line must start with 'Active session'; got: {:?}", lines[ 0 ]
  );
  assert!(
    lines[ 0 ].contains( "entries" ),
    "first line must contain entry count; got: {:?}", lines[ 0 ]
  );
  assert!(
    lines.len() >= 2 && lines[ 1 ].starts_with( "Project" ),
    "second line must start with 'Project'; got: {:?}", lines.get( 1 )
  );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-31 / T02: Truncation gate — message ≤ 50 chars shown in full
// ────────────────────────────────────────────────────────────────────────────

/// IT-31: Truncation gate — 40-char message shown in full, no ellipsis
///
/// ## Root Cause (tested behaviour)
/// Messages of 50 characters or fewer must NOT be truncated.
/// The full text must appear verbatim in the Last message section.
///
/// ## Verification
/// - stdout contains the full 40-char string
/// - stdout does NOT contain "..."
#[test]
fn it31_truncation_gate_short_message()
{
  // Exactly 40 characters (verified by counting):
  // "Fix typo in the readme file near line 10"
  //  3 + 1 + 4 + 1 + 2 + 1 + 3 + 1 + 6 + 1 + 4 + 1 + 4 + 1 + 4 + 1 + 2 = 40
  let msg_40 = "Fix typo in the readme file near line 10";
  assert_eq!( msg_40.chars().count(), 40, "fixture must be exactly 40 chars" );

  let root = tempfile::TempDir::new().unwrap();
  let project_path = root.path().join( "proj31" );
  std::fs::create_dir_all( &project_path ).unwrap();

  common::write_path_project_session_with_last_message(
    root.path(), &project_path, "session-it31", 2, msg_40
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", root.path().to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( msg_40 ), "full 40-char message must appear verbatim; got:\n{s}" );
  assert!( !s.contains( "..." ), "must NOT truncate a 40-char message; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-32 / T04: Truncation formula — message > 50 chars as first30...last30
// ────────────────────────────────────────────────────────────────────────────

/// IT-32: Truncation formula — 60-char message as {first30}...{last30}
///
/// ## Root Cause (tested behaviour)
/// Messages longer than 50 Unicode scalar values must be truncated to
/// `{first30}...{last30}` (exactly 63 output characters for a 60-char input).
///
/// ## Verification
/// - stdout contains "..."
/// - The first 30 chars of the source message appear before "..."
/// - The last 30 chars of the source message appear after "..."
/// - The full 60-char message does NOT appear verbatim
#[test]
fn it32_truncation_formula_long_message()
{
  // 60 chars total: 30 A's + 30 B's — distinct halves for easy verification.
  let first30  = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
  let last30   = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";
  let msg_60   = format!( "{first30}{last30}" );
  assert_eq!( msg_60.chars().count(), 60, "fixture must be exactly 60 chars" );

  let root = tempfile::TempDir::new().unwrap();
  let project_path = root.path().join( "proj32" );
  std::fs::create_dir_all( &project_path ).unwrap();

  common::write_path_project_session_with_last_message(
    root.path(), &project_path, "session-it32", 2, &msg_60
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", root.path().to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "..." ), "must use ... for 60-char message; got:\n{s}" );
  assert!( s.contains( first30 ), "must contain first 30 chars before ...; got:\n{s}" );
  assert!( s.contains( last30 ), "must contain last 30 chars after ...; got:\n{s}" );
  assert!( !s.contains( &msg_60 ), "must NOT show full 60-char message verbatim; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-33 / T05: No sessions in scope → "No active session found."
// ────────────────────────────────────────────────────────────────────────────

/// IT-33: Empty scope → "No active session found."
///
/// ## Root Cause (tested behaviour)
/// When no sessions exist under the cwd, summary mode must emit the sentinel
/// message `No active session found.` rather than empty output or an error.
///
/// ## Verification
/// - exit code is 0
/// - stdout contains "No active session found."
/// - stderr is empty
#[test]
fn it33_no_sessions_shows_not_found()
{
  let root = tempfile::TempDir::new().unwrap();
  let project_path = root.path().join( "empty_proj" );
  std::fs::create_dir_all( &project_path ).unwrap();
  // No session files written — storage is empty.

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", root.path().to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "No active session found." ),
    "expected sentinel message; got:\n{s}"
  );
  assert!( stderr( &out ).is_empty(), "stderr must be empty; got: {}", stderr( &out ) );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-34 / T06: Explicit scope::local keeps list mode
// ────────────────────────────────────────────────────────────────────────────

/// IT-34: Explicit `scope::` parameter activates list mode
///
/// ## Root Cause (tested behaviour)
/// Summary mode is only active when ALL parameters are absent.
/// An explicit `scope::local` must activate list mode regardless.
///
/// ## Verification
/// - stdout contains "Found" (list-mode header)
/// - stdout does NOT contain "Active session"
#[test]
fn it34_explicit_scope_keeps_list_mode()
{
  let root = tempfile::TempDir::new().unwrap();
  let project_path = root.path().join( "proj34" );
  std::fs::create_dir_all( &project_path ).unwrap();

  common::write_path_project_session_with_last_message(
    root.path(), &project_path, "session-it34", 2, "Some message"
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", root.path().to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".sessions" )
    .arg( "scope::local" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ), "explicit scope:: must activate list mode; got:\n{s}" );
  assert!( !s.contains( "Active session" ), "must NOT show summary with explicit scope::; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-35 / T07: Explicit limit::N keeps list mode
// ────────────────────────────────────────────────────────────────────────────

/// IT-35: Explicit `limit::` parameter activates list mode
///
/// ## Root Cause (tested behaviour)
/// Summary mode is only active when ALL parameters are absent.
/// An explicit `limit::5` must activate list mode regardless.
///
/// ## Verification
/// - stdout contains "Found" (list-mode header)
/// - stdout does NOT contain "Active session"
#[test]
fn it35_explicit_limit_keeps_list_mode()
{
  let root = tempfile::TempDir::new().unwrap();
  let project_path = root.path().join( "proj35" );
  std::fs::create_dir_all( &project_path ).unwrap();

  common::write_path_project_session_with_last_message(
    root.path(), &project_path, "session-it35", 2, "Some message"
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", root.path().to_str().unwrap() )
    .current_dir( &project_path )
    .arg( ".sessions" )
    .arg( "limit::5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found" ), "explicit limit:: must activate list mode; got:\n{s}" );
  assert!( !s.contains( "Active session" ), "must NOT show summary with explicit limit::; got:\n{s}" );
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
    .arg( ".sessions" )
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
