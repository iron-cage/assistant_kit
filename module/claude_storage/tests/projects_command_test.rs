//! Tests for `.projects` command — scope-aware session listing.
//!
//! ## Coverage
//!
//! Edge-case tests EC-1..EC-8 from `docs/cli/testing/param/scope.md` plus
//! behavioural tests for all four scopes with synthetic storage fixtures.
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
//! - `projects_path_encoding_test.rs` — path decode/display bug reproducers (IT-23..IT-26)
//! - `projects_family_display_test.rs` — family and agent session display (IT-1, IT-33, IT-36..IT-48)
//! - `projects_output_format_test.rs` — output format: path headers, agent collapse (IT-17..IT-29)
//! - `projects_scope_around_test.rs` — `scope::around` neighborhood semantics (IT-57..IT-59)
//! - `projects_zero_byte_count_bug.rs` — zero-byte session exclusion (IT-54..IT-56)

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
    .arg( ".projects" )
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
    .arg( ".projects" )
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
    .arg( ".projects" )
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
    .arg( ".projects" )
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
    .arg( ".projects" ).arg( "scope::relevant" )
    .output().unwrap();

  let upper = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".projects" ).arg( "scope::RELEVANT" )
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
    .arg( ".projects" )
    .arg( "scope::all" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "scope must be relevant|local|under|around|global, got all" ),
    "error must contain exact message; got: {err}"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-7: scope:: omitted → defaults to around (same output as explicit scope::around)
//
// Fixture: parent project + child project so that scope::local and scope::around
// produce different results. Around includes the child; local doesn't.
// (No ancestor projects in storage → around output equals under output here.)
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec7_omitted_scope_defaults_to_around()
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
    .arg( ".projects" )
    .arg( format!( "path::{}", parent.display() ) )
    .output().unwrap();

  let explicit = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::around" )
    .arg( format!( "path::{}", parent.display() ) )
    .output().unwrap();

  assert_exit( &implicit, 0 );
  let s = core::str::from_utf8( &implicit.stdout ).unwrap();
  // default scope must include descendant sessions (around includes under direction)
  assert!(
    s.contains( "session-child" ),
    "default scope must include descendant sessions (around behavior); got:\n{s}"
  );
  assert_eq!(
    implicit.stdout, explicit.stdout,
    "omitting scope:: must produce same output as scope::around"
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
    .arg( ".projects" ).arg( "scope::global" )
    .output().unwrap();

  let with_path = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".projects" ).arg( "scope::global" ).arg( "path::/nonexistent-subpath" )
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

// ────────────────────────────────────────────────────────────────────────────
// Behavioural: verbosity::0 → no header, just project paths
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
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( !s.contains( "Found" ),  "verbosity::0 must not emit 'Found N projects' header; got:\n{s}" );
  assert!( s.contains( "proj" ),    "verbosity::0 must output project path; got:\n{s}" );
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
    .arg( ".projects" )
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
    .arg( ".projects" ).arg( "verbosity::99" )
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
    .arg( ".projects" ).arg( "min_entries::-1" )
    .output().unwrap();
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "min_entries" ),
    "error must mention min_entries; got: {}",
    stderr( &out )
  );
}

// ────────────────────────────────────────────────────────────────────────────
// Coverage: agent::1 returns only agent sessions, agent::0 excludes them
// ────────────────────────────────────────────────────────────────────────────
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
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "agent::1" )
    .arg( "verbosity::1" )
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
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "agent::0" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-main" ), "agent::0 must include main session; got:\n{s}" );
  assert!( !s.contains( "agent-task-002" ), "agent::0 must exclude agent session; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Coverage: min_entries:: filters by actual entry count
// ────────────────────────────────────────────────────────────────────────────
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
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "min_entries::3" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-long" ), "min_entries::3 must include 6-entry session; got:\n{s}" );
  assert!( !s.contains( "session-short" ), "min_entries::3 must exclude 2-entry session; got:\n{s}" );
}

// ────────────────────────────────────────────────────────────────────────────
// Coverage: verbosity::2 shows project path header (grouped format)
// ────────────────────────────────────────────────────────────────────────────
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
    .arg( ".projects" )
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
// issue-025 regression: "Found 1 sessions:" uses wrong plural — must be
// "Found 1 session:" (singular).
//
// Root Cause: projects_routine always formats the count noun as "sessions"
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
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Found 1 project:" ),
    "with 1 result, header must use singular 'project' (not 'projects'); got:\n{s}"
  );
  assert!(
    !s.contains( "Found 1 projects:" ),
    "with 1 result, header must NOT use plural 'projects'; got:\n{s}"
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
  let project_a = root.path().join( "proj-a" );
  let project_b = root.path().join( "proj-b" );

  // Two sessions in two distinct project directories = two projects
  common::write_path_project_session( &storage_root, &project_a, "session-plural-a", 2 );
  common::write_path_project_session( &storage_root, &project_b, "session-plural-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Found 2 projects:" ),
    "with 2 distinct projects, header must use plural 'projects'; got:\n{s}"
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
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Found 0 projects:" ),
    "with 0 results, header must use plural 'projects' (zero takes plural in English); got:\n{s}"
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
    .arg( ".projects" ).arg( "verbosity::-1" )
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
// before projects_routine is entered.
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
    .arg( ".projects" ).arg( "agent::2" )
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
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "min_entries::0" )
    .arg( "verbosity::1" )
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
    .arg( ".projects" )
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
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "path::/" )
    .output()
    .unwrap();

  assert_exit( &out_global, 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-50: Project header always says "conversations", never "sessions"
// ────────────────────────────────────────────────────────────────────────────
/// IT-50: Project headers must always use "conversations" as the user-facing noun.
///
/// "sessions" is a storage-layer implementation detail invisible to users.
/// This test creates a project with sessions but no agents and verifies the
/// header reads "(N conversations)" not "(N sessions)".
#[ test ]
fn it_header_uses_conversations_not_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  common::write_path_project_session( &storage_root, &project, "session-a", 2 );
  common::write_path_project_session( &storage_root, &project, "session-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.contains( "sessions)" ),
    "Project header must not contain 'sessions)' — must say 'conversations)'\nOutput: {s}",
  );
  assert!(
    s.contains( "conversations)" ),
    "Project header must contain 'conversations)'\nOutput: {s}",
  );
}
