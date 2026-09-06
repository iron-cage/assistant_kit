//! Unit tests for `find_by_cwd`.
//!
//! Tests the cwd-matching rule `docs/feature/008_interactive_handoff.md` shares with
//! `clr chat`'s own session resolution, in isolation — no daemon, no subprocess.
//!
//! `SessionSummary` is `#[non_exhaustive]` in `claude_daemon_core`, so this crate
//! cannot build one via struct-literal syntax; fixtures go through `serde_json`
//! instead, the same wire shape `Request::ListSessions` itself produces.
#![ cfg( feature = "enabled" ) ]

use claude_runner::find_by_cwd;
use claude_daemon_core::SessionSummary;

fn session( id : &str, cwd : &std::path::Path, busy : bool ) -> SessionSummary
{
  serde_json::from_value( serde_json::json!( {
    "session_id" : id,
    "pid" : 1,
    "cwd" : cwd,
    "busy" : busy,
  } ) )
  .expect( "valid SessionSummary fixture" )
}

#[ test ]
fn no_sessions_returns_none()
{
  let dir = std::path::Path::new( "/tmp/does-not-matter" );
  assert!( find_by_cwd( &[], dir ).is_none() );
}

#[ test ]
fn exact_match_returns_the_session()
{
  let dir = tempfile::TempDir::new().expect( "tempdir" );
  let sessions = [ session( "abc", dir.path(), false ) ];
  let found = find_by_cwd( &sessions, dir.path() ).expect( "must match" );
  assert_eq!( found.session_id, "abc" );
}

#[ test ]
fn no_matching_cwd_returns_none()
{
  let dir_a = tempfile::TempDir::new().expect( "tempdir a" );
  let dir_b = tempfile::TempDir::new().expect( "tempdir b" );
  let sessions = [ session( "abc", dir_a.path(), false ) ];
  assert!( find_by_cwd( &sessions, dir_b.path() ).is_none() );
}

#[ test ]
fn returns_the_matching_session_among_several()
{
  let dir_a = tempfile::TempDir::new().expect( "tempdir a" );
  let dir_b = tempfile::TempDir::new().expect( "tempdir b" );
  let sessions =
  [
    session( "aaa", dir_a.path(), false ),
    session( "bbb", dir_b.path(), true ),
  ];
  let found = find_by_cwd( &sessions, dir_b.path() ).expect( "must match dir_b's session" );
  assert_eq!( found.session_id, "bbb" );
}

#[ test ]
fn trailing_slash_and_dot_component_still_match_via_canonicalize()
{
  let base = tempfile::TempDir::new().expect( "tempdir" );
  let proj = base.path().join( "proj" );
  std::fs::create_dir( &proj ).expect( "create proj subdir" );
  let sessions = [ session( "abc", &proj, false ) ];

  let with_trailing_slash = base.path().join( "proj/" );
  assert!(
    find_by_cwd( &sessions, &with_trailing_slash ).is_some(),
    "a trailing slash must not defeat the match"
  );

  let with_dot_component = base.path().join( "./proj" );
  assert!(
    find_by_cwd( &sessions, &with_dot_component ).is_some(),
    "a `.` path component must not defeat the match"
  );
}

#[ cfg( unix ) ]
#[ test ]
fn symlinked_dir_matches_its_real_target()
{
  let base   = tempfile::TempDir::new().expect( "tempdir" );
  let target = base.path().join( "real" );
  let link   = base.path().join( "link" );
  std::fs::create_dir( &target ).expect( "create real dir" );
  std::os::unix::fs::symlink( &target, &link ).expect( "create symlink" );

  let sessions = [ session( "abc", &target, false ) ];
  assert!(
    find_by_cwd( &sessions, &link ).is_some(),
    "a symlink resolving to the session's cwd must match"
  );
}

#[ test ]
fn nonexistent_dir_falls_back_to_raw_path_equality()
{
  let missing = std::path::Path::new( "/this/path/does/not/exist/at/all" );
  let sessions = [ session( "abc", missing, false ) ];
  assert!(
    find_by_cwd( &sessions, missing ).is_some(),
    "canonicalize fails on both sides for a nonexistent path, so the raw-path \
     fallback must still compare equal to itself"
  );
}
