//! `clt` CLI sanity tests
//!
//! ## Purpose
//!
//! Verify that the `claude_tools` crate and its `clt` binary compile and
//! link correctly against all aggregated Layer 2 crates. These tests do not
//! exercise runtime behaviour — the binary integration tests are in the
//! individual `claude_manager`, `claude_runner`, `claude_storage`, and
//! `claude_profile` crates.
//!
//! ## Coverage
//!
//! - The crate name and version are present (confirms build metadata)
//! - The `clt` binary is present in the build output
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `clt_package_name_is_claude_tools` | build metadata sanity |
//! | `clt_binary_is_present` | binary builds and runs |
//! | `clt_processes_command_accepted` | `.processes` routed via manager programmatic registration |
//! | `clt_sessions_accepts_scope_param` | storage `.sessions` accepts `scope::` (manager collision gone) |
//! | `clt_usage_command_accepted` | `.usage` routed via profile programmatic registration |
//! | `clt_paths_command_accepted` | `.paths` routed via profile programmatic registration |
//! | `clt_account_list_command_accepted` | `.account.list` routed via manager (first-wins) |

#[test]
fn clt_package_name_is_claude_tools()
{
  let name = env!( "CARGO_PKG_NAME" );
  assert_eq!( name, "claude_tools", "unexpected package name: {name}" );
}

#[test]
fn clt_binary_is_present()
{
  // Locate the `clt` binary in Cargo's output directory.
  // `cargo_bin!` panics if the binary is absent — indicating the [[bin]] entry
  // in Cargo.toml is missing or the `enabled` feature was not activated.
  let bin = assert_cmd::cargo::cargo_bin!( "clt" );
  assert!( bin.exists(), "clt binary not found at: {}", bin.display() );
}

/// Verify `.processes` is routed through `clt` via manager's programmatic registration.
#[test]
fn clt_processes_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "clt" )
  )
    .env( "HOME", home.path() )
    .args( [ ".processes" ] )
    .output()
    .unwrap();
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "clt .processes should exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// After `.sessions` → `.processes` rename in manager, storage's `.sessions`
/// (scope-based session listing) is now the winner in `clt`.  It accepts `scope::`.
#[test]
fn clt_sessions_accepts_scope_param()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "clt" )
  )
    .env( "HOME", home.path() )
    .args( [ ".sessions", "scope::local" ] )
    .output()
    .unwrap();
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "clt .sessions scope::local must succeed (storage variant accepts scope); stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// Verify `.usage` is routed through `clt` via profile's programmatic registration.
///
/// `.usage` reads `stats-cache.json` which may not exist in a fresh temp HOME,
/// so exit 2 (runtime error) is acceptable — it proves the command was found
/// and dispatched.  Exit 1 would mean unknown command (registration failure).
#[test]
fn clt_usage_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "clt" )
  )
    .env( "HOME", home.path() )
    .args( [ ".usage" ] )
    .output()
    .unwrap();
  let code = out.status.code().unwrap_or( -1 );
  assert_ne!(
    code, 1,
    "clt .usage must not exit 1 (unknown command); stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// Verify `.paths` is routed through `clt` via profile's programmatic registration.
#[test]
fn clt_paths_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "clt" )
  )
    .env( "HOME", home.path() )
    .args( [ ".paths" ] )
    .output()
    .unwrap();
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "clt .paths should exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// Verify `.account.list` is routed through `clt`.
///
/// `.account.list` belongs exclusively to `claude_profile` — `claude_manager`
/// does NOT register any `.account.*` commands. Profile owns all five account
/// commands; manager owns version, processes, and settings only.
#[test]
fn clt_account_list_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "clt" )
  )
    .env( "HOME", home.path() )
    .args( [ ".account.list" ] )
    .output()
    .unwrap();
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "clt .account.list should exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}
