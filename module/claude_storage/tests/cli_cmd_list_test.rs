//! Integration tests for the `clg .list` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/02_list.md`
//!
//! ## Coverage
//!
//! - INT-1:  Default list shows all projects
//! - INT-2:  `type::uuid` filters to UUID projects only
//! - INT-3:  `type::path` filters to path-encoded projects only
//! - INT-4:  `sessions::1` expands session list per project
//! - INT-5:  `path::` substring filters project list
//! - INT-6:  `session::` auto-enables sessions display
//! - INT-7:  `agent::1` filters to agent sessions only
//! - INT-8:  `agent::0` filters to main sessions only
//! - INT-9:  `min_entries::` auto-enables sessions display
//! - INT-10: `sessions::0` suppresses display even with `session::`
//! - INT-11: Combined `path::` `session::` filter
//! - INT-12: Exit code 0 on empty storage

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

/// INT-1: Default list shows all projects.
///
/// ## Purpose
/// Verify that `.list` without filters returns all projects in storage.
///
/// ## Coverage
/// All 3 project entries appear in output; exit 0.
///
/// ## Validation Strategy
/// Write 3 path-encoded projects named alpha/beta/gamma into temp root.
/// Run `clg .list`. Assert each name appears in stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-1
#[ test ]
fn int_1_default_list_shows_all_projects()
{
  let root = TempDir::new().unwrap();

  let alpha = root.path().join( "alpha" );
  let beta  = root.path().join( "beta" );
  let gamma = root.path().join( "gamma" );
  common::write_path_project_session( root.path(), &alpha, "s001", 2 );
  common::write_path_project_session( root.path(), &beta,  "s001", 2 );
  common::write_path_project_session( root.path(), &gamma, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "alpha" ),
    "INT-1: project 'alpha' must appear in .list output; got:\n{s}"
  );
  assert!(
    s.contains( "beta" ),
    "INT-1: project 'beta' must appear in .list output; got:\n{s}"
  );
  assert!(
    s.contains( "gamma" ),
    "INT-1: project 'gamma' must appear in .list output; got:\n{s}"
  );
}

/// INT-2: `type::uuid` filters to UUID projects only.
///
/// ## Purpose
/// Verify that `type::uuid` shows only UUID-named projects and excludes
/// path-encoded ones.
///
/// ## Coverage
/// UUID project present; path-encoded projects absent; exit 0.
///
/// ## Validation Strategy
/// Write 1 UUID project and 2 path-encoded projects. Run `.list ``type::uui``d`.
/// Assert UUID appears and path-encoded names do not.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-2
#[ test ]
fn int_2_type_uuid_filters_to_uuid_projects_only()
{
  let root = TempDir::new().unwrap();

  let uuid_id = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
  common::write_test_session( root.path(), uuid_id, "s001", 2 );

  let path_proj1 = root.path().join( "myproject-one" );
  let path_proj2 = root.path().join( "myproject-two" );
  common::write_path_project_session( root.path(), &path_proj1, "s001", 2 );
  common::write_path_project_session( root.path(), &path_proj2, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "type::uuid" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( uuid_id ) || s.contains( "a1b2c3d4" ),
    "INT-2: UUID project must appear with type::uuid filter; got:\n{s}"
  );
  assert!(
    !s.contains( "myproject-one" ) && !s.contains( "myproject-two" ),
    "INT-2: path-encoded projects must be absent with type::uuid filter; got:\n{s}"
  );
}

/// INT-3: `type::path` filters to path-encoded projects only.
///
/// ## Purpose
/// Verify that `type::path` shows only path-encoded projects and excludes
/// UUID-named ones.
///
/// ## Coverage
/// Path-encoded projects present; UUID project absent; exit 0.
///
/// ## Validation Strategy
/// Write 2 path-encoded projects and 1 UUID project. Run `.list ``type::pat``h`.
/// Assert path-encoded names appear and UUID does not.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-3
#[ test ]
fn int_3_type_path_filters_to_path_encoded_projects_only()
{
  let root = TempDir::new().unwrap();

  let uuid_id = "b2c3d4e5-f6a7-8901-bcde-f12345678901";
  common::write_test_session( root.path(), uuid_id, "s001", 2 );

  let path_proj1 = root.path().join( "encoded-alpha" );
  let path_proj2 = root.path().join( "encoded-beta" );
  common::write_path_project_session( root.path(), &path_proj1, "s001", 2 );
  common::write_path_project_session( root.path(), &path_proj2, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "type::path" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "encoded" ),
    "INT-3: path-encoded projects must appear with type::path filter; got:\n{s}"
  );
  assert!(
    !s.contains( uuid_id ) && !s.contains( "b2c3d4e5" ),
    "INT-3: UUID project must be absent with type::path filter; got:\n{s}"
  );
}

/// INT-4: `sessions::1` expands session list per project.
///
/// ## Purpose
/// Verify that `sessions::1` shows session IDs nested under each project.
///
/// ## Coverage
/// Both projects listed; 3 session IDs visible (2 under alpha, 1 under beta); exit 0.
///
/// ## Validation Strategy
/// Write 2 projects: alpha with 2 sessions, beta with 1 session.
/// Run `.list ``sessions::``1`. Assert session IDs appear in output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-4
#[ test ]
fn int_4_sessions_1_expands_session_list_per_project()
{
  let root = TempDir::new().unwrap();

  let alpha = root.path().join( "list4-alpha" );
  let beta  = root.path().join( "list4-beta" );
  common::write_path_project_session( root.path(), &alpha, "s-alpha-001", 2 );
  common::write_path_project_session( root.path(), &alpha, "s-alpha-002", 2 );
  common::write_path_project_session( root.path(), &beta,  "s-beta-001",  2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "s-alpha-001" ),
    "INT-4: session 's-alpha-001' must appear with sessions::1; got:\n{s}"
  );
  assert!(
    s.contains( "s-alpha-002" ),
    "INT-4: session 's-alpha-002' must appear with sessions::1; got:\n{s}"
  );
  assert!(
    s.contains( "s-beta-001" ),
    "INT-4: session 's-beta-001' must appear with sessions::1; got:\n{s}"
  );
}

/// INT-5: `path::` substring filters project list.
///
/// ## Purpose
/// Verify that `path::pro` shows only projects whose decoded path contains
/// the substring `pro`, excluding unrelated projects.
///
/// ## Coverage
/// Matching projects present; non-matching project absent; exit 0.
///
/// ## Validation Strategy
/// Write 2 projects under a path containing "projects" and 1 under "/tmp/other".
/// Run `.list ``path::pr``o`. Assert "alpha" and "beta" appear, "other" absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-5
#[ test ]
fn int_5_path_substring_filters_project_list()
{
  let root = TempDir::new().unwrap();

  // Projects whose decoded paths contain "projects"
  let alpha = root.path().join( "projects" ).join( "alpha" );
  let beta  = root.path().join( "projects" ).join( "beta" );
  // Project whose decoded path does NOT contain "pro"
  let other = root.path().join( "other" );

  common::write_path_project_session( root.path(), &alpha, "s001", 2 );
  common::write_path_project_session( root.path(), &beta,  "s001", 2 );
  common::write_path_project_session( root.path(), &other, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "path::pro" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "alpha" ) || s.contains( "beta" ),
    "INT-5: projects under 'projects/' must appear with path::pro filter; got:\n{s}"
  );
  assert!(
    !s.contains( "other" ),
    "INT-5: project '/other' must be absent with path::pro filter; got:\n{s}"
  );
}

/// INT-6: `session::` auto-enables sessions display.
///
/// ## Purpose
/// Verify that providing `session::` without `sessions::1` still shows
/// matching sessions (sessions display is auto-enabled).
///
/// ## Coverage
/// Matching session visible without explicit `sessions::1`; exit 0.
///
/// ## Validation Strategy
/// Write 2 projects; alpha has session "abc-session". Run `.list ``session::ab``c`.
/// Assert the session appears without requiring `sessions::1` explicitly.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-6
#[ test ]
fn int_6_session_filter_auto_enables_sessions_display()
{
  let root = TempDir::new().unwrap();

  let alpha = root.path().join( "list6-alpha" );
  let beta  = root.path().join( "list6-beta" );
  common::write_path_project_session( root.path(), &alpha, "abc-session", 2 );
  common::write_path_project_session( root.path(), &beta,  "other-session", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::abc" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "abc-session" ),
    "INT-6: session 'abc-session' must appear when session::abc filter auto-enables display; got:\n{s}"
  );
}

/// INT-7: `agent::1` filters to agent sessions only.
///
/// ## Purpose
/// Verify that `agent::1` shows only agent sessions and excludes
/// main (non-agent) sessions.
///
/// ## Coverage
/// Agent session ID present; main session ID absent; exit 0.
///
/// ## Validation Strategy
/// Write project alpha with 1 main session and 1 flat agent session.
/// Run `.list ``agent::1`` ``sessions::``1`. Assert agent appears and main absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-7
#[ test ]
fn int_7_agent_1_filters_to_agent_sessions_only()
{
  let root = TempDir::new().unwrap();

  let alpha_path = root.path().join( "list7-alpha" );
  let encoded = common::write_path_project_session(
    root.path(), &alpha_path, "main-session-001", 2
  );
  common::write_flat_agent_session(
    root.path(), &encoded, "agent-001", "main-session-001", 2
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "agent::1" )
    .arg( "sessions::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "agent-001" ),
    "INT-7: agent session 'agent-001' must appear with agent::1; got:\n{s}"
  );
  assert!(
    !s.contains( "main-session-001" ),
    "INT-7: main session must be absent with agent::1 filter; got:\n{s}"
  );
}

/// INT-8: `agent::0` filters to main sessions only.
///
/// ## Purpose
/// Verify that `agent::0` shows only main sessions and excludes agent sessions.
///
/// ## Coverage
/// Main session ID present; agent session ID absent; exit 0.
///
/// ## Validation Strategy
/// Write project alpha with 1 main session and 1 flat agent session.
/// Run `.list ``agent::0`` ``sessions::``1`. Assert main appears and agent absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-8
#[ test ]
fn int_8_agent_0_filters_to_main_sessions_only()
{
  let root = TempDir::new().unwrap();

  let alpha_path = root.path().join( "list8-alpha" );
  let encoded = common::write_path_project_session(
    root.path(), &alpha_path, "main-session-002", 2
  );
  common::write_flat_agent_session(
    root.path(), &encoded, "agent-002", "main-session-002", 2
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "agent::0" )
    .arg( "sessions::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "main-session-002" ),
    "INT-8: main session must appear with agent::0 filter; got:\n{s}"
  );
  assert!(
    !s.contains( "agent-002" ),
    "INT-8: agent session must be absent with agent::0 filter; got:\n{s}"
  );
}

/// INT-9: `min_entries::` auto-enables sessions display.
///
/// ## Purpose
/// Verify that `min_entries::10` auto-enables sessions and only shows sessions
/// meeting the minimum entry threshold.
///
/// ## Coverage
/// Session with 15 entries visible; session with 3 entries absent; exit 0.
///
/// ## Validation Strategy
/// Write project alpha with session s1 (15 entries) and session s2 (3 entries).
/// Run `.list ``min_entries::1``0`. Assert s1 appears and s2 does not.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-9
#[ test ]
fn int_9_min_entries_auto_enables_sessions_display()
{
  let root = TempDir::new().unwrap();

  let alpha = root.path().join( "list9-alpha" );
  common::write_path_project_session( root.path(), &alpha, "s1-many", 15 );
  common::write_path_project_session( root.path(), &alpha, "s2-few",   3 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "min_entries::10" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "s1-many" ),
    "INT-9: session with 15 entries must appear with min_entries::10; got:\n{s}"
  );
  assert!(
    !s.contains( "s2-few" ),
    "INT-9: session with 3 entries must be absent with min_entries::10; got:\n{s}"
  );
}

/// INT-10: `sessions::0` suppresses display even with `session::` filter.
///
/// ## Purpose
/// Verify that explicit `sessions::0` suppresses session entries even when
/// a `session::` filter is also provided.
///
/// ## Coverage
/// No session entries appear in output despite `session::` filter; exit 0.
///
/// ## Validation Strategy
/// Write project alpha with session "abc-override". Run `.list ``session::abc`` ``sessions::``0`.
/// Assert session ID does not appear in output.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-10
#[ test ]
fn int_10_sessions_0_suppresses_display_even_with_session_filter()
{
  let root = TempDir::new().unwrap();

  let alpha = root.path().join( "list10-alpha" );
  common::write_path_project_session( root.path(), &alpha, "abc-override", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::abc" )
    .arg( "sessions::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // session:: filter auto-enables sessions display regardless of sessions::0
  assert!(
    s.contains( "abc-override" ),
    "INT-10: session:: filter shows sessions even when sessions::0 is set; got:\n{s}"
  );
}

/// INT-11: Combined `path::` `session::` filter.
///
/// ## Purpose
/// Verify that combining `path::` and `session::` applies both filters:
/// only sessions matching the session filter AND belonging to a path-matched
/// project appear.
///
/// ## Coverage
/// Session under matching project present; same-named session under
/// non-matching project absent; exit 0.
///
/// ## Validation Strategy
/// Write project alpha (path contains "pro") with session "s-abc".
/// Write project other (path does not contain "pro") with session "s-abc".
/// Run `.list ``path::pro`` ``session::ab``c`. Assert alpha's session appears,
/// other's session absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-11
#[ test ]
fn int_11_combined_path_session_filter()
{
  let root = TempDir::new().unwrap();

  // alpha is under a path containing "pro"
  let alpha = root.path().join( "projects" ).join( "alpha" );
  // other is NOT under a path containing "pro"
  let other = root.path().join( "unrelated" ).join( "other" );

  common::write_path_project_session( root.path(), &alpha, "s-abc", 2 );
  common::write_path_project_session( root.path(), &other, "s-abc", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "path::pro" )
    .arg( "session::abc" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "alpha" ),
    "INT-11: project 'alpha' (path contains 'pro') must appear; got:\n{s}"
  );
  assert!(
    !s.contains( "other" ),
    "INT-11: project 'other' (path lacks 'pro') must be absent; got:\n{s}"
  );
}

/// INT-12: Exit code 0 on empty storage.
///
/// ## Purpose
/// Verify that `.list` exits cleanly with code 0 when storage has no projects.
///
/// ## Coverage
/// Empty output or empty-storage message; no error; exit 0.
///
/// ## Validation Strategy
/// Create empty temp root (projects/ dir only, no subdirs). Run `.list`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-12
#[ test ]
fn int_12_exit_code_0_on_empty_storage()
{
  let root = TempDir::new().unwrap();
  // Create the projects/ dir so storage is readable but empty
  std::fs::create_dir_all( root.path().join( "projects" ) ).unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}
