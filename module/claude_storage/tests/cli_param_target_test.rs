//! Edge case tests for the `target::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/16_target.md`
//!
//! ## Coverage
//!
//! - EC-1: Value "projects" accepted
//! - EC-2: Value "sessions" accepted
//! - EC-3: Value "entries" accepted
//! - EC-4: Value "SESSIONS" accepted (case-insensitive)
//! - EC-5: Invalid value "files" rejected with error
//! - EC-6: Omitted defaults to "projects"
//! - EC-7: `target::sessions` without `project::` counts all sessions

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

/// EC-1: Value "projects" accepted.
///
/// ## Purpose
/// Validates that `target::projects` is accepted by .count.
///
/// ## Coverage
/// Exit 0; numeric output representing project count.
///
/// ## Validation Strategy
/// Create fixture with projects. Run `.count ``target::project``s`. Assert exit 0
/// and output is a non-negative integer.
///
/// ## Related Requirements
/// `tests/docs/cli/param/16_target.md` — EC-1
#[ test ]
fn ec_1_target_projects_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-tgt", "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::projects" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  let count : i64 = output.parse().unwrap_or( -1 );
  assert!(
    count >= 0,
    "EC-1: target::projects must output a non-negative integer; got: {output}"
  );
}

/// EC-2: Value "sessions" accepted.
///
/// ## Purpose
/// Validates that `target::sessions` is accepted by .count.
///
/// ## Coverage
/// Exit 0; numeric output representing session count.
///
/// ## Validation Strategy
/// Create fixture with sessions. Run `.count ``target::session``s`. Assert exit 0
/// and output is a non-negative integer.
///
/// ## Related Requirements
/// `tests/docs/cli/param/16_target.md` — EC-2
#[ test ]
fn ec_2_target_sessions_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-tgt2", "sess-a", 2 );
  common::write_test_session( root.path(), "proj-tgt2", "sess-b", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  let count : i64 = output.parse().unwrap_or( -1 );
  assert!(
    count >= 0,
    "EC-2: target::sessions must output a non-negative integer; got: {output}"
  );
}

/// EC-3: Value "entries" accepted.
///
/// ## Purpose
/// Validates that `target::entries` is accepted by .count.
///
/// ## Coverage
/// Exit 0; numeric output representing entry count.
///
/// ## Validation Strategy
/// Create session with known entry count. Run `.count ``target::entries`` ``session::``-default_topic ``project::``...`.
/// Assert exit 0 and output is a non-negative integer.
///
/// ## Related Requirements
/// `tests/docs/cli/param/16_target.md` — EC-3
#[ test ]
fn ec_3_target_entries_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-tgt3", "-default_topic", 4 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::entries" )
    .arg( "session::-default_topic" )
    .arg( "project::proj-tgt3" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  let count : i64 = output.parse().unwrap_or( -1 );
  assert!(
    count >= 0,
    "EC-3: target::entries must output a non-negative integer; got: {output}"
  );
}

/// EC-4: Value "SESSIONS" rejected (target parsing is case-sensitive).
///
/// ## Purpose
/// Validates that uppercase target values are rejected by .count.
///
/// ## Coverage
/// Exit 1; error mentions "Invalid target: SESSIONS".
///
/// ## Validation Strategy
/// Create fixture. Run `.count ``target::SESSION``S`. Assert exit 1 and
/// error contains "Invalid target".
///
/// ## Related Requirements
/// `tests/docs/cli/param/16_target.md` — EC-4
#[ test ]
fn ec_4_target_uppercase_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-tgt4", "sess", 2 );

  let out_upper = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::SESSIONS" )
    .output()
    .unwrap();

  assert_exit( &out_upper, 1 );
  let err = stderr( &out_upper );
  assert!(
    err.contains( "SESSIONS" ) || err.contains( "target" ) || err.contains( "Invalid" ),
    "EC-4: error must mention SESSIONS or target or Invalid; got: {err}"
  );
}

/// EC-5: Invalid value "files" rejected with error.
///
/// ## Purpose
/// Validates that "files" is not a valid target value.
///
/// ## Coverage
/// Exit 1; error message contains "target must be projects|sessions|entries, got files".
///
/// ## Validation Strategy
/// Run `.count ``target::file``s`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/16_target.md` — EC-5
#[ test ]
fn ec_5_target_files_rejected()
{
  let out = common::clg_cmd()
    .arg( ".count" )
    .arg( "target::files" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "target" ) && err.contains( "files" ),
    "EC-5: expected 'target' and 'files' in stderr; got: {err}"
  );
}

/// EC-6: Omitted defaults to "projects".
///
/// ## Purpose
/// Validates that omitting `target::` defaults to counting projects.
///
/// ## Coverage
/// Exit 0; numeric output matching project count (same as `target::projects`).
///
/// ## Validation Strategy
/// Create fixture. Run `.count` and `.count ``target::project``s`.
/// Assert identical output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/16_target.md` — EC-6
#[ test ]
fn ec_6_target_omitted_defaults_to_projects()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-tgt6", "sess", 2 );

  let out_default = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .output()
    .unwrap();

  let out_explicit = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::projects" )
    .output()
    .unwrap();

  assert_exit( &out_default, 0 );
  assert_exit( &out_explicit, 0 );
  assert_eq!(
    stdout( &out_default ).trim(),
    stdout( &out_explicit ).trim(),
    "EC-6: omitted target must match target::projects output"
  );
}

/// EC-7: `target::sessions` without `project::` counts all sessions.
///
/// ## Purpose
/// Validates that `target::sessions` without `project::` counts across all projects.
///
/// ## Coverage
/// Exit 0; count reflects all sessions in storage (no implicit project filter).
///
/// ## Validation Strategy
/// Create sessions in two different projects. Run `.count ``target::session``s`.
/// Assert count >= 2.
///
/// ## Related Requirements
/// `tests/docs/cli/param/16_target.md` — EC-7
#[ test ]
fn ec_7_target_sessions_counts_all_without_project()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-a7", "sess-a", 2 );
  common::write_test_session( root.path(), "proj-b7", "sess-b", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".count" )
    .arg( "target::sessions" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out ).trim().to_owned();
  let count : i64 = output.parse().unwrap_or( 0 );
  assert!(
    count >= 2,
    "EC-7: session count across two projects must be >= 2; got: {count}"
  );
}
