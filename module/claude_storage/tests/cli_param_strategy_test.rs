//! Edge case tests for the `strategy::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/20_strategy.md`
//!
//! ## Coverage
//!
//! - EC-1: resume accepted
//! - EC-2: fresh accepted
//! - EC-3: Invalid value rejected
//! - EC-4: Case-insensitive: Resume accepted
//! - EC-5: Case-insensitive: FRESH accepted
//! - EC-6: Absent defaults to auto-detect (fresh when no history)
//! - EC-7: Absent defaults to auto-detect (resume when history exists)
//! - EC-8: resume forced overrides auto-detect fresh
//! - EC-9: fresh forced overrides auto-detect resume

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

/// Create storage history so .session.ensure reports "resume".
///
/// Encodes the `session_dir` path and writes a non-empty JSONL file into
/// `{home}/.claude/projects/{encoded}/`.
fn setup_history( home : &std::path::Path, session_dir : &std::path::Path )
{
  let storage_root = home.join( ".claude" );
  common::write_path_project_session( &storage_root, session_dir, "session-test", 2 );
}

/// EC-1: resume accepted.
///
/// ## Purpose
/// Validates that `strategy::resume` is accepted by .session.ensure.
///
/// ## Coverage
/// Exit 0; two lines output; line 2 is "resume".
///
/// ## Validation Strategy
/// Run `.session.ensure ``path::``... ``strategy::resum``e` with fresh HOME.
/// Assert exit 0 and line 2 is "resume".
///
/// ## Related Requirements
/// `tests/docs/cli/param/20_strategy.md` — EC-1
#[ test ]
fn ec_1_strategy_resume_accepted()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "strategy::resume" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  let lines : Vec< &str > = output.lines().collect();
  assert_eq!( lines.len(), 2, "EC-1: must output exactly 2 lines; got: {output}" );
  assert_eq!( lines[ 1 ], "resume", "EC-1: line 2 must be 'resume'; got: {}", lines[ 1 ] );
}

/// EC-2: fresh accepted.
///
/// ## Purpose
/// Validates that `strategy::fresh` is accepted by .session.ensure.
///
/// ## Coverage
/// Exit 0; two lines output; line 2 is "fresh".
///
/// ## Validation Strategy
/// Run `.session.ensure ``path::``... ``strategy::fres``h` with fresh HOME.
/// Assert exit 0 and line 2 is "fresh".
///
/// ## Related Requirements
/// `tests/docs/cli/param/20_strategy.md` — EC-2
#[ test ]
fn ec_2_strategy_fresh_accepted()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "strategy::fresh" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  let lines : Vec< &str > = output.lines().collect();
  assert_eq!( lines.len(), 2, "EC-2: must output exactly 2 lines; got: {output}" );
  assert_eq!( lines[ 1 ], "fresh", "EC-2: line 2 must be 'fresh'; got: {}", lines[ 1 ] );
}

/// EC-3: Invalid value rejected.
///
/// ## Purpose
/// Validates that `strategy::auto` is rejected as invalid.
///
/// ## Coverage
/// Exit 1; error message containing "strategy must be resume|fresh".
///
/// ## Validation Strategy
/// Run `.session.ensure ``path::``... ``strategy::aut``o`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/20_strategy.md` — EC-3
#[ test ]
fn ec_3_strategy_invalid_rejected()
{
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "strategy::auto" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "strategy" ) && ( err.contains( "resume" ) || err.contains( "fresh" ) ),
    "EC-3: error must mention strategy and valid values; got: {err}"
  );
}

/// EC-4: Case-insensitive: Resume accepted.
///
/// ## Purpose
/// Validates that `strategy::Resume` (mixed case) is accepted.
///
/// ## Coverage
/// Exit 0; line 2 is "resume" (normalized to lowercase).
///
/// ## Validation Strategy
/// Run `.session.ensure ``path::``... ``strategy::Resum``e`. Assert exit 0 and line 2 = "resume".
///
/// ## Related Requirements
/// `tests/docs/cli/param/20_strategy.md` — EC-4
#[ test ]
fn ec_4_strategy_resume_case_insensitive()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "strategy::Resume" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  let lines : Vec< &str > = output.lines().collect();
  assert_eq!( lines.len(), 2, "EC-4: must output exactly 2 lines; got: {output}" );
  assert_eq!( lines[ 1 ], "resume", "EC-4: line 2 must be 'resume'; got: {}", lines[ 1 ] );
}

/// EC-5: Case-insensitive: FRESH accepted.
///
/// ## Purpose
/// Validates that `strategy::FRESH` (all caps) is accepted.
///
/// ## Coverage
/// Exit 0; line 2 is "fresh".
///
/// ## Validation Strategy
/// Run `.session.ensure ``path::``... ``strategy::FRES``H`. Assert exit 0 and line 2 = "fresh".
///
/// ## Related Requirements
/// `tests/docs/cli/param/20_strategy.md` — EC-5
#[ test ]
fn ec_5_strategy_fresh_case_insensitive()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "strategy::FRESH" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  let lines : Vec< &str > = output.lines().collect();
  assert_eq!( lines.len(), 2, "EC-5: must output exactly 2 lines; got: {output}" );
  assert_eq!( lines[ 1 ], "fresh", "EC-5: line 2 must be 'fresh'; got: {}", lines[ 1 ] );
}

/// EC-6: Absent defaults to auto-detect (fresh when no history).
///
/// ## Purpose
/// Validates that omitting `strategy::` with no history defaults to "fresh".
///
/// ## Coverage
/// Exit 0; line 2 is "fresh" (no history → auto-detect → fresh).
///
/// ## Validation Strategy
/// Run `.session.ensure ``path::``...` with empty HOME (no history).
/// Assert line 2 is "fresh".
///
/// ## Related Requirements
/// `tests/docs/cli/param/20_strategy.md` — EC-6
#[ test ]
fn ec_6_strategy_absent_fresh_when_no_history()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  let lines : Vec< &str > = output.lines().collect();
  assert_eq!( lines.len(), 2, "EC-6: must output exactly 2 lines; got: {output}" );
  assert_eq!( lines[ 1 ], "fresh", "EC-6: line 2 must be 'fresh' with no history; got: {}", lines[ 1 ] );
}

/// EC-7: Absent defaults to auto-detect (resume when history exists).
///
/// ## Purpose
/// Validates that omitting `strategy::` with existing history defaults to "resume".
///
/// ## Coverage
/// Exit 0; line 2 is "resume" (history found → auto-detect → resume).
///
/// ## Validation Strategy
/// Create storage history for the session dir. Run `.session.ensure ``path::``...` with no strategy.
/// Assert line 2 is "resume".
///
/// ## Related Requirements
/// `tests/docs/cli/param/20_strategy.md` — EC-7
#[ test ]
fn ec_7_strategy_absent_resume_when_history_exists()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();
  // Create the session directory path we'll use (default topic = /-default_topic)
  let session_dir = base.path().join( "-default_topic" );
  setup_history( home.path(), &session_dir );

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  let lines : Vec< &str > = output.lines().collect();
  assert_eq!( lines.len(), 2, "EC-7: must output exactly 2 lines; got: {output}" );
  assert_eq!( lines[ 1 ], "resume", "EC-7: line 2 must be 'resume' with history; got: {}", lines[ 1 ] );
}

/// EC-8: resume forced overrides auto-detect fresh.
///
/// ## Purpose
/// Validates that `strategy::resume` forces "resume" even when auto-detect would be "fresh".
///
/// ## Coverage
/// Exit 0; line 2 is "resume" despite no history (auto-detect would be "fresh").
///
/// ## Validation Strategy
/// Run with empty HOME (no history) + `strategy::resume`.
/// Assert line 2 is "resume" (not "fresh").
///
/// ## Related Requirements
/// `tests/docs/cli/param/20_strategy.md` — EC-8
#[ test ]
fn ec_8_strategy_resume_overrides_auto_fresh()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();
  // No history → auto-detect would be "fresh"

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "strategy::resume" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  let lines : Vec< &str > = output.lines().collect();
  assert_eq!( lines.len(), 2, "EC-8: must output exactly 2 lines; got: {output}" );
  assert_eq!( lines[ 1 ], "resume", "EC-8: strategy::resume must force 'resume' even without history; got: {}", lines[ 1 ] );
}

/// EC-9: fresh forced overrides auto-detect resume.
///
/// ## Purpose
/// Validates that `strategy::fresh` forces "fresh" even when auto-detect would be "resume".
///
/// ## Coverage
/// Exit 0; line 2 is "fresh" despite existing history.
///
/// ## Validation Strategy
/// Create history then run `.session.ensure` with `strategy::fresh`.
/// Assert line 2 is "fresh" (not "resume").
///
/// ## Related Requirements
/// `tests/docs/cli/param/20_strategy.md` — EC-9
#[ test ]
fn ec_9_strategy_fresh_overrides_auto_resume()
{
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();
  let session_dir = base.path().join( "-default_topic" );
  setup_history( home.path(), &session_dir );
  // History exists → auto-detect would be "resume"

  let out = common::clg_cmd()
    .env( "HOME", home.path() )
    .arg( ".session.ensure" )
    .arg( format!( "path::{}", base.path().display() ) )
    .arg( "strategy::fresh" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  let lines : Vec< &str > = output.lines().collect();
  assert_eq!( lines.len(), 2, "EC-9: must output exactly 2 lines; got: {output}" );
  assert_eq!( lines[ 1 ], "fresh", "EC-9: strategy::fresh must force 'fresh' despite history; got: {}", lines[ 1 ] );
}
