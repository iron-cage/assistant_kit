//! Session path lifecycle command tests
//!
//! Integration tests for `.path`, `.exists`, `.session.dir`, and `.session.ensure`.
//!
//! ## Design Notes
//!
//! **Double-newline pitfall**: `execute_oneshot()` calls `println!("{}", content)` which
//! appends `\n`. Handler `OutputData` content must therefore NOT end with `\n`; otherwise
//! the binary emits `\n\n` and exact stdout checks like `assert_eq!(stdout, "text\n")` fail.
//! This manifested during TSK-068 in `exists_routine`, `session_dir_routine`, and
//! `session_ensure_routine` — trailing `\n` was removed from all `OutputData::new()` calls.
//!
//! **`strategy::` is label-only**: `strategy::fresh`/`strategy::resume` forces which label
//! line 2 of `.session.ensure` output shows. It does NOT wipe or recreate the directory.
//! The directory is always created idempotently via `create_dir_all`. The spec deliberately
//! omits wipe behavior; the plan's Algorithm A4 was aspirational and was not implemented.
//!
//! ## Coverage
//!
//! ### `.path`
//! - Default (cwd) computes correct storage path
//! - `path::` override computes path for given directory
//! - `topic::` appends encoded suffix
//! - Nonexistent path exits 0
//! - Empty topic rejected
//! - Slash in topic rejected
//!
//! ### `.exists`
//! - With history: exits 0, stdout `"sessions exist\n"` (exact)
//! - Without history: exits 1, stderr `"no sessions\n"` (exact, issue-033)
//! - `path::` override with history
//! - `topic::` scopes to topic storage
//! - Empty topic rejected
//!
//! ### `.session.dir`
//! - Default topic produces `{base}/-default_topic`
//! - Custom topic produces `{base}/-{topic}`
//! - Missing `path::` rejected
//! - Does not create directory
//! - Empty topic rejected
//! - Slash in topic rejected
//!
//! ### `.session.ensure`
//! - Creates directory when absent
//! - Reports `fresh` when no history
//! - Reports `resume` when history exists
//! - `strategy::resume` forces resume
//! - `strategy::fresh` forces fresh
//! - Custom topic respected
//! - Missing `path::` rejected
//! - Invalid strategy rejected
//! - Empty topic rejected

mod common;

use std::fs;
use std::path::PathBuf;

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Create storage history for a given directory path.
///
/// Encodes `project_path` and writes a non-empty `.jsonl` file into
/// `{home}/.claude/projects/{encoded}/`.
fn setup_history( home : &std::path::Path, project_path : &std::path::Path )
{
  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode_path should succeed" );
  let storage_dir = home.join( ".claude" ).join( "projects" ).join( &encoded );
  fs::create_dir_all( &storage_dir ).unwrap();
  fs::write( storage_dir.join( "session.jsonl" ), b"fake content\n" ).unwrap();
}

// ─── .path tests ─────────────────────────────────────────────────────────────

/// `.path` with no arguments returns storage path for cwd
#[ test ]
fn it_path_default_cwd()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".path" ] )
    .current_dir( project.path() )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}, stdout: {stdout}"
  );
  assert!(
    !stdout.trim().is_empty(),
    "Should output storage path. Got empty stdout"
  );
  assert!(
    stdout.contains( ".claude" ) || stdout.contains( "projects" ),
    "Output should contain storage path components. Got: {stdout}"
  );
}

/// `.path path::PATH` returns storage path for given directory
#[ test ]
fn it_path_explicit_path()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let project_path = project.path().to_str().unwrap().to_string();

  let output = common::clg_cmd()
    .args( [ ".path", &format!( "path::{project_path}" ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}, stdout: {stdout}"
  );
  assert!(
    stdout.contains( ".claude" ) || stdout.contains( "projects" ),
    "Output should contain storage path components. Got: {stdout}"
  );
}

/// `.path topic::NAME` appends topic suffix to storage path
#[ test ]
fn it_path_with_topic()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let project_path = project.path().to_str().unwrap().to_string();

  let output = common::clg_cmd()
    .args( [ ".path", &format!( "path::{project_path}" ), "topic::work" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}, stdout: {stdout}"
  );
  assert!(
    stdout.contains( "work" ),
    "Output should contain topic name. Got: {stdout}"
  );
}

/// `.path` exits 0 for nonexistent path
#[ test ]
fn it_path_nonexistent_exits_0()
{
  let home = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".path", "path::/tmp/nonexistent-path-for-test-xyz-abc" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );

  assert!(
    output.status.success(),
    "Should exit 0 for nonexistent path (path computation is filesystem-independent). stderr: {stderr}, stdout: {stdout}"
  );
}

/// `.path topic::` (empty) rejected
#[ test ]
fn it_path_empty_topic_rejected()
{
  let home = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".path", "topic::" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with empty topic. Got: {combined}"
  );
}

/// `.path topic::sub/dir` (slash in topic) rejected
#[ test ]
fn it_path_slash_in_topic_rejected()
{
  let home = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".path", "topic::sub/dir" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with slash in topic. Got: {combined}"
  );
}

// ─── .exists tests ────────────────────────────────────────────────────────────

/// `.exists` with history exits 0 and prints "sessions exist"
#[ test ]
fn it_exists_with_history_exits_0()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  setup_history( home.path(), project.path() );

  let output = common::clg_cmd()
    .args( [ ".exists", &format!( "path::{}", project.path().display() ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0 when history exists. stderr: {stderr}, stdout: {stdout}"
  );
  assert!(
    stdout.contains( "sessions exist" ),
    "Should print 'sessions exist'. Got stdout: {stdout}"
  );
  assert!(
    stderr.is_empty(),
    "stderr should be empty on success. Got: {stderr}"
  );
}

/// `.exists` without history exits 1 and prints "no sessions" on stderr
#[ test ]
fn it_exists_without_history_exits_1()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".exists", &format!( "path::{}", project.path().display() ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    !output.status.success(),
    "Should exit non-zero when no history. stdout: {stdout}, stderr: {stderr}"
  );
  assert!(
    stderr.contains( "no sessions" ),
    "Should print 'no sessions' to stderr. Got stderr: {stderr}"
  );
  assert!(
    stdout.is_empty(),
    "stdout should be empty on no-history. Got: {stdout}"
  );
}

/// `.exists` stderr is exactly "no sessions\n" when not found
///
/// ## Root Cause
///
/// `execute_oneshot` in `cli_main.rs` prepended `"Error: "` to every error message
/// via `eprintln!("Error: {error}")`. The unilang pipeline further wraps execution
/// errors with `"Execution error: Execution Error: "` prefix. Combined, the output
/// was `"Error: Execution error: Execution Error: no sessions"` — three layers of
/// wrapping — instead of the clean `"no sessions"` the spec requires.
///
/// ## Why Not Caught
///
/// The existing `it_exists_without_history_exits_1` test only asserts
/// `stderr.contains("no sessions")`, which passes even with the prefix wrapping.
/// No test pinned the exact stderr format, so the spec violation went undetected.
///
/// ## Fix Applied
///
/// `execute_oneshot` now calls `extract_user_message(&error)` which strips
/// `"Execution error: Execution Error: "` prefix and trims whitespace before
/// printing, producing the clean `"no sessions"` message the spec requires.
///
/// ## Prevention
///
/// Whenever a command has specified exact output (stdout or stderr), add an
/// `assert_eq!` exact-match test, not just `contains`. The stdout side had
/// `it_exists_stdout_exact_when_found`; the stderr side was missing its counterpart.
///
/// ## Pitfall
///
/// `execute_oneshot` adds `"Error: "` and the unilang pipeline adds its own prefix.
/// When writing handlers whose error output is consumed by shell scripts (any
/// command with informational exit-1 semantics), validate the exact stderr format
/// against the spec via an exact-match test, not just `contains`.
// test_kind: bug_reproducer(issue-033)
#[ test ]
fn it_exists_stderr_exact_when_no_history()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".exists", &format!( "path::{}", project.path().display() ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );

  assert!(
    !output.status.success(),
    "Should exit non-zero. stdout: {stdout}"
  );
  assert_eq!(
    stderr.as_ref(),
    "no sessions\n",
    "stderr must be exactly 'no sessions\\n' (spec: Exit 1: 'no sessions' on stderr)"
  );
}

/// `.exists` stdout is exactly "sessions exist\n" when found
#[ test ]
fn it_exists_stdout_exact_when_found()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  setup_history( home.path(), project.path() );

  let output = common::clg_cmd()
    .args( [ ".exists", &format!( "path::{}", project.path().display() ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );

  assert!(
    output.status.success(),
    "Should exit 0"
  );
  assert_eq!(
    stdout.as_ref(),
    "sessions exist\n",
    "stdout should be exactly 'sessions exist\\n'"
  );
}

/// `.exists` with `topic::` checks topic-specific storage
#[ test ]
fn it_exists_topic_checks_topic_storage()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  // Build the session directory path: {project}/-work
  let session_dir : PathBuf = project.path().join( "-work" );
  fs::create_dir_all( &session_dir ).unwrap();

  // Create storage history keyed to {project}/-work (the session directory itself)
  setup_history( home.path(), &session_dir );

  let output = common::clg_cmd()
    .args( [ ".exists", &format!( "path::{}", project.path().display() ), "topic::work" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0 when topic storage exists. stderr: {stderr}, stdout: {stdout}"
  );
  assert!(
    stdout.contains( "sessions exist" ),
    "Should report 'sessions exist'. Got: {stdout}"
  );
}

/// `.exists` empty topic rejected
#[ test ]
fn it_exists_empty_topic_rejected()
{
  let home = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".exists", "topic::" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with empty topic. Got: {combined}"
  );
}

// ─── .session.dir tests ───────────────────────────────────────────────────────

/// `.session.dir path::PATH` returns `{path}/-default_topic`
#[ test ]
fn it_session_dir_default_topic()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.dir", &format!( "path::{base}" ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}, stdout: {stdout}"
  );

  let expected = format!( "{base}/-default_topic\n" );
  assert_eq!(
    stdout.as_ref(),
    expected,
    "Output should be {{base}}/-default_topic"
  );
}

/// `.session.dir path::PATH topic::TOPIC` returns `{path}/-{topic}`
#[ test ]
fn it_session_dir_custom_topic()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.dir", &format!( "path::{base}" ), "topic::work" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}, stdout: {stdout}"
  );

  let expected = format!( "{base}/-work\n" );
  assert_eq!(
    stdout.as_ref(),
    expected,
    "Output should be {{base}}/-work"
  );
}

/// `.session.dir` without `path::` is rejected
#[ test ]
fn it_session_dir_missing_path_rejected()
{
  let home = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.dir" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail without path. Got: {combined}"
  );
}

/// `.session.dir` does NOT create the directory
#[ test ]
fn it_session_dir_does_not_create_directory()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.dir", &format!( "path::{base}" ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  assert!(
    output.status.success(),
    "Should exit 0"
  );

  // The session directory should NOT have been created
  let session_dir = project.path().join( "-default_topic" );
  assert!(
    !session_dir.exists(),
    ".session.dir must not create the directory. Found: {session_dir:?}"
  );
}

/// `.session.dir` empty topic rejected
#[ test ]
fn it_session_dir_empty_topic_rejected()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.dir", &format!( "path::{base}" ), "topic::" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let combined = format!(
    "{}{}",
    String::from_utf8_lossy( &output.stderr ),
    String::from_utf8_lossy( &output.stdout )
  );

  assert!(
    !output.status.success(),
    "Should fail with empty topic. Got: {combined}"
  );
}

/// `.session.dir` slash in topic rejected
#[ test ]
fn it_session_dir_slash_in_topic_rejected()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.dir", &format!( "path::{base}" ), "topic::sub/dir" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let combined = format!(
    "{}{}",
    String::from_utf8_lossy( &output.stderr ),
    String::from_utf8_lossy( &output.stdout )
  );

  assert!(
    !output.status.success(),
    "Should fail with slash in topic. Got: {combined}"
  );
}

// ─── .session.ensure tests ────────────────────────────────────────────────────

/// `.session.ensure` creates directory when absent
#[ test ]
fn it_session_ensure_creates_directory()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}, stdout: {stdout}"
  );

  // Directory should exist now
  let session_dir = project.path().join( "-default_topic" );
  assert!(
    session_dir.exists(),
    ".session.ensure should create the session directory. Path: {session_dir:?}"
  );
}

/// `.session.ensure` reports `fresh` when no history exists
#[ test ]
fn it_session_ensure_strategy_fresh_when_no_history()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}"
  );

  let lines : Vec< &str > = stdout.lines().collect();
  assert_eq!( lines.len(), 2, "Should output exactly 2 lines. Got: {stdout}" );
  assert_eq!(
    lines[ 1 ],
    "fresh",
    "Line 2 should be 'fresh' when no history. Got: {stdout}"
  );
}

/// `.session.ensure` reports `resume` when history exists
#[ test ]
fn it_session_ensure_strategy_resume_when_history_exists()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  // The session dir will be {project}/-default_topic
  let session_dir = project.path().join( "-default_topic" );
  fs::create_dir_all( &session_dir ).unwrap();
  setup_history( home.path(), &session_dir );

  let output = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}"
  );

  let lines : Vec< &str > = stdout.lines().collect();
  assert_eq!( lines.len(), 2, "Should output exactly 2 lines. Got: {stdout}" );
  assert_eq!(
    lines[ 1 ],
    "resume",
    "Line 2 should be 'resume' when history exists. Got: {stdout}"
  );
}

/// `.session.ensure` line 1 is the absolute session directory path
#[ test ]
fn it_session_ensure_line1_is_session_dir_path()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ), "topic::work" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );

  assert!( output.status.success(), "Should exit 0" );

  let lines : Vec< &str > = stdout.lines().collect();
  assert_eq!( lines.len(), 2, "Should output exactly 2 lines. Got: {stdout}" );

  let expected_dir = format!( "{base}/-work" );
  assert_eq!(
    lines[ 0 ],
    expected_dir,
    "Line 1 should be the absolute session directory path. Got: {stdout}"
  );
}

/// `.session.ensure strategy::resume` forces resume even when no history
#[ test ]
fn it_session_ensure_force_resume_overrides_fresh()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  // No history — auto-detect would give "fresh"
  let output = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ), "strategy::resume" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}"
  );

  let lines : Vec< &str > = stdout.lines().collect();
  assert_eq!( lines.len(), 2, "Should output 2 lines. Got: {stdout}" );
  assert_eq!(
    lines[ 1 ],
    "resume",
    "Forced resume should override auto-detect fresh. Got: {stdout}"
  );
}

/// `.session.ensure strategy::fresh` forces fresh even when history exists
#[ test ]
fn it_session_ensure_force_fresh_overrides_resume()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  // History exists — auto-detect would give "resume"
  let session_dir = project.path().join( "-default_topic" );
  fs::create_dir_all( &session_dir ).unwrap();
  setup_history( home.path(), &session_dir );

  let output = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ), "strategy::fresh" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}"
  );

  let lines : Vec< &str > = stdout.lines().collect();
  assert_eq!( lines.len(), 2, "Should output 2 lines. Got: {stdout}" );
  assert_eq!(
    lines[ 1 ],
    "fresh",
    "Forced fresh should override auto-detect resume. Got: {stdout}"
  );
}

/// `.session.ensure` is idempotent (second call does not fail)
#[ test ]
fn it_session_ensure_idempotent()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  // First call
  let out1 = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute (first call)" );
  assert!( out1.status.success(), "First call should succeed" );

  // Second call (directory already exists)
  let out2 = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute (second call)" );

  let stderr2 = String::from_utf8_lossy( &out2.stderr );
  let stdout2 = String::from_utf8_lossy( &out2.stdout );

  assert!(
    out2.status.success(),
    "Second call should also succeed. stderr: {stderr2}, stdout: {stdout2}"
  );
}

/// `.session.ensure` without `path::` rejected
#[ test ]
fn it_session_ensure_missing_path_rejected()
{
  let home = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.ensure" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let combined = format!(
    "{}{}",
    String::from_utf8_lossy( &output.stderr ),
    String::from_utf8_lossy( &output.stdout )
  );

  assert!(
    !output.status.success(),
    "Should fail without path. Got: {combined}"
  );
}

/// `.session.ensure strategy::invalid` rejected
#[ test ]
fn it_session_ensure_invalid_strategy_rejected()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ), "strategy::auto" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );
  let combined = format!( "{stderr}{stdout}" );

  assert!(
    !output.status.success(),
    "Should fail with invalid strategy. Got: {combined}"
  );
  assert!(
    combined.contains( "strategy" ),
    "Error should mention 'strategy'. Got: {combined}"
  );
}

/// `.session.ensure` empty topic rejected
#[ test ]
fn it_session_ensure_empty_topic_rejected()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ), "topic::" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let combined = format!(
    "{}{}",
    String::from_utf8_lossy( &output.stderr ),
    String::from_utf8_lossy( &output.stdout )
  );

  assert!(
    !output.status.success(),
    "Should fail with empty topic. Got: {combined}"
  );
}

// ─── .path additional coverage ───────────────────────────────────────────────

/// `.path` output is a single line ending with `/` (IT-5)
#[ test ]
fn it_path_output_single_line_ending_slash()
{
  let home = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".path", "path::/tmp/test-dir-path-format-check" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}"
  );

  let content = stdout.trim_end_matches( '\n' );
  let lines : Vec< &str > = content.split( '\n' ).filter( | l | !l.is_empty() ).collect();
  assert_eq!(
    lines.len(), 1,
    "Output must be exactly one non-empty line. Got: {stdout}"
  );
  assert!(
    content.ends_with( '/' ),
    "Output must end with '/'. Got: {stdout}"
  );
}

/// `.path path::~` expands tilde to home directory (IT-7)
#[ test ]
fn it_path_tilde_expansion()
{
  let home = tempfile::TempDir::new().unwrap();
  let home_str = home.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".path", "path::~/myproject" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0 after tilde expansion. stderr: {stderr}"
  );
  assert!(
    !stdout.contains( '~' ),
    "Output must not contain literal '~' — tilde must be expanded. Got: {stdout}"
  );
  assert!(
    stdout.contains( home_str ),
    "Output must contain expanded home directory. Got: {stdout}"
  );
}

/// `.path path::.` resolves to cwd (IT-8)
#[ test ]
fn it_path_dot_resolves_to_cwd()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();

  let out_no_args = common::clg_cmd()
    .args( [ ".path" ] )
    .current_dir( project.path() )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute (no args)" );

  let out_dot = common::clg_cmd()
    .args( [ ".path", "path::." ] )
    .current_dir( project.path() )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute (path::.)" );

  assert!( out_no_args.status.success(), "no-args form should succeed" );
  assert!( out_dot.status.success(), "path::. form should succeed" );

  assert_eq!(
    String::from_utf8_lossy( &out_dot.stdout ),
    String::from_utf8_lossy( &out_no_args.stdout ),
    "path::. must produce same output as bare .path from same cwd"
  );
}

// ─── .session.dir additional coverage ────────────────────────────────────────

/// `.session.dir path::~` expands tilde in `path::` (IT-5)
#[ test ]
fn it_session_dir_tilde_expansion()
{
  let home = tempfile::TempDir::new().unwrap();
  let home_str = home.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.dir", "path::~/myproject" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0 after tilde expansion. stderr: {stderr}"
  );
  assert!(
    !stdout.contains( '~' ),
    "Output must not contain literal '~' — tilde must be expanded. Got: {stdout}"
  );
  assert!(
    stdout.contains( home_str ),
    "Output must contain expanded home path. Got: {stdout}"
  );
}

/// `.session.dir path::.` resolves to cwd (IT-6)
#[ test ]
fn it_session_dir_dot_resolves_to_cwd()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let out_dot = common::clg_cmd()
    .args( [ ".session.dir", "path::." ] )
    .current_dir( project.path() )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute (path::.)" );

  let out_explicit = common::clg_cmd()
    .args( [ ".session.dir", &format!( "path::{base}" ) ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute (explicit path)" );

  assert!( out_dot.status.success(), "path::. form should succeed" );
  assert!( out_explicit.status.success(), "explicit path form should succeed" );

  assert_eq!(
    String::from_utf8_lossy( &out_dot.stdout ),
    String::from_utf8_lossy( &out_explicit.stdout ),
    "path::. must produce same output as explicit cwd path"
  );
}

/// `.session.dir` exits 0 even when given path does not exist on disk (IT-10)
#[ test ]
fn it_session_dir_exits_0_nonexistent_path()
{
  let home = tempfile::TempDir::new().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.dir", "path::/tmp/nonexistent-project-session-dir-xyz-abc" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stderr = String::from_utf8_lossy( &output.stderr );
  let stdout = String::from_utf8_lossy( &output.stdout );

  assert!(
    output.status.success(),
    ".session.dir must exit 0 even for nonexistent path (path is computed, not accessed). stderr: {stderr}, stdout: {stdout}"
  );
}

/// `.session.ensure` custom topic used in output path
#[ test ]
fn it_session_ensure_custom_topic_in_output()
{
  let home = tempfile::TempDir::new().unwrap();
  let project = tempfile::TempDir::new().unwrap();
  let base = project.path().to_str().unwrap();

  let output = common::clg_cmd()
    .args( [ ".session.ensure", &format!( "path::{base}" ), "topic::my_session" ] )
    .env( "HOME", home.path() )
    .output()
    .expect( "Failed to execute" );

  let stdout = String::from_utf8_lossy( &output.stdout );
  let stderr = String::from_utf8_lossy( &output.stderr );

  assert!(
    output.status.success(),
    "Should exit 0. stderr: {stderr}"
  );

  let lines : Vec< &str > = stdout.lines().collect();
  assert_eq!( lines.len(), 2, "Should output 2 lines. Got: {stdout}" );
  assert!(
    lines[ 0 ].ends_with( "/-my_session" ),
    "Line 1 should end with '/-my_session'. Got: {}",
    lines[ 0 ]
  );

  // Also verify the directory was created with the right name
  let session_dir = project.path().join( "-my_session" );
  assert!(
    session_dir.exists(),
    "Session dir with custom topic should be created. Path: {session_dir:?}"
  );
}
