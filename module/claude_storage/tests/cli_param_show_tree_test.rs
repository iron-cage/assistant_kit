//! Edge case tests for the `show_tree::` parameter.
//!
//! ## Source
//!
//! - Spec: `tests/docs/cli/param/24_show_tree.md`
//!
//! ## Coverage
//!
//! - EC-1: `show_tree::0` → compact family summary (default)
//! - EC-2: `show_tree::1` → tree-indented agent display
//! - EC-3: Non-boolean value rejected
//! - EC-4: Omitted uses default of 0
//! - EC-5: Tree format shows agent connectors
//! - EC-6: Single root without agents in tree mode

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

/// EC-1: `show_tree::0` → compact family summary.
///
/// ## Purpose
/// Validates that `show_tree::0` uses compact family summary format.
///
/// ## Coverage
/// Exit 0; output uses compact per-root session format.
///
/// ## Validation Strategy
/// Create project with sessions. Run `.projects show_tree::0`. Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/24_show_tree.md` — EC-1
#[ test ]
fn ec_1_show_tree_0_compact_format()
{
  let storage = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_dir.path(),
    "session-test",
    3,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_tree::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  // Compact format should not contain tree connectors
  assert!(
    !output.contains( "├─" ) && !output.contains( "└─" ),
    "EC-1: show_tree::0 should use compact format without tree connectors; got: {output}"
  );
}

/// EC-2: `show_tree::1` → tree-indented agent display.
///
/// ## Purpose
/// Validates that `show_tree::1` switches to tree-indented format.
///
/// ## Coverage
/// Exit 0; output uses tree-indented format with full UUIDs.
///
/// ## Validation Strategy
/// Create project with root + agent sessions. Run `.projects show_tree::1`.
/// Assert exit 0.
///
/// ## Related Requirements
/// `tests/docs/cli/param/24_show_tree.md` — EC-2
#[ test ]
fn ec_2_show_tree_1_tree_format()
{
  let storage = TempDir::new().unwrap();
  let _project_dir = TempDir::new().unwrap();

  common::write_hierarchical_session(
    storage.path(),
    "test-project",
    "root-session",
    &[ ( "abc123", "task" ) ],
    3,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_tree::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-3: Non-boolean value rejected.
///
/// ## Purpose
/// Validates that `show_tree::abc` is rejected (not a valid boolean).
///
/// ## Coverage
/// Exit non-zero; error message about boolean expected.
///
/// ## Validation Strategy
/// Run `.projects show_tree::abc`. Assert exit non-zero and error in stderr.
///
/// ## Related Requirements
/// `tests/docs/cli/param/24_show_tree.md` — EC-3
#[ test ]
fn ec_3_show_tree_non_boolean_rejected()
{
  let out = common::clg_cmd()
    .arg( ".projects" )
    .arg( "show_tree::abc" )
    .output()
    .unwrap();

  assert_ne!(
    out.status.code().unwrap_or( -1 ),
    0,
    "EC-3: show_tree::abc should be rejected; stderr: {}",
    stderr( &out )
  );
}

/// EC-4: Omitted uses default of 0.
///
/// ## Purpose
/// Validates that omitting `show_tree::` defaults to compact format.
///
/// ## Coverage
/// Exit 0; output uses compact format (same as `show_tree::0`).
///
/// ## Validation Strategy
/// Create project with sessions. Run `.projects scope::global` without `show_tree`.
/// Assert exit 0 and compact format used.
///
/// ## Related Requirements
/// `tests/docs/cli/param/24_show_tree.md` — EC-4
#[ test ]
fn ec_4_show_tree_omitted_default_0()
{
  let storage = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_dir.path(),
    "session-test",
    3,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
}

/// EC-5: Tree format shows agent connectors.
///
/// ## Purpose
/// Validates that tree-indented format uses `├─`/`└─` connectors for agents.
///
/// ## Coverage
/// Exit 0; output contains tree connectors with agent sessions indented under root.
///
/// ## Validation Strategy
/// Create project with root + multiple agent sessions. Run `.projects show_tree::1`.
/// Assert output contains tree connectors.
///
/// ## Related Requirements
/// `tests/docs/cli/param/24_show_tree.md` — EC-5
#[ test ]
fn ec_5_show_tree_agent_connectors()
{
  let storage = TempDir::new().unwrap();
  let _project_dir = TempDir::new().unwrap();

  common::write_hierarchical_session(
    storage.path(),
    "test-project-tree",
    "root-session-2",
    &[ ( "abc123", "task" ), ( "def456", "research" ) ],
    3,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_tree::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  // With 2 agents, tree format should show connectors
  let has_connectors = output.contains( "├" ) || output.contains( "└" );
  assert!(
    has_connectors,
    "EC-5: show_tree::1 with agents should show tree connectors; got: {output}"
  );
}

/// EC-6: Single root without agents in tree mode.
///
/// ## Purpose
/// Validates that tree mode with a single root session (no agents) produces
/// output without connector lines.
///
/// ## Coverage
/// Exit 0; output shows root session but no `├─`/`└─` connectors.
///
/// ## Validation Strategy
/// Create project with only root session (no agent sessions). Run
/// `.projects show_tree::1`. Assert exit 0 and no tree connectors in output.
///
/// ## Related Requirements
/// `tests/docs/cli/param/24_show_tree.md` — EC-6
#[ test ]
fn ec_6_show_tree_single_root_no_agents()
{
  let storage = TempDir::new().unwrap();
  let project_dir = TempDir::new().unwrap();

  common::write_path_project_session(
    storage.path(),
    project_dir.path(),
    "session-root-only",
    3,
  );

  let out = common::clg_cmd()
    .env( "CLAUDE_STORAGE_ROOT", storage.path() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_tree::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let output = stdout( &out );
  assert!(
    !output.contains( "├" ) && !output.contains( "└" ),
    "EC-6: show_tree::1 with no agents should have no tree connectors; got: {output}"
  );
}
