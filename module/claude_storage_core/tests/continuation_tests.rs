//! Integration tests for continuation detection.
//!
//! These tests verify that `check_continuation` and `to_storage_path_for`
//! correctly detect conversation history in `~/.claude/projects/`.
//!
//! # Test Strategy
//!
//! Tests create real storage directories under `~/.claude/projects/` using
//! the canonical `encode_path` encoding, write fixture files, run detection,
//! then clean up. This exercises the full detection path against real filesystem.
//!
//! # Encoding Note
//!
//! Storage paths use v1 lossy encoding via `encode_path()`. Only `/` and `_`
//! are replaced with `-`. Characters like `.@#%&` pass through unchanged.
//! This matches Claude Code's actual storage behavior.

use std::path::{ Path, PathBuf };
use tempfile::TempDir;
use claude_storage_core::{ encode_path, continuation };

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
  let session_dir = temp_dir.path().join( "test-session" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir );
  std::fs::create_dir_all( &claude_storage ).unwrap();

  // No conversation files — should return false
  assert!( !continuation::check_continuation( &session_dir ) );

  let _ = std::fs::remove_dir_all( &claude_storage );
}

#[ test ]
fn check_continuation_returns_true_with_jsonl_conversation_file()
{
  let temp_dir = TempDir::new().unwrap();
  let session_dir = temp_dir.path().join( "test-session" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( "ce2efe82-3c31-40d9-a6b1-33c22c13aea5.jsonl" ),
    r#"{"message":"test"}"#,
  ).unwrap();

  assert!( continuation::check_continuation( &session_dir ) );

  let _ = std::fs::remove_dir_all( &claude_storage );
}

// ============================================================================
// check_continuation — excluded file types
// ============================================================================

#[ test ]
fn check_continuation_skips_agent_definition_files()
{
  let temp_dir = TempDir::new().unwrap();
  let session_dir = temp_dir.path().join( "test-session-agent" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( "agent-custom.jsonl" ),
    r#"{"agent":"definition"}"#,
  ).unwrap();

  // Agent definition files don't count as conversations
  assert!( !continuation::check_continuation( &session_dir ) );

  let _ = std::fs::remove_dir_all( &claude_storage );
}

#[ test ]
fn check_continuation_skips_empty_files()
{
  let temp_dir = TempDir::new().unwrap();
  let session_dir = temp_dir.path().join( "test-session-empty" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( "empty.jsonl" ),
    "",  // 0-byte file (crash artifact)
  ).unwrap();

  // Empty files don't count as valid conversations
  assert!( !continuation::check_continuation( &session_dir ) );

  let _ = std::fs::remove_dir_all( &claude_storage );
}

// ============================================================================
// check_continuation — conversation file variants
// ============================================================================

#[ test ]
fn check_continuation_detects_conversation_json()
{
  let temp_dir = TempDir::new().unwrap();
  let session_dir = temp_dir.path().join( "test-session-conv-json" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( "conversation.json" ),
    r#"{"test":true}"#,
  ).unwrap();

  assert!( continuation::check_continuation( &session_dir ), "conversation.json should be detected" );

  let _ = std::fs::remove_dir_all( &claude_storage );
}

#[ test ]
fn check_continuation_detects_claude_dotfile()
{
  let temp_dir = TempDir::new().unwrap();
  let session_dir = temp_dir.path().join( "test-session-claude-dotfile" );
  std::fs::create_dir_all( &session_dir ).unwrap();

  let claude_storage = make_claude_storage( &session_dir );
  std::fs::create_dir_all( &claude_storage ).unwrap();
  std::fs::write(
    claude_storage.join( ".claude_history" ),
    "test content",
  ).unwrap();

  assert!( continuation::check_continuation( &session_dir ), ".claude* file should be detected" );

  let _ = std::fs::remove_dir_all( &claude_storage );
}

// ============================================================================
// Helper
// ============================================================================

/// Build the `~/.claude/projects/{encoded}` path for a given session directory.
///
/// Uses `encode_path()` (v1 lossy encoding: only `/` and `_` → `-`).
fn make_claude_storage( session_dir : &Path ) -> PathBuf
{
  let home_dir = std::env::var( "HOME" ).expect( "HOME must be set" );
  let encoded = encode_path( session_dir ).expect( "path must be encodable" );
  PathBuf::from( home_dir )
    .join( ".claude" )
    .join( "projects" )
    .join( encoded )
}
