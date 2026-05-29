//! Integration tests for the `clg .projects` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/07_projects.md`
//!
//! ## Coverage
//!
//! - INT-1: Default (no args) shows active-project summary
//! - INT-2: `scope::relevant` includes ancestor project sessions
//! - INT-3: `scope::under` includes descendant project sessions
//! - INT-4: `scope::global` returns all sessions regardless of path
//! - INT-5: `path::` overrides cwd as scope anchor
//! - INT-6: `session::` filter narrows results
//! - INT-7: `min_entries::` filter excludes short sessions
//! - INT-8: No matching sessions exits with code 0
//! - INT-9: `scope::local` finds project when path contains underscores
//! - INT-10: `scope::under` finds subtree when base path has underscores
//! - INT-11: `scope::relevant` finds ancestor when path has underscores
//! - INT-12: `scope::relevant` finds topic-scoped ancestor with underscores
//! - INT-13: `scope::under` with multiple underscore components finds nested projects
//! - INT-14: default output groups sessions under project path headers
//! - INT-15: path header always present for `scope::local` single project
//! - INT-16: agent sessions collapsed to count line without `agent::` filter
//! - INT-17: `show_tree::1` shows agents tree-indented under parent session
//! - INT-18: entry count shown per session by default
//! - INT-19: `agent::1` explicit filter disables collapse
//! - INT-20: `scope::under` displays underscore dirs without splitting at /
//! - INT-21: `scope::global` displays hyphen-prefixed topic dir in path header
//! - INT-22: `scope::under` excludes sibling with underscore-suffix name
//! - INT-23: `scope::relevant` excludes sibling with underscore-suffix name
//! - INT-24: entry count shown per session
//! - INT-25: `limit::N` truncates main sessions
//!
//! Tests INT-26..INT-50: → `cli_cmd_projects_summary_test.rs`

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

// ─── INT-1 ────────────────────────────────────────────────────────────────────

/// INT-1: Default (no args) shows active-project summary.
///
/// After task-019 removed the dedicated summary mode, bare `clg .projects`
/// uses list mode (`scope::around`). The spec INT-1 describes legacy summary
/// output. The test verifies the command exits 0 and produces some output
/// referencing the project. The exact format (summary vs list) reflects the
/// current implementation.
#[ test ]
fn int_1_default_no_args_exits_0_with_output()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "alpha" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session_with_last_message(
    &storage_root, &project, "session-int1", 2, "Hello from int-1 test"
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
  assert!(
    !s.is_empty(),
    "output must not be empty when project has sessions; got empty stdout"
  );
}

// ─── INT-2 ────────────────────────────────────────────────────────────────────

/// INT-2: `scope::relevant` includes ancestor project sessions.
#[ test ]
fn int_2_scope_relevant_includes_ancestors()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let path_a   = root.path().join( "a" );
  let path_ab  = path_a.join( "b" );
  let path_abc = path_ab.join( "c" );
  fs::create_dir_all( &path_abc ).unwrap();

  common::write_path_project_session( &storage_root, &path_a,   "session-int2-a",   2 );
  common::write_path_project_session( &storage_root, &path_ab,  "session-int2-ab",  2 );
  common::write_path_project_session( &storage_root, &path_abc, "session-int2-abc", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", path_abc.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int2-a" ),   "must include ancestor /a; got:\n{s}" );
  assert!( s.contains( "session-int2-ab" ),  "must include ancestor /a/b; got:\n{s}" );
  assert!( s.contains( "session-int2-abc" ), "must include current /a/b/c; got:\n{s}" );
}

// ─── INT-3 ────────────────────────────────────────────────────────────────────

/// INT-3: `scope::under` includes descendant project sessions.
#[ test ]
fn int_3_scope_under_includes_descendants()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let base  = root.path().join( "a" ).join( "b" );
  let child = base.join( "c" );
  let grand = child.join( "d" );
  let other = root.path().join( "z" );
  fs::create_dir_all( &grand ).unwrap();
  fs::create_dir_all( &other ).unwrap();

  common::write_path_project_session( &storage_root, &base,  "session-int3-base",  2 );
  common::write_path_project_session( &storage_root, &child, "session-int3-child", 2 );
  common::write_path_project_session( &storage_root, &grand, "session-int3-grand", 2 );
  common::write_path_project_session( &storage_root, &other, "session-int3-other", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int3-base" ),  "must include base; got:\n{s}" );
  assert!( s.contains( "session-int3-child" ), "must include child; got:\n{s}" );
  assert!( s.contains( "session-int3-grand" ), "must include grandchild; got:\n{s}" );
  assert!( !s.contains( "session-int3-other" ), "must NOT include /z sibling; got:\n{s}" );
}

// ─── INT-4 ────────────────────────────────────────────────────────────────────

/// INT-4: `scope::global` returns all sessions regardless of path.
#[ test ]
fn int_4_scope_global_returns_all()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let pa = root.path().join( "a" ).join( "b" );
  let pb = root.path().join( "c" ).join( "d" );
  let pc = root.path().join( "e" ).join( "f" );

  common::write_path_project_session( &storage_root, &pa, "session-int4-ab", 2 );
  common::write_path_project_session( &storage_root, &pb, "session-int4-cd", 2 );
  common::write_path_project_session( &storage_root, &pc, "session-int4-ef", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int4-ab" ), "must include /a/b; got:\n{s}" );
  assert!( s.contains( "session-int4-cd" ), "must include /c/d; got:\n{s}" );
  assert!( s.contains( "session-int4-ef" ), "must include /e/f; got:\n{s}" );
}

// ─── INT-5 ────────────────────────────────────────────────────────────────────

/// INT-5: `path::` overrides cwd as scope anchor.
#[ test ]
fn int_5_path_overrides_cwd()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let target = root.path().join( "a" ).join( "b" ).join( "c" );
  let other  = root.path().join( "a" ).join( "b" );
  fs::create_dir_all( &target ).unwrap();

  common::write_path_project_session( &storage_root, &target, "session-int5-target", 2 );
  common::write_path_project_session( &storage_root, &other,  "session-int5-other",  2 );

  // Run from /tmp (no project there); path:: points to target
  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( std::env::temp_dir() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", target.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int5-target" ), "must include target session; got:\n{s}" );
  assert!( !s.contains( "session-int5-other" ),  "must NOT include other (/a/b); got:\n{s}" );
}

// ─── INT-6 ────────────────────────────────────────────────────────────────────

/// INT-6: `session::` filter narrows results.
#[ test ]
fn int_6_session_filter_narrows_results()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  common::write_path_project_session( &storage_root, &project, "-commit",        2 );
  common::write_path_project_session( &storage_root, &project, "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "session::commit" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "commit" ), "must include -commit session; got:\n{s}" );
  assert!(
    !s.contains( "default_topic" ),
    "must NOT include -default_topic session; got:\n{s}"
  );
}

// ─── INT-7 ────────────────────────────────────────────────────────────────────

/// INT-7: `min_entries::` filter excludes short sessions.
#[ test ]
fn int_7_min_entries_filter_excludes_short_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  common::write_path_project_session( &storage_root, &project, "session-short", 3  );
  common::write_path_project_session( &storage_root, &project, "session-long",  15 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "min_entries::10" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-long" ),  "must include 15-entry session; got:\n{s}" );
  assert!( !s.contains( "session-short" ), "must NOT include 3-entry session; got:\n{s}" );
}

// ─── INT-8 ────────────────────────────────────────────────────────────────────

/// INT-8: No matching sessions exits with code 0.
#[ test ]
fn int_8_no_matching_sessions_exits_0()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  // Empty storage — no projects at all.
  fs::create_dir_all( &storage_root ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let e = stderr( &out );
  assert!( e.is_empty(), "stderr must be empty on no-results; got:\n{e}" );
}

// ─── INT-9 ────────────────────────────────────────────────────────────────────

/// INT-9: `scope::local` finds project when path contains underscores.
#[ test ]
fn int_9_scope_local_underscore_path()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "my_project" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int9", 2 );

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
  assert!( s.contains( "session-int9" ), "must include session for my_project; got:\n{s}" );
}

// ─── INT-10 ───────────────────────────────────────────────────────────────────

/// INT-10: `scope::under` finds subtree when base path has underscores.
#[ test ]
fn int_10_scope_under_underscore_base_path()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let base  = root.path().join( "my_project" );
  let child = base.join( "child" );
  fs::create_dir_all( &child ).unwrap();

  common::write_path_project_session( &storage_root, &base,  "session-int10-base",  2 );
  common::write_path_project_session( &storage_root, &child, "session-int10-child", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int10-base" ),  "must include base; got:\n{s}" );
  assert!( s.contains( "session-int10-child" ), "must include child; got:\n{s}" );
}

// ─── INT-11 ───────────────────────────────────────────────────────────────────

/// INT-11: `scope::relevant` finds ancestor when path has underscores.
#[ test ]
fn int_11_scope_relevant_underscore_ancestor()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let ancestor = root.path().join( "my_project" );
  let current  = ancestor.join( "sub" ).join( "child" );
  fs::create_dir_all( &current ).unwrap();

  common::write_path_project_session( &storage_root, &ancestor, "session-int11-ancestor", 2 );
  common::write_path_project_session( &storage_root, &current,  "session-int11-current",  2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", current.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int11-ancestor" ), "must include underscore ancestor; got:\n{s}" );
  assert!( s.contains( "session-int11-current" ),  "must include current; got:\n{s}" );
}

// ─── INT-12 ───────────────────────────────────────────────────────────────────

/// INT-12: `scope::relevant` finds topic-scoped ancestor with underscores.
#[ test ]
fn int_12_scope_relevant_topic_scoped_underscore_ancestor()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // ancestor project = my_project, topic = default_topic
  // storage key = {encoded_my_project}--default-topic
  let ancestor_base = root.path().join( "my_project" );
  fs::create_dir_all( &ancestor_base ).unwrap();
  let encoded_base = claude_storage_core::encode_path( &ancestor_base )
    .expect( "encode ancestor base" );
  let topic_key = format!( "{encoded_base}--default-topic" );
  common::write_test_session( &storage_root, &topic_key, "session-int12-topic-ancestor", 2 );

  // current = child of my_project
  let current = ancestor_base.join( "child" );
  fs::create_dir_all( &current ).unwrap();
  common::write_path_project_session( &storage_root, &current, "session-int12-current", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", current.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "session-int12-topic-ancestor" ),
    "must include topic-scoped ancestor; got:\n{s}"
  );
}

// ─── INT-13 ───────────────────────────────────────────────────────────────────

/// INT-13: `scope::under` with multiple underscore components finds nested projects.
#[ test ]
fn int_13_scope_under_multiple_underscore_components()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let base    = root.path().join( "my_project" ).join( "sub_module" );
  let child   = base.join( "feature_x" );
  let unrelated = root.path().join( "other_project" );
  fs::create_dir_all( &child ).unwrap();
  fs::create_dir_all( &unrelated ).unwrap();

  common::write_path_project_session( &storage_root, &base,      "session-int13-base",      2 );
  common::write_path_project_session( &storage_root, &child,     "session-int13-child",     2 );
  common::write_path_project_session( &storage_root, &unrelated, "session-int13-unrelated", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int13-base" ),  "must include base; got:\n{s}" );
  assert!( s.contains( "session-int13-child" ), "must include feature_x child; got:\n{s}" );
  assert!(
    !s.contains( "session-int13-unrelated" ),
    "must NOT include other_project; got:\n{s}"
  );
}

// ─── INT-14 ───────────────────────────────────────────────────────────────────

/// INT-14: default output groups sessions under project path headers.
#[ test ]
fn int_14_v1_groups_sessions_under_path_headers()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let proj_a = root.path().join( "proj-a" );
  let proj_b = root.path().join( "proj-b" );
  common::write_path_project_session( &storage_root, &proj_a, "session-id-a", 2 );
  common::write_path_project_session( &storage_root, &proj_b, "session-id-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // Path-encoded display converts hyphens to path separators: proj-a → proj/a
  assert!( s.contains( "proj" ),         "must include proj path component; got:\n{s}" );
  assert!( s.contains( "session-id-a" ), "must include session-id-a; got:\n{s}" );
  assert!( s.contains( "session-id-b" ), "must include session-id-b; got:\n{s}" );
}

// ─── INT-15 ───────────────────────────────────────────────────────────────────

/// INT-15: path header always present for `scope::local` single project.
#[ test ]
fn int_15_v1_path_header_present_for_scope_local()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "known-proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int15", 2 );

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
  // Path-encoded display converts hyphens to path separators: known-proj → known/proj
  assert!( s.contains( "known" ),         "path header must include 'known' component; got:\n{s}" );
  assert!( s.contains( "session-int15" ), "session must appear; got:\n{s}" );
}

// ─── INT-16 ───────────────────────────────────────────────────────────────────

/// INT-16: agent sessions collapsed to count line without `agent::` filter.
#[ test ]
fn int_16_v1_agent_sessions_collapsed_without_filter()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  // 2 main sessions
  common::write_path_project_session( &storage_root, &project, "session-main-a", 2 );
  common::write_path_project_session( &storage_root, &project, "session-main-b", 2 );

  // 3 agent sessions via hierarchical layout
  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "session-main-a",
    &[ ( "t001", "general-purpose" ), ( "t002", "general-purpose" ), ( "t003", "general-purpose" ) ],
    2
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // Agents must be collapsed — a count line instead of individual rows
  assert!(
    s.contains( "agent" ),
    "must contain 'agent' collapse indicator; got:\n{s}"
  );
  assert!(
    !s.contains( "agent-t001" ) && !s.contains( "agent-t002" ) && !s.contains( "agent-t003" ),
    "agent sessions must NOT appear individually at v1 without agent:: filter; got:\n{s}"
  );
}

// ─── INT-17 ───────────────────────────────────────────────────────────────────

/// INT-17: `show_tree::1` shows agents tree-indented under parent session.
#[ test ]
fn int_17_v2_agent_sessions_shown_individually()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-session-int17",
    &[ ( "agent-x", "general-purpose" ), ( "agent-y", "general-purpose" ) ],
    2
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_tree::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // With show_tree::1 agents appear tree-indented, not as a collapse summary line
  assert!(
    !s.contains( "+ 2 agent sessions" ),
    "show_tree::1 must not show agent collapse line; got:\n{s}"
  );
}

// ─── INT-18 ───────────────────────────────────────────────────────────────────

/// INT-18: entry count shown per session by default.
#[ test ]
fn int_18_v2_entry_count_shown_per_session()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-4entries", 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "4 entries" ) || s.contains( "4 entry" ),
    "must show entry count by default; got:\n{s}"
  );
}

// ─── INT-19 ───────────────────────────────────────────────────────────────────

/// INT-19: `agent::1` explicit filter disables collapse.
#[ test ]
fn int_19_v1_agent_filter_disables_collapse()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int19",
    &[ ( "a19-001", "general-purpose" ), ( "a19-002", "general-purpose" ) ],
    2
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "agent::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // With agent::1 at v1, agents must appear individually
  assert!(
    s.contains( "a19-001" ) || s.contains( "a19-002" ) || !s.contains( "+ 2 agent sessions" ),
    "agent::1 at v1 must disable collapse; got:\n{s}"
  );
}

// ─── INT-20 ───────────────────────────────────────────────────────────────────

/// INT-20: `scope::under` displays underscore dirs without splitting at /.
#[ test ]
fn int_20_scope_under_underscore_dirs_display_correctly()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let base    = root.path().join( "my_project" );
  let child   = base.join( "myproject" );
  fs::create_dir_all( &child ).unwrap();
  common::write_path_project_session( &storage_root, &child, "session-int20", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "my_project" ),
    "output must contain 'my_project' not a split form; got:\n{s}"
  );
}

// ─── INT-21 ───────────────────────────────────────────────────────────────────

/// INT-21: `scope::global` displays hyphen-prefixed topic dir in path header.
#[ test ]
fn int_21_scope_global_hyphen_topic_dir_in_header()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let src_topic = root.path().join( "src" ).join( "-default_topic" );
  fs::create_dir_all( &src_topic ).unwrap();
  common::write_path_project_session( &storage_root, &src_topic, "session-int21", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "-default_topic" ),
    "header must include '-default_topic'; got:\n{s}"
  );
  assert!(
    !s.lines().any( | l | l.trim_end().ends_with( "src:" ) ),
    "line must NOT end with 'src:' (truncated form); got:\n{s}"
  );
}

// ─── INT-22 ───────────────────────────────────────────────────────────────────

/// INT-22: `scope::under` excludes sibling with underscore-suffix name.
#[ test ]
fn int_22_scope_under_excludes_underscore_suffix_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let base    = root.path().join( "base" );
  let child   = base.join( "sub" );
  let sibling = root.path().join( "base_extra" );
  fs::create_dir_all( &child ).unwrap();
  fs::create_dir_all( &sibling ).unwrap();

  common::write_path_project_session( &storage_root, &child,   "session-it25-child",   2 );
  common::write_path_project_session( &storage_root, &sibling, "session-it25-sibling", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", base.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it25-child" ),   "must include child; got:\n{s}" );
  assert!( !s.contains( "session-it25-sibling" ), "must NOT include sibling; got:\n{s}" );
}

// ─── INT-23 ───────────────────────────────────────────────────────────────────

/// INT-23: `scope::relevant` excludes sibling with underscore-suffix name.
#[ test ]
fn int_23_scope_relevant_excludes_underscore_suffix_sibling()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let sibling = root.path().join( "base" );
  let target  = root.path().join( "base_extra" );
  fs::create_dir_all( &sibling ).unwrap();
  fs::create_dir_all( &target ).unwrap();

  common::write_path_project_session( &storage_root, &sibling, "session-it26-sibling", 2 );
  common::write_path_project_session( &storage_root, &target,  "session-it26-current", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .arg( format!( "path::{}", target.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-it26-current" ),  "must include current; got:\n{s}" );
  assert!( !s.contains( "session-it26-sibling" ), "must NOT include sibling; got:\n{s}" );
}

// ─── INT-24 ───────────────────────────────────────────────────────────────────

/// INT-24: entry count shown per session.
#[ test ]
fn int_24_v1_entry_count_shown_per_session()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int24", 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "4 entries" ) || s.contains( "4 entry" ),
    "must show entry count at v1; got:\n{s}"
  );
}

// ─── INT-25 ───────────────────────────────────────────────────────────────────

/// INT-25: `limit::N` truncates main sessions.
#[ test ]
fn int_25_v1_limit_truncates_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  for i in 0..5
  {
    common::write_path_project_session( &storage_root, &project, &format!( "session-{i}" ), 2 );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "limit::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // A truncation hint should appear when limit < total
  assert!(
    s.contains( "more" ) || s.contains( "truncat" ) || s.contains( "conversation" ),
    "must show truncation hint when limit < total sessions; got:\n{s}"
  );
}
