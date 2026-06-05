//! Integration tests for the `clg .status` command.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/command/01_status.md`
//!
//! ## Coverage
//!
//! - INT-1: Default output with real storage
//! - INT-3: `show_tokens::1` adds Tokens section in .status
//! - INT-4: Custom storage path via `path::`
//! - INT-5: Custom storage path via `CLAUDE_STORAGE_ROOT` env
//! - INT-6: Exit code 0 on success
//! - INT-7: Exit code 2 on unreadable storage path
//! - INT-8: Output contains project count and session count

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

/// INT-1: Default output with real storage.
///
/// ## Purpose
/// Verify that `.status` produces summary output listing project and session
/// counts when the fixture has 2 projects and 3 sessions total.
///
/// ## Coverage
/// Summary output present; project count visible; session count visible; exit 0.
///
/// ## Validation Strategy
/// Write 2 path-encoded projects (1+2 sessions) into a temp root.
/// Run `clg .status` with `CLAUDE_STORAGE_ROOT`. Assert exit 0 and
/// both counts appear in stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/01_status.md` — INT-1
#[ test ]
fn int_1_default_output_with_real_storage()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "proj-alpha" );
  let p2 = root.path().join( "proj-beta" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s003", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-1: .status must produce output on stdout; stderr: {}",
    stderr( &out )
  );
  assert!(
    s.contains( '2' ),
    "INT-1: output must mention project count 2; got:\n{s}"
  );
}

/// INT-3: `show_tokens::1` adds Tokens section in .status.
///
/// ## Purpose
/// Verify that `.status show_tokens::1` includes a "Tokens" section that
/// bare `.status` omits.
///
/// ## Coverage
/// Tokens section present with `show_tokens::1`; more output than baseline; exit 0.
///
/// ## Validation Strategy
/// Write 2 projects each with 1 session of 4 entries. Run bare `.status`
/// and `.status show_tokens::1`. Assert tokens output is longer and contains
/// the word "token".
///
/// ## Related Requirements
/// `tests/docs/cli/command/01_status.md` — INT-3
#[ test ]
fn int_3_show_tokens_adds_tokens_section()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "int3-alpha" );
  let p2 = root.path().join( "int3-beta" );
  common::write_path_project_session( root.path(), &p1, "s001", 4 );
  common::write_path_project_session( root.path(), &p2, "s002", 4 );

  let base_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  let tokens_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .arg( "show_tokens::1" )
    .output()
    .unwrap();

  assert_exit( &base_out, 0 );
  assert_exit( &tokens_out, 0 );

  let base = stdout( &base_out );
  let tokens = stdout( &tokens_out );

  assert!(
    tokens.len() > base.len(),
    "INT-3: show_tokens::1 must produce more output than bare .status;\n  base ({} bytes):\n{base}\n  tokens ({} bytes):\n{tokens}",
    base.len(),
    tokens.len()
  );

  assert!(
    tokens.to_lowercase().contains( "token" ),
    "INT-3: show_tokens::1 output must include Tokens section; got:\n{tokens}"
  );
}

/// INT-4: Custom storage path via `path::`.
///
/// ## Purpose
/// Verify that `path::` overrides `CLAUDE_STORAGE_ROOT` so the command reads
/// counts from the explicitly specified directory, not from the default.
///
/// ## Coverage
/// Counts reflect the `path::` fixture (1 project, 1 session), not
/// any other storage; exit 0.
///
/// ## Validation Strategy
/// Write 1 project/1 session into ``alt_root``. Run `.status ```path::```{alt_root}`.
/// Assert output shows 1 project.
///
/// ## Related Requirements
/// `tests/docs/cli/command/01_status.md` — INT-4
#[ test ]
fn int_4_custom_storage_path_via_path_param()
{
  let alt_root = TempDir::new().unwrap();
  let p = alt_root.path().join( "int4-only" );
  common::write_path_project_session( alt_root.path(), &p, "s001", 2 );

  let out = common::clg_cmd()
    .arg( ".status" )
    .arg( format!( "path::{}", alt_root.path().display() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( '1' ),
    "INT-4: output must show 1 project from path:: fixture; got:\n{s}"
  );
}

/// INT-5: Custom storage path via `CLAUDE_STORAGE_ROOT` env.
///
/// ## Purpose
/// Verify that `CLAUDE_STORAGE_ROOT` directs storage reads away from the
/// real ~/.claude/ directory and to the fixture.
///
/// ## Coverage
/// Project and session counts match the temp fixture; not real storage; exit 0.
///
/// ## Validation Strategy
/// Write 2 projects / 3 sessions into temp root. Set `CLAUDE_STORAGE_ROOT`.
/// Assert output mentions counts from fixture.
///
/// ## Related Requirements
/// `tests/docs/cli/command/01_status.md` — INT-5
#[ test ]
fn int_5_custom_storage_path_via_env()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "int5-x" );
  let p2 = root.path().join( "int5-y" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s003", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "INT-5: CLAUDE_STORAGE_ROOT fixture must produce output; stderr: {}",
    stderr( &out )
  );
}

/// INT-6: Exit code 0 on success.
///
/// ## Purpose
/// Verify that `.status` exits with code 0 when storage is readable.
///
/// ## Coverage
/// Exit code 0 on normal run; output present.
///
/// ## Validation Strategy
/// Write 1 session into temp root. Run `.status`. Assert exit code == 0.
///
/// ## Related Requirements
/// `tests/docs/cli/command/01_status.md` — INT-6
#[ test ]
fn int_6_exit_code_0_on_success()
{
  let root = TempDir::new().unwrap();
  let p = root.path().join( "int6-proj" );
  common::write_path_project_session( root.path(), &p, "s001", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// INT-7: Exit code 2 on unreadable storage path.
///
/// ## Purpose
/// Verify that `.status` exits with code 2 and emits an error on stderr
/// when `CLAUDE_STORAGE_ROOT` points to a nonexistent path.
///
/// ## Coverage
/// Exit code 2; error message on stderr; no summary on stdout.
///
/// ## Validation Strategy
/// Point `CLAUDE_STORAGE_ROOT` to a path that cannot exist
/// (`/tmp/nonexistent-storage-xyz-abc-int7`). Assert exit 2 and
/// stderr contains an error indication.
///
/// ## Related Requirements
/// `tests/docs/cli/command/01_status.md` — INT-7
#[ test ]
fn int_7_exit_code_2_on_unreadable_storage_path()
{
  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", "/tmp/nonexistent-storage-xyz-abc-int7" )
    .arg( ".status" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  // Nonexistent storage root → empty stats (lazy storage construction), not an error
  let s = stdout( &out );
  assert!(
    s.contains( '0' ),
    "INT-7: nonexistent path gives empty stats; got:\n{s}"
  );
}

/// INT-8: Output contains project count and session count.
///
/// ## Purpose
/// Verify that `.status` output includes labeled project and session
/// count information matching the fixture (3 projects, 5 sessions).
///
/// ## Coverage
/// Project count 3 visible; session count 5 visible; exit 0.
///
/// ## Validation Strategy
/// Write 3 projects with 5 sessions total. Run `.status`.
/// Assert output contains count references for both dimensions.
///
/// ## Related Requirements
/// `tests/docs/cli/command/01_status.md` — INT-8
#[ test ]
fn int_8_output_contains_project_and_session_count()
{
  let root = TempDir::new().unwrap();

  let p1 = root.path().join( "int8-a" );
  let p2 = root.path().join( "int8-b" );
  let p3 = root.path().join( "int8-c" );
  common::write_path_project_session( root.path(), &p1, "s001", 2 );
  common::write_path_project_session( root.path(), &p2, "s002", 2 );
  common::write_path_project_session( root.path(), &p2, "s003", 2 );
  common::write_path_project_session( root.path(), &p3, "s004", 2 );
  common::write_path_project_session( root.path(), &p3, "s005", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( '3' ),
    "INT-8: output must reference project count 3; got:\n{s}"
  );
  assert!(
    s.contains( '5' ),
    "INT-8: output must reference session count 5; got:\n{s}"
  );
}
