//! Integration tests for continuation detection.
//!
//! These tests verify that `check_continuation` and `to_storage_path_for`
//! correctly detect conversation history in `~/.claude/projects/`.
//!
//! # Test Strategy
//!
//! Tests create real storage directories under a temporary home directory using
//! the canonical `encode_path` encoding, write fixture files, run detection,
//! and verify the result. `HOME` is overridden per-test via `set_var` (safe
//! because nextest runs each test in its own process). This exercises the full
//! detection path against real filesystem without touching the host `~/.claude/`.
//!
//! # Encoding Note
//!
//! Storage paths use v1 lossy encoding via `encode_path()`. Only `/` and `_`
//! are replaced with `-`. Characters like `.@#%&` pass through unchanged.
//! This matches Claude Code's actual storage behavior.

use std::path::{ Path, PathBuf };
use tempfile::TempDir;
use claude_storage_core::{ encode_path, continuation, SessionId };

// ============================================================================
// to_storage_path_for
// ============================================================================

#[ test ]
fn to_storage_path_for_constructs_home_relative_path()
{
  let session_dir = Path::new( "/home/user/project" );
  let storage = continuation::to_storage_path_for( session_dir );

  let Some( path ) = storage else
  {
    panic!( "to_storage_path_for returned None (HOME not set?)" );
  };

  let path_str = path.display().to_string();
  assert!( path_str.contains( ".claude/projects/" ), "path must be under .claude/projects/: {path_str}" );
  assert!( path_str.ends_with( "-home-user-project" ), "must end with encoded path: {path_str}" );
}

#[ test ]
fn to_storage_path_for_returns_none_for_empty_path()
{
  // encode_path fails for empty paths, so to_storage_path_for returns None
  let session_dir = Path::new( "/" );
  let result = continuation::to_storage_path_for( session_dir );

  // Either None (encode fails) or a path — just verify no panic
  // encode_path("/") returns Err, so this should be None
  assert!( result.is_none(), "empty/root path should return None" );
}

// ============================================================================
// check_continuation — basic cases
// ============================================================================

#[ test ]
fn check_continuation_returns_false_for_nonexistent_directory()
{
  let temp_dir = TempDir::new().unwrap();
  let session_dir = temp_dir.path().join( "nonexistent" );

  assert!( !continuation::check_continuation( &session_dir ) );
}

#[ test ]
fn check_continuation_returns_false_without_conversation_files()
{
  let temp_dir = TempDir::new().unwrap();
  let home_dir = TempDir::new().unwrap();
  std::env::set_var( "HOME", home_dir.path() );
  let session_dir = temp_dir.path().join( "test-session" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir, home_dir.path() );
  std::fs::create_dir_all( &claude_storage ).unwrap();

  // No conversation files — should return false
  assert!( !continuation::check_continuation( &session_dir ) );
}

#[ test ]
fn check_continuation_returns_true_with_jsonl_conversation_file()
{
  let temp_dir = TempDir::new().unwrap();
  let home_dir = TempDir::new().unwrap();
  std::env::set_var( "HOME", home_dir.path() );
  let session_dir = temp_dir.path().join( "test-session" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir, home_dir.path() );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( "ce2efe82-3c31-40d9-a6b1-33c22c13aea5.jsonl" ),
    r#"{"message":"test"}"#,
  ).unwrap();

  assert!( continuation::check_continuation( &session_dir ) );
}

// ============================================================================
// check_continuation — excluded file types
// ============================================================================

#[ test ]
fn check_continuation_skips_agent_definition_files()
{
  let temp_dir = TempDir::new().unwrap();
  let home_dir = TempDir::new().unwrap();
  std::env::set_var( "HOME", home_dir.path() );
  let session_dir = temp_dir.path().join( "test-session-agent" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir, home_dir.path() );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( "agent-custom.jsonl" ),
    r#"{"agent":"definition"}"#,
  ).unwrap();

  // Agent definition files don't count as conversations
  assert!( !continuation::check_continuation( &session_dir ) );
}

#[ test ]
fn check_continuation_skips_empty_files()
{
  let temp_dir = TempDir::new().unwrap();
  let home_dir = TempDir::new().unwrap();
  std::env::set_var( "HOME", home_dir.path() );
  let session_dir = temp_dir.path().join( "test-session-empty" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir, home_dir.path() );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( "empty.jsonl" ),
    "",  // 0-byte file (crash artifact)
  ).unwrap();

  // Empty files don't count as valid conversations
  assert!( !continuation::check_continuation( &session_dir ) );
}

// ============================================================================
// check_continuation — conversation file variants
// ============================================================================

#[ test ]
fn check_continuation_detects_conversation_json()
{
  let temp_dir = TempDir::new().unwrap();
  let home_dir = TempDir::new().unwrap();
  std::env::set_var( "HOME", home_dir.path() );
  let session_dir = temp_dir.path().join( "test-session-conv-json" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir, home_dir.path() );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( "conversation.json" ),
    r#"{"test":true}"#,
  ).unwrap();

  assert!( continuation::check_continuation( &session_dir ), "conversation.json should be detected" );
}

#[ test ]
fn check_continuation_detects_claude_dotfile()
{
  let temp_dir = TempDir::new().unwrap();
  let home_dir = TempDir::new().unwrap();
  std::env::set_var( "HOME", home_dir.path() );
  let session_dir = temp_dir.path().join( "test-session-claude-dotfile" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir, home_dir.path() );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( ".claude_history" ),
    "test content",
  ).unwrap();

  assert!( continuation::check_continuation( &session_dir ), ".claude* file should be detected" );
}

// ============================================================================
// most_recent_session_in_dir
// ============================================================================

#[ test ]
fn most_recent_session_in_dir_empty_dir_returns_none()
{
  let storage = TempDir::new().unwrap();
  let result  = continuation::most_recent_session_in_dir( storage.path() );
  assert!( result.is_none(), "empty dir must return None" );
}

#[ test ]
fn most_recent_session_in_dir_single_file_returns_its_uuid()
{
  let storage = TempDir::new().unwrap();
  std::fs::write( storage.path().join( "uuid-a.jsonl" ), b"{}" ).unwrap();
  let result = continuation::most_recent_session_in_dir( storage.path() );
  assert_eq!( result, Some( SessionId::new( "uuid-a" ) ) );
}

#[ test ]
fn most_recent_session_in_dir_returns_most_recent_of_two()
{
  let storage = TempDir::new().unwrap();
  // Write file A first, then sleep 10ms, then write file B — B is newer.
  std::fs::write( storage.path().join( "uuid-a.jsonl" ), b"{}" ).unwrap();
  std::thread::sleep( core::time::Duration::from_millis( 10 ) );
  std::fs::write( storage.path().join( "uuid-b.jsonl" ), b"{}" ).unwrap();

  let result = continuation::most_recent_session_in_dir( storage.path() );
  assert_eq!( result, Some( SessionId::new( "uuid-b" ) ), "must return the more recently written file" );
}

#[ test ]
fn most_recent_session_in_dir_skips_agent_files()
{
  let storage = TempDir::new().unwrap();
  std::fs::write( storage.path().join( "agent-abc.jsonl" ), b"{}" ).unwrap();
  let result = continuation::most_recent_session_in_dir( storage.path() );
  assert!( result.is_none(), "agent-* files must be excluded" );
}

#[ test ]
fn most_recent_session_in_dir_skips_empty_files()
{
  let storage = TempDir::new().unwrap();
  std::fs::write( storage.path().join( "zero.jsonl" ), b"" ).unwrap(); // 0-byte
  let result = continuation::most_recent_session_in_dir( storage.path() );
  assert!( result.is_none(), "0-byte files must be excluded" );
}

#[ test ]
fn most_recent_session_in_dir_skips_non_jsonl()
{
  let storage = TempDir::new().unwrap();
  std::fs::write( storage.path().join( "conversation.json" ), b"{}" ).unwrap();
  let result = continuation::most_recent_session_in_dir( storage.path() );
  assert!( result.is_none(), "non-.jsonl files must be excluded" );
}

#[ test ]
fn most_recent_session_id_encodes_cwd_and_finds_file()
{
  let home_dir = TempDir::new().unwrap();
  std::env::set_var( "HOME", home_dir.path() );
  let session_dir = TempDir::new().unwrap();
  let claude_storage = make_claude_storage( session_dir.path(), home_dir.path() );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write( claude_storage.join( "test-uuid.jsonl" ), b"{}" ).unwrap();

  let result = continuation::most_recent_session_id( session_dir.path() );
  assert_eq!( result, Some( SessionId::new( "test-uuid" ) ) );
}

// ============================================================================
// Helper
// ============================================================================

/// Build the `{home_dir}/.claude/projects/{encoded}` path for a given session directory.
///
/// Uses `encode_path()` (v1 lossy encoding: only `/` and `_` → `-`).
fn make_claude_storage( session_dir : &Path, home_dir : &Path ) -> PathBuf
{
  let encoded = encode_path( session_dir ).expect( "path must be encodable" );
  home_dir
    .join( ".claude" )
    .join( "projects" )
    .join( encoded )
}
