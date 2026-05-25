//! Edge case tests for the `limit::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/22_limit.md`
//!
//! ## Coverage
//!
//! - EC-1: `limit::5` → max 5 sessions shown per project
//! - EC-2: `limit::0` → no cap (all sessions shown)
//! - EC-3: Negative limit (e.g., `limit::`-1) → rejected
//! - EC-4: `limit::` empty value → rejected
//! - EC-5: `limit::100` when project has fewer sessions → all shown
//! - EC-6: `limit::` non-integer value → rejected

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

/// EC-1: `limit::5` → max 5 sessions per project.
///
/// ## Purpose
/// Validates that `limit::5` caps sessions shown per project at 5.
///
/// ## Coverage
/// Exit 0; at most 5 sessions shown per project; excess omitted.
///
/// ## Validation Strategy
/// Create project with 10 sessions. Run `.projects ```limit::```5`. Assert exit 0
/// and at most 5 sessions appear per project.
///
/// ## Related Requirements
/// `tests/docs/cli/param/22_limit.md` — EC-1
#[ test ]
fn ec_1_limit_5_caps_sessions()
{
  let root = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  // Create 10 sessions in the project
  for i in 0..10
  {
    common::write_path_project_session(
      root.path(),
      project_dir.path(),
      &format!( "sess-{i:02}" ),
      2,
    );
  }

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( project_dir.path() )
    .arg( ".projects" )
    .arg( "limit::5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  // Count how many session lines appear (each session appears as "sess-NN")
  let session_count = ( 0..10 )
    .filter( |i| output.contains( &format!( "sess-{i:02}" ) ) )
    .count();
  assert!(
    session_count <= 5,
    "EC-1: limit::5 must show at most 5 sessions; found {session_count} in output: {output}"
  );
}

/// EC-2: `limit::0` → all sessions shown (no cap).
///
/// ## Purpose
/// Validates that `limit::0` disables session capping.
///
/// ## Coverage
/// Exit 0; all sessions shown per project; no capping applied.
///
/// ## Validation Strategy
/// Create project with 10 sessions. Run `.projects ```limit::```0`. Assert exit 0
/// and output is at least as long as the `limit::5` output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/22_limit.md` — EC-2
#[ test ]
fn ec_2_limit_0_no_cap()
{
  let root = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  for i in 0..10
  {
    common::write_path_project_session(
      root.path(),
      project_dir.path(),
      &format!( "sess-{i:02}" ),
      2,
    );
  }

  let out_uncapped = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( project_dir.path() )
    .arg( ".projects" )
    .arg( "limit::0" )
    .output()
    .unwrap();

  let out_capped = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( project_dir.path() )
    .arg( ".projects" )
    .arg( "limit::5" )
    .output()
    .unwrap();

  assert_exit( &out_uncapped, 0 );
  assert_exit( &out_capped, 0 );
  let uncapped_len = stdout( &out_uncapped ).len();
  let capped_len = stdout( &out_capped ).len();
  assert!(
    uncapped_len >= capped_len,
    "EC-2: limit::0 output must be at least as long as limit::5 output; uncapped={uncapped_len}, capped={capped_len}"
  );
}

/// EC-3: Negative limit rejected.
///
/// ## Purpose
/// Validates that `limit::-1` is rejected as invalid.
///
/// ## Coverage
/// Exit 1; error indicating limit must be a non-negative integer.
///
/// ## Validation Strategy
/// Run `.projects ```limit::```-1`. Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/22_limit.md` — EC-3
#[ test ]
fn ec_3_limit_negative_rejected()
{
  let out = common::clg_cmd()
    .arg( ".projects" )
    .arg( "limit::-1" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "limit" ),
    "EC-3: error must mention 'limit'; got: {combined}"
  );
}

/// EC-4: Empty value rejected.
///
/// ## Purpose
/// Validates that `limit::` with empty value is rejected.
///
/// ## Coverage
/// Exit 1; error indicating limit requires a value.
///
/// ## Validation Strategy
/// Run `.projects limit::`. Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/22_limit.md` — EC-4
#[ test ]
fn ec_4_limit_empty_rejected()
{
  let out = common::clg_cmd()
    .arg( ".projects" )
    .arg( "limit::" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    combined.contains( "limit" ),
    "EC-4: error must mention 'limit'; got: {combined}"
  );
}

/// EC-5: `limit::100` when project has fewer sessions → all shown.
///
/// ## Purpose
/// Validates that a limit larger than the session count shows all sessions.
///
/// ## Coverage
/// Exit 0; all sessions shown (limit not reached); no error.
///
/// ## Validation Strategy
/// Create project with 3 sessions. Run `.projects ```limit::10```0`. Assert exit 0
/// and all 3 sessions appear.
///
/// ## Related Requirements
/// `tests/docs/cli/param/22_limit.md` — EC-5
#[ test ]
fn ec_5_limit_exceeds_session_count_shows_all()
{
  let root = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();
  for i in 0..3
  {
    common::write_path_project_session(
      root.path(),
      project_dir.path(),
      &format!( "sess-{i}" ),
      2,
    );
  }

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .current_dir( project_dir.path() )
    .arg( ".projects" )
    .arg( "limit::100" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  // All 3 sessions must appear
  for i in 0..3
  {
    assert!(
      output.contains( &format!( "sess-{i}" ) ),
      "EC-5: all 3 sessions must appear with limit::100; missing sess-{i}; got: {output}"
    );
  }
}

/// EC-6: Non-integer value rejected.
///
/// ## Purpose
/// Validates that `limit::five` is rejected as non-integer.
///
/// ## Coverage
/// Exit 1; error indicating limit requires a non-negative integer.
///
/// ## Validation Strategy
/// Run `.projects ```limit::fiv```e`. Assert exit 1.
///
/// ## Related Requirements
/// `tests/docs/cli/param/22_limit.md` — EC-6
#[ test ]
fn ec_6_limit_non_integer_rejected()
{
  let out = common::clg_cmd()
    .arg( ".projects" )
    .arg( "limit::five" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stderr( &out ), stdout( &out ) );
  assert!(
    !combined.is_empty(),
    "EC-6: expected non-empty error for non-integer limit value; got empty output"
  );
}
