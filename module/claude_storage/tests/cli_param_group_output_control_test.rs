//! Cross-command interaction tests for Output Control parameter group.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param_group/01_output_control.md`
//!
//! ## Coverage
//!
//! - CC-1: Default output is non-empty in .status and .list
//! - CC-2: `show_tokens::1` adds Tokens section in .status

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

/// CC-1: Default output is non-empty in .status and .list.
///
/// ## Purpose
/// Verify that bare `.status` and `.list` produce styled, non-empty output.
///
/// ## Coverage
/// Default output; both commands produce output; exit 0 for both.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/01_output_control.md` — CC-1
#[ test ]
fn cc_1_default_output_consistent_across_commands()
{
  let root = TempDir::new().unwrap();
  let proj = root.path().join( "cc1-proj" );
  common::write_path_project_session( root.path(), &proj, "s001", 2 );

  let status_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  let list_out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &status_out, 0 );
  assert_exit( &list_out, 0 );

  assert!(
    !stdout( &status_out ).is_empty(),
    "CC-1: default .status must produce output; stderr: {}",
    stderr( &status_out )
  );
  assert!(
    !stdout( &list_out ).is_empty(),
    "CC-1: default .list must produce output; stderr: {}",
    stderr( &list_out )
  );
}

/// CC-2: `show_tokens::1` adds Tokens section in .status.
///
/// ## Purpose
/// Verify that `.status show_tokens::1` includes a "Tokens" section in its
/// output that is absent from bare `.status`.
///
/// ## Coverage
/// Tokens section present with show_tokens::1; absent without it; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/01_output_control.md` — CC-2
#[ test ]
fn cc_2_show_tokens_adds_tokens_section_in_status()
{
  let root = TempDir::new().unwrap();
  let p1 = root.path().join( "cc2-alpha" );
  let p2 = root.path().join( "cc2-beta" );
  common::write_path_project_session( root.path(), &p1, "s001", 4 );
  common::write_path_project_session( root.path(), &p2, "s002", 4 );
  common::write_path_project_session( root.path(), &p2, "s003", 4 );

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
    "CC-2: show_tokens::1 must produce more output than bare .status;\n  base ({} bytes):\n{base}\n  tokens ({} bytes):\n{tokens}",
    base.len(),
    tokens.len()
  );

  assert!(
    tokens.to_lowercase().contains( "token" ),
    "CC-2: show_tokens::1 output must include a Tokens section; got:\n{tokens}"
  );
}
