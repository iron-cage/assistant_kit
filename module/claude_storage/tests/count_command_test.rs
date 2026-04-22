//! Tests for `.count` command — conversation counting.
//!
//! ## Coverage
//!
//! - `target::conversations` requires `project::` parameter
//! - `target::conversations project::<id>` outputs bare integer conversation count
//! - `target::conversations project::<id>` with 1 session → "1" (singular edge case)
//! - `target::conversations project::<id>` with 0 sessions → "0" (empty project)
//! - `target::conversations project::<nonexistent>` → error (invalid project)

mod common;

// ────────────────────────────────────────────────────────────────────────────
// IT-T04: `.count target::conversations project::<id>` outputs bare integer
// ────────────────────────────────────────────────────────────────────────────
/// IT-T04: `.count target::conversations` outputs a bare integer conversation count.
///
/// With two sessions in the project and the 1:1 identity mapping, expects count = 2.
#[ test ]
fn it_count_target_conversations()
{
  use tempfile::TempDir;
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj-count-t04" );
  let encoded = common::write_path_project_session( &storage_root, &project, "sess-t04-a", 2 );
  common::write_path_project_session( &storage_root, &project, "sess-t04-b", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".count" )
    .arg( "target::conversations" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert!(
    out.status.success(),
    ".count target::conversations must exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
  let s = String::from_utf8_lossy( &out.stdout ).trim().to_string();
  assert!(
    s.parse::< usize >().is_ok(),
    "output must be a bare integer; got: '{s}'",
  );
  assert_eq!( s, "2", "expected 2 conversations (1:1 with sessions); got: '{s}'" );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-T05: `.count target::conversations` without `project::` returns error
// ────────────────────────────────────────────────────────────────────────────
/// IT-T05: `.count target::conversations` without `project::` must fail with a clear error.
///
/// Conversations require project scope; omitting `project::` is an error.
#[ test ]
fn it_count_conversations_requires_project()
{
  use tempfile::TempDir;
  let root = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".count" )
    .arg( "target::conversations" )
    .output()
    .unwrap();

  assert!(
    !out.status.success(),
    ".count target::conversations without project:: must exit non-0; got: {:?}",
    out.status.code(),
  );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-T06: `.count target::conversations project::<id>` with 1 session → "1"
// ────────────────────────────────────────────────────────────────────────────
/// IT-T06: `.count target::conversations` with exactly 1 session outputs "1".
///
/// Singular edge case complementary to IT-T04 (count=2).
/// With the 1:1 identity mapping one session = one conversation, so count must be "1".
#[ test ]
fn it_count_one_conversation()
{
  use tempfile::TempDir;
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj-count-t06" );
  let encoded = common::write_path_project_session( &storage_root, &project, "sess-t06-only", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".count" )
    .arg( "target::conversations" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert!(
    out.status.success(),
    ".count target::conversations must exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
  let s = String::from_utf8_lossy( &out.stdout ).trim().to_string();
  assert_eq!( s, "1", "1 session → 1 conversation (1:1 identity mapping); got: '{s}'" );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-T07: `.count target::conversations project::<id>` with 0 sessions → "0"
// ────────────────────────────────────────────────────────────────────────────
/// IT-T07: `.count target::conversations` on an empty project outputs "0".
///
/// Project directory exists in storage but has no JSONL files.
/// Must exit 0 and output "0", not crash or error.
#[ test ]
fn it_count_zero_conversations_empty_project()
{
  use tempfile::TempDir;
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj-count-t07-empty" );

  // Create project directory without any session files
  let encoded = claude_storage_core::encode_path( &project )
    .expect( "encode project path" );
  let dir = storage_root.join( "projects" ).join( &encoded );
  std::fs::create_dir_all( &dir ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".count" )
    .arg( "target::conversations" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert!(
    out.status.success(),
    ".count target::conversations on empty project must exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
  let s = String::from_utf8_lossy( &out.stdout ).trim().to_string();
  assert_eq!( s, "0", "empty project → 0 conversations; got: '{s}'" );
}

// ────────────────────────────────────────────────────────────────────────────
// IT-T08: `.count target::conversations project::<nonexistent>` → error
// ────────────────────────────────────────────────────────────────────────────
/// IT-T08: `.count target::conversations` with a nonexistent project must fail.
///
/// A valid encoded project ID with no storage directory is an error condition.
/// Must exit non-0 — silently returning 0 would hide a user mistake.
#[ test ]
fn it_count_conversations_nonexistent_project_fails()
{
  use tempfile::TempDir;
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Encode a path but intentionally do NOT create the project dir in storage
  let fake_project = root.path().join( "nonexistent-proj-t08" );
  let encoded = claude_storage_core::encode_path( &fake_project )
    .expect( "encode project path" );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".count" )
    .arg( "target::conversations" )
    .arg( format!( "project::{encoded}" ) )
    .output()
    .unwrap();

  assert!(
    !out.status.success(),
    ".count target::conversations with nonexistent project must exit non-0; stdout: {}; stderr: {}",
    String::from_utf8_lossy( &out.stdout ),
    String::from_utf8_lossy( &out.stderr ),
  );
}
