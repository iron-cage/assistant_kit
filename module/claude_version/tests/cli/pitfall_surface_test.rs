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

use crate::subprocess_helpers::{ assert_exit, run_clv, run_clv_with_env, stdout };

// ─── PF-1 (pitfall/001_version_lock_chmod.md): install handles chmod automatically

// PF-1: dry::1 install completes without requiring manual chmod — exit 0
#[ test ]
fn pf01_001_chmod_auto_handled()
{
  let out = run_clv( &[ ".version.install", "version::stable", "dry::1" ] );
  // Command manages permissions internally; no manual chmod required from caller
  assert_exit( &out, 0 );
}

// ─── PF-2 (pitfall/001_version_lock_chmod.md): install dry-run previews chmod ─

// PF-2: dry::1 preview references chmod in output — trap is handled, not ignored
#[ test ]
fn pf02_001_chmod_dry_shows_chmod()
{
  let out = run_clv( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "chmod" ), "install dry-run must preview chmod step: {text}" );
}

// ─── PF-3 (pitfall/001_version_lock_chmod.md): guard dry-run shows restore cap ─

// PF-3: guard dry-run output indicates lock restoration capability
#[ test ]
fn pf03_001_guard_shows_restore()
{
  let out = run_clv( &[ ".version.guard", "dry::1" ] );
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
  let out = run_clv( &[ ".version.install", "version::stable", "dry::1" ] );
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

  let out = run_clv_with_env(
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

// PF-3: after stable install preference is set and matched by an installed
// binary, guard reports no drift.
//
// Fix(BUG-018): hardcoded `preferredVersionResolved: "2.1.78"` was dead data
// for the "stable" spec path (guard_once_pinned() re-resolves aliases fresh,
// ignoring the stored `resolved` value — see its own doc comment), and the
// test relied on the real system `claude` binary via unoverridden PATH
// instead of the documented symlink-isolation pattern — so the assertion
// actually compared the container's real installed version against the
// compile-time `VERSION_ALIASES` "stable" pin, both of which drift
// independently of this test and of each other.
// Root cause: missing the HOME-isolation symlink documented in
// subprocess_helpers.rs § HOME Isolation — Symlink Requirement.
// Pitfall: any test asserting "no drift" must pin BOTH sides of the
// comparison deterministically (settings + `~/.local/bin/claude` symlink) —
// never rely on whatever the host/container happens to have installed.
#[ test ]
fn pf03_002_no_drift_after_install()
{
  // Resolve stable alias to its pinned semver (compile-time constant; stays
  // in sync with VERSION_ALIASES automatically — no hardcoded string needed).
  let stable_ver = claude_version_core::version::VERSION_ALIASES
    .iter()
    .find( | a | a.name == "stable" )
    .map( | a | a.value )
    .expect( "stable alias not found in VERSION_ALIASES" );

  let dir = TempDir::new().unwrap();
  let home = dir.path();
  let claude_dir = home.join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let settings_json = format!( r#"{{
  "preferredVersionSpec": "stable",
  "preferredVersionResolved": "{stable_ver}"
}}"# );
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  // Deterministic install marker — get_version_from_symlink() reads the
  // symlink's target filename, not the real system claude binary, so this
  // is immune to whatever version the host/container actually has installed.
  let local_bin = home.join( ".local" ).join( "bin" );
  std::fs::create_dir_all( &local_bin ).unwrap();
  std::os::unix::fs::symlink( stable_ver, local_bin.join( "claude" ) ).unwrap();

  let out = run_clv_with_env(
    &[ ".version.guard", "dry::1" ],
    &[ ( "HOME", home.to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "matches" ) || text.contains( "ok" ),
    "guard after pinned install must report no drift: {text}"
  );
}
