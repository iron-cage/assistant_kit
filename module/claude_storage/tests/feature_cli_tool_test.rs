//! Feature tests for the CLI tool.
//!
//! ## Source
//!
//! - Spec: `tests/docs/feature/01_cli_tool.md`
//! - Feature doc: `docs/feature/001_cli_tool.md`
//!
//! ## Coverage
//!
//! - FT-1: One-shot command executes and exits cleanly
//! - FT-2: Unknown command rejected with non-zero exit
//! - FT-3: Path-encoded project returned in project list
//! - FT-4: UUID-named project returned in project list
//! - FT-5: Flat-layout (B7) project sessions accessible
//! - FT-6: Hierarchical-layout (B13) project sessions accessible

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

/// FT-1: One-shot command executes and exits cleanly.
///
/// ## Purpose
/// Verify that the CLI tool's one-shot invocation mode runs `.status` and exits 0
/// when at least one project is present in storage.
///
/// ## Coverage
/// One-shot mode; `.status` stdout non-empty; exit 0.
///
/// ## Validation Strategy
/// Write one project session to a temp storage root, run `clg .status` with
/// `CLAUDE_STORAGE_ROOT` pointing to it. Assert exit 0 and stdout non-empty.
///
/// ## Related Requirements
/// docs/feature/001_cli_tool.md — one-shot invocation mode
#[ test ]
fn ft_1_one_shot_command_executes_and_exits_cleanly()
{
  let root = TempDir::new().unwrap();
  let project = root.path().join( "proj" );
  common::write_path_project_session( root.path(), &project, "session-ft1", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  assert!(
    !stdout( &out ).is_empty(),
    "FT-1: .status must produce output; got silence; stderr: {}",
    stderr( &out )
  );
}

/// FT-2: Unknown command rejected with non-zero exit.
///
/// ## Purpose
/// Verify that the CLI rejects an unknown command with exit 1 and an error
/// message on stderr, without panicking or printing a stack trace.
///
/// ## Coverage
/// Unknown command path; exit 1; error on stderr; no panic.
///
/// ## Validation Strategy
/// Execute `clg .nonexistent_command`. Assert exit 1, stderr non-empty,
/// and no "panicked at" text in stderr.
///
/// ## Related Requirements
/// docs/feature/001_cli_tool.md — one-shot invocation mode
#[ test ]
fn ft_2_unknown_command_rejected_with_non_zero_exit()
{
  let out = common::clg_cmd()
    .env( "HOME", "/tmp" )
    .arg( ".nonexistent_command" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "FT-2: unknown command must emit error on stderr; got silence"
  );
  assert!(
    !err.contains( "panicked at" ),
    "FT-2: unknown command must not panic; stderr:\n{err}"
  );
}

/// FT-3: Path-encoded project returned in project list.
///
/// ## Purpose
/// Verify that a path-encoded project identifier appears in `.list` output.
///
/// ## Coverage
/// Path-encoded project scheme; project visible in `.list`; exit 0.
///
/// ## Validation Strategy
/// Write a path-encoded project, run `clg .list`, assert the project path
/// appears in stdout. Assert exit 0.
///
/// ## Related Requirements
/// docs/feature/001_cli_tool.md — path-encoded project scheme
#[ test ]
fn ft_3_path_encoded_project_returned_in_project_list()
{
  let root = TempDir::new().unwrap();
  let project = root.path().join( "myproject" );
  common::write_path_project_session( root.path(), &project, "session-ft3", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "myproject" ),
    "FT-3: path-encoded project must appear in .list output; got:\n{s}"
  );
}

/// FT-4: UUID-named project returned in project list.
///
/// ## Purpose
/// Verify that a UUID-named project and a path-encoded project both appear
/// in the same `.list` output.
///
/// ## Coverage
/// UUID project scheme; both project types in one listing; exit 0.
///
/// ## Validation Strategy
/// Write one path-encoded and one UUID-named project, run `clg .list`,
/// assert both identifiers appear in stdout. Assert exit 0.
///
/// ## Related Requirements
/// docs/feature/001_cli_tool.md — UUID-based project scheme
#[ test ]
fn ft_4_uuid_named_project_returned_in_project_list()
{
  let root = TempDir::new().unwrap();
  let project = root.path().join( "myproject" );
  common::write_path_project_session( root.path(), &project, "session-ft4-path", 2 );

  let uuid = "8d795a1c-c81d-4010-8d29-b4e678272419";
  common::write_test_session( root.path(), uuid, "session-ft4-uuid", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( uuid ) || s.contains( "8d795a1c" ),
    "FT-4: UUID project must appear in .list output; got:\n{s}"
  );
  assert!(
    s.contains( "myproject" ),
    "FT-4: path-encoded project must appear alongside UUID project; got:\n{s}"
  );
}

/// FT-5: Flat-layout (B7) project sessions accessible.
///
/// ## Purpose
/// Verify that sessions in the flat layout (`.jsonl` directly in the project
/// directory) are visible via `.list sessions::1`, with no layout error.
///
/// ## Coverage
/// Flat-layout (B7) storage transparency; sessions appear in listing; exit 0.
///
/// ## Validation Strategy
/// Write a flat-layout project (`write_path_project_session` produces
/// `{project_id}/{session_id}.jsonl` directly), run `clg .list sessions::1`,
/// assert stdout non-empty and no layout error on stderr. Assert exit 0.
///
/// ## Related Requirements
/// docs/feature/001_cli_tool.md — flat layout (B7) transparent handling
#[ test ]
fn ft_5_flat_layout_project_sessions_accessible()
{
  let root = TempDir::new().unwrap();
  let project = root.path().join( "flatproj" );
  common::write_path_project_session( root.path(), &project, "session-ft5", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let err = stderr( &out );
  assert!(
    !s.is_empty(),
    "FT-5: flat-layout sessions must appear in .list sessions::1; stderr: {err}"
  );
  assert!(
    !err.contains( "unrecognized layout" ),
    "FT-5: flat-layout must not produce layout error; stderr:\n{err}"
  );
}

/// FT-6: Hierarchical-layout (B13) project sessions accessible.
///
/// ## Purpose
/// Verify that sessions in the hierarchical layout (subagent directory structure,
/// B13+) appear in `.list sessions::1` output alongside flat-layout sessions.
///
/// ## Coverage
/// Hierarchical-layout (B13+) storage transparency; both layouts in one listing; exit 0.
///
/// ## Validation Strategy
/// Write one flat-layout and one hierarchical project (`write_hierarchical_path_session`),
/// run `clg .list sessions::1`, assert stdout non-empty and no layout error on
/// stderr. Assert exit 0.
///
/// ## Related Requirements
/// docs/feature/001_cli_tool.md — hierarchical layout (B13+) transparent handling
#[ test ]
fn ft_6_hierarchical_layout_project_sessions_accessible()
{
  let root = TempDir::new().unwrap();

  let flat_project = root.path().join( "flatproj" );
  common::write_path_project_session( root.path(), &flat_project, "session-ft6-flat", 2 );

  let hier_project = root.path().join( "hierproj" );
  common::write_hierarchical_path_session(
    root.path(),
    &hier_project,
    "session-ft6-hier",
    &[ ( "agent-001", "task" ) ],
    2,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let err = stderr( &out );
  assert!(
    !s.is_empty(),
    "FT-6: hierarchical-layout sessions must appear in .list sessions::1; stderr: {err}"
  );
  assert!(
    !err.contains( "unrecognized layout" ),
    "FT-6: hierarchical-layout must not produce layout error; stderr:\n{err}"
  );
}
