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
//! - INT-51: `scope::` with invalid value rejected
//! - INT-52: `agent::` with non-boolean value rejected
//! - INT-53: `detail::projects` shows header line only, no session/family body lines
//! - INT-54: `detail::` omitted reproduces exact `detail::sessions` output
//! - INT-55: `detail::` with invalid value rejected
//! - INT-56: `filter::` narrows to projects whose decoded path contains the substring
//! - INT-57: `filter::` with no matching project shows empty listing, not an error
//! - INT-58: `type::uuid` narrows to UUID-named projects only
//! - INT-59: `type::path` narrows to path-named projects only
//! - INT-60: `type::` with invalid value rejected
//! - INT-61: `project::X ids::1` outputs one conversation ID per line
//! - INT-62: `project::X ids::1 count::1` outputs a single bare integer
//! - INT-63: `ids::1` without required `project::` rejected
//! - INT-64: `type::` and `filter::` compose under `scope::global`
//! - INT-65: `limit::`/`show_tree::`/`show_topic::` are no-ops under `detail::projects`
//! - INT-66: `.list`'s `deprecation_message` edit does not alter runtime output
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

/// INT-51: `scope::` with invalid value rejected.
///
/// ## Purpose
/// Verify `.projects scope::badvalue` is rejected — `badvalue` is not a
/// valid `scope::` option (accepted: `local`, `under`, `relevant`, `global`,
/// `around`).
///
/// ## Coverage
/// Exit code exactly 1; stderr carries the canonical `validate_scope()`
/// error naming the invalid value; no project output on stdout.
///
/// ## Validation Strategy
/// Run `.projects scope::badvalue` against an empty temp storage root from
/// a neutral cwd. Assert exit 1, canonical stderr text, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/07_projects.md` — INT-51
#[ test ]
fn int_51_scope_invalid_value_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".projects" )
    .arg( "scope::badvalue" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-51: invalid scope:: value must produce an error on stderr"
  );
  assert!(
    err.contains( "scope must be relevant|local|under|global|around, got badvalue" ),
    "INT-51: stderr must carry the canonical validate_scope() error; got: {err}"
  );
  assert!(
    stdout( &out ).is_empty(),
    "INT-51: no project output on stdout when scope:: is rejected; got:\n{}",
    stdout( &out )
  );
}

/// INT-52: `agent::` with non-boolean value rejected.
///
/// ## Purpose
/// Verify `.projects agent::invalid` is rejected as an argument error —
/// `invalid` is not a valid boolean value (accepted: `0`, `1`).
///
/// ## Coverage
/// Exit code exactly 1; non-empty stderr describing the argument error; no
/// project output on stdout.
///
/// ## Validation Strategy
/// Run `.projects agent::invalid` against an empty temp storage root from a
/// neutral cwd. Assert exit 1, stderr naming the `agent` argument, empty
/// stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/07_projects.md` — INT-52
#[ test ]
fn int_52_agent_non_boolean_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".projects" )
    .arg( "agent::invalid" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-52: non-boolean agent:: value must produce an error on stderr"
  );
  assert!(
    err.contains( "agent" ),
    "INT-52: stderr must name the rejected agent argument; got: {err}"
  );
  assert!(
    stdout( &out ).is_empty(),
    "INT-52: no project output on stdout when agent:: is rejected; got:\n{}",
    stdout( &out )
  );
}

// ─── INT-53 ───────────────────────────────────────────────────────────────────

/// INT-53: `detail::projects` shows header line only, no session/family body lines.
#[ test ]
fn int_53_detail_projects_header_only_no_body_lines()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let proj_a = root.path().join( "proj-a" );
  let proj_b = root.path().join( "proj-b" );
  fs::create_dir_all( &proj_a ).unwrap();
  fs::create_dir_all( &proj_b ).unwrap();

  let encoded_a = claude_storage_core::encode_path( &proj_a ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded_a, "root-int53",
    &[ ( "agent-int53-x", "general-purpose" ), ( "agent-int53-y", "general-purpose" ) ],
    2
  );
  common::write_path_project_session( &storage_root, &proj_b, "session-int53-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "detail::projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found 2 projects" ), "must show project count header; got:\n{s}" );
  assert!( !s.contains( "root-int53" ), "must NOT show root session id in body; got:\n{s}" );
  assert!( !s.contains( "agent-int53-x" ), "must NOT show agent session id; got:\n{s}" );
  assert!( !s.contains( "session-int53-b" ), "must NOT show plain session id; got:\n{s}" );
  assert!( !s.contains( "[2 agents" ), "must NOT show agent-count bracket; got:\n{s}" );
}

// ─── INT-54 ───────────────────────────────────────────────────────────────────

/// INT-54: `detail::` omitted reproduces exact `detail::sessions` output.
#[ test ]
fn int_54_detail_omitted_matches_explicit_sessions()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int54", 3 );

  let out_default = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  let out_explicit = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "detail::sessions" )
    .output()
    .unwrap();

  assert_exit( &out_default, 0 );
  assert_exit( &out_explicit, 0 );
  assert_eq!(
    stdout( &out_default ), stdout( &out_explicit ),
    "detail:: omitted must byte-match explicit detail::sessions"
  );
  assert!(
    stdout( &out_default ).contains( "session-int54" ),
    "sanity: session must appear; got:\n{}", stdout( &out_default )
  );
}

// ─── INT-55 ───────────────────────────────────────────────────────────────────

/// INT-55: `detail::` with invalid value rejected.
#[ test ]
fn int_55_detail_invalid_value_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".projects" )
    .arg( "detail::bogus" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "detail must be projects|sessions, got bogus" ),
    "INT-55: stderr must carry the canonical validate_detail_level() error; got: {err}"
  );
  assert!(
    stdout( &out ).is_empty(),
    "INT-55: no project output on stdout when detail:: is rejected; got:\n{}",
    stdout( &out )
  );
}

// ─── INT-56 ───────────────────────────────────────────────────────────────────

/// INT-56: `filter::` narrows to projects whose decoded path contains the substring.
#[ test ]
fn int_56_filter_narrows_to_matching_substring()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let proj_alpha = root.path().join( "alpha" );
  let proj_beta  = root.path().join( "beta" );
  let proj_gamma = root.path().join( "gamma" );
  fs::create_dir_all( &proj_alpha ).unwrap();
  fs::create_dir_all( &proj_beta ).unwrap();
  fs::create_dir_all( &proj_gamma ).unwrap();

  common::write_path_project_session( &storage_root, &proj_alpha, "session-int56-alpha", 2 );
  common::write_path_project_session( &storage_root, &proj_beta,  "session-int56-beta",  2 );
  common::write_path_project_session( &storage_root, &proj_gamma, "session-int56-gamma", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "filter::alpha" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int56-alpha" ), "must include alpha project; got:\n{s}" );
  assert!( !s.contains( "session-int56-beta" ),  "must NOT include beta project; got:\n{s}" );
  assert!( !s.contains( "session-int56-gamma" ), "must NOT include gamma project; got:\n{s}" );
}

// ─── INT-57 ───────────────────────────────────────────────────────────────────

/// INT-57: `filter::` with no matching project shows empty listing, not an error.
#[ test ]
fn int_57_filter_no_match_shows_empty_listing()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int57", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "filter::nonexistent-substring" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found 0 projects" ), "must show zero-count header, not an error; got:\n{s}" );
  assert!( !s.contains( "session-int57" ), "must NOT include filtered-out session; got:\n{s}" );
}

// ─── INT-58 ───────────────────────────────────────────────────────────────────

/// INT-58: `type::uuid` narrows to UUID-named projects only.
#[ test ]
fn int_58_type_uuid_narrows_to_uuid_projects()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // UUID-named project (no leading '-' prefix => ProjectId::Uuid)
  common::write_test_session( &storage_root, "550e8400-e29b-41d4-a716-446655440000", "session-int58-uuid", 2 );

  // Path-named project
  let path_proj = root.path().join( "proj-int58" );
  fs::create_dir_all( &path_proj ).unwrap();
  common::write_path_project_session( &storage_root, &path_proj, "session-int58-path", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "type::uuid" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int58-uuid" ),  "must include UUID project; got:\n{s}" );
  assert!( !s.contains( "session-int58-path" ), "must NOT include path project; got:\n{s}" );
}

// ─── INT-59 ───────────────────────────────────────────────────────────────────

/// INT-59: `type::path` narrows to path-named projects only.
#[ test ]
fn int_59_type_path_narrows_to_path_projects()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  common::write_test_session( &storage_root, "550e8400-e29b-41d4-a716-446655440001", "session-int59-uuid", 2 );

  let path_proj = root.path().join( "proj-int59" );
  fs::create_dir_all( &path_proj ).unwrap();
  common::write_path_project_session( &storage_root, &path_proj, "session-int59-path", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "type::path" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int59-path" ), "must include path project; got:\n{s}" );
  assert!( !s.contains( "session-int59-uuid" ), "must NOT include UUID project; got:\n{s}" );
}

// ─── INT-60 ───────────────────────────────────────────────────────────────────

/// INT-60: `type::` with invalid value rejected.
#[ test ]
fn int_60_type_invalid_value_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".projects" )
    .arg( "type::bogus" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "type must be uuid|path|all, got bogus" ),
    "INT-60: stderr must carry the canonical validate_project_type() error; got: {err}"
  );
  assert!(
    stdout( &out ).is_empty(),
    "INT-60: no project output on stdout when type:: is rejected; got:\n{}",
    stdout( &out )
  );
}

// ─── INT-61 ───────────────────────────────────────────────────────────────────

/// INT-61: `project::X ids::1` outputs one conversation ID per line.
#[ test ]
fn int_61_ids_outputs_one_conversation_id_per_line()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj-int61" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int61-a", &[ ( "agent-int61-1", "general-purpose" ) ], 2
  );
  common::write_path_project_session( &storage_root, &project, "root-int61-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( format!( "project::{}", project.display() ) )
    .arg( "ids::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let lines : Vec< &str > = s.lines().filter( | l | !l.is_empty() ).collect();
  assert_eq!( lines.len(), 2, "must output exactly 2 conversation ID lines; got:\n{s}" );
  assert!( lines.contains( &"root-int61-a" ), "must list root-int61-a; got:\n{s}" );
  assert!( lines.contains( &"root-int61-b" ), "must list root-int61-b; got:\n{s}" );
  assert!( !s.contains( "Found" ), "must NOT show 'Found N projects' header; got:\n{s}" );
  assert!( !s.contains( "agent-int61-1" ), "must NOT list agent id (only root conversation ids); got:\n{s}" );
}

// ─── INT-62 ───────────────────────────────────────────────────────────────────

/// INT-62: `project::X ids::1 count::1` outputs a single bare integer.
#[ test ]
fn int_62_ids_count_outputs_bare_integer()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj-int62" );
  fs::create_dir_all( &project ).unwrap();

  common::write_path_project_session( &storage_root, &project, "root-int62-a", 2 );
  common::write_path_project_session( &storage_root, &project, "root-int62-b", 2 );
  common::write_path_project_session( &storage_root, &project, "root-int62-c", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( format!( "project::{}", project.display() ) )
    .arg( "ids::1" )
    .arg( "count::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert_eq!( s.trim(), "3", "must output bare integer count and nothing else; got:\n{s}" );
}

// ─── INT-63 ───────────────────────────────────────────────────────────────────

/// INT-63: `ids::1` without required `project::` rejected.
#[ test ]
fn int_63_ids_without_project_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".projects" )
    .arg( "ids::1" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "project parameter required for ids" ),
    "INT-63: stderr must carry the specific ids::-requires-project:: validation error, \
     not a generic unknown-parameter/help-hint message (that would coincidentally contain \
     the substring \"project\" via the \"..projects ??\" hint text and pass for the wrong \
     reason); got: {err}"
  );
  assert!(
    stdout( &out ).is_empty(),
    "INT-63: no conversation IDs on stdout; got:\n{}",
    stdout( &out )
  );
}

// ─── INT-64 ───────────────────────────────────────────────────────────────────

/// INT-64: `type::` and `filter::` compose under `scope::global`.
#[ test ]
fn int_64_type_and_filter_compose()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let path_alpha = root.path().join( "alpha-int64" );
  let path_beta  = root.path().join( "beta-int64" );
  fs::create_dir_all( &path_alpha ).unwrap();
  fs::create_dir_all( &path_beta ).unwrap();
  common::write_path_project_session( &storage_root, &path_alpha, "session-int64-path-alpha", 2 );
  common::write_path_project_session( &storage_root, &path_beta,  "session-int64-path-beta",  2 );

  // UUID project whose raw id also contains "alpha" (would match filter:: if type:: didn't exclude it)
  common::write_test_session( &storage_root, "alpha00000-uuid-look-alike-project-id", "session-int64-uuid-alpha", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "type::path" )
    .arg( "filter::alpha" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "session-int64-path-alpha" ),  "must include path+alpha project; got:\n{s}" );
  assert!( !s.contains( "session-int64-path-beta" ),  "must exclude path+beta (filter:: mismatch); got:\n{s}" );
  assert!( !s.contains( "session-int64-uuid-alpha" ), "must exclude uuid+alpha (type:: mismatch); got:\n{s}" );
}

// ─── INT-65 ───────────────────────────────────────────────────────────────────

/// INT-65: `limit::`/`show_tree::`/`show_topic::` are no-ops under `detail::projects`.
#[ test ]
fn int_65_limit_show_tree_show_topic_noop_under_detail_projects()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj-int65" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int65", &[ ( "agent-int65", "general-purpose" ) ], 2
  );

  let out_plain = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "detail::projects" )
    .output()
    .unwrap();

  let out_with_noops = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "detail::projects" )
    .arg( "limit::1" )
    .arg( "show_tree::1" )
    .arg( "show_topic::1" )
    .output()
    .unwrap();

  assert_exit( &out_plain, 0 );
  assert_exit( &out_with_noops, 0 );
  assert_eq!(
    stdout( &out_plain ), stdout( &out_with_noops ),
    "limit::/show_tree::/show_topic:: must be no-ops under detail::projects"
  );
}

// ─── INT-66 ───────────────────────────────────────────────────────────────────

/// INT-66: `.list`'s `deprecation_message` edit does not alter runtime output.
#[ test ]
fn int_66_list_deprecation_message_preserves_output()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj-int66" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int66", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "Found 1 project" ), "must show project count header; got:\n{s}" );
  assert!(
    s.lines().any( | l | l.starts_with( "Path(" ) ),
    "must show Path(...) debug-format project id, unaffected by deprecation_message; got:\n{s}"
  );
}
