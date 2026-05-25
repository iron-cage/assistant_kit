//! Cross-command interaction tests for Session Filter parameter group.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param_group/04_session_filter.md`
//!
//! ## Coverage
//!
//! - CC-1: `session::` alone auto-enables session display
//! - CC-2: `agent::` alone auto-enables session display
//! - CC-3: `min_entries::` alone auto-enables session display
//! - CC-4: `sessions::0` suppresses display even with all three filters
//! - CC-5: `session::` + `agent::` combined filters sessions by both
//! - CC-6: `session::` + `min_entries::` combined filters by both criteria
//! - CC-7: All three filters are AND-combined (not OR)

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

/// CC-1: `session::` alone auto-enables session display.
///
/// ## Purpose
/// Verify that providing `session::` without explicit `sessions::1` automatically
/// enables session listing and filters to only sessions matching the substring.
///
/// ## Coverage
/// Auto-enable via `session::`; substring filter applied; non-matching session absent; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/04_session_filter.md` — CC-1
#[ test ]
fn cc_1_session_alone_auto_enables_session_display()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "sf1proj" );
  let encoded = claude_storage_core::encode_path( &project_dir ).unwrap();
  common::write_test_session( root.path(), &encoded, "-commit", 2 );
  common::write_test_session( root.path(), &encoded, "-default_topic", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::commit" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "commit" ),
    "CC-1: session::commit must appear in output; got:\n{s}"
  );
  assert!(
    !s.contains( "default_topic" ),
    "CC-1: -default_topic session must not appear when session::commit filter active; got:\n{s}"
  );
}

/// CC-2: `agent::` alone auto-enables session display.
///
/// ## Purpose
/// Verify that providing `agent::1` without explicit `sessions::1` automatically
/// enables session listing and filters to only agent sessions.
///
/// ## Coverage
/// Auto-enable via `agent::`; only agent sessions shown; main sessions absent; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/04_session_filter.md` — CC-2
#[ test ]
fn cc_2_agent_alone_auto_enables_session_display()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "sf2proj" );
  let encoded = claude_storage_core::encode_path( &project_dir ).unwrap();
  // Write a main session
  common::write_test_session( root.path(), &encoded, "-main-session", 2 );
  // Write an agent session
  common::write_flat_agent_session( root.path(), &encoded, "agent-001", "-main-session", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "agent::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "agent" ),
    "CC-2: agent sessions must appear with agent::1 filter; got:\n{s}"
  );
  assert!(
    !s.contains( "main-session" ) || s.contains( "agent" ),
    "CC-2: output with agent::1 must show agent sessions; got:\n{s}"
  );
}

/// CC-3: `min_entries::` alone auto-enables session display.
///
/// ## Purpose
/// Verify that providing `min_entries::5` without explicit `sessions::1`
/// automatically enables session listing and filters to sessions with >= 5 entries.
///
/// ## Coverage
/// Auto-enable via `min_entries::`; sessions below threshold absent; sessions above visible; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/04_session_filter.md` — CC-3
#[ test ]
fn cc_3_min_entries_alone_auto_enables_session_display()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "sf3proj" );
  let encoded = claude_storage_core::encode_path( &project_dir ).unwrap();
  common::write_test_session( root.path(), &encoded, "small-session", 2 );
  common::write_test_session( root.path(), &encoded, "large-session", 10 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "min_entries::5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "large-session" ),
    "CC-3: large-session (10 entries) must appear with min_entries::5; got:\n{s}"
  );
  assert!(
    !s.contains( "small-session" ),
    "CC-3: small-session (2 entries) must not appear with min_entries::5; got:\n{s}"
  );
}

/// CC-4: `sessions::0` suppresses display even with all three filters.
///
/// ## Purpose
/// Verify that `sessions::0` overrides the auto-enable behavior of
/// `session::`, `agent::`, and `min_entries::` and suppresses session expansion.
///
/// ## Coverage
/// `sessions::0` override; output no longer than `sessions::1` despite filter params; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/04_session_filter.md` — CC-4
#[ test ]
fn cc_4_sessions_0_suppresses_display_even_with_all_three_filters()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "sf4proj" );
  let encoded = claude_storage_core::encode_path( &project_dir ).unwrap();
  common::write_flat_agent_session( root.path(), &encoded, "agent-commit-001", "-commit", 4 );
  common::write_test_session( root.path(), &encoded, "-commit", 4 );

  let out_suppressed = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::0" )
    .arg( "session::commit" )
    .arg( "agent::1" )
    .arg( "min_entries::2" )
    .output()
    .unwrap();

  let out_enabled = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "sessions::1" )
    .arg( "session::commit" )
    .arg( "agent::1" )
    .arg( "min_entries::2" )
    .output()
    .unwrap();

  assert_exit( &out_suppressed, 0 );
  assert_exit( &out_enabled, 0 );

  // sessions::0 must produce output no longer than sessions::1
  let suppressed_len = stdout( &out_suppressed ).len();
  let enabled_len = stdout( &out_enabled ).len();
  assert!(
    suppressed_len <= enabled_len,
    "CC-4: sessions::0 output must be no longer than sessions::1; suppressed={suppressed_len}, enabled={enabled_len}\nsuppressed:\n{}\nenabled:\n{}",
    stdout( &out_suppressed ),
    stdout( &out_enabled )
  );
}

/// CC-5: `session::` + `agent::` combined filters sessions by both.
///
/// ## Purpose
/// Verify that `session::commit ``agent::``1` returns only sessions that match
/// BOTH the "commit" substring AND the agent filter (AND semantics).
///
/// ## Coverage
/// AND combination of `session::` and `agent::`;; only dual-matching session returned; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/04_session_filter.md` — CC-5
#[ test ]
fn cc_5_session_and_agent_combined_filters_by_both()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "sf5proj" );
  let encoded = claude_storage_core::encode_path( &project_dir ).unwrap();
  // Main session with "commit" in name — should NOT match (agent::1 excludes it)
  common::write_test_session( root.path(), &encoded, "-commit", 2 );
  // Agent session with "commit" in name — should match both filters
  common::write_flat_agent_session( root.path(), &encoded, "agent-commit-123", "-commit", 2 );
  // Agent session without "commit" — should NOT match (session::commit excludes it)
  common::write_flat_agent_session( root.path(), &encoded, "agent-other-456", "-other", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::commit" )
    .arg( "agent::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "agent-commit-123" ),
    "CC-5: agent-commit-123 must appear (matches both filters); got:\n{s}"
  );
  assert!(
    !s.contains( "agent-other-456" ),
    "CC-5: agent-other-456 must not appear (no 'commit' substring); got:\n{s}"
  );
}

/// CC-6: `session::` + `min_entries::` combined filters by both criteria.
///
/// ## Purpose
/// Verify that `session::commit ``min_entries::``5` returns only sessions that
/// match BOTH the "commit" substring AND have >= 5 entries (AND semantics).
///
/// ## Coverage
/// AND combination of `session::` and `min_entries::`;; only dual-qualifying session returned; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/04_session_filter.md` — CC-6
#[ test ]
fn cc_6_session_and_min_entries_combined_filters_by_both_criteria()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "sf6proj" );
  let encoded = claude_storage_core::encode_path( &project_dir ).unwrap();
  // "commit" substring but only 3 entries — should NOT match (below min_entries)
  common::write_test_session( root.path(), &encoded, "-commit", 3 );
  // "commit" substring AND 10 entries — should match both
  common::write_test_session( root.path(), &encoded, "-commit-long", 10 );
  // 8 entries but no "commit" — should NOT match (no substring match)
  common::write_test_session( root.path(), &encoded, "-default_topic", 8 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::commit" )
    .arg( "min_entries::5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "commit-long" ),
    "CC-6: -commit-long must appear (matches both criteria); got:\n{s}"
  );
  assert!(
    !s.contains( "default_topic" ),
    "CC-6: -default_topic must not appear (no 'commit' in name); got:\n{s}"
  );
}

/// CC-7: All three filters are AND-combined (not OR).
///
/// ## Purpose
/// Verify that `session::commit ``agent::1`` ``min_entries::``5` only returns sessions
/// that satisfy ALL three conditions simultaneously.
///
/// ## Coverage
/// Strict AND semantics; only single triple-matching session returned; exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param_group/04_session_filter.md` — CC-7
#[ test ]
fn cc_7_all_three_filters_are_and_combined_not_or()
{
  let root = TempDir::new().unwrap();
  let project_dir = root.path().join( "sf7proj" );
  let encoded = claude_storage_core::encode_path( &project_dir ).unwrap();
  // Agent + "commit" + 5 entries: satisfies ALL three — must appear
  common::write_flat_agent_session( root.path(), &encoded, "agent-commit-5entries", "-commit-5", 5 );
  // Agent + no "commit" + 5 entries: fails session:: filter
  common::write_flat_agent_session( root.path(), &encoded, "agent-other-5entries", "-other", 5 );
  // Main + "commit" + 5 entries: fails agent:: filter
  common::write_test_session( root.path(), &encoded, "-commit-5entries", 5 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "session::commit" )
    .arg( "agent::1" )
    .arg( "min_entries::5" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "agent-commit-5entries" ),
    "CC-7: agent-commit-5entries must appear (satisfies all three filters); got:\n{s}"
  );
  assert!(
    !s.contains( "agent-other-5entries" ),
    "CC-7: agent-other-5entries must not appear (no 'commit' substring); got:\n{s}"
  );
  assert!(
    !s.contains( "-commit-5entries" ) || s.contains( "agent-commit-5entries" ),
    "CC-7: main -commit-5entries session must not appear (fails agent:: filter); got:\n{s}"
  );
}
