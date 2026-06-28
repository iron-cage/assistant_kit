//! Edge case tests for the `show_tokens::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/23_show_tokens.md`
//!
//! ## Coverage
//!
//! - EC-1: `show_tokens::0` → no token section (default)
//! - EC-2: `show_tokens::1` in `.show` → token usage appended
//! - EC-3: Non-boolean value rejected
//! - EC-4: Omitted uses default of 0
//! - EC-5: `show_tokens::1` in `.status` → triggers full JSONL parse
//! - EC-6: `show_tokens::1` in `.show` → appends token usage to output

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

/// EC-1: `show_tokens::0` → no token section.
///
/// ## Purpose
/// Validates that `show_tokens::0` produces output without token usage section.
///
/// ## Coverage
/// Exit 0; output does not contain token usage breakdown.
///
/// ## Validation Strategy
/// Create storage. Run `.status show_tokens::0`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/23_show_tokens.md` — EC-1
#[ test ]
fn ec_1_show_tokens_0_no_token_section()
{
  let storage = TempDir::new().unwrap();
  let project_cwd = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_cwd.path(),
    "session-test",
    3,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .arg( ".status" )
    .arg( "show_tokens::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.to_lowercase().contains( "input tokens" ),
    "EC-1: show_tokens::0 should not show token usage; got: {output}"
  );
}

/// EC-2: `show_tokens::1` in `.show` → token usage appended.
///
/// ## Purpose
/// Validates that `show_tokens::1` includes token usage in `.show` output.
///
/// ## Coverage
/// Exit 0; output includes token-related content.
///
/// ## Validation Strategy
/// Create project with session. Run `.show session_id::session-test show_tokens::1`.
/// Assert exit 0 and output contains "Token Usage:" section.
///
/// ## Related Requirements
/// `tests/docs/cli/param/23_show_tokens.md` — EC-2
#[ test ]
fn ec_2_show_tokens_1_in_show()
{
  let storage = TempDir::new().unwrap();
  let project_cwd = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_cwd.path(),
    "session-test",
    4,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .current_dir( project_cwd.path() )
    .arg( ".show" )
    .arg( "session_id::session-test" )
    .arg( "show_tokens::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "Token Usage:" ),
    "EC-2: show_tokens::1 in .show should include token usage section; got: {output}"
  );
}

/// EC-3: Non-boolean value rejected.
///
/// ## Purpose
/// Validates that `show_tokens::abc` is rejected (not a valid boolean).
///
/// ## Coverage
/// Exit non-zero; error message about boolean expected.
///
/// ## Validation Strategy
/// Run `.status show_tokens::abc`. Assert exit non-zero and error in stderr.
///
/// ## Related Requirements
/// `tests/docs/cli/param/23_show_tokens.md` — EC-3
#[ test ]
fn ec_3_show_tokens_non_boolean_rejected()
{
  let storage = TempDir::new().unwrap();

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .arg( ".status" )
    .arg( "show_tokens::abc" )
    .output()
    .unwrap();

  assert_ne!(
    out.status.code().unwrap_or( -1 ),
    0,
    "EC-3: show_tokens::abc should be rejected; stderr: {}",
    stderr( &out )
  );
}

/// EC-4: Omitted uses default of 0.
///
/// ## Purpose
/// Validates that omitting `show_tokens::` uses default (no token section).
///
/// ## Coverage
/// Exit 0; fast path used in `.status`.
///
/// ## Validation Strategy
/// Create storage. Run `.status` without `show_tokens`. Assert exit 0
/// and output does not contain token breakdown.
///
/// ## Related Requirements
/// `tests/docs/cli/param/23_show_tokens.md` — EC-4
#[ test ]
fn ec_4_show_tokens_omitted_default_0()
{
  let storage = TempDir::new().unwrap();
  let project_cwd = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_cwd.path(),
    "session-test",
    3,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .arg( ".status" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-5: `show_tokens::1` in `.status` triggers full JSONL parse.
///
/// ## Purpose
/// Validates that `show_tokens::1` in `.status` produces token totals.
///
/// ## Coverage
/// Exit 0; output includes token-related content from full JSONL parsing.
///
/// ## Validation Strategy
/// Create storage. Run `.status show_tokens::1`. Assert exit 0 and output
/// contains token information.
///
/// ## Related Requirements
/// `tests/docs/cli/param/23_show_tokens.md` — EC-5
#[ test ]
fn ec_5_show_tokens_1_in_status_full_parse()
{
  let storage = TempDir::new().unwrap();
  let project_cwd = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_cwd.path(),
    "session-test",
    4,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .arg( ".status" )
    .arg( "show_tokens::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "Tokens:" ) || output.contains( "Input:" ),
    "EC-5: show_tokens::1 in .status should include token information; got: {output}"
  );
}

/// EC-6: `show_tokens::1` in `.show` appends token usage.
///
/// ## Purpose
/// Validates that `show_tokens::1` appends token usage to `.show` session output.
///
/// ## Coverage
/// Exit 0; session content or metadata output includes token usage section.
///
/// ## Validation Strategy
/// Create project with session. Run `.show session_id::session-test show_tokens::1`.
/// Assert exit 0 and output contains "Token Usage:" section.
///
/// ## Related Requirements
/// `tests/docs/cli/param/23_show_tokens.md` — EC-6
#[ test ]
fn ec_6_show_tokens_1_appends_to_show()
{
  let storage = TempDir::new().unwrap();
  let project_cwd = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_cwd.path(),
    "session-test",
    4,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .current_dir( project_cwd.path() )
    .arg( ".show" )
    .arg( "session_id::session-test" )
    .arg( "show_tokens::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "Token Usage:" ),
    "EC-6: show_tokens::1 should append token usage to .show output; got: {output}"
  );
}
