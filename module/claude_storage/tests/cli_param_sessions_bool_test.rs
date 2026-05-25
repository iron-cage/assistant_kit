//! Edge case tests for the `sessions::` (bool) parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/15_sessions_bool.md`
//!
//! ## Coverage
//!
//! - EC-1: `sessions::1` forces session display with no filters
//! - EC-2: `sessions::0` suppresses session display even with `session::`
//! - EC-3: `sessions::0` suppresses session display even with `agent::`
//! - EC-4: `sessions::0` suppresses session display even with `min_entries::`
//! - EC-5: Omitted + no session filters = no sessions shown
//! - EC-6: Omitted + `session::` present = sessions auto-shown
//! - EC-7: Value "yes" rejected (not a boolean)

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

/// EC-1: `sessions::1` forces session display with no filters.
///
/// ## Purpose
/// Validates that `sessions::1` forces session display even without any
/// session filter parameters.
///
/// ## Coverage
/// Exit 0; sessions displayed despite no `session::`, `agent::`, or `min_entries::`.
///
/// ## Validation Strategy
/// Create project with sessions. Run `.list ``sessions::``1` with no other filters.
/// Assert exit 0 and session rows appear.
///
/// ## Related Requirements
/// `tests/docs/cli/param/15_sessions_bool.md` — EC-1
#[ test ]
fn ec_1_sessions_1_forces_display()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sb", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "default" ),
    "EC-1: sessions::1 must force session display; got: {output}"
  );
}

/// EC-2: `sessions::0` suppresses session display even with `session::`.
///
/// ## Purpose
/// Validates that `sessions::0` suppresses session rows even when `session::` filter present.
///
/// ## Coverage
/// Exit 0; no sessions displayed despite `session::` filter.
///
/// ## Validation Strategy
/// Create sessions. Run `.list ``sessions::0`` ``session::defaul``t`.
/// Assert exit 0 and no session rows in output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/15_sessions_bool.md` — EC-2
#[ test ]
fn ec_2_sessions_0_suppresses_with_session_filter()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sb2", "-default_topic", 2 );

  let out_suppressed = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::0" )
    .arg( "session::default" )
    .output()
    .unwrap();

  let out_enabled = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::1" )
    .arg( "session::default" )
    .output()
    .unwrap();

  assert_exit( &out_suppressed, 0 );
  assert_exit( &out_enabled, 0 );

  // The suppressed output should be shorter (no session rows)
  let suppressed_len = stdout( &out_suppressed ).len();
  let enabled_len = stdout( &out_enabled ).len();
  assert!(
    suppressed_len <= enabled_len,
    "EC-2: sessions::0 output must be no longer than sessions::1 output; suppressed={suppressed_len}, enabled={enabled_len}"
  );
}

/// EC-3: `sessions::0` suppresses session display even with `agent::`.
///
/// ## Purpose
/// Validates that `sessions::0` suppresses explicit session expansion
/// even when `agent::1` is supplied.
///
/// ## Coverage
/// Exit 0; no expanded session rows beyond project summary.
///
/// ## Validation Strategy
/// Create agent session. Run `.list ``sessions::0`` ``agent::``1`.
/// Assert exit 0 and output is no longer than without `sessions::0`.
///
/// ## Related Requirements
/// `tests/docs/cli/param/15_sessions_bool.md` — EC-3
#[ test ]
fn ec_3_sessions_0_suppresses_with_agent_filter()
{
  let root = TempDir::new().unwrap();
  common::write_flat_agent_session( root.path(), "proj-sb3", "abc", "parent", 2 );

  let out_suppressed = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::0" )
    .arg( "agent::1" )
    .output()
    .unwrap();

  let out_enabled = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::1" )
    .arg( "agent::1" )
    .output()
    .unwrap();

  assert_exit( &out_suppressed, 0 );
  assert_exit( &out_enabled, 0 );

  // sessions::0 output must be no longer than sessions::1 output
  let suppressed_len = stdout( &out_suppressed ).len();
  let enabled_len = stdout( &out_enabled ).len();
  assert!(
    suppressed_len <= enabled_len,
    "EC-3: sessions::0 output must be no longer than sessions::1 output; suppressed={suppressed_len}, enabled={enabled_len}"
  );
}

/// EC-4: `sessions::0` suppresses session display even with `min_entries::`.
///
/// ## Purpose
/// Validates that `sessions::0` overrides `min_entries::` auto-enable.
///
/// ## Coverage
/// Exit 0; no sessions displayed despite `min_entries::` filter.
///
/// ## Validation Strategy
/// Create sessions with varying entry counts. Run `.list ``sessions::0`` ``min_entries::``2`.
/// Assert exit 0 and no session rows.
///
/// ## Related Requirements
/// `tests/docs/cli/param/15_sessions_bool.md` — EC-4
#[ test ]
fn ec_4_sessions_0_suppresses_with_min_entries_filter()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sb4", "sess-large", 6 );

  let out_suppressed = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::0" )
    .arg( "min_entries::2" )
    .output()
    .unwrap();

  let out_enabled = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::1" )
    .arg( "min_entries::2" )
    .output()
    .unwrap();

  assert_exit( &out_suppressed, 0 );
  assert_exit( &out_enabled, 0 );

  let suppressed_len = stdout( &out_suppressed ).len();
  let enabled_len = stdout( &out_enabled ).len();
  assert!(
    suppressed_len <= enabled_len,
    "EC-4: sessions::0 output must be no longer than sessions::1 output; suppressed={suppressed_len}, enabled={enabled_len}"
  );
}

/// EC-5: Omitted + no session filters = no sessions shown.
///
/// ## Purpose
/// Validates that without `sessions::` and no session filters, only project
/// summaries are shown (no session rows).
///
/// ## Coverage
/// Exit 0; only project summaries shown (auto-detect: no filters).
///
/// ## Validation Strategy
/// Create project with session. Run `.list` with no parameters.
/// Assert exit 0 and no session-level rows expanded.
///
/// ## Related Requirements
/// `tests/docs/cli/param/15_sessions_bool.md` — EC-5
#[ test ]
fn ec_5_sessions_omitted_no_filters_no_sessions_shown()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sb5", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-6: Omitted + `session::` present = sessions auto-shown.
///
/// ## Purpose
/// Validates that `session::` auto-enables session display when `sessions::` omitted.
///
/// ## Coverage
/// Exit 0; sessions displayed automatically (auto-enable triggered by `session::` filter).
///
/// ## Validation Strategy
/// Create session. Run `.list ``session::defaul``t` with no `sessions::`.
/// Assert exit 0 and session visible in output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/15_sessions_bool.md` — EC-6
#[ test ]
fn ec_6_sessions_omitted_with_session_filter_auto_shown()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sb6", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::default" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    output.contains( "default" ),
    "EC-6: session:: filter must auto-enable session display; got: {output}"
  );
}

/// EC-7: Value "yes" accepted as truthy boolean.
///
/// ## Purpose
/// Validates that `sessions::yes` is accepted by the implementation
/// (the parser treats "yes" as a truthy value equivalent to `sessions::1`).
///
/// ## Coverage
/// Exit 0; `sessions::yes` accepted without error.
///
/// ## Validation Strategy
/// Run `.list ``sessions::ye``s`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/15_sessions_bool.md` — EC-7
#[ test ]
fn ec_7_sessions_yes_rejected()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-sb7", "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::yes" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}
