//! Edge case tests for the `session_id::` parameter (direct identifier).
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/14_session_id.md`
//!
//! ## Coverage
//!
//! - EC-1: Named session ID (e.g., -`default_topic`) accepted
//! - EC-2: UUID session ID accepted
//! - EC-3: Empty value rejected
//! - EC-4: Unknown session ID exits with error
//! - EC-5: Required in .export — missing exits with 1
//! - EC-6: Optional in .show — absent shows project
//! - EC-7: Whitespace-only value rejected

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

/// EC-1: Named session ID (e.g., -`default_topic`) accepted.
///
/// ## Purpose
/// Validates that a named session ID is accepted in .show.
///
/// ## Coverage
/// Exit 0; content from -`default_topic` session displayed.
///
/// ## Validation Strategy
/// Create session "-`default_topic`". Run `.show ``session_id::``-default_topic`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/14_session_id.md` — EC-1
#[ test ]
fn ec_1_session_id_named_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sid", "-default_topic", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-default_topic" )
    .arg( "project::proj-sid" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty(),
    "EC-1: named session_id must produce output; got empty"
  );
}

/// EC-2: UUID session ID accepted.
///
/// ## Purpose
/// Validates that a UUID-format session ID is accepted in .show.
///
/// ## Coverage
/// Exit 0; content from UUID session displayed.
///
/// ## Validation Strategy
/// Create session with UUID name. Run `.show ``session_id::``{uuid}`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/14_session_id.md` — EC-2
#[ test ]
fn ec_2_session_id_uuid_accepted()
{
  let root = TempDir::new().unwrap();
  let uuid = "8d795a1c-c81d-4010-8d29-b4e678272419";
  common::write_test_session( root.path(), "proj-sid2", uuid, 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( format!( "session_id::{uuid}" ) )
    .arg( "project::proj-sid2" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty(),
    "EC-2: UUID session_id must produce output; got empty"
  );
}

/// EC-3: Empty value rejected.
///
/// ## Purpose
/// Validates that `session_id::` with empty value is rejected.
///
/// ## Coverage
/// Exit 1; error about empty `session_id` value.
///
/// ## Validation Strategy
/// Run `.show session_id::`. Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/14_session_id.md` — EC-3
#[ test ]
fn ec_3_session_id_empty_rejected()
{
  let out = common::clg_cmd()
    .arg( ".show" )
    .arg( "session_id::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "session" ),
    "EC-3: error must mention 'session'; got: {combined}"
  );
}

/// EC-4: Unknown session ID exits with error.
///
/// ## Purpose
/// Validates that an unknown `session_id` produces a not-found error.
///
/// ## Coverage
/// Exit 1; error message "session not found: -nonexistent-session-zzz".
///
/// ## Validation Strategy
/// Run `.show ``session_id::``-nonexistent-session-zzz`. Assert exit 1 and error.
///
/// ## Related Requirements
/// `tests/docs/cli/param/14_session_id.md` — EC-4
#[ test ]
fn ec_4_session_id_unknown_exits_with_error()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sid4", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".show" )
    .arg( "session_id::-nonexistent-session-zzz" )
    .arg( "project::proj-sid4" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "session" ) && ( combined.contains( "not found" ) || combined.contains( "nonexistent" ) ),
    "EC-4: error must mention session not found; got: {combined}"
  );
}

/// EC-5: Required in .export — missing exits with 1.
///
/// ## Purpose
/// Validates that .export requires `session_id::`.
///
/// ## Coverage
/// Exit 1; error about missing `session_id::` for .export.
///
/// ## Validation Strategy
/// Create fixture. Run `.export ``output::``...` with no `session_id`.
/// Assert exit 1 and error mentions `session_id`.
///
/// ## Related Requirements
/// `tests/docs/cli/param/14_session_id.md` — EC-5
#[ test ]
fn ec_5_session_id_required_in_export()
{
  let root = TempDir::new().unwrap();
  let out_dir = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sid5", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".export" )
    .arg( format!( "output::{}", out_dir.path().join( "out.md" ).display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "session" ) || combined.contains( "output" ),
    "EC-5: error must mention session_id or output requirement; got: {combined}"
  );
}

/// EC-6: Optional in .show — absent shows project.
///
/// ## Purpose
/// Validates that omitting `session_id::` in .show shows project-level view.
///
/// ## Coverage
/// Exit 0; project view shown (not a single-session view, not an error).
///
/// ## Validation Strategy
/// Create project with session. Run `.show` from project dir with no `session_id`.
/// Assert exit 0 and no error.
///
/// ## Related Requirements
/// `tests/docs/cli/param/14_session_id.md` — EC-6
#[ test ]
fn ec_6_session_id_optional_in_show_shows_project()
{
  let root = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  common::write_path_project_session( root.path(), project_dir.path(), "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( project_dir.path() )
    .arg( ".show" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-7: Whitespace-only value rejected.
///
/// ## Purpose
/// Validates that a whitespace-only `session_id` value is rejected.
///
/// ## Coverage
/// Exit 1; error about whitespace-only `session_id` value.
///
/// ## Validation Strategy
/// Run `.show ``session_id::``   ` (spaces only). Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/14_session_id.md` — EC-7
#[ test ]
fn ec_7_session_id_whitespace_only_rejected()
{
  let out = common::clg_cmd()
    .arg( ".show" )
    .arg( "session_id::   " )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  let combined_lower = combined.to_lowercase();
  assert!(
    combined_lower.contains( "session" ) || combined_lower.contains( "empty" ),
    "EC-7: error must mention session or empty; got: {combined}"
  );
}
