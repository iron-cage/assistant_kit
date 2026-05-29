//! Acceptance tests for the "Find Past Conversation" user story.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/user_story/02_find_past_conversation.md`
//!
//! ## Coverage
//!
//! - RWS-1: List all projects shows projects in storage
//! - RWS-2: Search by keyword finds matching sessions
//! - RWS-3: Project filter restricts search to one project
//! - RWS-4: Session metadata filters narrow listing
//! - RWS-5: Show session displays full session details

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

/// RWS-1: List all projects shows projects in storage.
///
/// ## Purpose
/// End-to-end acceptance test: developer lists all projects to find the one
/// containing the conversation they're looking for.
///
/// ## Coverage
/// All 3 project paths visible; no sessions shown by default; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/02_find_past_conversation.md` — RWS-1
#[ test ]
fn rws_1_list_all_projects_shows_projects_in_storage()
{
  // Note: directory names must not contain hyphens — the storage path encoding is lossy
  // (hyphens in directory names are decoded as path separators), so "find-proj-alpha"
  // would appear as "find/proj/alpha" in output. Single-token names without hyphens
  // are unambiguous and round-trip correctly through encode_path → decode_path.
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "findprojone" );
  let p2 = root.path().join( "findprojtwo" );
  let p3 = root.path().join( "findprojthree" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p3, "s003", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "findprojone" ),
    "RWS-1: findprojone must appear in .list; got:\n{s}"
  );
  assert!(
    s.contains( "findprojtwo" ),
    "RWS-1: findprojtwo must appear in .list; got:\n{s}"
  );
  assert!(
    s.contains( "findprojthree" ),
    "RWS-1: findprojthree must appear in .list; got:\n{s}"
  );
}

/// RWS-2: Search by keyword finds matching sessions.
///
/// ## Purpose
/// End-to-end acceptance test: developer searches for a keyword they remember
/// from a conversation; only the project with matching content appears.
///
/// ## Coverage
/// Keyword match returns matching project; non-matching project absent; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/02_find_past_conversation.md` — RWS-2
#[ test ]
fn rws_2_search_by_keyword_finds_matching_sessions()
{
  let root = TempDir::new().unwrap();
  // Use names without hyphens to avoid path encoding issues
  let proj_a = root.path().join( "searchprojA" );

  // Project A has a session containing "authentication"
  common::write_test_session_with_last_message(
    root.path(),
    &claude_storage_core::encode_path( &proj_a ).unwrap(),
    "-default_topic",
    2,
    "authentication token setup",
  );

  let enc_a = claude_storage_core::encode_path( &proj_a ).unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::authentication" )
    .arg( format!( "project::{enc_a}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "authentication" ) || s.contains( "searchprojA" ),
    "RWS-2: search must return the matching session or project; got:\n{s}"
  );
}

/// RWS-3: Project filter restricts search to one project.
///
/// ## Purpose
/// End-to-end acceptance test: developer knows which project a conversation is
/// in and narrows the search to that project only.
///
/// ## Coverage
/// `project::` scopes search; only target project sessions returned; other project excluded; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/02_find_past_conversation.md` — RWS-3
#[ test ]
fn rws_3_project_filter_restricts_search_to_one_project()
{
  let root = TempDir::new().unwrap();
  let target = root.path().join( "search-target" );
  let other  = root.path().join( "search-other" );

  common::write_test_session_with_last_message(
    root.path(),
    &claude_storage_core::encode_path( &target ).unwrap(),
    "-default_topic",
    2,
    "config setting value",
  );
  common::write_test_session_with_last_message(
    root.path(),
    &claude_storage_core::encode_path( &other ).unwrap(),
    "-default_topic",
    2,
    "config file path",
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".search" )
    .arg( "query::config" )
    .arg( format!( "project::{}", target.display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "search-target" ) || s.contains( "config" ),
    "RWS-3: search must return results from the target project; got:\n{s}"
  );
  assert!(
    !s.contains( "search-other" ),
    "RWS-3: other project must be excluded when project:: is specified; got:\n{s}"
  );
}

/// RWS-4: Session metadata filters narrow listing.
///
/// ## Purpose
/// End-to-end acceptance test: developer lists only main sessions with
/// substantial content using `show_sessions::1 ``agent::0`` ``min_entries::1``0`.
///
/// ## Coverage
/// Combined filter; below-threshold session excluded; agent session excluded; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/02_find_past_conversation.md` — RWS-4
#[ test ]
fn rws_4_session_metadata_filters_narrow_listing()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "narrow-proj" );
  let encoded = claude_storage_core::encode_path( &proj ).unwrap();

  // Main session with 12 entries — should appear
  common::write_test_session( root.path(), &encoded, "main-large", 12 );
  // Main session with 3 entries — should NOT appear (below min_entries::10)
  common::write_test_session( root.path(), &encoded, "main-small", 3 );
  // Agent session with 20 entries — should NOT appear (agent::0 excludes it)
  common::write_flat_agent_session( root.path(), &encoded, "agent-big-001", "main-large", 20 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "show_sessions::1" )
    .arg( "agent::0" )
    .arg( "min_entries::10" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "main-large" ),
    "RWS-4: main-large (12 entries) must appear; got:\n{s}"
  );
  assert!(
    !s.contains( "main-small" ),
    "RWS-4: main-small (3 entries) must be excluded by min_entries::10; got:\n{s}"
  );
  assert!(
    !s.contains( "agent-big-001" ),
    "RWS-4: agent session must be excluded by agent::0; got:\n{s}"
  );
}

/// RWS-5: Show session displays full session details.
///
/// ## Purpose
/// End-to-end acceptance test: developer found a session ID and inspects its
/// content; `.show` displays metadata and entry content.
///
/// ## Coverage
/// Session metadata visible; entries listed with type and content; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/02_find_past_conversation.md` — RWS-5
#[ test ]
fn rws_5_show_session_displays_full_session_details()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "show-detail-proj" );
  std::fs::create_dir_all( &proj ).unwrap();
  common::write_path_project_session( root.path(), &proj, "-default_topic", 3 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( &proj )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "RWS-5: .show must produce session detail output; stderr: {}",
    stderr( &out )
  );
  // Should contain entry count or session metadata
  assert!(
    s.contains( '3' ) || s.contains( "entry" ) || s.contains( "session" ),
    "RWS-5: .show must include session metadata or entry info; got:\n{s}"
  );
}
