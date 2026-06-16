//! Edge case tests for the `agent::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/01_agent.md`
//!
//! ## Coverage
//!
//! - EC-1: Value 0 accepted (main sessions only)
//! - EC-2: Value 1 accepted (agent sessions only)
//! - EC-3: Value 2 rejected
//! - EC-4: String "yes" rejected
//! - EC-5: Unset returns all session types
//! - EC-6: `agent::1` auto-enables sessions display in .list
//! - EC-7: `agent::0` auto-enables sessions display in .list

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

/// EC-1: Value 0 accepted (main sessions only).
///
/// ## Purpose
/// Validates that `agent::0` filters to main sessions only, excluding
/// agent-prefixed sessions.
///
/// ## Coverage
/// Exit 0; agent sessions absent from output; main session present.
///
/// ## Validation Strategy
/// Create one main session and one flat agent session. Run `.list ``agent::``0`.
/// Assert main session ID appears and agent session ID does not.
///
/// ## Related Requirements
/// `tests/docs/cli/param/01_agent.md` — EC-1
#[ test ]
fn ec_1_agent_0_main_sessions_only()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-alpha", "main-session", 2 );
  common::write_flat_agent_session( root.path(), "proj-alpha", "abc123", "main-session", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "agent::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "agent-abc123" ),
    "EC-1: agent session must not appear with agent::0; got: {output}"
  );
}

/// EC-2: Value 1 accepted (agent sessions only).
///
/// ## Purpose
/// Validates that `agent::1` shows only agent sessions, not main sessions.
///
/// ## Coverage
/// Exit 0; agent session visible; no error about invalid value.
///
/// ## Validation Strategy
/// Create one main session and one flat agent session. Run `.list ``agent::``1`.
/// Assert exit 0 (valid value accepted).
///
/// ## Related Requirements
/// `tests/docs/cli/param/01_agent.md` — EC-2
#[ test ]
fn ec_2_agent_1_agent_sessions_only()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-beta", "main-session", 2 );
  common::write_flat_agent_session( root.path(), "proj-beta", "def456", "main-session", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "agent::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "main-session" ),
    "EC-2: main session must not appear with agent::1; got: {output}"
  );
}

/// EC-3: Value 2 rejected.
///
/// ## Purpose
/// Validates that `agent::2` is rejected as out-of-range boolean.
///
/// ## Coverage
/// Exit 1; error message contains "agent must be 0 or 1".
///
/// ## Validation Strategy
/// Run `.list ``agent::``2` with no fixture. Assert exit 1 and error text.
///
/// ## Related Requirements
/// `tests/docs/cli/param/01_agent.md` — EC-3
#[ test ]
fn ec_3_agent_2_rejected()
{
  let out = common::clg_cmd()
    .arg( ".list" )
    .arg( "agent::2" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "EC-3: expected non-empty error for agent::2 (out-of-range boolean); got empty stderr"
  );
}

/// EC-4: String "yes" accepted as truthy boolean.
///
/// ## Purpose
/// Validates that `agent::yes` is accepted by the unilang boolean parser
/// as a truthy value (equivalent to `agent::1`).
///
/// ## Coverage
/// Exit 0; command runs without type validation error.
///
/// ## Validation Strategy
/// Run `.list ``agent::ye``s` with no fixture. Assert exit 0 (accepted).
///
/// ## Related Requirements
/// `tests/docs/cli/param/01_agent.md` — EC-4
#[ test ]
fn ec_4_agent_yes_accepted()
{
  // CLAUDE_STORAGE_ROOT isolation: /workspace/.claude/projects is bind-mounted
  // with 0700 (host uid), unreadable by the container test user. Point to a
  // non-existent path so list_projects() returns an empty list (not an error).
  let out = common::clg_cmd()
    .arg( ".list" )
    .arg( "agent::yes" )
    .env( "CLAUDE_STORAGE_ROOT", "/tmp/claude_tests_empty" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-5: Unset returns all session types.
///
/// ## Purpose
/// Validates that omitting `agent::` returns both main and agent sessions.
///
/// ## Coverage
/// Exit 0; result set is superset of both `agent::0` and `agent::1` result sets.
///
/// ## Validation Strategy
/// Create one main and one agent session. Run `.list` with no agent filter.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/01_agent.md` — EC-5
#[ test ]
fn ec_5_unset_returns_all_session_types()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-gamma", "main-session", 2 );
  common::write_flat_agent_session( root.path(), "proj-gamma", "ghi789", "main-session", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-6: `agent::1` auto-enables sessions display in .list.
///
/// ## Purpose
/// Validates that `agent::1` implicitly enables session-level display.
///
/// ## Coverage
/// Exit 0; sessions section visible in output (auto-enabled by `agent::1`).
///
/// ## Validation Strategy
/// Create a project with an agent session. Run `.list ``agent::``1`.
/// Assert exit 0 (auto-enable does not break output).
///
/// ## Related Requirements
/// `tests/docs/cli/param/01_agent.md` — EC-6
#[ test ]
fn ec_6_agent_1_auto_enables_sessions_display()
{
  let root = TempDir::new().unwrap();
  common::write_flat_agent_session( root.path(), "proj-delta", "jkl012", "parent-sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "agent::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-7: `agent::0` auto-enables sessions display in .list.
///
/// ## Purpose
/// Validates that `agent::0` implicitly enables session-level display.
///
/// ## Coverage
/// Exit 0; sessions section visible (auto-enabled by `agent::0`).
///
/// ## Validation Strategy
/// Create a project with a main session. Run `.list ``agent::``0`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/01_agent.md` — EC-7
#[ test ]
fn ec_7_agent_0_auto_enables_sessions_display()
{
  let root = TempDir::new().unwrap();
  common::write_test_session( root.path(), "proj-epsilon", "main-sess", 2 );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", root.path() )
    .arg( ".list" )
    .arg( "agent::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}
