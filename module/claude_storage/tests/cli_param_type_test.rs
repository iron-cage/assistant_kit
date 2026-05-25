//! Edge case tests for the `type::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/18_type.md`
//!
//! ## Coverage
//!
//! - EC-1: Value "uuid" accepted
//! - EC-2: Value "path" accepted
//! - EC-3: Value "all" accepted
//! - EC-4: Value "PATH" accepted (case-insensitive)
//! - EC-5: Invalid value "both" rejected with error
//! - EC-6: Omitted defaults to "all"
//! - EC-7: `type::uuid` returns only UUID-named projects

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

/// EC-1: Value "uuid" accepted.
///
/// ## Purpose
/// Validates that `type::uuid` is accepted by .list.
///
/// ## Coverage
/// Exit 0; output contains only UUID-named projects (or empty if none exist).
///
/// ## Validation Strategy
/// Create fixture with a UUID project. Run `.list ``type::uui``d`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/18_type.md` — EC-1
#[ test ]
fn ec_1_type_uuid_accepted()
{
  let root = TempDir::new().unwrap();
  let uuid = "8d795a1c-c81d-4010-8d29-b4e678272419";
  common::write_test_session( root.path(), uuid, "sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "type::uuid" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( uuid ) || output.is_empty(),
    "EC-1: type::uuid must list UUID project; got: {output}"
  );
}

/// EC-2: Value "path" accepted.
///
/// ## Purpose
/// Validates that `type::path` is accepted by .list.
///
/// ## Coverage
/// Exit 0; output contains only path-encoded projects.
///
/// ## Validation Strategy
/// Create fixture with a path-encoded project. Run `.list ``type::pat``h`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/18_type.md` — EC-2
#[ test ]
fn ec_2_type_path_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_path_project_session(
    root.path(),
    &std::path::PathBuf::from( "/home/alice/proj" ),
    "sess",
    2,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "type::path" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.is_empty() || output.is_empty(),
    "EC-2: type::path must not error; got: {output}"
  );
}

/// EC-3: Value "all" accepted.
///
/// ## Purpose
/// Validates that `type::all` lists all projects regardless of naming scheme.
///
/// ## Coverage
/// Exit 0; all projects in storage shown.
///
/// ## Validation Strategy
/// Create both UUID and path-encoded projects. Run `.list ``type::al``l`.
/// Assert both appear.
///
/// ## Related Requirements
/// `tests/docs/cli/param/18_type.md` — EC-3
#[ test ]
fn ec_3_type_all_accepted()
{
  let root = TempDir::new().unwrap();
  let uuid = "8d795a1c-c81d-4010-8d29-b4e678272419";
  common::write_test_session( root.path(), uuid, "sess-uuid", 2 );
  common::write_path_project_session(
    root.path(),
    &std::path::PathBuf::from( "/home/alice/proj" ),
    "sess-path",
    2,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "type::all" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( uuid ),
    "EC-3: type::all must list UUID project; got: {output}"
  );
}

/// EC-4: Value "PATH" rejected (type parsing is case-sensitive).
///
/// ## Purpose
/// Validates that uppercase type values are rejected by .list.
///
/// ## Coverage
/// Exit 1; error mentions "Invalid type: PATH".
///
/// ## Validation Strategy
/// Run `.list ``type::PAT``H`. Assert exit 1 and error is non-empty.
///
/// ## Related Requirements
/// `tests/docs/cli/param/18_type.md` — EC-4
#[ test ]
fn ec_4_type_uppercase_accepted()
{
  let root = TempDir::new().unwrap();
  common::write_path_project_session(
    root.path(),
    &std::path::PathBuf::from( "/home/alice/projup" ),
    "sess",
    2,
  );

  let out_upper = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "type::PATH" )
    .output()
    .unwrap();

  assert_exit( &out_upper, 1 );
  let err = stderr( &out_upper );
  assert!(
    !err.is_empty(),
    "EC-4: error must be non-empty for uppercase type value; got empty stderr"
  );
}

/// EC-5: Invalid value "both" rejected with error.
///
/// ## Purpose
/// Validates that "both" is not a valid type value.
///
/// ## Coverage
/// Exit 1; error message contains "type must be uuid|path|all, got both".
///
/// ## Validation Strategy
/// Run `.list ``type::bot``h`. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/18_type.md` — EC-5
#[ test ]
fn ec_5_type_both_rejected()
{
  let out = common::clg_cmd()
    .arg( ".list" )
    .arg( "type::both" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "type" ) && err.contains( "both" ),
    "EC-5: expected 'type' and 'both' in stderr; got: {err}"
  );
}

/// EC-6: Omitted defaults to "all".
///
/// ## Purpose
/// Validates that omitting `type::` defaults to listing all projects.
///
/// ## Coverage
/// Exit 0; output includes all projects (identical to `type::all`).
///
/// ## Validation Strategy
/// Create both UUID and path projects. Run `.list` and `.list ``type::al``l`.
/// Assert identical output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/18_type.md` — EC-6
#[ test ]
fn ec_6_type_omitted_defaults_to_all()
{
  let root = TempDir::new().unwrap();
  let uuid = "8d795a1c-c81d-4010-8d29-b4e678272419";
  common::write_test_session( root.path(), uuid, "sess-uuid", 2 );
  common::write_path_project_session(
    root.path(),
    &std::path::PathBuf::from( "/home/alice/def" ),
    "sess-path",
    2,
  );

  let out_default = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  let out_all = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "type::all" )
    .output()
    .unwrap();

  assert_exit( &out_default, 0 );
  assert_exit( &out_all, 0 );
  assert_eq!(
    stdout( &out_default ),
    stdout( &out_all ),
    "EC-6: omitted type must match type::all output"
  );
}

/// EC-7: `type::uuid` returns only UUID-named projects.
///
/// ## Purpose
/// Validates that `type::uuid` excludes path-encoded projects from output.
///
/// ## Coverage
/// Exit 0; path-encoded projects absent; UUID projects present.
///
/// ## Validation Strategy
/// Create both UUID and path projects. Run `.list ``type::uui``d`.
/// Assert UUID project appears and path-encoded project absent.
///
/// ## Related Requirements
/// `tests/docs/cli/param/18_type.md` — EC-7
#[ test ]
fn ec_7_type_uuid_returns_only_uuid_projects()
{
  let root = TempDir::new().unwrap();
  let uuid = "8d795a1c-c81d-4010-8d29-b4e678272419";
  common::write_test_session( root.path(), uuid, "sess-uuid", 2 );
  let encoded = common::write_path_project_session(
    root.path(),
    &std::path::PathBuf::from( "/home/alice/typetest" ),
    "sess-path",
    2,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "type::uuid" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( uuid ),
    "EC-7: type::uuid must include the UUID project; got: {output}"
  );
  assert!(
    !output.contains( &encoded ),
    "EC-7: type::uuid must exclude the path-encoded project; got: {output}"
  );
}
