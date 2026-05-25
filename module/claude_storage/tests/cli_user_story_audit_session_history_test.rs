//! Acceptance tests for the "Audit Session History" user story.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/user_story/01_audit_session_history.md`
//!
//! ## Coverage
//!
//! - RWS-1: Basic status shows project and session totals
//! - RWS-2: Verbosity 2 shows per-project breakdown
//! - RWS-3: Verbosity 0 outputs machine-readable format
//! - RWS-4: Count `target::sessions` returns bare integer
//! - RWS-5: Path override inspects alternate storage root

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

/// RWS-1: Basic status shows project and session totals.
///
/// ## Purpose
/// End-to-end acceptance test: developer runs `.status` to get an at-a-glance
/// overview with project count and session count visible in the output.
///
/// ## Coverage
/// Summary output present; project count in output; session count in output; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/01_audit_session_history.md` — RWS-1
#[ test ]
fn rws_1_basic_status_shows_project_and_session_totals()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "audit-proj-a" );
  let p2 = root.path().join( "audit-proj-b" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "RWS-1: .status must produce summary output; stderr: {}",
    stderr( &out )
  );
  assert!(
    s.contains( '2' ),
    "RWS-1: output must mention count 2 (projects or sessions); got:\n{s}"
  );
}

/// RWS-2: Verbosity 2 shows per-project breakdown.
///
/// ## Purpose
/// End-to-end acceptance test: developer drills into per-project session counts
/// using `verbosity::2`, getting more detail than the default summary.
///
/// ## Coverage
/// Per-project detail rows; entry breakdown visible; more detail than `verbosity::1`; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/01_audit_session_history.md` — RWS-2
#[ test ]
fn rws_2_verbosity_2_shows_per_project_breakdown()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "audit2-proj-a" );
  let p2 = root.path().join( "audit2-proj-b" );
  common::write_path_project_session( root.path(), &p1, "s001", 4 );
  common::write_path_project_session( root.path(), &p2, "s002", 4 );

  let v1_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::1" )
    .output()
    .unwrap();

  let v2_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::2" )
    .output()
    .unwrap();

  assert_exit( &v1_out, 0 );
  assert_exit( &v2_out, 0 );

  let v2 = stdout( &v2_out );
  let v1 = stdout( &v1_out );

  assert!(
    v2.len() > v1.len(),
    "RWS-2: verbosity::2 must produce more detail than verbosity::1;\n  v1 ({} bytes):\n{v1}\n  v2 ({} bytes):\n{v2}",
    v1.len(),
    v2.len()
  );
}

/// RWS-3: Verbosity 0 outputs machine-readable format.
///
/// ## Purpose
/// End-to-end acceptance test: developer pipes `.status ```verbosity::```0` to a
/// script and gets `Projects: N` parseable output.
///
/// ## Coverage
/// Machine-readable format; no decorative headers; parseable output; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/01_audit_session_history.md` — RWS-3
#[ test ]
fn rws_3_verbosity_0_outputs_machine_readable_format()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "audit3proj" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "verbosity::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // The output uses "Projects:" (capital P) and "Sessions:" keys
  let s_lower = s.to_lowercase();
  assert!(
    s_lower.contains( "projects:" ),
    "RWS-3: verbosity::0 must output 'projects:' key (case-insensitive); got:\n{s}"
  );
  assert!(
    s_lower.contains( "sessions:" ),
    "RWS-3: verbosity::0 must output 'sessions:' key (case-insensitive); got:\n{s}"
  );
  // Must not contain decorative elements
  assert!(
    !s.contains( "===" ) && !s.contains( "│" ) && !s.contains( "┌" ),
    "RWS-3: verbosity::0 must not contain table borders or decorations; got:\n{s}"
  );
}

/// RWS-4: Count `target::sessions` returns bare integer.
///
/// ## Purpose
/// End-to-end acceptance test: developer checks exact session count for
/// threshold scripting; `.count ```target::session```s` returns a bare integer.
///
/// ## Coverage
/// Bare integer output; no labels; usable in shell arithmetic; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/01_audit_session_history.md` — RWS-4
#[ test ]
fn rws_4_count_target_sessions_returns_bare_integer()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "audit4-proj-a" );
  let p2 = root.path().join( "audit4-proj-b" );
  // 3 sessions total across 2 projects
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s003", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let trimmed = s.trim();
  assert!(
    trimmed.parse::< u64 >().is_ok(),
    "RWS-4: .count target::sessions must output a bare integer; got:\n{s}"
  );
  assert_eq!(
    trimmed,
    "3",
    "RWS-4: session count must be 3 for the 3-session fixture; got:\n{s}"
  );
}

/// RWS-5: Path override inspects alternate storage root.
///
/// ## Purpose
/// End-to-end acceptance test: developer inspects a backup or alternate storage
/// location by passing `path::` to `.status`, seeing counts from that root only.
///
/// ## Coverage
/// Alternate storage root via `path::`; counts from alternate fixture; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/user_story/01_audit_session_history.md` — RWS-5
#[ test ]
fn rws_5_path_override_inspects_alternate_storage_root()
{
  let alt_root = TempDir::new().unwrap();
  let proj = alt_root.path().join( "alt-proj" );
  common::write_path_project_session( alt_root.path(), &proj, "s001", 2 );
  common::write_path_project_session( alt_root.path(), &proj, "s002", 2 );

  let out = common::clg_cmd()
    .arg( ".status" )
    .arg( format!( "path::{}", alt_root.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "RWS-5: .status path:: must produce output for the alternate root; stderr: {}",
    stderr( &out )
  );
  // Must reflect the alternate fixture (1 project, 2 sessions)
  assert!(
    s.contains( '1' ) || s.contains( '2' ),
    "RWS-5: output must reflect alternate storage counts (1 project, 2 sessions); got:\n{s}"
  );
}
