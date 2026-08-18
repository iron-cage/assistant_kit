//! Edge case tests for the `since_days::` parameter on `.projects`.
//!
//! ## Coverage
//!
//! EC-1 through EC-5 per `tests/docs/cli/param/27_since_days.md` —
//! recency-window inclusion/exclusion, zero-day (last 24 hours) semantics,
//! negative-value rejection, omitted default (no filtering), and
//! non-integer rejection.
//!
//! ## Test Case Index
//!
//! | ID | Test Name | Category |
//! |----|-----------|----------|
//! | EC-1 | Window includes recent session, excludes old session | Filter Behavior |
//! | EC-2 | `since_days::0` shows a session touched today | Boundary |
//! | EC-3 | Negative value rejected | Validation |
//! | EC-4 | Omitted means no window filtering | Default |
//! | EC-5 | Non-integer value rejected | Type Validation |

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

/// Set a session file's modification time to `now - days` days.
fn set_mtime_days_ago( path : &std::path::Path, days : u64 )
{
  let t = std::time::SystemTime::now() - core::time::Duration::from_secs( days * 86_400 );
  let f = std::fs::OpenOptions::new().write( true ).open( path )
    .expect( "open session file for mtime update" );
  f.set_times( std::fs::FileTimes::new().set_modified( t ) )
    .expect( "set session file mtime" );
}

/// EC-1: Window includes recent session, excludes old session.
///
/// ## Purpose
/// Validates that `since_days::20` keeps a session modified 5 days ago and
/// drops one modified 25 days ago.
///
/// ## Coverage
/// Recent session ID present; old session ID absent; conversation count
/// header reflects the filtered set.
///
/// ## Validation Strategy
/// Two sessions in one project with mtimes now-5d and now-25d; run
/// `.projects scope::global since_days::20`; assert inclusion/exclusion.
///
/// ## Related Requirements
/// `tests/docs/cli/param/27_since_days.md` — EC-1
#[ test ]
fn ec_1_since_days_window_includes_recent_excludes_old()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj_window" );

  let enc = common::write_path_project_session( &storage_root, &project, "recent11", 2 );
  common::write_path_project_session( &storage_root, &project, "older222", 2 );

  let dir = storage_root.join( "projects" ).join( &enc );
  set_mtime_days_ago( &dir.join( "recent11.jsonl" ), 5 );
  set_mtime_days_ago( &dir.join( "older222.jsonl" ), 25 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "since_days::20" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "recent11" ), "EC-1: session 5 days old must be inside a 20-day window; got:\n{s}" );
  assert!( !s.contains( "older222" ), "EC-1: session 25 days old must be outside a 20-day window; got:\n{s}" );
  assert!( s.contains( "1 conversation" ), "EC-1: header must count only the windowed session; got:\n{s}" );
}

/// EC-2: `since_days::0` shows a session touched today.
///
/// ## Purpose
/// Validates the zero-day boundary: `0` means the most recent 24 hours,
/// not an empty window.
///
/// ## Coverage
/// A freshly written session (mtime = now) appears with `since_days::0`.
///
/// ## Validation Strategy
/// Write a session (mtime is current), run `.projects scope::global
/// since_days::0`, assert the session is listed.
///
/// ## Related Requirements
/// `tests/docs/cli/param/27_since_days.md` — EC-2
#[ test ]
fn ec_2_since_days_zero_shows_today()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj_today" );

  common::write_path_project_session( &storage_root, &project, "todayaaa", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "since_days::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "todayaaa" ), "EC-2: session touched now must survive since_days::0; got:\n{s}" );
}

/// EC-3: Negative value rejected.
///
/// ## Purpose
/// Validates that `since_days::-1` is rejected as invalid.
///
/// ## Coverage
/// Exit 1; error output mentions `since_days`.
///
/// ## Validation Strategy
/// Run `.projects since_days::-1`. Assert exit 1 and the error names the
/// parameter.
///
/// ## Related Requirements
/// `tests/docs/cli/param/27_since_days.md` — EC-3
#[ test ]
fn ec_3_since_days_negative_rejected()
{
  let out = common::clg_cmd()
    .arg( ".projects" )
    .arg( "since_days::-1" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "since_days" ),
    "EC-3: error must mention 'since_days'; got: {combined}"
  );
}

/// EC-4: Omitted means no window filtering.
///
/// ## Purpose
/// Validates the baseline regression: without `since_days::`, sessions of
/// any age are listed — the parameter is purely additive.
///
/// ## Coverage
/// Both a 5-day-old and a 25-day-old session appear in bare output.
///
/// ## Validation Strategy
/// Same fixture as EC-1, run `.projects scope::global` with no window,
/// assert both sessions and the unfiltered count.
///
/// ## Related Requirements
/// `tests/docs/cli/param/27_since_days.md` — EC-4
#[ test ]
fn ec_4_since_days_omitted_no_filtering()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj_window" );

  let enc = common::write_path_project_session( &storage_root, &project, "recent11", 2 );
  common::write_path_project_session( &storage_root, &project, "older222", 2 );

  let dir = storage_root.join( "projects" ).join( &enc );
  set_mtime_days_ago( &dir.join( "recent11.jsonl" ), 5 );
  set_mtime_days_ago( &dir.join( "older222.jsonl" ), 25 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "recent11" ), "EC-4: recent session must appear without a window; got:\n{s}" );
  assert!( s.contains( "older222" ), "EC-4: old session must appear without a window; got:\n{s}" );
  assert!( s.contains( "2 conversations" ), "EC-4: header must count both sessions; got:\n{s}" );
}

/// EC-5: Non-integer value rejected.
///
/// ## Purpose
/// Validates that `since_days::abc` is rejected (not a valid integer).
///
/// ## Coverage
/// Exit non-zero; coercion error on the `since_days` argument.
///
/// ## Validation Strategy
/// Run `.projects since_days::abc`. Assert exit non-zero.
///
/// ## Related Requirements
/// `tests/docs/cli/param/27_since_days.md` — EC-5
#[ test ]
fn ec_5_since_days_non_integer_rejected()
{
  let out = common::clg_cmd()
    .arg( ".projects" )
    .arg( "since_days::abc" )
    .output()
    .unwrap();

  assert_ne!(
    out.status.code().unwrap_or( -1 ),
    0,
    "EC-5: since_days::abc should be rejected; stderr: {}",
    stderr( &out )
  );
}
