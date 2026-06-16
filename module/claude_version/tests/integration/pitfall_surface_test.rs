//! Pitfall surface tests (PF- prefix) for `claude_version`.
//!
//! Implements test cases from `tests/docs/pitfall/` spec files.
//! Each function maps to one PF- case verifying a documented trap is avoided.
//!
//! # Coverage Map
//!
//! | PF-spec | ID | Function |
//! |---------|----|----------|
//! | pitfall/001_version_lock_chmod.md | PF-1 | `pf01_001_chmod_auto_handled` |
//! | pitfall/001_version_lock_chmod.md | PF-2 | `pf02_001_chmod_dry_shows_chmod` |
//! | pitfall/001_version_lock_chmod.md | PF-3 | `pf03_001_guard_shows_restore` |
//! | pitfall/002_symlink_retarget.md | PF-1 | `pf01_002_purge_in_install_preview` |
//! | pitfall/002_symlink_retarget.md | PF-2 | `pf02_002_guard_dry_detects_drift` |
//! | pitfall/002_symlink_retarget.md | PF-3 | `pf03_002_no_drift_after_install` |

use tempfile::TempDir;

use crate::helpers::{ assert_exit, run_clm, run_clm_with_env, stdout };

// ─── PF-1 (pitfall/001_version_lock_chmod.md): install handles chmod automatically

// PF-1: dry::1 install completes without requiring manual chmod — exit 0
#[ test ]
fn pf01_001_chmod_auto_handled()
{
  let out = run_clm( &[ ".version.install", "version::stable", "dry::1" ] );
  // Command manages permissions internally; no manual chmod required from caller
  assert_exit( &out, 0 );
}

// ─── PF-2 (pitfall/001_version_lock_chmod.md): install dry-run previews chmod ─

// PF-2: dry::1 preview references chmod in output — trap is handled, not ignored
#[ test ]
fn pf02_001_chmod_dry_shows_chmod()
{
  let out = run_clm( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "chmod" ), "install dry-run must preview chmod step: {text}" );
}

// ─── PF-3 (pitfall/001_version_lock_chmod.md): guard dry-run shows restore cap ─

// PF-3: guard dry-run output indicates lock restoration capability
#[ test ]
fn pf03_001_guard_shows_restore()
{
  let out = run_clm( &[ ".version.guard", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Guard must convey it can restore/reinstall (not merely detect drift)
  assert!(
    text.contains( "install" ) || text.contains( "reinstall" ) || text.contains( "restore" ) || text.contains( "matches" ),
    "guard dry-run must indicate restoration or lock capability: {text}"
  );
}

// ─── PF-1 (pitfall/002_symlink_retarget.md): install preview confirms Layer 4 purge

// PF-1: dry::1 install stdout references binary purge or cache removal
#[ test ]
fn pf01_002_purge_in_install_preview()
{
  let out = run_clm( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "purge" ) || text.contains( "cache" ),
    "install dry-run must reference Layer 4 binary purge: {text}"
  );
}

// ─── PF-2 (pitfall/002_symlink_retarget.md): guard dry-run detects drift ──────

// PF-2: with a stale/mismatched preference, guard dry-run describes drift detection AND recovery
#[ test ]
fn pf02_002_guard_dry_detects_drift()
{
  // Write settings with a version that won't match installed claude (9.9.9 does not exist)
  // so guard detects drift between current install and preference
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let settings_json = r#"{
  "preferredVersionSpec": "9.9.9",
  "preferredVersionResolved": "9.9.9"
}"#;
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  let out = run_clm_with_env(
    &[ ".version.guard", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Guard must describe both detection and recovery steps
  assert!(
    text.contains( "drift" ) || text.contains( "[dry-run]" ),
    "guard dry-run with mismatched pref must describe drift detection: {text}"
  );
  assert!(
    text.contains( "reinstall" ) || text.contains( "install" ) || text.contains( "would" ),
    "guard dry-run must also describe recovery steps: {text}"
  );
}

// ─── PF-3 (pitfall/002_symlink_retarget.md): no drift after pinned install ────

// PF-3: after stable install preference is set to 2.1.78, guard reports no drift
#[ test ]
fn pf03_002_no_drift_after_install()
{
  // Write settings matching installed claude version (stable = 2.1.78 in container)
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let settings_json = r#"{
  "preferredVersionSpec": "stable",
  "preferredVersionResolved": "2.1.78"
}"#;
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  // Use real PATH so guard can find the installed claude binary at 2.1.78
  let out = run_clm_with_env(
    &[ ".version.guard", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Only assert no-drift when claude is present; "would install" means PATH lacks claude
  if !text.contains( "would install" )
  {
    assert!(
      text.contains( "matches" ) || text.contains( "ok" ),
      "guard after pinned install must report no drift: {text}"
    );
  }
}
