//! CLI integration tests for `claude_assets` (`cla` binary).
// allow: test doc comments reference many function names; backtick-wrapping all is noisy
#![ allow( clippy::doc_markdown ) ]
//!
//! ## Test Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | cli01 | `cla .kinds` with PRO_CLAUDE set | Exit 0; ≥6 lines showing kinds |
//! | cli02 | `cla .list` with PRO_CLAUDE unset | Exit 2; error mentions PRO_CLAUDE |
//! | cli03 | `cla .list kind::rule` with source dir empty | Exit 0; "No artifacts" message |
//! | cli04 | `cla .install kind::rule name::rust` | Exit 0; creates symlink |
//! | cli05 | `cla .install kind::rule name::nonexistent` | Exit 2; "not found" error |
//! | cli06 | `cla .install kind::rule name::rust` second time | Exit 0; "Reinstalled" message |
//! | cli07 | `cla .uninstall kind::rule name::rust` installed | Exit 0; "Uninstalled" message |
//! | cli08 | `cla .uninstall kind::rule name::rust` not installed | Exit 0; "Not installed" message |
//! | cli09 | `cla .list kind::rule` with 2 source, 1 installed | Exit 0; ● and ○ markers |
//! | cli10 | `cla .install` without kind:: | Exit 1; error mentions kind:: |
//! | cli11 | `cla .install kind::invalid name::x` | Exit 1; "unknown kind" error |

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

// ── helpers ───────────────────────────────────────────────────────────────────

fn cla() -> Command
{
  Command::cargo_bin( "cla" ).unwrap()
}

/// Write a dummy `.md` source file for `kind`/`name` in `src_dir/<kind>s/`.
fn write_source_file( src_dir : &std::path::Path, kind : &str, name : &str )
{
  let dir = src_dir.join( format!( "{kind}s" ) );
  fs::create_dir_all( &dir ).unwrap();
  fs::write( dir.join( format!( "{name}.md" ) ), b"# test" ).unwrap();
}

// ── cli01 ─────────────────────────────────────────────────────────────────────

/// cli01: `.kinds` with PRO_CLAUDE set exits 0 and shows all 6 kinds.
///
/// Root Cause: kinds_routine must succeed even when .claude/ is absent.
/// Why Not Caught: no test existed.
/// Fix Applied: kinds_routine reads only env var, no filesystem access required.
/// Prevention: always run .kinds as a smoke test after any routine change.
/// Pitfall: if PRO_CLAUDE is set to a nonexistent path, .kinds still works (display only).
#[ test ]
fn cli01_kinds_exits_0_with_six_kinds()
{
  let dir = TempDir::new().unwrap();
  let out = cla()
    .args( [ ".kinds" ] )
    .env( "PRO_CLAUDE", dir.path() )
    .output()
    .unwrap();

  assert!( out.status.success(), "exit must be 0, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let lines : Vec< _ > = stdout.lines().filter( |l| !l.trim().is_empty() ).collect();
  assert!( lines.len() >= 6, "must show at least 6 kinds, got {}: {stdout}", lines.len() );
  assert!( stdout.contains( "rule" ),    "must mention rule, got: {stdout}" );
  assert!( stdout.contains( "command" ), "must mention command, got: {stdout}" );
}

// ── cli02 ─────────────────────────────────────────────────────────────────────

/// cli02: `.list` without PRO_CLAUDE exits 2 with actionable error.
///
/// Root Cause: AssetPaths::from_env() must return a typed error, not panic.
/// Why Not Caught: no test existed.
/// Fix Applied: from_env() returns AssetPathsError::EnvVarNotSet; mapped to InternalError (exit 2).
/// Prevention: always test with both PRO_CLAUDE and PRO unset.
/// Pitfall: PRO may be set in CI; clear both vars explicitly.
#[ test ]
fn cli02_list_without_env_exits_2()
{
  let out = cla()
    .args( [ ".list" ] )
    .env_remove( "PRO_CLAUDE" )
    .env_remove( "PRO" )
    .output()
    .unwrap();

  assert_eq!( out.status.code(), Some( 2 ), "exit must be 2, got: {:?}", out.status );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "PRO_CLAUDE" ),
    "error must mention PRO_CLAUDE, got: {stderr}",
  );
}

// ── cli03 ─────────────────────────────────────────────────────────────────────

/// cli03: `.list kind::rule` with empty source dir exits 0 and says "No artifacts".
///
/// Root Cause: graceful degradation — missing source dir must not be an error.
/// Why Not Caught: no test existed.
/// Fix Applied: list_available() returns empty vec when source dir absent.
/// Prevention: always test with a fresh, empty PRO_CLAUDE dir.
/// Pitfall: if .list returns exit 1 here, `cla .list` fails for new repos.
#[ test ]
fn cli03_list_empty_source_exits_0()
{
  let dir = TempDir::new().unwrap();
  let out = cla()
    .args( [ ".list", "kind::rule" ] )
    .env( "PRO_CLAUDE", dir.path() )
    .output()
    .unwrap();

  assert!( out.status.success(), "exit must be 0, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "No artifacts" ),
    "must say 'No artifacts', got: {stdout}",
  );
}

// ── cli04 ─────────────────────────────────────────────────────────────────────

/// cli04: `.install kind::rule name::rust` creates a symlink; exit 0.
///
/// Root Cause: install must use symlink(), not copy().
/// Why Not Caught: no test existed.
/// Fix Applied: install() calls std::os::unix::fs::symlink().
/// Prevention: verify symlink with read_link() after install.
/// Pitfall: stat shows same content for copy and symlink; read_link() distinguishes.
#[ test ]
fn cli04_install_creates_symlink()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  write_source_file( src.path(), "rule", "rust" );

  let out = cla()
    .args( [ ".install", "kind::rule", "name::rust" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .output()
    .unwrap();

  assert!( out.status.success(), "exit must be 0, got: {:?}", out.status.code() );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( "Installed" ) || stdout.contains( "install" ), "must confirm install, got: {stdout}" );

  let link = tgt.path().join( ".claude" ).join( "rules" ).join( "rust.md" );
  assert!( fs::read_link( &link ).is_ok(), "target must be a symlink, got: {link:?}" );
}

// ── cli05 ─────────────────────────────────────────────────────────────────────

/// cli05: `.install kind::rule name::nonexistent` exits 2 with "not found" error.
///
/// Root Cause: install must fail clearly when source artifact is absent.
/// Why Not Caught: no test existed.
/// Fix Applied: install() returns AssetError::SourceNotFound for absent source.
/// Prevention: always test with a name that doesn't exist in the source.
/// Pitfall: if exit code is 1 (not 2), clt scripts may misinterpret as usage error.
#[ test ]
fn cli05_install_nonexistent_exits_2()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();

  let out = cla()
    .args( [ ".install", "kind::rule", "name::nonexistent" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .output()
    .unwrap();

  assert_eq!( out.status.code(), Some( 2 ), "exit must be 2, got: {:?}", out.status );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.to_lowercase().contains( "not found" ) || stderr.contains( "nonexistent" ),
    "error must mention 'not found' or the name, got: {stderr}",
  );
}

// ── cli06 ─────────────────────────────────────────────────────────────────────

/// cli06: installing twice is idempotent — second call exits 0 with "Reinstalled".
///
/// Root Cause: repeated installs must succeed, not fail on existing symlink.
/// Why Not Caught: no test existed.
/// Fix Applied: install() removes and recreates the symlink on second call.
/// Prevention: always run install twice and assert both succeed.
/// Pitfall: if second call errors, automation scripts that `cla .install` unconditionally will break.
#[ test ]
fn cli06_install_idempotent_reinstalls()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  write_source_file( src.path(), "rule", "go" );

  // First install.
  cla()
    .args( [ ".install", "kind::rule", "name::go" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .assert()
    .success();

  // Second install — must be idempotent.
  let out = cla()
    .args( [ ".install", "kind::rule", "name::go" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .output()
    .unwrap();

  assert!( out.status.success(), "second install must exit 0, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Reinstalled" ) || stdout.contains( "install" ),
    "must confirm reinstall, got: {stdout}",
  );
}

// ── cli07 ─────────────────────────────────────────────────────────────────────

/// cli07: `.uninstall kind::rule name::rust` on installed artifact exits 0 with "Uninstalled".
///
/// Root Cause: uninstall must remove symlink and confirm removal.
/// Why Not Caught: no test existed.
/// Fix Applied: uninstall() removes symlink and returns Uninstalled action.
/// Prevention: verify symlink is absent after uninstall.
/// Pitfall: if uninstall exits 2, it may be confused with a data-unavailable error.
#[ test ]
fn cli07_uninstall_installed_artifact()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  write_source_file( src.path(), "rule", "rust" );

  // Install first.
  cla()
    .args( [ ".install", "kind::rule", "name::rust" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .assert()
    .success();

  let link = tgt.path().join( ".claude" ).join( "rules" ).join( "rust.md" );
  assert!( fs::symlink_metadata( &link ).is_ok(), "symlink must exist before uninstall" );

  // Uninstall.
  let out = cla()
    .args( [ ".uninstall", "kind::rule", "name::rust" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .output()
    .unwrap();

  assert!( out.status.success(), "exit must be 0, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Uninstalled" ) || stdout.contains( "uninstall" ),
    "must confirm uninstall, got: {stdout}",
  );
  assert!( !link.exists(), "symlink must be absent after uninstall" );
}

// ── cli08 ─────────────────────────────────────────────────────────────────────

/// cli08: `.uninstall kind::rule name::rust` when not installed exits 0 (not error).
///
/// Root Cause: uninstall of absent artifact must be idempotent (not error).
/// Why Not Caught: no test existed.
/// Fix Applied: uninstall() returns NotInstalled action for absent path.
/// Prevention: always test uninstall on a never-installed name.
/// Pitfall: if uninstall exits 2 here, `make clean` style scripts break.
#[ test ]
fn cli08_uninstall_not_installed_exits_0()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();

  let out = cla()
    .args( [ ".uninstall", "kind::rule", "name::ghost" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .output()
    .unwrap();

  assert!( out.status.success(), "exit must be 0, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "Not installed" ) || stdout.contains( "not installed" ),
    "must say 'Not installed', got: {stdout}",
  );
}

// ── cli09 ─────────────────────────────────────────────────────────────────────

/// cli09: `.list kind::rule` shows ● for installed and ○ for available.
///
/// Root Cause: list_all() must merge available and installed with correct markers.
/// Why Not Caught: no test existed.
/// Fix Applied: list_routine() prints "●" for Installed and "○" for Available.
/// Prevention: install one of two rules, then assert both markers appear.
/// Pitfall: if list only shows installed, available artifacts are invisible to the user.
#[ test ]
fn cli09_list_shows_installed_and_available_markers()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  write_source_file( src.path(), "rule", "rust" );
  write_source_file( src.path(), "rule", "python" );

  // Install only rust.
  cla()
    .args( [ ".install", "kind::rule", "name::rust" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .assert()
    .success();

  let out = cla()
    .args( [ ".list", "kind::rule" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .output()
    .unwrap();

  assert!( out.status.success(), "exit must be 0, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( '●' ), "must show ● for installed, got: {stdout}" );
  assert!( stdout.contains( '○' ), "must show ○ for available, got: {stdout}" );
}

// ── cli10 ─────────────────────────────────────────────────────────────────────

/// cli10: `.install` without kind:: exits 1 (argument missing = usage error).
///
/// Root Cause: kind:: is required; missing it is a usage error (exit 1, not 2).
/// Why Not Caught: no test existed.
/// Fix Applied: require_str() returns ArgumentMissing (exit 1) for empty kind.
/// Prevention: always test required params with absent values.
/// Pitfall: exit 2 for missing args would be confused with runtime errors.
#[ test ]
fn cli10_install_without_kind_exits_1()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();

  let out = cla()
    .args( [ ".install", "name::rust" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .output()
    .unwrap();

  assert_eq!( out.status.code(), Some( 1 ), "exit must be 1, got: {:?}", out.status );
}

// ── cli11 ─────────────────────────────────────────────────────────────────────

/// cli11: `.install kind::invalid name::x` exits 1 for unknown kind.
///
/// Root Cause: invalid kind string is a usage error (exit 1).
/// Why Not Caught: no test existed.
/// Fix Applied: parse_kind() returns ArgumentTypeMismatch (exit 1) for unknown string.
/// Prevention: always test with a kind string not in the supported set.
/// Pitfall: exit 2 for unknown kinds conflates input errors with runtime errors.
#[ test ]
fn cli11_install_invalid_kind_exits_1()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();

  let out = cla()
    .args( [ ".install", "kind::invalid", "name::x" ] )
    .env( "PRO_CLAUDE", src.path() )
    .current_dir( tgt.path() )
    .output()
    .unwrap();

  assert_eq!( out.status.code(), Some( 1 ), "exit must be 1, got: {:?}", out.status );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "unknown kind" ) || stderr.contains( "invalid" ),
    "must mention unknown kind, got: {stderr}",
  );
}
