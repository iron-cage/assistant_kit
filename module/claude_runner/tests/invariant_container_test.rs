//! Structural tests for invariant/010: Container-Only Test Execution.
//!
//! ## What
//!
//! These tests verify that the two enforcement layers required by
//! `docs/invariant/010_container_only_test_execution.md` are structurally present
//! and correctly configured:
//!
//! 1. **Nextest setup script**: `.config/setup-require-container` registered in
//!    `.config/nextest.toml`; checks three detection signals
//! 2. **Shell layer**: verified at the shell level (not directly testable in Rust);
//!    the `verb/test` rejection of `VERB_LAYER` and the `verb/test.d/l0` hard-error stub
//!    are integration-tested manually.
//!
//! ## Tests
//!
//! - `workspace_nextest_toml_registers_setup_script` (IT-1): workspace `.config/nextest.toml`
//!   contains `setup-scripts = true` and `require-container` reference
//! - `setup_script_file_exists` (IT-2): `.config/setup-require-container` exists at workspace root
//! - `setup_script_checks_dockerenv` (IT-3): script body checks `/.dockerenv` (signal 1)
//! - `setup_script_checks_containerenv` (IT-4): script body checks `/run/.containerenv` (signal 2)
//! - `setup_script_checks_runbox_var` (IT-5): script body checks `RUNBOX_CONTAINER` (signal 3)
//!
//! ## Self-Verifying Invariant
//!
//! This test file itself runs inside the container (enforced by the nextest setup script).
//! If any of these tests execute at all, signal 3 (`RUNBOX_CONTAINER=1`) was already satisfied
//! by `verb/test.d/l1`. The structural assertions guard against accidental misconfiguration
//! of the enforcement scripts.

use std::fs;
use std::path::Path;

fn workspace_root() -> std::path::PathBuf
{
  Path::new( env!( "CARGO_MANIFEST_DIR" ) )
    .parent()
    .expect( "module/ dir must have parent" )
    .parent()
    .expect( "workspace root must be 2 levels up from crate" )
    .to_path_buf()
}

#[ test ]
fn workspace_nextest_toml_registers_setup_script()
{
  let config_path = workspace_root().join( ".config/nextest.toml" );
  assert!(
    config_path.exists(),
    "Missing workspace .config/nextest.toml — required by invariant/010"
  );
  let content = fs::read_to_string( &config_path )
    .unwrap_or_else( |e| panic!( "Cannot read {}: {e}", config_path.display() ) ); // display() not Copy, can't inline
  assert!(
    content.contains( "setup-scripts = true" ),
    "`.config/nextest.toml` must contain `setup-scripts = true`"
  );
  assert!(
    content.contains( "require-container" ),
    "`.config/nextest.toml` must reference `require-container` setup script"
  );
}

#[ test ]
fn setup_script_file_exists()
{
  let script_path = workspace_root().join( ".config/setup-require-container" );
  assert!(
    script_path.exists(),
    "Missing workspace `.config/setup-require-container` — required by invariant/010"
  );
}

#[ test ]
fn setup_script_checks_dockerenv()
{
  let script_path = workspace_root().join( ".config/setup-require-container" );
  let content = fs::read_to_string( &script_path )
    .unwrap_or_else( |e| panic!( "Cannot read {}: {e}", script_path.display() ) ); // display() not Copy, can't inline
  assert!(
    content.contains( "/.dockerenv" ),
    "setup-require-container must check signal 1: `/.dockerenv` file existence"
  );
}

#[ test ]
fn setup_script_checks_containerenv()
{
  let script_path = workspace_root().join( ".config/setup-require-container" );
  let content = fs::read_to_string( &script_path )
    .unwrap_or_else( |e| panic!( "Cannot read {}: {e}", script_path.display() ) ); // display() not Copy, can't inline
  assert!(
    content.contains( "/run/.containerenv" ),
    "setup-require-container must check signal 2: `/run/.containerenv` file existence"
  );
}

#[ test ]
fn setup_script_checks_runbox_var()
{
  let script_path = workspace_root().join( ".config/setup-require-container" );
  let content = fs::read_to_string( &script_path )
    .unwrap_or_else( |e| panic!( "Cannot read {}: {e}", script_path.display() ) ); // display() not Copy, can't inline
  assert!(
    content.contains( "RUNBOX_CONTAINER" ),
    "setup-require-container must check signal 3: `RUNBOX_CONTAINER=1` env var"
  );
}
