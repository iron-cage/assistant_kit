//! Edge-case tests for `.projects` scope parameter acceptance and rejection.
//!
//! ## Coverage
//!
//! EC-1..EC-8 from `docs/cli/testing/param/scope.md`:
//! - EC-1..EC-4: each valid scope value accepted (exit 0)
//! - EC-5: scope is case-insensitive
//! - EC-6: invalid scope value rejected (exit 1)
//! - EC-7: omitted scope defaults to `around`
//! - EC-8: `scope::global` ignores the `path::` parameter
//!
//! ## Related Files
//!
//! - `projects_scope_test.rs` — scope behavioral semantics (which sessions are returned)
//! - `projects_command_test.rs` — filter and output formatting tests
//! - `projects_family_display_test.rs` — family/agent hierarchy display (IT-36..IT-48)
//! - `projects_scope_around_test.rs` — `scope::around` neighborhood semantics (IT-57..IT-59)

mod common;

use tempfile::TempDir;

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

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

// ────────────────────────────────────────────────────────────────────────────
// EC-1: scope::local accepted — exit 0
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec1_scope_local_accepted()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::local" )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-2: scope::relevant accepted — exit 0
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec2_scope_relevant_accepted()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::relevant" )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-3: scope::under accepted — exit 0 with path::
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec3_scope_under_accepted()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( format!( "path::{}", root.path().display() ) )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-4: scope::global accepted — exit 0
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec4_scope_global_accepted()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();
  assert_exit( &out, 0 );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-5: scope::RELEVANT (uppercase) is case-insensitive — same output as lowercase
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec5_scope_case_insensitive()
{
  let root = TempDir::new().unwrap();
  let home = root.path().to_str().unwrap();

  let lower = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".projects" ).arg( "scope::relevant" )
    .output().unwrap();

  let upper = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".projects" ).arg( "scope::RELEVANT" )
    .output().unwrap();

  assert_exit( &lower, 0 );
  assert_exit( &upper, 0 );
  assert_eq!(
    lower.stdout, upper.stdout,
    "scope::relevant and scope::RELEVANT must produce identical stdout"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-6: scope::all (invalid) → exit 1 with exact error message
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec6_invalid_scope_rejected()
{
  let root = TempDir::new().unwrap();
  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::all" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "scope must be relevant|local|under|around|global, got all" ),
    "error must contain exact message; got: {err}"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-7: scope:: omitted → defaults to around (same output as explicit scope::around)
//
// Fixture: parent project + child project so that scope::local and scope::around
// produce different results. Around includes the child; local doesn't.
// (No ancestor projects in storage → around output equals under output here.)
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec7_omitted_scope_defaults_to_around()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  let parent = root.path().join( "parent_proj" );
  let child  = parent.join( "child_sub" );
  common::write_path_project_session( &storage_root, &parent, "session-parent", 1 );
  common::write_path_project_session( &storage_root, &child,  "session-child",  1 );

  let implicit = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( format!( "path::{}", parent.display() ) )
    .output().unwrap();

  let explicit = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::around" )
    .arg( format!( "path::{}", parent.display() ) )
    .output().unwrap();

  assert_exit( &implicit, 0 );
  let s = core::str::from_utf8( &implicit.stdout ).unwrap();
  // default scope must include descendant sessions (around includes under direction)
  assert!(
    s.contains( "session-child" ),
    "default scope must include descendant sessions (around behavior); got:\n{s}"
  );
  assert_eq!(
    implicit.stdout, explicit.stdout,
    "omitting scope:: must produce same output as scope::around"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-8: scope::global ignores path:: — output identical with or without path::
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn ec8_global_ignores_path()
{
  let root = TempDir::new().unwrap();
  let home = root.path().to_str().unwrap();

  let without_path = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".projects" ).arg( "scope::global" )
    .output().unwrap();

  let with_path = common::clg_cmd()
    .env( "HOME", home )
    .arg( ".projects" ).arg( "scope::global" ).arg( "path::/nonexistent-subpath" )
    .output().unwrap();

  assert_exit( &without_path, 0 );
  assert_exit( &with_path, 0 );
  assert_eq!(
    without_path.stdout, with_path.stdout,
    "scope::global must produce identical output regardless of path::"
  );
}

// ────────────────────────────────────────────────────────────────────────────
// EC-9: scope::under with path::/ → exit 1 (unencodable root path)
//
// Validates that passing path::/ to a scope that encodes the base path
// fails with exit 1 and a clear error. scope::global is exempt because it
// never encodes the path. See also EC-8 which confirms scope::global succeeds.
// ────────────────────────────────────────────────────────────────────────────
#[test]
fn root_path_rejected_for_non_global_scope()
{
  let root = TempDir::new().unwrap();

  // scope::under with path::/ must fail (unencodable base path)
  let out_under = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::under" )
    .arg( "path::/" )
    .output()
    .unwrap();

  assert_ne!(
    out_under.status.code().unwrap_or( -1 ), 0,
    "scope::under path::/ must exit non-zero; got exit 0"
  );
  assert!(
    stderr( &out_under ).contains( "path is empty after normalization" )
      || stderr( &out_under ).contains( "Failed to encode" ),
    "error must mention path encoding failure; got: {}",
    stderr( &out_under )
  );

  // scope::global with path::/ must still succeed (global ignores path)
  let out_global = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "path::/" )
    .output()
    .unwrap();

  assert_exit( &out_global, 0 );
}
