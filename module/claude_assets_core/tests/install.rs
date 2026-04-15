//! Real-filesystem integration tests for `claude_assets_core`.
// allow: test doc comments reference many function names; backtick-wrapping all is noisy
#![ allow( clippy::doc_markdown ) ]
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | inst01 | install() creates a symlink | read_link() succeeds |
//! | inst02 | install() creates target subdir | dir exists after install |
//! | inst03 | install() is idempotent (re-links) | second install returns Reinstalled |
//! | inst04 | install() refuses non-symlink target | AssetError::NotASymlink |
//! | inst05 | uninstall() removes symlink | path absent after uninstall |
//! | inst06 | uninstall() on absent path returns NotInstalled | action == NotInstalled |
//! | inst07 | uninstall() refuses non-symlink | AssetError::NotASymlink |
//! | inst08 | list_available() empty when source dir absent | returns empty vec |
//! | inst09 | list_available() returns source names | 3 .md files → 3 names |
//! | inst10 | list_installed() returns only symlinks | 2 installed, 1 regular file |
//! | inst11 | list_all() merges available and installed | correct InstallStatus per name |
//! | inst12 | AssetPaths::from_env() errors when both vars unset | AssetPathsError::EnvVarNotSet |
//! | inst13 | Directory-layout install creates dir symlink | Skill symlink → source dir |

use claude_assets_core::{
  artifact::ArtifactKind,
  error::AssetError,
  install::{ InstallAction, install, uninstall },
  paths::{ AssetPaths, AssetPathsError },
  registry::{ InstallStatus, list_all, list_available, list_installed },
};
use std::fs;
use tempfile::TempDir;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Create an `AssetPaths` pointing at two temp directories.
fn make_paths( src_dir : &std::path::Path, tgt_dir : &std::path::Path ) -> AssetPaths
{
  AssetPaths::new( src_dir.to_path_buf(), tgt_dir.to_path_buf() )
}

/// Write a dummy source file for the given kind + name.
fn write_source( paths : &AssetPaths, kind : ArtifactKind, name : &str )
{
  let dir = paths.source_dir( kind );
  fs::create_dir_all( &dir ).unwrap();
  match kind.layout()
  {
    claude_assets_core::artifact::ArtifactLayout::File =>
    {
      let ext = kind.file_extension().unwrap_or( "" );
      fs::write( dir.join( format!( "{name}.{ext}" ) ), b"# test" ).unwrap();
    }
    claude_assets_core::artifact::ArtifactLayout::Directory =>
    {
      fs::create_dir_all( dir.join( name ) ).unwrap();
    }
  }
}

// ── inst01 ────────────────────────────────────────────────────────────────────

/// inst01: install() creates a symlink confirmed by read_link().
///
/// Root Cause: install() must create a symlink (via create_symlink helper), never copy().
/// Why Not Caught: no test existed.
/// Fix Applied: install() calls symlink() and the test verifies with read_link().
/// Prevention: always verify symlink with read_link() in install tests.
/// Pitfall: fs::metadata() follows symlinks; use read_link() to confirm symlink type.
#[ test ]
fn inst01_install_creates_symlink()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );
  write_source( &paths, ArtifactKind::Rule, "rust" );

  let report = install( &paths, ArtifactKind::Rule, "rust" ).unwrap();
  assert_eq!( report.action, InstallAction::Installed );

  let tgt_path = paths.target_dir( ArtifactKind::Rule ).join( "rust.md" );
  assert!( fs::read_link( &tgt_path ).is_ok(), "target must be a symlink" );
}

// ── inst02 ────────────────────────────────────────────────────────────────────

/// inst02: install() creates the target .claude/rules/ subdir when absent.
///
/// Root Cause: .claude/rules/ may not exist in a fresh project checkout.
/// Why Not Caught: no test existed.
/// Fix Applied: install() calls create_dir_all() on the target subdir.
/// Prevention: always install into a fresh temp dir without pre-creating target.
/// Pitfall: if target dir is created by a prior test, this test won't catch regression.
#[ test ]
fn inst02_install_creates_target_subdir()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );
  write_source( &paths, ArtifactKind::Rule, "python" );

  // Target .claude/rules/ must NOT exist yet.
  assert!( !paths.target_dir( ArtifactKind::Rule ).exists() );

  install( &paths, ArtifactKind::Rule, "python" ).unwrap();
  assert!( paths.target_dir( ArtifactKind::Rule ).is_dir() );
}

// ── inst03 ────────────────────────────────────────────────────────────────────

/// inst03: install() is idempotent — second call re-links and returns Reinstalled.
///
/// Root Cause: repeated installs must succeed, not fail on existing symlink.
/// Why Not Caught: no test existed.
/// Fix Applied: install() removes existing symlink before re-linking.
/// Prevention: always run install twice in idempotency tests.
/// Pitfall: Reinstalled ≠ error; the symlink still points correctly after.
#[ test ]
fn inst03_install_is_idempotent()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );
  write_source( &paths, ArtifactKind::Command, "commit" );

  install( &paths, ArtifactKind::Command, "commit" ).unwrap();
  let r2 = install( &paths, ArtifactKind::Command, "commit" ).unwrap();
  assert_eq!( r2.action, InstallAction::Reinstalled );

  // Symlink still valid after re-link.
  let tgt_path = paths.target_dir( ArtifactKind::Command ).join( "commit.md" );
  assert!( fs::read_link( &tgt_path ).is_ok() );
}

// ── inst04 ────────────────────────────────────────────────────────────────────

/// inst04: install() refuses to replace a regular file at the target path.
///
/// Root Cause: silently overwriting a regular file would destroy user data.
/// Why Not Caught: no test existed.
/// Fix Applied: install() checks symlink_metadata() and returns NotASymlink for regular files.
/// Prevention: never remove non-symlinks in install; guard with is_symlink().
/// Pitfall: fs::metadata() follows the link — symlink_metadata() must be used instead.
#[ test ]
fn inst04_install_refuses_non_symlink_target()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );
  write_source( &paths, ArtifactKind::Rule, "go" );

  // Place a regular file where the symlink would go.
  let tgt_dir = paths.target_dir( ArtifactKind::Rule );
  fs::create_dir_all( &tgt_dir ).unwrap();
  fs::write( tgt_dir.join( "go.md" ), b"regular file" ).unwrap();

  let err = install( &paths, ArtifactKind::Rule, "go" ).unwrap_err();
  assert!(
    matches!( err, AssetError::NotASymlink { .. } ),
    "expected NotASymlink, got: {err}",
  );

  // Original file must still be present.
  assert!( tgt_dir.join( "go.md" ).exists() );
}

// ── inst05 ────────────────────────────────────────────────────────────────────

/// inst05: uninstall() removes an existing symlink.
///
/// Root Cause: uninstall must clean up the target symlink.
/// Why Not Caught: no test existed.
/// Fix Applied: uninstall() removes the file after symlink_metadata() confirms it's a link.
/// Prevention: verify path is absent after uninstall.
/// Pitfall: symlink may be dangling (source deleted); symlink_metadata() still detects it.
#[ test ]
fn inst05_uninstall_removes_symlink()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );
  write_source( &paths, ArtifactKind::Agent, "planner" );

  install( &paths, ArtifactKind::Agent, "planner" ).unwrap();
  let tgt_path = paths.target_dir( ArtifactKind::Agent ).join( "planner.md" );
  assert!( fs::symlink_metadata( &tgt_path ).is_ok() );

  let report = uninstall( &paths, ArtifactKind::Agent, "planner" ).unwrap();
  assert_eq!( report.action, InstallAction::Uninstalled );
  assert!( !tgt_path.exists() );
}

// ── inst06 ────────────────────────────────────────────────────────────────────

/// inst06: uninstall() on absent path returns NotInstalled (not an error).
///
/// Root Cause: uninstall of a never-installed artifact must be harmless.
/// Why Not Caught: no test existed.
/// Fix Applied: uninstall() returns NotInstalled when symlink_metadata() yields NotFound.
/// Prevention: always check action == NotInstalled for absent-uninstall case.
/// Pitfall: returning an error here would break idempotent teardown scripts.
#[ test ]
fn inst06_uninstall_absent_returns_not_installed()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );

  let report = uninstall( &paths, ArtifactKind::Rule, "nonexistent" ).unwrap();
  assert_eq!( report.action, InstallAction::NotInstalled );
}

// ── inst07 ────────────────────────────────────────────────────────────────────

/// inst07: uninstall() refuses to remove a regular file (data-loss guard).
///
/// Root Cause: uninstall must only remove symlinks it created, never regular files.
/// Why Not Caught: no test existed.
/// Fix Applied: uninstall() calls symlink_metadata() and returns NotASymlink.
/// Prevention: AF6 anti-faking check — always test with a real regular file.
/// Pitfall: fs::metadata() follows symlinks; only symlink_metadata() detects the link.
#[ test ]
fn inst07_uninstall_refuses_regular_file()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );

  let tgt_dir = paths.target_dir( ArtifactKind::Hook );
  fs::create_dir_all( &tgt_dir ).unwrap();
  let regular_file = tgt_dir.join( "pre_commit.yaml" );
  fs::write( &regular_file, b"hooks:" ).unwrap();

  let err = uninstall( &paths, ArtifactKind::Hook, "pre_commit" ).unwrap_err();
  assert!(
    matches!( err, AssetError::NotASymlink { .. } ),
    "expected NotASymlink, got: {err}",
  );
  assert!( regular_file.exists(), "regular file must not be deleted" );
}

// ── inst08 ────────────────────────────────────────────────────────────────────

/// inst08: list_available() returns empty vec when source dir absent.
///
/// Root Cause: graceful degradation — missing source dir is not an error.
/// Why Not Caught: no test existed.
/// Fix Applied: list_available() returns Ok(vec![]) when source dir doesn't exist.
/// Prevention: always test with a source dir that doesn't exist.
/// Pitfall: returning an error here breaks `cla .list` in repos with no rules yet.
#[ test ]
fn inst08_list_available_empty_when_source_absent()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );
  // No source/rules dir created.

  let available = list_available( &paths, ArtifactKind::Rule ).unwrap();
  assert!( available.is_empty() );
}

// ── inst09 ────────────────────────────────────────────────────────────────────

/// inst09: list_available() returns correct names from source dir.
///
/// Root Cause: must enumerate .md files and strip extension.
/// Why Not Caught: no test existed.
/// Fix Applied: list_available() scans source dir and applies artifact_name().
/// Prevention: verify length and membership of returned names.
/// Pitfall: non-.md files in the source dir must be ignored.
#[ test ]
fn inst09_list_available_returns_source_names()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );

  write_source( &paths, ArtifactKind::Rule, "rust" );
  write_source( &paths, ArtifactKind::Rule, "python" );
  write_source( &paths, ArtifactKind::Rule, "go" );
  // Extra file with wrong extension — must be ignored.
  let dir = paths.source_dir( ArtifactKind::Rule );
  fs::write( dir.join( "ignore.txt" ), b"" ).unwrap();

  let mut available = list_available( &paths, ArtifactKind::Rule ).unwrap();
  available.sort();
  assert_eq!( available, vec![ "go", "python", "rust" ] );
}

// ── inst10 ────────────────────────────────────────────────────────────────────

/// inst10: list_installed() returns only symlink names (skips regular files).
///
/// Root Cause: list_installed must distinguish installed symlinks from regular files.
/// Why Not Caught: no test existed.
/// Fix Applied: list_installed() uses symlink_metadata() and filters non-symlinks.
/// Prevention: place a regular file alongside symlinks and assert it's excluded.
/// Pitfall: is_symlink() after metadata() follows the link; symlink_metadata() doesn't.
#[ test ]
fn inst10_list_installed_returns_only_symlinks()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );

  write_source( &paths, ArtifactKind::Rule, "rust" );
  write_source( &paths, ArtifactKind::Rule, "python" );
  install( &paths, ArtifactKind::Rule, "rust" ).unwrap();
  install( &paths, ArtifactKind::Rule, "python" ).unwrap();

  // Add a regular file — must NOT appear in list_installed().
  let tgt_dir = paths.target_dir( ArtifactKind::Rule );
  fs::write( tgt_dir.join( "stale.md" ), b"regular" ).unwrap();

  let mut installed = list_installed( &paths, ArtifactKind::Rule ).unwrap();
  installed.sort();
  assert_eq!( installed, vec![ "python", "rust" ] );
}

// ── inst11 ────────────────────────────────────────────────────────────────────

/// inst11: list_all() merges available and installed with correct statuses.
///
/// Root Cause: CLI .list command needs unified view with Install/Available marker.
/// Why Not Caught: no test existed.
/// Fix Applied: list_all() merges list_available() and list_installed().
/// Prevention: verify both statuses are present and entries are sorted.
/// Pitfall: an installed entry absent from source must also appear as Installed.
#[ test ]
fn inst11_list_all_merges_with_correct_status()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );

  write_source( &paths, ArtifactKind::Rule, "a" );
  write_source( &paths, ArtifactKind::Rule, "b" );
  install( &paths, ArtifactKind::Rule, "a" ).unwrap();
  // "b" is available but not installed.

  let all = list_all( &paths, ArtifactKind::Rule ).unwrap();
  assert_eq!( all.len(), 2 );

  let a_status = all.iter().find( | ( n, _ ) | n == "a" ).map( | ( _, s ) | *s );
  let b_status = all.iter().find( | ( n, _ ) | n == "b" ).map( | ( _, s ) | *s );
  assert_eq!( a_status, Some( InstallStatus::Installed ) );
  assert_eq!( b_status, Some( InstallStatus::Available ) );
}

// ── inst12 ────────────────────────────────────────────────────────────────────

/// inst12: AssetPaths::from_env() returns EnvVarNotSet when both vars are absent.
///
/// Root Cause: clear actionable error required when env is not configured.
/// Why Not Caught: no test existed.
/// Fix Applied: from_env() checks PRO_CLAUDE then PRO; if neither set, returns error.
/// Prevention: run in a controlled env that clears both vars.
/// Pitfall: CI machines may have PRO set from a prior test; always clear both.
#[ test ]
fn inst12_from_env_errors_when_vars_unset()
{
  // Remove both vars for this test.
  std::env::remove_var( "PRO_CLAUDE" );
  std::env::remove_var( "PRO" );

  let err = AssetPaths::from_env().unwrap_err();
  assert!(
    matches!( err, AssetPathsError::EnvVarNotSet ),
    "expected EnvVarNotSet, got: {err}",
  );
  let msg = err.to_string();
  assert!( msg.contains( "PRO_CLAUDE" ), "error must mention PRO_CLAUDE, got: {msg}" );
}

// ── inst13 ────────────────────────────────────────────────────────────────────

/// inst13: install() creates a directory symlink for Skill (Directory layout).
///
/// Root Cause: skills are directory trees; symlink must point to dir, not copy it.
/// Why Not Caught: no test existed.
/// Fix Applied: install() uses create_symlink() — dispatches to the correct platform API.
/// Prevention: confirm with read_link() that the target is a symlink to a dir.
/// Pitfall: fs::copy() silently copies file trees; read_link() fails on copies.
#[ test ]
fn inst13_install_directory_layout_creates_dir_symlink()
{
  let src = TempDir::new().unwrap();
  let tgt = TempDir::new().unwrap();
  let paths = make_paths( src.path(), tgt.path() );
  write_source( &paths, ArtifactKind::Skill, "tsk" );

  let report = install( &paths, ArtifactKind::Skill, "tsk" ).unwrap();
  assert_eq!( report.action, InstallAction::Installed );

  let tgt_path = paths.target_dir( ArtifactKind::Skill ).join( "tsk" );
  let link_target = fs::read_link( &tgt_path ).expect( "target must be a symlink" );
  assert!( link_target.ends_with( "skills/tsk" ), "symlink must point to source dir, got: {link_target:?}" );
}
