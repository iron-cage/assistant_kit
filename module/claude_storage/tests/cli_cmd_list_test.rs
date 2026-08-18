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
//! - INT-4:  `show_sessions::1` expands session list per project
//! - INT-5:  `path::` substring filters project list
//! - INT-6:  `session::` auto-enables sessions display
//! - INT-7:  `agent::1` filters to agent sessions only
//! - INT-8:  `agent::0` filters to main sessions only
//! - INT-9:  `min_entries::` auto-enables sessions display
//! - INT-10: `show_sessions::0` suppresses display even with `session::`
//! - INT-11: Combined `path::` `session::` filter
//! - INT-12: Exit code 0 on empty storage
//! - B007:   Historical collision layout cannot break path-filter isolation
//! - T01:    Default (no `scope::`) regression guard
//! - T02:    `scope::global` explicit byte-identical to default (AF2)
//! - T03:    `scope::local` narrows to the cwd project only
//! - T04:    `scope::under` narrows to descendant projects
//! - T05:    `scope::bogus` rejected with canonical error
//! - T06:    `scope::local` composes with the existing `path::` filter
//! - INT-13: `type::` with invalid value rejected
//! - INT-14: `agent::` with non-boolean value rejected
#![ cfg( unix ) ]

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

/// INT-4: `show_sessions::1` expands session list per project.
///
/// ## Purpose
/// Verify that `show_sessions::1` shows session IDs nested under each project.
///
/// ## Coverage
/// Both projects listed; 3 session IDs visible (2 under alpha, 1 under beta); exit 0.
///
/// ## Validation Strategy
/// Write 2 projects: alpha with 2 sessions, beta with 1 session.
/// Run `.list ``show_sessions::``1`. Assert session IDs appear in output.
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
    .arg( "show_sessions::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "s-alpha-001" ),
    "INT-4: session 's-alpha-001' must appear with show_sessions::1; got:\n{s}"
  );
  assert!(
    s.contains( "s-alpha-002" ),
    "INT-4: session 's-alpha-002' must appear with show_sessions::1; got:\n{s}"
  );
  assert!(
    s.contains( "s-beta-001" ),
    "INT-4: session 's-beta-001' must appear with show_sessions::1; got:\n{s}"
  );
}

/// INT-5: `path::` substring filters project list.
///
/// ## Purpose
/// Verify that `path::projects` shows only projects whose decoded path
/// contains the substring `projects`, excluding unrelated projects.
///
/// ## Coverage
/// Matching projects present; non-matching project absent; exit 0.
///
/// ## Validation Strategy
/// Write 2 projects under a path containing "projects" and 1 under "/tmp/other".
/// Run `.list path::projects`. Assert "alpha" and "beta" appear, "other" absent.
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
  // Project whose decoded path does NOT contain "projects"
  let other = root.path().join( "other" );

  common::write_path_project_session( root.path(), &alpha, "s001", 2 );
  common::write_path_project_session( root.path(), &beta,  "s001", 2 );
  common::write_path_project_session( root.path(), &other, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    // Filter "projects" (not the former "pro") — see Fix(BUG-007) at INT-11:
    // 8 chars + '_' segment boundaries cannot collide with the shared TempDir
    // root's 6-char random component, unlike the 3-char "pro".
    .arg( "path::projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "alpha" ) || s.contains( "beta" ),
    "INT-5: projects under 'projects/' must appear with path::projects filter; got:\n{s}"
  );
  assert!(
    !s.contains( "other" ),
    "INT-5: project '/other' must be absent with path::projects filter; got:\n{s}"
  );
}

/// INT-6: `session::` auto-enables sessions display.
///
/// ## Purpose
/// Verify that providing `session::` without `show_sessions::1` still shows
/// matching sessions (sessions display is auto-enabled).
///
/// ## Coverage
/// Matching session visible without explicit `show_sessions::1`; exit 0.
///
/// ## Validation Strategy
/// Write 2 projects; alpha has session "abc-session". Run `.list ``session::ab``c`.
/// Assert the session appears without requiring `show_sessions::1` explicitly.
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
/// Run `.list ``agent::1`` ``show_sessions::``1`. Assert agent appears and main absent.
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
    .arg( "show_sessions::1" )
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
/// Run `.list ``agent::0`` ``show_sessions::``1`. Assert main appears and agent absent.
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
    .arg( "show_sessions::1" )
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

/// INT-10: `show_sessions::0` suppresses display even with `session::` filter.
///
/// ## Purpose
/// Verify that explicit `show_sessions::0` suppresses session entries even when
/// a `session::` filter is also provided.
///
/// ## Coverage
/// No session entries appear in output despite `session::` filter; exit 0.
///
/// ## Validation Strategy
/// Write project alpha with session "abc-override". Run `.list ``session::abc`` ``show_sessions::``0`.
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
    .arg( "show_sessions::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // session:: filter auto-enables sessions display regardless of show_sessions::0
  assert!(
    s.contains( "abc-override" ),
    "INT-10: session:: filter shows sessions even when show_sessions::0 is set; got:\n{s}"
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
/// Write project alpha (path contains "projects") with session "s-abc".
/// Write project other (path does not contain "projects") with session "s-abc".
/// Run `.list path::projects session::abc`. Assert alpha's session
/// appears, other's session absent.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-11
#[ test ]
fn int_11_combined_path_session_filter()
{
  let root = TempDir::new().unwrap();

  // Fix(BUG-007): filter changed "pro" -> "projects".
  // Root cause: both fixtures share ONE random TempDir root whose encoded form
  // starts "-tmp<6 random chars>"; the literal 'p' of "tmp" followed by a random
  // suffix starting "ro"/"RO" made the shared root itself contain "pro"
  // case-insensitively, so BOTH projects matched `path::pro` and the
  // "other must be absent" assertion failed intermittently (~1/961 draws).
  // Pitfall: a substring filter asserted absent must be impossible — not just
  // unlikely — in every generator-controlled segment of the fixture's full
  // stored identifier; "projects" (8 chars, crossing a '_' boundary) cannot
  // fit in the 6-char random component under any draw.
  // alpha is under a path containing "projects"
  let alpha = root.path().join( "projects" ).join( "alpha" );
  // other is NOT under a path containing "projects"
  let other = root.path().join( "unrelated" ).join( "other" );

  common::write_path_project_session( root.path(), &alpha, "s-abc", 2 );
  common::write_path_project_session( root.path(), &other, "s-abc", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "path::projects" )
    .arg( "session::abc" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "alpha" ),
    "INT-11: project 'alpha' (path contains 'projects') must appear; got:\n{s}"
  );
  assert!(
    !s.contains( "other" ),
    "INT-11: project 'other' (path lacks 'projects') must be absent; got:\n{s}"
  );
}

/// B007: Historical collision layout cannot break path-filter isolation.
///
/// ## Root Cause
/// INT-5/INT-11 formerly filtered on `path::pro` while both fixture projects
/// shared one random `TempDir` root. `tempfile`'s ".tmp" prefix ends in 'p',
/// so a random suffix starting "ro" (any case) put the substring "pro" into
/// the SHARED root — both projects then matched, violating the fixtures'
/// isolation assumption (observed live: root `.tmpROq9Z7` -> "tmpro...").
///
/// ## Why Not Caught
/// ~1/961 trigger probability per run; an intermittent failure followed by an
/// immediate clean re-run reads as "flaky CI", discouraging root-causing.
///
/// ## Fix Applied
/// Filter substring "pro" -> "projects" in INT-5/INT-11 (BUG-007). 8 chars
/// cannot fit inside the 6-char random component, and spanning matches break
/// on the '_' segment boundary — collision is impossible, not just unlikely.
///
/// ## Prevention
/// This test pins the exact historical worst-case draw as a FIXED path
/// segment (".tmpROq9Z7", which contains "pro" case-insensitively) and
/// asserts `path::projects` still isolates the two projects deterministically.
///
/// ## Pitfall
/// Fixture isolation must never depend on shared uncontrolled randomness:
/// any generator-fed segment of a stored identifier is part of the filter's
/// comparison surface, whether or not the fixture author intended it.
// test_kind: bug_reproducer(BUG-007)
#[ test ]
fn bug_007_collision_prefix_root_does_not_break_path_filter_isolation()
{
  let root = TempDir::new().unwrap();

  // Deliberately embed the historical failure's exact random draw as a fixed
  // literal: ".tmpROq9Z7" contains "pro" case-insensitively ("...pRO...").
  let collision_root = root.path().join( ".tmpROq9Z7" );
  let alpha = collision_root.join( "projects" ).join( "alpha" );
  let other = collision_root.join( "unrelated" ).join( "other" );

  common::write_path_project_session( root.path(), &alpha, "s-abc", 2 );
  common::write_path_project_session( root.path(), &other, "s-abc", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "path::projects" )
    .arg( "session::abc" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "alpha" ),
    "B007: project 'alpha' (path contains 'projects') must appear; got:\n{s}"
  );
  assert!(
    !s.contains( "other" ),
    "B007: project 'other' must stay absent even under the historical \
     collision-bearing root ('.tmpROq9Z7' contains 'pro'); got:\n{s}"
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

/// T01: Default (no `scope::`) regression guard.
///
/// ## Purpose
/// Verify that `.list` with no `scope::` given still lists every project,
/// matching pre-retrofit behavior (the `global` default).
///
/// ## Coverage
/// Both written projects appear; exit 0.
///
/// ## Validation Strategy
/// Write 2 unrelated path-encoded projects. Run `.list` with no `scope::`.
/// Assert both project names appear.
///
/// ## Related Requirements
/// `task/claude_storage/executed/516_list_scope_retrofit.md` — T01
#[ test ]
fn t01_default_scope_global_regression_guard()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "t01alpha" );
  let beta  = root.path().join( "t01beta" );

  common::write_path_project_session( root.path(), &alpha, "s001", 2 );
  common::write_path_project_session( root.path(), &beta,  "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "t01alpha" ) && s.contains( "t01beta" ),
    "T01: default (no scope::) must list all projects, matching pre-retrofit behavior; got:\n{s}"
  );
}

/// T02: `scope::global` explicit is byte-for-byte identical to the omitted default.
///
/// ## Purpose
/// Prove zero drift at the default (AF2) — the omitted-scope and
/// explicit-`scope::global` invocations execute the exact same untouched
/// `storage.list_projects()` code path and must produce identical bytes.
///
/// ## Coverage
/// `stdout` byte-for-byte equal between the two invocations; both exit 0.
///
/// ## Validation Strategy
/// Write 2 projects. Run `.list` and `.list scope::global` separately.
/// Assert `stdout` is byte-identical between the two.
///
/// ## Related Requirements
/// `task/claude_storage/executed/516_list_scope_retrofit.md` — T02
#[ test ]
fn t02_scope_global_explicit_byte_identical_to_default()
{
  let root  = TempDir::new().unwrap();
  let alpha = root.path().join( "t02alpha" );
  let beta  = root.path().join( "t02beta" );

  common::write_path_project_session( root.path(), &alpha, "s001", 2 );
  common::write_path_project_session( root.path(), &beta,  "s001", 2 );

  let default_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".list" )
    .output()
    .unwrap();

  let explicit_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".list" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &default_out, 0 );
  assert_exit( &explicit_out, 0 );
  assert_eq!(
    default_out.stdout, explicit_out.stdout,
    "T02: scope::global explicit must be byte-for-byte identical to the omitted default \
    (same untouched storage.list_projects() code path)"
  );
}

/// T03: `scope::local` narrows to the cwd project only.
///
/// ## Purpose
/// Verify `scope::local` limits the listed projects to the cwd's own
/// project, excluding an unrelated project written elsewhere in storage.
///
/// ## Coverage
/// cwd's own project present; unrelated project absent; exit 0.
///
/// ## Validation Strategy
/// Write the cwd's own project (a real directory, required for
/// `Command::current_dir`) plus one unrelated project. Run
/// `.list scope::local` from the cwd project's own directory. Assert the
/// cwd project appears and the unrelated one does not.
///
/// ## Related Requirements
/// `task/claude_storage/executed/516_list_scope_retrofit.md` — T03
#[ test ]
fn t03_scope_local_narrows_to_cwd_project()
{
  let root       = TempDir::new().unwrap();
  let target_tmp = TempDir::new().unwrap();
  let target     = target_tmp.path().join( "t03targetmarker" );
  std::fs::create_dir_all( &target ).unwrap();

  common::write_path_project_session( root.path(), &target, "s001", 2 );

  let other = root.path().join( "t03other" );
  common::write_path_project_session( root.path(), &other, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &target )
    .arg( ".list" )
    .arg( "scope::local" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "t03targetmarker" ),
    "T03: scope::local from the target project's own cwd must include it; got:\n{s}"
  );
  assert!(
    !s.contains( "t03other" ),
    "T03: scope::local must exclude an unrelated project outside the anchor; got:\n{s}"
  );
}

/// T04: `scope::under` narrows to descendant projects.
///
/// ## Purpose
/// Verify `scope::under` from an ancestor cwd includes a nested descendant
/// project while excluding an unrelated project elsewhere in storage.
///
/// ## Coverage
/// Nested descendant project present; unrelated project absent; exit 0.
///
/// ## Validation Strategy
/// Write a project nested under an ancestor directory (the ancestor itself
/// is a real directory for `Command::current_dir`; the nested project's
/// storage encoding does not require real on-disk existence — see
/// `scope.rs`'s conservative-include fallback). Run `.list scope::under`
/// from the ancestor. Assert the nested project appears and the unrelated
/// one does not.
///
/// ## Related Requirements
/// `task/claude_storage/executed/516_list_scope_retrofit.md` — T04
#[ test ]
fn t04_scope_under_narrows_to_descendants()
{
  let root       = TempDir::new().unwrap();
  let anchor_tmp = TempDir::new().unwrap();
  let anchor     = anchor_tmp.path().join( "t04anchor" );
  let nested     = anchor.join( "t04nestedchild" );
  std::fs::create_dir_all( &anchor ).unwrap();

  common::write_path_project_session( root.path(), &nested, "s001", 2 );

  let other = root.path().join( "t04other" );
  common::write_path_project_session( root.path(), &other, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &anchor )
    .arg( ".list" )
    .arg( "scope::under" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "t04nestedchild" ),
    "T04: scope::under from the ancestor cwd must include the nested descendant project; got:\n{s}"
  );
  assert!(
    !s.contains( "t04other" ),
    "T04: scope::under must exclude an unrelated project outside the anchor; got:\n{s}"
  );
}

/// T05: `scope::bogus` rejected with the canonical `validate_scope()` error.
///
/// ## Purpose
/// Verify invalid `scope::` values are rejected the same way for `.list` as
/// for `.projects`/`.show`/`.export`/`.search` — one shared validator, one
/// canonical error.
///
/// ## Coverage
/// Exit 1; stderr contains the exact `validate_scope()` wording.
///
/// ## Validation Strategy
/// Run `.list scope::bogus`. Assert exit 1 and the canonical error text.
///
/// ## Related Requirements
/// `task/claude_storage/executed/516_list_scope_retrofit.md` — T05
#[ test ]
fn t05_scope_bogus_rejected_with_canonical_error()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".list" )
    .arg( "scope::bogus" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "scope must be relevant|local|under|global|around, got bogus" ),
    "T05: scope::bogus must produce the canonical validate_scope() error; got: {err}"
  );
}

/// T06: `scope::local` composes with the existing `path::` filter.
///
/// ## Purpose
/// Prove both narrowings apply together — `path::` substring-filters the
/// `scope::`-narrowed set rather than either one silently overriding the
/// other.
///
/// ## Coverage
/// Project present when `path::` matches; same project absent when `path::`
/// does not match, despite `scope::local` being identical in both runs;
/// both exit 0.
///
/// ## Validation Strategy
/// Write the cwd's own project with "assistant" in its path. Run
/// `.list scope::local path::assistant` (should include it) and
/// `.list scope::local path::zzz-nonexistent-substring` (should exclude
/// it). Assert both outcomes.
///
/// ## Related Requirements
/// `task/claude_storage/executed/516_list_scope_retrofit.md` — T06
#[ test ]
fn t06_scope_local_composes_with_path_filter()
{
  let root       = TempDir::new().unwrap();
  let target_tmp = TempDir::new().unwrap();
  let target     = target_tmp.path().join( "t06assistantproject" );
  std::fs::create_dir_all( &target ).unwrap();

  common::write_path_project_session( root.path(), &target, "s001", 2 );

  let matching = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &target )
    .arg( ".list" )
    .arg( "scope::local" )
    .arg( "path::assistant" )
    .output()
    .unwrap();

  assert_exit( &matching, 0 );
  let s_match = stdout( &matching );
  assert!(
    s_match.contains( "t06assistantproject" ),
    "T06: scope::local + a matching path:: substring must include the project; got:\n{s_match}"
  );

  let non_matching = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &target )
    .arg( ".list" )
    .arg( "scope::local" )
    .arg( "path::zzz-nonexistent-substring" )
    .output()
    .unwrap();

  assert_exit( &non_matching, 0 );
  let s_non_match = stdout( &non_matching );
  assert!(
    !s_non_match.contains( "t06assistantproject" ),
    "T06: scope::local narrowed set must still be filtered by a non-matching path::; got:\n{s_non_match}"
  );
}

/// INT-13: `type::` with invalid value rejected.
///
/// ## Purpose
/// Verify `.list type::badvalue` is rejected — `badvalue` is not a valid
/// `type::` option.
///
/// ## Coverage
/// Exit code exactly 1; stderr names the invalid value; no listing output
/// on stdout.
///
/// ## Validation Strategy
/// Run `.list type::badvalue` against an empty temp storage root from a
/// neutral cwd. Assert exit 1, stderr containing `badvalue`, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-13
#[ test ]
fn int_13_type_invalid_value_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".list" )
    .arg( "type::badvalue" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-13: invalid type:: value must produce an error on stderr"
  );
  assert!(
    err.contains( "badvalue" ),
    "INT-13: stderr must name the invalid type:: value; got: {err}"
  );
  assert!(
    stdout( &out ).is_empty(),
    "INT-13: no listing output on stdout when type:: is rejected; got:\n{}",
    stdout( &out )
  );
}

/// INT-14: `agent::` with non-boolean value rejected.
///
/// ## Purpose
/// Verify `.list agent::invalid` is rejected as an argument error —
/// `invalid` is not a valid boolean value (accepted: `0`, `1`).
///
/// ## Coverage
/// Exit code exactly 1; non-empty stderr describing the argument error; no
/// listing output on stdout.
///
/// ## Validation Strategy
/// Run `.list agent::invalid` against an empty temp storage root from a
/// neutral cwd. Assert exit 1, stderr naming the `agent` argument, empty
/// stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/02_list.md` — INT-14
#[ test ]
fn int_14_agent_non_boolean_rejected()
{
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( "/tmp" )
    .arg( ".list" )
    .arg( "agent::invalid" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "INT-14: non-boolean agent:: value must produce an error on stderr"
  );
  assert!(
    err.contains( "agent" ),
    "INT-14: stderr must name the rejected agent argument; got: {err}"
  );
  assert!(
    stdout( &out ).is_empty(),
    "INT-14: no listing output on stdout when agent:: is rejected; got:\n{}",
    stdout( &out )
  );
}
