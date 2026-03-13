//! Integration tests for `SessionManager` and `Strategy`.
//!
//! # Moved From
//!
//! These tests were moved from `claude_profile` to `claude_runner_core`
//! when `SessionManager` ownership was transferred (task-034).
//! `SessionManager` manages LOCAL invocation directories (`-{topic}/`),
//! not `~/.claude/projects/` — it belongs with process execution logic.

use claude_runner_core::{ SessionManager, Strategy };
use core::str::FromStr;
use tempfile::TempDir;

// ============================================================================
// SessionManager — session_dir format
// ============================================================================

#[ test ]
fn session_dir_format()
{
  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  let dir = mgr.session_dir( "test-session" );

  assert!( dir.ends_with( "sessions/-test-session" ) );
}

#[ test ]
fn session_dir_special_names()
{
  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  let dir = mgr.session_dir( "my-debug-session" );
  assert!( dir.ends_with( "-my-debug-session" ) );

  let dir = mgr.session_dir( "session123" );
  assert!( dir.ends_with( "-session123" ) );

  let dir = mgr.session_dir( "default" );
  assert!( dir.ends_with( "-default" ) );
}

// ============================================================================
// SessionManager — deprecated session_exists (v1.x behavior)
// ============================================================================

#[ test ]
fn session_exists_returns_false_for_nonexistent()
{
  //! Verify deprecated `session_exists()` behavior for nonexistent session.
  //!
  //! **Note:** This tests the deprecated v1.x detection method.
  //! Use `claude_storage_core::continuation::check_continuation()` instead.

  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  #[ allow( deprecated ) ]
  {
    assert!( !mgr.session_exists( "nonexistent" ) );
  }
}

#[ test ]
fn session_exists_returns_false_without_history_file()
{
  //! Verify deprecated `session_exists()` requires `.claude_history` file.

  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  let session_dir = mgr.session_dir( "test" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  #[ allow( deprecated ) ]
  {
    assert!( !mgr.session_exists( "test" ) );
  }
}

#[ test ]
fn session_exists_returns_true_with_history_file()
{
  //! Verify deprecated `session_exists()` detects `.claude_history` file.

  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  let session_dir = mgr.session_dir( "test" );
  std::fs::create_dir_all( &session_dir ).unwrap();
  std::fs::write( session_dir.join( ".claude_history" ), "" ).unwrap();

  #[ allow( deprecated ) ]
  {
    assert!( mgr.session_exists( "test" ) );
  }
}

// ============================================================================
// SessionManager — ensure_session
// ============================================================================

#[ test ]
fn ensure_session_creates_directory()
{
  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  let session_dir = mgr.ensure_session( "test", Strategy::Resume ).unwrap();

  assert!( session_dir.exists() );
  assert!( session_dir.ends_with( "-test" ) );
}

#[ test ]
fn ensure_session_is_idempotent()
{
  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  let dir1 = mgr.ensure_session( "test", Strategy::Resume ).unwrap();
  let dir2 = mgr.ensure_session( "test", Strategy::Resume ).unwrap();

  assert_eq!( dir1, dir2 );
  assert!( dir1.exists() );
}

#[ test ]
fn ensure_session_resume_preserves_existing()
{
  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  let session_dir = mgr.ensure_session( "test", Strategy::Resume ).unwrap();
  let marker_file = session_dir.join( "marker.txt" );
  std::fs::write( &marker_file, "existing" ).unwrap();

  mgr.ensure_session( "test", Strategy::Resume ).unwrap();

  assert!( marker_file.exists() );
  assert_eq!( std::fs::read_to_string( &marker_file ).unwrap(), "existing" );
}

#[ test ]
fn ensure_session_fresh_deletes_existing()
{
  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  let session_dir = mgr.ensure_session( "test", Strategy::Resume ).unwrap();
  let marker_file = session_dir.join( "marker.txt" );
  std::fs::write( &marker_file, "old" ).unwrap();

  mgr.ensure_session( "test", Strategy::Fresh ).unwrap();

  assert!( !marker_file.exists() );
}

#[ test ]
fn ensure_session_fresh_creates_clean_directory()
{
  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  let session_dir = mgr.ensure_session( "test", Strategy::Resume ).unwrap();
  std::fs::write( session_dir.join( "file1.txt" ), "data1" ).unwrap();
  std::fs::write( session_dir.join( "file2.txt" ), "data2" ).unwrap();

  let new_dir = mgr.ensure_session( "test", Strategy::Fresh ).unwrap();

  let entries : Vec< _ > = std::fs::read_dir( &new_dir ).unwrap().collect();
  assert_eq!( entries.len(), 0 );
}

#[ test ]
fn ensure_session_fresh_creates_new_when_nonexistent()
{
  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "sessions" );
  let mgr = SessionManager::new( &sessions_root );

  // Fresh on non-existent should succeed
  let session_dir = mgr.ensure_session( "new-session", Strategy::Fresh ).unwrap();

  assert!( session_dir.exists() );
  assert!( session_dir.ends_with( "-new-session" ) );
}

#[ test ]
fn sessions_base_dir_returns_correct_path()
{
  let temp = TempDir::new().unwrap();
  let sessions_root = temp.path().join( "test-sessions" );
  let mgr = SessionManager::new( &sessions_root );

  assert!( mgr.sessions_base_dir().ends_with( "test-sessions" ) );
}

// ============================================================================
// Strategy — FromStr
// ============================================================================

#[ test ]
fn strategy_fromstr_resume()
{
  let result = Strategy::from_str( "resume" );
  assert!( matches!( result, Ok( Strategy::Resume ) ) );
}

#[ test ]
fn strategy_fromstr_fresh()
{
  let result = Strategy::from_str( "fresh" );
  assert!( matches!( result, Ok( Strategy::Fresh ) ) );
}

#[ test ]
fn strategy_fromstr_uppercase_fails()
{
  assert!( Strategy::from_str( "RESUME" ).is_err(), "uppercase should fail" );
  assert!( Strategy::from_str( "FRESH" ).is_err(), "uppercase should fail" );
}

#[ test ]
fn strategy_fromstr_empty_fails()
{
  assert!( Strategy::from_str( "" ).is_err(), "empty string should fail" );
}

#[ test ]
fn strategy_fromstr_invalid_fails()
{
  let result = Strategy::from_str( "invalid" );
  assert!( result.is_err() );

  if let Err( msg ) = result
  {
    assert!( msg.contains( "resume" ), "error should mention valid options: {msg}" );
    assert!( msg.contains( "fresh" ), "error should mention valid options: {msg}" );
  }
}

#[ test ]
fn strategy_fromstr_mixed_case_fails()
{
  for case in [ "Resume", "Fresh", "RESUME", "FRESH", "reSume", "frEsh" ]
  {
    assert!( Strategy::from_str( case ).is_err(), "'{case}' should fail (case sensitive)" );
  }
}
