//! Integration tests for `.version.guard` — E14.
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 400 | No preference set → defaults to stable, exit 0 | P | 0 |
//! | 401 | `dry::1` no preference → defaults to stable, exit 0 | P | 0 |
//! | 402 | HOME not set → defaults to stable, exit 0 | P | 0 |
//! | 403 | preference=latest, `dry::1` → "no version pin to guard" | P | 0 |
//! | 406 | `dry::1 force::1` → dry wins, no install | P | 0 |
//! | 409 | `interval::0` behaves as one-shot (default) | P | 0 |
//! | 410 | stale `preferredVersionResolved` → re-resolves alias | P | 0 |
//! | 411 | `version::9.9.9 dry::1` → override shown in output, exit 0 | P | 0 |
//! | 412 | `bogus::x` → unknown parameter, exit 1 | N | 1 |
//! | 413 | `version::9.9.9 force::1 dry::1` → dry wins, override shown | P | 0 |
//! | 414 | `version::` (empty) → exit 1 | N | 1 |
//! | 415 | watch loop continues after install error (`bug_reproducer`) | P | 124 |
//! | 416 | `version::latest dry::1` override → "no version pin to guard" | P | 0 |
//! | 417 | `dry::1 v::0` → output shorter than `v::1` | P | 0 |

use tempfile::TempDir;

use crate::subprocess_helpers::{
  assert_exit, run_clv, run_clv_with_env, stderr, stdout, write_settings,
};

// ─── E14: version guard ─────────────────────────────────────────────────────

// TC-400: no preference set → defaults to stable, exit 0
#[ test ]
fn tc400_guard_no_preference()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.guard" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "stable" ),
    "guard with no preference must default to stable: {text}"
  );
}

// TC-401: dry::1 no preference → defaults to stable, exit 0
#[ test ]
fn tc401_guard_dry_no_preference()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.guard", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "stable" ),
    "dry guard with no preference must default to stable: {text}"
  );
}

// TC-402: HOME not set → defaults to stable, exit 0 (resilient)
#[ test ]
fn tc402_guard_no_home()
{
  let out = run_clv_with_env(
    &[ ".version.guard" ],
    &[ ( "HOME", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "stable" ),
    "guard without HOME must default to stable: {text}"
  );
}

// TC-403: preference=latest, dry::1 → "no version pin to guard"
#[ test ]
fn tc403_guard_latest_dry()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write settings with latest preference (resolved=null).
  let settings_json = r#"{
  "preferredVersionSpec": "latest",
  "preferredVersionResolved": null
}"#;
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  let out = run_clv_with_env(
    &[ ".version.guard", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "latest" ),
    "guard with latest preference must mention latest: {text}"
  );
}

// TC-406: dry::1 force::1 → dry wins, no install
#[ test ]
fn tc406_guard_dry_force_no_install()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write settings with pinned preference.
  let settings_json = r#"{
  "preferredVersionSpec": "stable",
  "preferredVersionResolved": "2.1.78"
}"#;
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  let out = run_clv_with_env(
    &[ ".version.guard", "dry::1", "force::1" ],
    &[ ( "HOME", home ), ( "PATH", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run]" ),
    "dry must prevail over force: {text}"
  );
}

// TC-410: stale preferredVersionResolved → guard re-resolves alias
//
// Root Cause
//
// After an alias bump (e.g. month 2.1.50 → 2.1.74) the stored
// `preferredVersionResolved` in settings.json becomes stale.
// `guard_once()` must re-resolve the alias through the current table
// rather than blindly trusting the stored resolved value.
//
// Why Not Caught
//
// Previous tests never wrote a stale resolved value that diverged
// from the compile-time alias table.
//
// Fix Applied
//
// `guard_once()` calls `resolve_version_spec()` on `spec` before
// using it, so the current alias value always wins.
//
// Prevention
//
// This test explicitly writes a stale resolved value and asserts the
// output uses the current alias mapping.
//
// Pitfall
//
// Any code path that reads `preferredVersionResolved` for comparison
// must re-resolve the alias name first; the stored value is advisory.
#[ test ]
fn tc410_guard_reresoves_stale_alias()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Stale resolved value from a previous alias mapping.
  let settings_json = r#"{
  "preferredVersionSpec": "month",
  "preferredVersionResolved": "2.1.50"
}"#;
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  let out = run_clv_with_env(
    &[ ".version.guard", "dry::1" ],
    &[ ( "HOME", home ), ( "PATH", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Must use re-resolved value (2.1.74), not stale stored value (2.1.50).
  assert!(
    text.contains( "2.1.74" ),
    "guard must re-resolve alias to current value 2.1.74: {text}"
  );
  assert!(
    !text.contains( "2.1.50" ),
    "guard must NOT use stale stored resolved 2.1.50: {text}"
  );
}

// TC-409: interval::0 behaves as one-shot (default)
#[ test ]
fn tc409_guard_interval_zero_oneshot()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.guard", "interval::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "stable" ),
    "interval::0 must behave as one-shot with stable default: {text}"
  );
}

// TC-411: version::9.9.9 dry::1 → override shown in output, exit 0
#[ test ]
fn tc411_guard_version_override_dry()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.guard", "version::9.9.9", "dry::1" ],
    &[ ( "HOME", home ), ( "PATH", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "9.9.9" ),
    "version override 9.9.9 must appear in dry-run output: {text}"
  );
}

// TC-412: bogus::x → unknown parameter, exit 1
#[ test ]
fn tc412_guard_unknown_param_rejected()
{
  let out = run_clv( &[ ".version.guard", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// TC-413: version::9.9.9 force::1 dry::1 → dry wins, override shown in output
#[ test ]
fn tc413_guard_version_override_dry_wins_over_force()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.guard", "version::9.9.9", "force::1", "dry::1" ],
    &[ ( "HOME", home ), ( "PATH", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "9.9.9" ),
    "version override 9.9.9 must appear in dry-run output: {text}"
  );
  assert!(
    text.contains( "[dry-run]" ),
    "dry::1 must produce [dry-run] prefix: {text}"
  );
}

// TC-414: version:: (empty value) → exit 1, stderr mentions version
#[ test ]
fn tc414_guard_version_empty_rejected()
{
  let out = run_clv( &[ ".version.guard", "version::" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "version" ),
    "stderr must mention 'version' for empty version:: error: {err}"
  );
}

// TC-415: watch loop continues after install error  —  bug_reproducer(BUG-005)
//
// # Root Cause
// The watch loop in `version_guard_routine` calls `return result` when
// `guard_once()` returns `Err`. This exits the entire watch daemon on the
// first transient install failure (e.g. ETXTBSY "Text file busy").
// In watch mode the error should be logged and the loop should continue.
//
// # Why Not Caught
// No test existed for the watch+error code path.  TC-409 only covers
// `interval::0` (one-shot), which never reaches the looping branch.
//
// # Fix Applied
// Remove `return result` from the Err branch in the watch loop; let the
// loop fall through to `sleep` and continue to the next iteration.
//
// # Prevention
// Uses `timeout 2 clv .version.guard interval::1` with empty PATH to force
// install failure on every iteration.  If the loop exits early (bug), the
// process returns exit 2 before the 2-second deadline.  If the loop
// continues (fix), `timeout` kills it and returns 124.
//
// # Pitfall
// One-shot mode (`interval::0`) MUST still propagate errors — only the
// watch branch (`interval > 0`) should swallow and continue.
#[ test ]
fn tc415_watch_loop_continues_after_install_error()
{
  let dir  = tempfile::TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Drift state: preferred = 9.9.9 but no claude installed → install attempted.
  // Empty PATH ensures curl/bash are not found → perform_install returns Err.
  let settings_json = r#"{"preferredVersionSpec":"9.9.9","preferredVersionResolved":"9.9.9"}"#;
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  let out = std::process::Command::new( "/usr/bin/timeout" )
    .args( [ "2", bin, ".version.guard", "interval::1" ] )
    .env( "HOME", home )
    .env( "PATH", "" )
    .output()
    .expect( "timeout command failed to launch" );

  let err = String::from_utf8_lossy( &out.stderr );
  let code = out.status.code().unwrap_or( -1 );

  // With the bug: loop returns on first error → exits with code 2 in < 1 second.
  // With the fix: loop continues → timeout kills it at 2 seconds → exit code 124.
  assert_eq!(
    code, 124,
    "watch loop must survive install errors and run until killed (expected 124, got {code})\nstderr: {err}"
  );
  // At least 2 log lines confirm the loop ran more than once.
  let line_count = err.lines().filter( | l | !l.trim().is_empty() ).count();
  assert!(
    line_count >= 2,
    "stderr must show at least 2 log lines, proving the loop continued past the first error\nstderr: {err}"
  );
}

// TC-416: version::latest dry::1 override → "no version pin to guard"
//
// Root Cause: The `version::` override parameter bypasses settings lookup (FR-21).
//   When the supplied spec resolves to `latest`, `guard_once_latest(dry=true)` must
//   be called and return "preferred = latest (no version pin to guard)".
// Why Not Caught: TC-403 tests settings-driven latest, but not the override path.
//   A regression in the override dispatch could silently skip `guard_once_latest`.
// Fix Applied: test exercises the explicit `version::latest` code path in isolation.
// Prevention: Any refactor of guard's override dispatch must keep this branch covered.
// Pitfall: Do not confuse with TC-403 (settings preference = latest); this test
//   uses version:: override and requires NO preference stored in settings.
#[ test ]
fn tc416_guard_version_latest_override_dry()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No settings written — override must not read from settings (FR-21).
  let out = run_clv_with_env(
    &[ ".version.guard", "version::latest", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no version pin to guard" ),
    "version::latest override with dry::1 must output 'no version pin to guard': {text}"
  );
}

// TC-417: dry::1 v::0 → output shorter than v::1
#[ test ]
fn tc417_guard_v0_shorter_than_v1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );  // no preference → defaults to stable

  let out0 = run_clv_with_env(
    &[ ".version.guard", "dry::1", "v::0" ],
    &[ ( "HOME", home ) ],
  );
  let out1 = run_clv_with_env(
    &[ ".version.guard", "dry::1", "v::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out0, 0 );
  assert_exit( &out1, 0 );
  let s0 = stdout( &out0 );
  let s1 = stdout( &out1 );
  assert!(
    s0.len() < s1.len(),
    "v::0 output ({} chars) must be shorter than v::1 ({} chars): v0={s0:?} v1={s1:?}",
    s0.len(), s1.len()
  );
}
