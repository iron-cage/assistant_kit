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
//! | `ast_account_list_command_accepted` | `.accounts` routed via profile programmatic registration |
//! | `ast_journal_list_command_accepted` | `.journal.list` routed via journal_viewer YAML aggregation |

fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: cd module/assistant && ./verb/test\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

fn run_ast( home : &std::path::Path, args : &[ &str ] ) -> std::process::Output
{
  assert_container();
  std::process::Command::new( assert_cmd::cargo::cargo_bin!( "ast" ) )
    .env( "HOME", home )
    .args( args )
    .output()
    .unwrap()
}

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
  let out  = run_ast( home.path(), &[ ".processes" ] );
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
  let out  = run_ast( home.path(), &[ ".projects", "scope::local" ] );
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
  let out  = run_ast( home.path(), &[ ".usage" ] );
  let code = out.status.code().unwrap_or( -1 );
  assert!(
    code == 0 || code == 2,
    "ast.usage must exit 0 or 2 (not {code}); stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// Verify `.paths` is routed through `ast` via profile's programmatic registration.
#[test]
fn ast_paths_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out  = run_ast( home.path(), &[ ".paths" ] );
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "ast.paths should exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// Verify `.accounts` is routed through `ast`.
///
/// `.accounts` belongs exclusively to `claude_profile` — `claude_version`
/// does NOT register any `.account.*` commands. Profile owns all five account
/// commands; manager owns version, processes, and settings only.
#[test]
fn ast_account_list_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out  = run_ast( home.path(), &[ ".accounts" ] );
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "ast .accounts should exit 0; stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// Verify `.journal.list` is routed through `ast` via YAML aggregation from
/// `claude_journal_viewer`.
///
/// `.journal.list` is registered via the static YAML aggregation in `assistant/build.rs`
/// (not programmatic registration), which aggregates `claude_journal.commands.yaml`.
/// An empty journal dir is fine — the command lists 0 events and exits 0.
#[ test ]
fn ast_journal_list_command_accepted()
{
  let home = tempfile::TempDir::new().unwrap();
  let out  = run_ast( home.path(), &[ ".journal.list" ] );
  assert_eq!(
    out.status.code().unwrap_or( -1 ), 0,
    "ast .journal.list should exit 0 (empty journal is valid); stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}
