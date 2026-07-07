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
//! | 418 | `format::json` watch mode → raw JSON passthrough, no dot separator | P | 124 |
//! | 419 | `hot_swap_binary` trace fires when a live claude process is detected (task 313 T01) | P | 1 |
//! | 420 | `unlock_versions_dir` trace fires on install (task 313 T03) | P | 1 |
//! | 421 | `lock_version` trace fires via curated bash-only PATH (task 313 T04) | P | 0 |
//! | 422 | `perform_install` trace fires on install (task 313 T05) | P | 1 |
//! | 423 | `store_preferred_version` trace fires via idempotent-skip (task 313 T06) | P | 0 |

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

// TC-418: format::json watch mode → raw JSON passthrough, no dot-separated wrapper
//
// # Root Cause
// The ok-path branch discriminator in the watch loop compared the raw check
// output against the literal string "ok" to decide bare-vs-detailed text
// formatting. Under `format::json`, `check_installed_guard()` never emits
// that literal string — it emits a JSON object — so the comparison always
// fell through to the "detailed" branch and embedded the whole JSON blob
// inside the compact dot-separated line, corrupting it for JSON consumers.
//
// # Why Not Caught
// No test exercised `format::json` combined with watch mode (`interval::N>0`);
// the Test Matrix only covered format::text at both verbosity levels.
//
// # Fix Applied
// The ok-path branch now matches on `opts.format` first: `OutputFormat::Json`
// prints the check result verbatim (one JSON line per iteration); only
// `OutputFormat::Text` uses the bare-vs-detailed dot-separated wrapper.
//
// # Prevention
// Fakes an installed version via the `~/.local/bin/claude` symlink
// convention (`get_version_from_symlink`) so the guard reports a match
// ("ok") without any network/install activity, then confirms the stderr
// line is a well-formed JSON object with no ` · ` text-format separator.
//
// # Pitfall
// `format::text` (the default) must still show the compact dot-separated
// wrapper — this test only asserts on the `format::json` case; the text
// path is covered separately by TC-415/417 and IT-20.
#[ test ]
fn tc418_watch_mode_json_format_passthrough()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Fake an installed version via symlink so the guard hits the "match" (ok)
  // path deterministically, without any real install/network activity.
  let local_bin = dir.path().join( ".local" ).join( "bin" );
  std::fs::create_dir_all( &local_bin ).unwrap();
  std::fs::write( local_bin.join( "9.9.9" ), "" ).unwrap();
  std::os::unix::fs::symlink( "9.9.9", local_bin.join( "claude" ) ).unwrap();

  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let settings_json = r#"{"preferredVersionSpec":"9.9.9","preferredVersionResolved":"9.9.9"}"#;
  std::fs::write( claude_dir.join( "settings.json" ), settings_json ).unwrap();

  let bin = env!( "CARGO_BIN_EXE_claude_version" );
  let out = std::process::Command::new( "/usr/bin/timeout" )
    .args( [ "2", bin, ".version.guard", "interval::1", "format::json" ] )
    .env( "HOME", home )
    .output()
    .expect( "timeout command failed to launch" );

  let err = String::from_utf8_lossy( &out.stderr );
  let lines : Vec< &str > = err.lines().filter( | l | !l.trim().is_empty() ).collect();
  assert!( !lines.is_empty(), "expected at least one watch-mode log line; stderr: {err}" );

  for line in &lines
  {
    assert!(
      !line.contains( " · " ),
      "format::json watch-mode line must not contain the text-format dot separator: {line:?}"
    );
    assert!(
      line.starts_with( '{' ) && line.ends_with( '}' ),
      "format::json watch-mode line must be a raw JSON object, got: {line:?}"
    );
    assert!(
      line.contains( "\"status\":\"ok\"" ),
      "expected a matched (ok) status in the JSON passthrough line, got: {line:?}"
    );
  }
}

// ─── Task 313: parameter-trace subprocess-isolated tests (T01, T03-T06) ────

// TC-419 (task 313 T01): hot_swap_binary trace fires when a live claude
// process is detected.
//
// Combines process_isolation_test.rs's dummy-live-process technique
// (symlinked claude -> sleep, augmented spawn-time PATH) so
// find_claude_processes() detects a real live process via /proc, with a
// no-preference-set HOME so `.version.guard interval::0` finds nothing
// installed and proceeds to perform_install. The CLI subprocess itself runs
// with an empty PATH so the subsequent curl|bash spawn fails fast — no real
// network call — after hot_swap_binary's trace has already fired
// unconditionally.
#[ cfg( unix ) ]
#[ test ]
fn tc419_hot_swap_binary_traces_with_live_process()
{
  let tmp_bin     = TempDir::new().unwrap();
  let tmp_bin_dir = tmp_bin.path();
  let sleep_bin   = if std::path::Path::new( "/usr/bin/sleep" ).exists() { "/usr/bin/sleep" } else { "/bin/sleep" };
  std::os::unix::fs::symlink( sleep_bin, tmp_bin_dir.join( "claude" ) ).unwrap();

  let orig_path = std::env::var( "PATH" ).unwrap_or_default();
  let aug_path  = format!( "{}:{}", tmp_bin_dir.display(), orig_path );

  let mut dummy = std::process::Command::new( "claude" )
    .arg( "300" )
    .env( "PATH", &aug_path )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "failed to spawn dummy claude process" );

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.guard", "interval::0" ],
    &[ ( "HOME", home ), ( "PATH", "" ) ],
  );
  let err = stderr( &out );

  // Clean up dummy process before asserting so it never leaks on failure.
  let _ = dummy.kill();
  let _ = dummy.wait();

  assert!(
    err.contains( "hot_swap_binary" ),
    "stderr must contain hot_swap_binary trace line when a live claude process is detected: {err}"
  );
}

// TC-420 (task 313 T03): unlock_versions_dir trace fires on install.
//
// unlock_versions_dir() runs unconditionally near the top of
// perform_install(), before the curl|bash pipe. An empty PATH forces bash
// itself to be not-found, so perform_install fails fast with no real network
// call (same technique as TC-415), after the trace has already fired.
#[ test ]
fn tc420_unlock_versions_dir_traces_on_install()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.install", "version::stable" ],
    &[ ( "HOME", home ), ( "PATH", "" ) ],
  );
  let err = stderr( &out );
  assert!(
    err.contains( "unlock_versions_dir" ),
    "stderr must contain unlock_versions_dir trace line: {err}"
  );
}

// TC-421 (task 313 T04): lock_version trace fires via a curated bash-only
// PATH.
//
// A curated PATH containing only a real `bash` binary (no `curl`) lets the
// curl|bash pipe's second stage still exit 0 (POSIX last-command-wins,
// absent pipefail) even though `curl` itself is not-found — so
// perform_install's status.success() is true with zero real network
// contact, reaching lock_version safely.
#[ cfg( unix ) ]
#[ test ]
fn tc421_lock_version_traces_via_curated_path()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let real_bash     = if std::path::Path::new( "/usr/bin/bash" ).exists() { "/usr/bin/bash" } else { "/bin/bash" };
  let bash_only_dir = TempDir::new().unwrap();
  std::os::unix::fs::symlink( real_bash, bash_only_dir.path().join( "bash" ) ).unwrap();

  let out = run_clv_with_env(
    &[ ".version.install", "version::stable" ],
    &[ ( "HOME", home ), ( "PATH", bash_only_dir.path().to_str().unwrap() ) ],
  );
  let err = stderr( &out );
  assert!(
    err.contains( "lock_version" ),
    "stderr must contain lock_version trace line: {err}"
  );
}

// TC-422 (task 313 T05): perform_install trace fires on install.
//
// Empty PATH forces bash itself to be not-found, so perform_install fails
// fast with no real network call (same technique as TC-415), after the
// trace has already fired as the function's first statement.
#[ test ]
fn tc422_perform_install_traces_on_install()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clv_with_env(
    &[ ".version.install", "version::stable" ],
    &[ ( "HOME", home ), ( "PATH", "" ) ],
  );
  let err = stderr( &out );
  assert!(
    err.contains( "perform_install" ),
    "stderr must contain perform_install trace line: {err}"
  );
}

// TC-423 (task 313 T06): store_preferred_version trace fires via the
// idempotent-skip path.
//
// Reuses process_isolation_test.rs's deterministic version-symlink technique
// (lines 86-108) rather than tc358's ambient-system-state technique (which
// is non-deterministic): resolve stable's pinned semver at test time, write
// a real empty file named that version into an isolated HOME's
// `.local/bin/`, symlink claude to it. get_installed_version()'s symlink
// check then deterministically matches resolved, forcing the
// idempotent-skip branch (commands/version.rs:110), which calls
// store_preferred_version without ever entering perform_install — no real
// network call.
#[ cfg( unix ) ]
#[ test ]
fn tc423_store_preferred_version_traces_on_idempotent_skip()
{
  let stable_ver = claude_version_core::version::VERSION_ALIASES
    .iter()
    .find( | a | a.name == "stable" )
    .map( | a | a.value )
    .expect( "stable alias not found in VERSION_ALIASES" );

  let dir  = TempDir::new().unwrap();
  let home = dir.path();
  let local_bin = home.join( ".local" ).join( "bin" );
  std::fs::create_dir_all( &local_bin ).unwrap();
  std::fs::write( local_bin.join( stable_ver ), "" ).unwrap();
  std::os::unix::fs::symlink( stable_ver, local_bin.join( "claude" ) ).unwrap();
  write_settings( home, &[] );

  let out = run_clv_with_env(
    &[ ".version.install", "version::stable" ],
    &[ ( "HOME", home.to_str().unwrap() ), ( "PATH", "" ) ],
  );
  let err = stderr( &out );
  assert!(
    err.contains( "store_preferred_version" ),
    "stderr must contain store_preferred_version trace line via the idempotent-skip path: {err}"
  );
}
