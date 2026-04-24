//! `ast` CLI sanity tests
//!
//! ## Purpose
//!
//! Verify that the `assistant` crate and its `ast` binary compile and
//! link correctly against all aggregated Layer 2 crates. These tests do not
//! exercise runtime behaviour — the binary integration tests are in the
//! individual `claude_version`, `claude_runner`, `claude_storage`, and
//! `claude_profile` crates.
//!
//! ## Coverage
//!
//! - The crate name and version are present (confirms build metadata)
//! - The `ast` binary is present in the build output
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `ast_package_name_is_assistant` | build metadata sanity |
//! | `ast_binary_is_present` | binary builds and runs |
//! | `ast_processes_command_accepted` | `.processes` routed via manager programmatic registration |
//! | `ast_projects_accepts_scope_param` | storage `.projects` accepts `scope::` (manager collision gone) |
//! | `ast_usage_command_accepted` | `.usage` routed via profile programmatic registration |
//! | `ast_paths_command_accepted` | `.paths` routed via profile programmatic registration |
//! | `ast_account_list_command_accepted` | `.account.list` routed via manager (first-wins) |

#[test]
fn ast_package_name_is_assistant()
{
  let name = env!( "CARGO_PKG_NAME" );
  assert_eq!( name, "assistant", "unexpected package name: {name}" );
}

#[test]
fn ast_binary_is_present()
{
  // Locate the `ast` binary in Cargo's output directory.
  // `cargo_bin!` panics if the binary is absent — indicating the [[bin]] entry
  // in Cargo.toml is missing or the `enabled` feature was not activated.
  let bin = assert_cmd::cargo::cargo_bin!( "ast" );
  assert!( bin.exists(), "astbinary not found at: {}", bin.display() );
}

/// Verify `.processes` is routed through `ast` via manager's programmatic registration.
#[test]
fn ast_processes_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "ast" )
  )
    .env( "HOME", home.path() )
    .args( [ ".processes" ] )
    .output()
    .unwrap();
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "ast.processes should exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// After storage renamed `.sessions` → `.projects` (task-015), `.projects`
/// is the scope-based session listing command in `ast`.  It accepts `scope::`.
#[test]
fn ast_projects_accepts_scope_param()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "ast" )
  )
    .env( "HOME", home.path() )
    .args( [ ".projects", "scope::local" ] )
    .output()
    .unwrap();
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "ast.projects scope::local must succeed (storage variant accepts scope); stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// Verify `.usage` is routed through `ast` via profile's programmatic registration.
///
/// `.usage` reads `stats-cache.json` which may not exist in a fresh temp HOME,
/// so exit 2 (runtime error) is acceptable — it proves the command was found
/// and dispatched.  Exit 1 would mean unknown command (registration failure).
#[test]
fn ast_usage_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "ast" )
  )
    .env( "HOME", home.path() )
    .args( [ ".usage" ] )
    .output()
    .unwrap();
  let code = out.status.code().unwrap_or( -1 );
  assert_ne!(
    code, 1,
    "ast.usage must not exit 1 (unknown command); stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// Verify `.paths` is routed through `ast` via profile's programmatic registration.
#[test]
fn ast_paths_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "ast" )
  )
    .env( "HOME", home.path() )
    .args( [ ".paths" ] )
    .output()
    .unwrap();
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "ast.paths should exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// Verify `.account.list` is routed through `ast`.
///
/// `.account.list` belongs exclusively to `claude_profile` — `claude_version`
/// does NOT register any `.account.*` commands. Profile owns all five account
/// commands; manager owns version, processes, and settings only.
#[test]
fn ast_account_list_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "ast" )
  )
    .env( "HOME", home.path() )
    .args( [ ".account.list" ] )
    .output()
    .unwrap();
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "ast.account.list should exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}
