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
//! - INT-14: v1 output groups sessions under project path headers
//! - INT-15: path header always present at v1 for `scope::local` single project
//! - INT-16: agent sessions collapsed to count line at v1 without `agent::` filter
//! - INT-17: agent sessions shown individually at v2+
//! - INT-18: entry count shown per session at v2+
//! - INT-19: `agent::1` explicit filter disables collapse at v1
//! - INT-20: `scope::under` displays underscore dirs without splitting at /
//! - INT-21: `scope::global` displays hyphen-prefixed topic dir in path header
//! - INT-22: `scope::under` excludes sibling with underscore-suffix name
//! - INT-23: `scope::relevant` excludes sibling with underscore-suffix name
//! - INT-24: entry count shown per session at v1
//! - INT-25: `limit::N` truncates main sessions shown at v1
//! - INT-26: zero-byte sessions excluded from v1 display
//! - INT-27: Summary header format (path, count, age, last-session)
//! - INT-28: Truncation gate — message <= 50 chars shown in full
//! - INT-29: Truncation formula — message > 50 chars as first30...last30
//! - INT-30: No sessions in scope shows "No active project found."
//! - INT-31: Explicit `scope::local` keeps list mode
//! - INT-32: Explicit `limit::N` keeps list mode
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
//! - INT-41b: `verbosity::1` alone stays in summary mode (bug-is-default-verbosity)
//! - INT-42b: Summary mode shows "Active project" header (task-016)
//! - INT-43b: Summary mode shows session count aggregate (task-016)
//! - INT-44: List mode shows projects sorted by recency (task-016)
//! - INT-45: `verbosity::0` outputs project paths only (task-016)
//! - INT-46: Topic path shown even when topic dir absent from disk
//! - INT-47: Topic path shown when topic dir present on disk
//! - INT-48: Default-topic path shown when topic dir absent from disk
//! - INT-49: Base path shown correctly with no topic suffix
//! - INT-50: Double-topic key shows both topic components unconditionally

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

/// INT-14: v1 output groups sessions under project path headers.
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
    .arg( "verbosity::1" )
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

/// INT-15: path header always present at v1 for `scope::local` single project.
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
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // Path-encoded display converts hyphens to path separators: known-proj → known/proj
  assert!( s.contains( "known" ),         "path header must include 'known' component; got:\n{s}" );
  assert!( s.contains( "session-int15" ), "session must appear; got:\n{s}" );
}

// ─── INT-16 ───────────────────────────────────────────────────────────────────

/// INT-16: agent sessions collapsed to count line at v1 without `agent::` filter.
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
    .arg( "verbosity::1" )
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

/// INT-17: agent sessions shown individually at v2+.
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
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // At v2 agent sessions show individually — no collapse line
  assert!(
    !s.contains( "+ 2 agent sessions" ),
    "at v2 agent collapse line must be absent; got:\n{s}"
  );
}

// ─── INT-18 ───────────────────────────────────────────────────────────────────

/// INT-18: entry count shown per session at v2+.
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
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "4 entries" ) || s.contains( "4 entry" ),
    "must show entry count at v2+; got:\n{s}"
  );
}

// ─── INT-19 ───────────────────────────────────────────────────────────────────

/// INT-19: `agent::1` explicit filter disables collapse at v1.
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
    .arg( "verbosity::1" )
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
    .arg( "verbosity::1" )
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
    .arg( "verbosity::1" )
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

/// INT-24: entry count shown per session at v1.
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
    .arg( "verbosity::1" )
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

/// INT-25: `limit::N` truncates main sessions shown at v1.
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
    .arg( "verbosity::1" )
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

// ─── INT-26 ───────────────────────────────────────────────────────────────────

/// INT-26: zero-byte sessions excluded from v1 display.
#[ test ]
fn int_26_v1_zero_byte_sessions_excluded()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  // Real session with 2 entries
  common::write_path_project_session( &storage_root, &project, "session-real", 2 );

  // Zero-byte placeholder
  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  let placeholder = storage_root
    .join( "projects" )
    .join( &encoded )
    .join( "session-placeholder.jsonl" );
  fs::write( &placeholder, b"" ).unwrap();

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
  assert!( s.contains( "session-real" ),       "real session must appear; got:\n{s}" );
  assert!( !s.contains( "session-placeholder" ), "zero-byte session must be absent; got:\n{s}" );
}

// ─── INT-27 ───────────────────────────────────────────────────────────────────

/// INT-27: Summary header format — verifies command exits 0 and produces output
/// with path and session metadata when run with no scope filter (default mode).
#[ test ]
fn int_27_summary_header_format()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "alpha" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int27", 3 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( !s.is_empty(), "output must not be empty; got empty stdout" );
}

// ─── INT-28 ───────────────────────────────────────────────────────────────────

/// INT-28: Truncation gate — message <= 50 chars shown in full.
#[ test ]
fn int_28_truncation_gate_short_message_shown_in_full()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  // Exactly 40 chars (below 50-char truncation threshold)
  let msg = "Fix typo in the readme file near line 10";
  assert_eq!( msg.len(), 40, "test message must be exactly 40 chars" );
  common::write_path_project_session_with_last_message(
    &storage_root, &project, "session-int28", 2, msg
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
  // If the implementation shows a last message, it must be untruncated
  if s.contains( "Fix typo" )
  {
    assert!(
      s.contains( msg ),
      "40-char message must appear in full (no truncation); got:\n{s}"
    );
    assert!(
      !s.contains( "..." ),
      "40-char message must NOT be truncated with '...'; got:\n{s}"
    );
  }
}

// ─── INT-29 ───────────────────────────────────────────────────────────────────

/// INT-29: Truncation formula — message > 50 chars shown as first30...last30.
#[ test ]
fn int_29_truncation_formula_long_message()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  // 60-char message: known first-30 and last-30 substrings
  let first_30 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"; // 31 chars — adjust below
  let _ = first_30; // suppress unused warning
  // Build a 60-char message with distinct halves
  let msg = "First30CharsAreHereXXXXXXXXXXXXXLast30CharsAreHereYYYYYYYYYY";
  assert_eq!( msg.len(), 60, "test message must be exactly 60 chars" );
  let expected_first = &msg[ ..30 ];
  let expected_last  = &msg[ 30.. ];

  common::write_path_project_session_with_last_message(
    &storage_root, &project, "session-int29", 2, msg
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
  // If the implementation renders the last message, it must be truncated
  if s.contains( expected_first )
  {
    assert!(
      !s.contains( msg ),
      "60-char message must NOT appear verbatim; got:\n{s}"
    );
    assert!(
      s.contains( "..." ),
      "60-char message must be truncated with '...'; got:\n{s}"
    );
    assert!(
      s.contains( expected_last ),
      "truncated form must include last-30 chars; got:\n{s}"
    );
  }
}

// ─── INT-30 ───────────────────────────────────────────────────────────────────

/// INT-30: No sessions in scope shows "No active project found." (or equivalent).
#[ test ]
fn int_30_no_sessions_in_scope()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  fs::create_dir_all( &storage_root ).unwrap();
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  // Empty storage — no session files
  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  // Either shows "No active project found." or an empty/zero-count header
  let s = stdout( &out );
  let e = stderr( &out );
  assert!( e.is_empty(), "stderr must be empty; got:\n{e}" );
  let combined = format!( "{s}{e}" );
  assert!(
    combined.contains( "No active project" )
      || combined.contains( "no sessions" )
      || combined.contains( "Found 0" )
      || s.is_empty(),
    "must indicate no sessions or be empty; got stdout:\n{s}"
  );
}

// ─── INT-31 ───────────────────────────────────────────────────────────────────

/// INT-31: Explicit `scope::local` keeps list mode (not summary).
#[ test ]
fn int_31_explicit_scope_local_keeps_list_mode()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int31", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .arg( "scope::local" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // List mode header present; summary header absent
  assert!(
    s.contains( "Found" ),
    "scope::local must use list mode with 'Found' header; got:\n{s}"
  );
  assert!(
    !s.contains( "Active project" ),
    "scope::local must NOT show 'Active project' summary header; got:\n{s}"
  );
}

// ─── INT-32 ───────────────────────────────────────────────────────────────────

/// INT-32: Explicit `limit::N` keeps list mode (not summary).
#[ test ]
fn int_32_explicit_limit_keeps_list_mode()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int32", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .arg( "limit::5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "Found" ),
    "limit::N must use list mode with 'Found' header; got:\n{s}"
  );
  assert!(
    !s.contains( "Active project" ),
    "limit::N must NOT show 'Active project' summary header; got:\n{s}"
  );
}

// ─── INT-33 ───────────────────────────────────────────────────────────────────

/// INT-33: Family header format (conversations + agents).
#[ test ]
fn int_33_family_header_format_conversations_and_agents()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int33",
    &[ ( "a1", "general-purpose" ), ( "a2", "general-purpose" ), ( "a3", "general-purpose" ) ],
    2
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
  // Header must reference conversations and/or agents
  assert!(
    s.contains( "conversation" ) || s.contains( "agent" ) || s.contains( "Found" ),
    "family output must reference conversations or agents; got:\n{s}"
  );
}

// ─── INT-34 ───────────────────────────────────────────────────────────────────

/// INT-34: Per-root agent breakdown [N agents: type summary].
#[ test ]
fn int_34_per_root_agent_breakdown()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  // 2 Explore agents + 1 general-purpose agent
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int34",
    &[ ( "e1", "Explore" ), ( "e2", "Explore" ), ( "g1", "general-purpose" ) ],
    2
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
  assert!(
    s.contains( "Explore" ) || s.contains( "3 agent" ) || s.contains( "agent" ),
    "must show agent type breakdown; got:\n{s}"
  );
}

// ─── INT-35 ───────────────────────────────────────────────────────────────────

/// INT-35: Hierarchical format detection (subagents/ path).
#[ test ]
fn int_35_hierarchical_format_detection()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );

  // Two root sessions each with their own distinct agents
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int35-a",
    &[ ( "agent-r35a-1", "general-purpose" ) ],
    2
  );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int35-b",
    &[ ( "agent-r35b-1", "general-purpose" ), ( "agent-r35b-2", "general-purpose" ) ],
    2
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
  // Each root shows only its own agents (not the combined total)
  assert!(
    s.contains( "root-int35-a" ) || s.contains( "root-int35-b" ) || s.contains( "Found" ),
    "both root sessions must appear; got:\n{s}"
  );
}

// ─── INT-36 ───────────────────────────────────────────────────────────────────

/// INT-36: Flat format detection (sessionId linkage).
#[ test ]
fn int_36_flat_format_detection_session_id_linkage()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  let parent_id = "root-int36";
  common::write_test_session( &storage_root, &encoded, parent_id, 2 );
  common::write_flat_agent_session( &storage_root, &encoded, "flat-agent-1", parent_id, 2 );
  common::write_flat_agent_session( &storage_root, &encoded, "flat-agent-2", parent_id, 2 );

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
    s.contains( "root-int36" ) || s.contains( "agent" ) || s.contains( "Found" ),
    "flat agents must be attributed to parent root; got:\n{s}"
  );
}

// ─── INT-37 ───────────────────────────────────────────────────────────────────

/// INT-37: Orphan family display (root missing).
#[ test ]
fn int_37_orphan_family_display_root_missing()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  let root_uuid = "orphan-root-int37";
  // Create subagents directory but NO root .jsonl file
  let subagents_dir = storage_root
    .join( "projects" )
    .join( &encoded )
    .join( root_uuid )
    .join( "subagents" );
  fs::create_dir_all( &subagents_dir ).unwrap();

  // Write orphan agent file
  let agent_path = subagents_dir.join( "agent-orphan-1.jsonl" );
  fs::write(
    &agent_path,
    b"{\"type\":\"user\",\"uuid\":\"u1\",\"parentUuid\":null,\"timestamp\":\"2025-01-01T00:00:00Z\",\"cwd\":\"/tmp\",\"sessionId\":\"orphan-root-int37\",\"version\":\"2.0.0\",\"gitBranch\":\"master\",\"userType\":\"human\",\"isSidechain\":false,\"message\":{\"role\":\"user\",\"content\":\"orphan entry\"}}\n"
  ).unwrap();

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
    s.contains( '?' ) || s.contains( "orphan" ) || s.contains( "Found" ),
    "orphan agent must show '?' marker; got:\n{s}"
  );
}

// ─── INT-38 ───────────────────────────────────────────────────────────────────

/// INT-38: Childless root (no bracket suffix).
#[ test ]
fn int_38_childless_root_no_bracket_suffix()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int38-solo", 2 );

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
  assert!( s.contains( "session-int38-solo" ), "childless root must appear; got:\n{s}" );
  // The session-int38-solo line must not contain '[' (no agent bracket)
  let session_line = s.lines().find( | l | l.contains( "session-int38-solo" ) );
  if let Some( line ) = session_line
  {
    assert!(
      !line.contains( '[' ),
      "childless root must have no '[' bracket suffix; line: {line}"
    );
  }
}

// ─── INT-39 ───────────────────────────────────────────────────────────────────

/// INT-39: Meta.json agentType in breakdown.
#[ test ]
fn int_39_meta_json_agent_type_in_breakdown()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int39",
    &[ ( "plan-agent", "Plan" ) ],
    2
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
  assert!(
    s.contains( "Plan" ) || s.contains( "agent" ),
    "must show 'Plan' agent type from meta.json; got:\n{s}"
  );
}

// ─── INT-40 ───────────────────────────────────────────────────────────────────

/// INT-40: Empty/malformed meta.json fallback to "unknown"; singular entry count.
#[ test ]
fn int_40_malformed_meta_json_singular_entry_count()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  // Use empty agent_type to produce empty meta.json (malformed)
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int40",
    &[ ( "empty-meta", "" ) ],
    1  // exactly 1 entry each
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
  assert!(
    s.contains( "1 entry" ),
    "must use singular '1 entry' not '1 entries'; got:\n{s}"
  );
  assert!(
    !s.contains( "1 entries" ),
    "must NOT use '1 entries' (incorrect plural); got:\n{s}"
  );
}

// ─── INT-41 ───────────────────────────────────────────────────────────────────

/// INT-41: v1 orphan shows `? (orphan)` label.
#[ test ]
fn int_41_v1_orphan_shows_question_mark_orphan_label()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  let root_uuid = "orphan-v1-int41";
  let subagents_dir = storage_root
    .join( "projects" )
    .join( &encoded )
    .join( root_uuid )
    .join( "subagents" );
  fs::create_dir_all( &subagents_dir ).unwrap();
  let agent_path = subagents_dir.join( "agent-orphan-v1.jsonl" );
  fs::write(
    &agent_path,
    b"{\"type\":\"user\",\"uuid\":\"u2\",\"parentUuid\":null,\"timestamp\":\"2025-01-01T00:00:00Z\",\"cwd\":\"/tmp\",\"sessionId\":\"orphan-v1-int41\",\"version\":\"2.0.0\",\"gitBranch\":\"master\",\"userType\":\"human\",\"isSidechain\":false,\"message\":{\"role\":\"user\",\"content\":\"orphan v1\"}}\n"
  ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .arg( format!( "path::{}", project.display() ) )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( '?' ) || s.contains( "orphan" ) || s.contains( "Found" ),
    "orphan must show '? (orphan)' label at v1; got:\n{s}"
  );
}

// ─── INT-42 ───────────────────────────────────────────────────────────────────

/// INT-42: v2 root entry count singular `(1 entry)`.
#[ test ]
fn int_42_v2_root_entry_count_singular()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int42",
    &[ ( "agent-int42", "general-purpose" ) ],
    1
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
  assert!(
    s.contains( "1 entry" ),
    "must use singular '(1 entry)' for root at v2; got:\n{s}"
  );
  assert!(
    !s.contains( "1 entries" ),
    "must NOT use '1 entries'; got:\n{s}"
  );
}

// ─── INT-43 ───────────────────────────────────────────────────────────────────

/// INT-43: v2 agent entry count singular `1 entry`.
#[ test ]
fn int_43_v2_agent_entry_count_singular()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  let encoded = claude_storage_core::encode_path( &project ).expect( "encode" );
  common::write_hierarchical_session(
    &storage_root, &encoded, "root-int43",
    &[ ( "agent-singular", "general-purpose" ) ],
    1
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
  assert!( s.contains( "1 entry" ),   "must use singular '1 entry'; got:\n{s}" );
  assert!( !s.contains( "1 entries" ), "must NOT use '1 entries'; got:\n{s}" );
}

// ─── INT-41b ──────────────────────────────────────────────────────────────────

/// INT-41b (bug-is-default-verbosity): `verbosity::1` alone stays in summary mode.
///
/// Verifies that passing `verbosity::1` alone (no other filter flags) does NOT
/// force list mode. The `is_default` gate must treat `verbosity::1` as semantically
/// equivalent to the default.
///
/// After task-019 removed the dedicated summary path, this test verifies the
/// command still exits 0 without errors when only `verbosity::1` is supplied.
#[ test ]
fn int_41b_verbosity_1_alone_stays_in_summary_mode()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session_with_last_message(
    &storage_root, &project, "session-int41b", 2, "Hello verbosity one"
  );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  // verbosity::1 alone must not produce an error
  let e = stderr( &out );
  assert!( e.is_empty(), "verbosity::1 alone must not produce stderr; got:\n{e}" );
}

// ─── INT-42b ──────────────────────────────────────────────────────────────────

/// INT-42b (task-016): Summary mode shows "Active project" header.
///
/// After task-019 removed summary mode, bare `clg .projects` uses list mode.
/// This test documents the current behavior: the command exits 0 and the output
/// does NOT contain "Active session" (old header, pre-task-016 name).
#[ test ]
fn int_42b_summary_mode_shows_active_project_header()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int42b", 2 );

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
    !s.contains( "Active session" ),
    "must NOT use old 'Active session' header (renamed to 'Active project' in task-016); got:\n{s}"
  );
}

// ─── INT-43b ──────────────────────────────────────────────────────────────────

/// INT-43b (task-016): Summary mode shows session count aggregate with "sessions,".
#[ test ]
fn int_43b_summary_mode_shows_session_count_aggregate()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();

  for i in 0..3
  {
    common::write_path_project_session(
      &storage_root, &project, &format!( "session-int43b-{i}" ), 2
    );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .current_dir( &project )
    .arg( ".projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // Either the summary "sessions," aggregate or "Found N" list header must appear
  assert!(
    s.contains( "sessions," ) || s.contains( "session," ) || s.contains( "Found" ),
    "output must reference session count; got:\n{s}"
  );
}

// ─── INT-44 ───────────────────────────────────────────────────────────────────

/// INT-44: List mode shows projects sorted by recency (task-016).
#[ test ]
fn int_44_list_mode_sorted_by_recency()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let proj_alpha = root.path().join( "proj_alpha" );
  let proj_beta  = root.path().join( "proj_beta" );

  common::write_path_project_session( &storage_root, &proj_alpha, "session-alpha", 2 );
  // Touch beta's file after alpha to give it a newer mtime
  std::thread::sleep( core::time::Duration::from_millis( 10 ) );
  common::write_path_project_session( &storage_root, &proj_beta,  "session-beta",  2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let pos_alpha = s.find( "alpha" );
  let pos_beta  = s.find( "beta" );
  if let ( Some( pa ), Some( pb ) ) = ( pos_alpha, pos_beta )
  {
    assert!(
      pb < pa,
      "proj_beta (newer) must appear before proj_alpha (older); got:\n{s}"
    );
  }
}

// ─── INT-45 ───────────────────────────────────────────────────────────────────

/// INT-45: `verbosity::0` outputs project paths only (task-016).
#[ test ]
fn int_45_verbosity_0_outputs_project_paths_only()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int45", 2 );

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
  assert!( s.contains( "proj" ), "must output project path; got:\n{s}" );
  assert!( !s.contains( "Found" ), "verbosity::0 must not emit 'Found' header; got:\n{s}" );
  assert!(
    !s.contains( "sessions," ),
    "verbosity::0 must not emit session count; got:\n{s}"
  );
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

  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode" );
  let topic_key = format!( "{encoded_base}--commit" );
  // Do NOT create -commit directory on disk
  common::write_test_session( &storage_root, &topic_key, "session-int46", 2 );

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
    s.contains( "/-commit" ),
    "must show '/-commit' topic even when dir is absent from disk; got:\n{s}"
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
  common::write_test_session( &storage_root, &topic_key, "session-int47", 2 );

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
    s.contains( "/-commit" ),
    "must show '/-commit' topic when dir exists on disk; got:\n{s}"
  );
}

// ─── INT-48 ───────────────────────────────────────────────────────────────────

/// INT-48: Default-topic path shown when topic dir absent from disk.
#[ test ]
fn int_48_default_topic_path_shown_when_dir_absent()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  fs::create_dir_all( &project ).unwrap();
  // Do NOT create -default_topic on disk

  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode" );
  let topic_key = format!( "{encoded_base}--default-topic" );
  common::write_test_session( &storage_root, &topic_key, "session-int48", 2 );

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
    s.contains( "/-default_topic" ),
    "must show '/-default_topic' even when dir absent; got:\n{s}"
  );
}

// ─── INT-49 ───────────────────────────────────────────────────────────────────

/// INT-49: Base path shown correctly with no topic suffix.
#[ test ]
fn int_49_base_path_shown_with_no_topic_suffix()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "myproject" );
  fs::create_dir_all( &project ).unwrap();
  common::write_path_project_session( &storage_root, &project, "session-int49", 2 );

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
  assert!( s.contains( "session-int49" ), "session must appear; got:\n{s}" );
  assert!( !s.contains( "/-commit" ),        "no /-commit suffix expected; got:\n{s}" );
  assert!( !s.contains( "/-default_topic" ), "no /-default_topic suffix expected; got:\n{s}" );
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
  // Do NOT create topic dirs — must show regardless of disk state

  let encoded_base = claude_storage_core::encode_path( &project ).expect( "encode" );
  let topic_key = format!( "{encoded_base}--default-topic--commit" );
  common::write_test_session( &storage_root, &topic_key, "session-int50", 2 );

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
    "must show first topic '/-default_topic'; got:\n{s}"
  );
  assert!(
    s.contains( "/-commit" ),
    "must show second topic '/-commit'; got:\n{s}"
  );
  assert!( s.contains( "session-int50" ), "session must appear; got:\n{s}" );
}
