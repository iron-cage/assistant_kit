//! Edge case tests for the `show_stat::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/19_show_stat.md`
//!
//! ## Coverage
//!
//! - EC-1: `show_stat::0` → no statistics footer (default)
//! - EC-2: `show_stat::1` → statistics footer appended to content
//! - EC-3: Non-boolean value rejected
//! - EC-4: Omitted uses default of 0
//! - EC-5: No effect in metadata mode (`show_metadata::1`)
//! - EC-6: Combined with `show_tokens::1`

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

/// EC-1: `show_stat::0` → no statistics footer.
///
/// ## Purpose
/// Validates that `show_stat::0` produces content without statistics footer.
///
/// ## Coverage
/// Exit 0; output does not contain entry count breakdown.
///
/// ## Validation Strategy
/// Create project with session. Run `.show session_id::session-test show_stat::0`.
/// Assert exit 0 and output does not contain "Session Metadata:" footer.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_show_stat.md` — EC-1
#[ test ]
fn ec_1_show_stat_0_no_footer()
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
    .current_dir( project_cwd.path() )
    .arg( ".show" )
    .arg( "session_id::session-test" )
    .arg( "show_stat::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "Session Metadata:" ),
    "EC-1: show_stat::0 should not show statistics footer; got: {output}"
  );
}

/// EC-2: `show_stat::1` → statistics footer appended.
///
/// ## Purpose
/// Validates that `show_stat::1` appends a statistics section to content output.
///
/// ## Coverage
/// Exit 0; output contains entry count breakdown.
///
/// ## Validation Strategy
/// Create project with session. Run `.show session_id::session-test show_stat::1`.
/// Assert exit 0 and output contains "Session Metadata:" footer.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_show_stat.md` — EC-2
#[ test ]
fn ec_2_show_stat_1_footer_shown()
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
    .arg( "show_stat::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "Session Metadata:" ),
    "EC-2: show_stat::1 should show statistics footer; got: {output}"
  );
}

/// EC-3: Non-boolean value rejected.
///
/// ## Purpose
/// Validates that `show_stat::abc` is rejected (not a valid boolean).
///
/// ## Coverage
/// Exit non-zero; error message about boolean expected.
///
/// ## Validation Strategy
/// Run `.show show_stat::abc`. Assert exit non-zero and error in stderr.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_show_stat.md` — EC-3
#[ test ]
fn ec_3_show_stat_non_boolean_rejected()
{
  let out = common::clg_cmd()
    .arg( ".show" )
    .arg( "show_stat::abc" )
    .output()
    .unwrap();

  assert_ne!(
    out.status.code().unwrap_or( -1 ),
    0,
    "EC-3: show_stat::abc should be rejected; stderr: {}",
    stderr( &out )
  );
}

/// EC-4: Omitted uses default of 0.
///
/// ## Purpose
/// Validates that omitting `show_stat::` uses default (no footer).
///
/// ## Coverage
/// Exit 0; output equivalent to `show_stat::0`.
///
/// ## Validation Strategy
/// Create project with session. Run `.show session_id::session-test` without
/// `show_stat`. Assert exit 0 and no "Session Metadata:" footer.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_show_stat.md` — EC-4
#[ test ]
fn ec_4_show_stat_omitted_default_0()
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
    .current_dir( project_cwd.path() )
    .arg( ".show" )
    .arg( "session_id::session-test" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "Session Metadata:" ),
    "EC-4: omitting show_stat should not show statistics footer; got: {output}"
  );
}

/// EC-5: No effect in metadata mode (`metadata::1`).
///
/// ## Purpose
/// Validates that `show_stat::1` has no visible effect when `show_metadata::1` is also set.
///
/// ## Coverage
/// Exit 0; metadata mode output regardless of `show_stat` setting.
///
/// ## Validation Strategy
/// Create project with session. Run `.show session_id::session-test show_stat::1
/// show_metadata::1`. Assert exit 0 and no "Session Metadata:" footer
/// (metadata mode uses its own format, ignoring `show_stat`).
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_show_stat.md` — EC-5
#[ test ]
fn ec_5_show_stat_no_effect_in_metadata_mode()
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
    .current_dir( project_cwd.path() )
    .arg( ".show" )
    .arg( "session_id::session-test" )
    .arg( "show_stat::1" )
    .arg( "show_metadata::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "Session Metadata:" ),
    "EC-5: show_stat::1 in metadata mode should not show content-mode footer; got: {output}"
  );
}

/// EC-6: Combined with `show_tokens::1`.
///
/// ## Purpose
/// Validates that `show_stat::1` and `show_tokens::1` can be used together.
///
/// ## Coverage
/// Exit 0; output includes both statistics footer and token usage section.
///
/// ## Validation Strategy
/// Create project with session. Run `.show session_id::session-test
/// show_stat::1 show_tokens::1`. Assert exit 0 and output contains both
/// "Session Metadata:" and "Token Usage:" sections.
///
/// ## Related Requirements
/// `tests/docs/cli/param/19_show_stat.md` — EC-6
#[ test ]
fn ec_6_show_stat_combined_with_show_tokens()
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
    .arg( "show_stat::1" )
    .arg( "show_tokens::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "Session Metadata:" ),
    "EC-6: combined mode should include statistics footer; got: {output}"
  );
  assert!(
    output.contains( "Token Usage:" ),
    "EC-6: combined mode should include token usage section; got: {output}"
  );
}
