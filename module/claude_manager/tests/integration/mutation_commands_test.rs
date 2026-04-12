//! Integration tests for 4 mutation commands.
//!
//! # Test Matrix
//!
//! ## E5: `version install`
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 300 | `dry::1` → `[dry-run]` prefix, exit 0 | P | 0 |
//! | 301 | `version::stable dry::1` → preview shows `stable` | P | 0 |
//! | 302 | `version::1.2.3 dry::1` → preview shows exact version | P | 0 |
//! | 303 | `dry::1 force::1` → dry wins | P | 0 |
//! | 304 | `version::STABLE` → wrong case, exit 1 | N | 1 |
//! | 305 | `version::""` (empty) → exit 1 | N | 1 |
//! | 306 | `version::1.2` → two-part semver rejected | N | 1 |
//! | 307 | `version::x` → unknown alias, exit 1 | N | 1 |
//! | 308 | absent `version::` with `dry::1` → uses `stable` | P | 0 |
//! | 309 | `version::month dry::1` → resolves to pinned semver | P | 0 |
//! | 350 | `version::latest dry::1` → `autoUpdates = true` in preview | P | 0 |
//! | 351 | `version::stable dry::1` → `autoUpdates = false` in preview | P | 0 |
//! | 352 | `version::2.1.50 dry::1` → `autoUpdates = false` in preview | P | 0 |
//! | 353 | `version::latest dry::1` → previews unlock actions | P | 0 |
//! | 354 | `version::01.02.03` → leading zeros rejected | N | 1 |
//! | 355 | `version::0.0.0 dry::1` → single-zero parts valid | P | 0 |
//! | 356 | `dry::1` output mentions preferred version storage | P | 0 |
//! | 357 | `dry::1` does NOT write preference keys to settings | P | 0 |
//! | 358 | idempotent skip still stores preference | P | 0 |
//! | 359 | `version::stable dry::1` → output includes Layer 4 purge line | P | 0 |
//! | 360 | `version::latest dry::1` → output does NOT contain "purge" | P | 0 |
//! | 361 | `dry::1 format::json` → JSON output, exit 0 | P | 0 |
//! | 362 | `format::JSON` (uppercase) → exit 1 | N | 1 |
//!
//! ## E14: `version guard`
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
//!
//! ## E7: `processes kill`
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 310 | no processes → `no active processes`, exit 0 | P | 0 |
//! | 311 | `dry::1` no processes → `no active processes` | P | 0 |
//! | 312 | `dry::1 force::1` no processes → `no active processes` | P | 0 |
//! | 313 | `v::0` → accepted, exit 0 | P | 0 |
//! | 314 | `format::JSON` (uppercase) → exit 1 | N | 1 |
//! | 315 | `let _ = send_sigterm/sigkill` removed — errors now propagated | verify | — |
//! | 316 | `dry::1 format::json` → JSON output, exit 0 | P | 0 |
//!
//! ## E10: `settings set`
//!
//! | TC  | Description | P/N | Exit |
//! |-----|-------------|-----|------|
//! | 320 | no `key::` → exit 1 | N | 1 |
//! | 321 | `key::` but no `value::` → exit 1 | N | 1 |
//! | 322 | `value::true` → stores boolean `true` | P | 0 |
//! | 323 | `value::false` → stores boolean `false` | P | 0 |
//! | 324 | `value::0` → stores number `0` (NOT boolean) | P | 0 |
//! | 325 | `value::42` → stores number `42` | P | 0 |
//! | 326 | `value::hello` → stores quoted `"hello"` | P | 0 |
//! | 327 | `value::` (empty) → exit 1, error mentions "value" | N | 1 |
//! | 328 | creates file when absent | P | 0 |
//! | 329 | updates existing key (no duplication) | P | 0 |
//! | 330 | `dry::1` → shows preview, no file change | P | 0 |
//! | 331 | HOME not set → exit 2 | N | 2 |
//! | 332 | `key::""` (empty key) → exit 1 | N | 1 |
//! | 333 | adds new key to existing file | P | 0 |
//! | 334 | `dry::1` + `value::` empty → exit 1 (validation before dry-run) | N | 1 |
//!
//!
//! # Lessons Learned
//!
//! - **`/proc` is global state**: `find_claude_processes()` scans the real `/proc`
//!   regardless of subprocess environment. Tests for `processes kill` cannot assume
//!   zero processes — they must handle both "no processes" and "processes exist" paths.
//!   Setting `PATH=""` only hides the `claude` binary from subprocess, not from `/proc`.
//!
//! - **`write_settings()` helper writes all values as quoted strings** (e.g., `"true"`
//!   not `true`). Tests that verify JSON type preservation after `settings set` must
//!   re-read the actual file written by the command, not the helper's output.

use tempfile::TempDir;

use crate::helpers::{ assert_exit, run_clm, run_clm_with_env, stderr, stdout, write_settings };

// ─── E5: version install ─────────────────────────────────────────────────────

// TC-300: dry::1 → [dry-run] prefix, exit 0
#[ test ]
fn tc300_version_install_dry_shows_prefix()
{
  let out = run_clm( &[ ".version.install", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "must contain [dry-run]: {text}" );
}

// TC-301: version::stable dry::1 → preview shows stable
#[ test ]
fn tc301_version_install_dry_stable()
{
  let out = run_clm( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "stable" ), "must contain stable: {text}" );
}

// TC-302: version::1.2.3 dry::1 → shows exact version
#[ test ]
fn tc302_version_install_dry_exact_semver()
{
  let out = run_clm( &[ ".version.install", "version::1.2.3", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "1.2.3" ), "must contain 1.2.3: {text}" );
}

// TC-303: dry::1 force::1 → dry wins
#[ test ]
fn tc303_version_install_dry_wins_over_force()
{
  let out = run_clm( &[ ".version.install", "dry::1", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry must win over force: {text}" );
}

// TC-304: version::STABLE → wrong case, exit 1
#[ test ]
fn tc304_version_install_wrong_case_exits_1()
{
  let out = run_clm( &[ ".version.install", "version::STABLE" ] );
  assert_exit( &out, 1 );
}

// TC-305: version::"" (empty) → exit 1
#[ test ]
fn tc305_version_install_empty_version_exits_1()
{
  let out = run_clm( &[ ".version.install", "version::" ] );
  assert_exit( &out, 1 );
}

// TC-306: version::1.2 → two-part semver rejected
#[ test ]
fn tc306_version_install_two_part_semver_exits_1()
{
  let out = run_clm( &[ ".version.install", "version::1.2" ] );
  assert_exit( &out, 1 );
}

// TC-307: version::x → unknown alias, exit 1
#[ test ]
fn tc307_version_install_unknown_alias_exits_1()
{
  let out = run_clm( &[ ".version.install", "version::x" ] );
  assert_exit( &out, 1 );
}

// TC-308: absent version:: with dry::1 → defaults to stable
#[ test ]
fn tc308_version_install_absent_version_defaults_to_stable()
{
  let out = run_clm( &[ ".version.install", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "stable" ), "default version must be stable: {text}" );
}

// TC-309: version::month dry::1 → resolves to pinned semver
#[ test ]
fn tc309_version_install_dry_month()
{
  let out = run_clm( &[ ".version.install", "version::month", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "month" ), "must contain alias name 'month': {text}" );
  assert!( text.contains( "2.1.74" ), "must contain resolved version 2.1.74: {text}" );
}

// TC-350: version::latest dry::1 → autoUpdates = true in preview
#[ test ]
fn tc350_version_install_dry_latest_auto_updates_true()
{
  let out = run_clm( &[ ".version.install", "version::latest", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "autoUpdates = true" ), "latest must preview autoUpdates = true: {text}" );
}

// TC-351: version::stable dry::1 → autoUpdates = false in preview
#[ test ]
fn tc351_version_install_dry_stable_auto_updates_false()
{
  let out = run_clm( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "autoUpdates = false" ), "stable must preview autoUpdates = false: {text}" );
  assert!( text.contains( "DISABLE_AUTOUPDATER = 1" ), "stable must preview DISABLE_AUTOUPDATER: {text}" );
  assert!( text.contains( "chmod 555" ), "stable must preview chmod 555: {text}" );
}

// TC-352: version::2.1.50 dry::1 → autoUpdates = false in preview
#[ test ]
fn tc352_version_install_dry_semver_auto_updates_false()
{
  let out = run_clm( &[ ".version.install", "version::2.1.50", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "autoUpdates = false" ), "semver must preview autoUpdates = false: {text}" );
  assert!( text.contains( "DISABLE_AUTOUPDATER = 1" ), "semver must preview DISABLE_AUTOUPDATER: {text}" );
}

// TC-353: version::latest dry::1 → previews unlock actions
#[ test ]
fn tc353_version_install_dry_latest_shows_unlock()
{
  let out = run_clm( &[ ".version.install", "version::latest", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "remove env.DISABLE_AUTOUPDATER" ), "latest must show remove: {text}" );
  assert!( text.contains( "unlocked" ), "latest must show unlocked: {text}" );
}

// TC-359: version::stable dry::1 → output includes Layer 4 purge line
#[ test ]
fn tc359_version_install_dry_stable_includes_purge_line()
{
  let out = run_clm( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "purge stale cached binaries" ),
    "pinned install must preview Layer 4 purge: {text}"
  );
}

// TC-360: version::latest dry::1 → output does NOT contain purge (latest skips Layer 4)
#[ test ]
fn tc360_version_install_dry_latest_no_purge_line()
{
  let out = run_clm( &[ ".version.install", "version::latest", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "purge" ),
    "latest install must NOT mention purge (Layer 4 is pinned-only): {text}"
  );
}

// TC-361: dry::1 format::json → JSON output, exit 0
#[ test ]
fn tc361_version_install_dry_format_json()
{
  let out = run_clm( &[ ".version.install", "dry::1", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '{' ),
    "format::json dry-run must produce JSON object: {text}"
  );
}

// TC-362: format::JSON (uppercase) → exit 1
#[ test ]
fn tc362_version_install_format_uppercase_rejected()
{
  let out = run_clm( &[ ".version.install", "format::JSON" ] );
  assert_exit( &out, 1 );
}

// ─── E7: processes kill ───────────────────────────────────────────────────────

// TC-310: .processes.kill dry::1 exits 0 — shows [dry-run] or "no active processes"
#[ test ]
fn tc310_processes_kill_dry_exits_0()
{
  let out = run_clm( &[ ".processes.kill", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no active processes" ) || text.contains( "[dry-run]" ),
    "must be dry-run preview or no processes: {text}"
  );
}

// TC-311: .processes.kill dry::1 → preview mentions SIGTERM
#[ test ]
fn tc311_processes_kill_dry_mentions_sigterm()
{
  let out = run_clm( &[ ".processes.kill", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  if !text.contains( "no active processes" )
  {
    assert!( text.contains( "SIGTERM" ), "dry-run must mention SIGTERM: {text}" );
  }
}

// TC-312: .processes.kill dry::1 force::1 → dry wins, mentions SIGKILL
#[ test ]
fn tc312_processes_kill_dry_force_mentions_sigkill()
{
  let out = run_clm( &[ ".processes.kill", "dry::1", "force::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  if !text.contains( "no active processes" )
  {
    assert!( text.contains( "SIGKILL" ), "dry+force must mention SIGKILL: {text}" );
  }
}

// TC-315: verify signal errors are no longer silently swallowed
//
// Root Cause: `let _ = send_sigterm(p.pid)` and `let _ = send_sigkill(p.pid)`
//   discarded all signal delivery errors, making exit code 2 unreachable when a
//   signal failed for any reason other than "process survived" (caught by the
//   trailing `remaining > 0` check).
// Why Not Caught: no test exercised the signal-error path — triggering it
//   requires a process that exists in the Claude process list but rejects signals,
//   which is not reproducible in a clean test environment without injection.
// Fix Applied: `let _` replaced with proper Result collection; Err is returned
//   immediately if any signal delivery fails.
// Prevention: AF check below verifies the `let _` pattern is absent at source level.
// Pitfall: `find_claude_processes()` reads real `/proc`; tests cannot inject fake
//   PIDs into the process list, so the new error path is verified via code inspection
//   only. Functional regression is covered by TC-310–312 (happy paths still work).
#[ test ]
fn tc315_processes_kill_no_let_underscore_on_send_sig()
{
  // Verify at source level that `let _ = send_sig` is absent from commands.rs.
  // This is an AF (anti-faking) check — the only reliable test for a code path
  // that cannot be triggered through the binary without process injection.
  let src = std::fs::read_to_string( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/commands.rs" ) )
    .expect( "could not read commands.rs for AF check" );
  assert!(
    !src.contains( "let _ = send_sigterm" ),
    "let _ = send_sigterm must be absent — signal errors must be propagated",
  );
  assert!(
    !src.contains( "let _ = send_sigkill" ),
    "let _ = send_sigkill must be absent — signal errors must be propagated",
  );
}

// TC-313: v::0 → accepted, exit 0
#[ test ]
fn tc313_processes_kill_v0_accepted()
{
  let out = run_clm( &[ ".processes.kill", "v::0" ] );
  assert_exit( &out, 0 );
}

// TC-314: format::JSON (uppercase) → exit 1
#[ test ]
fn tc314_processes_kill_format_uppercase_rejected()
{
  let out = run_clm( &[ ".processes.kill", "format::JSON" ] );
  assert_exit( &out, 1 );
}

// TC-316: dry::1 format::json → JSON output, exit 0
#[ test ]
fn tc316_processes_kill_dry_format_json()
{
  let out = run_clm( &[ ".processes.kill", "dry::1", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '{' ),
    "format::json must produce JSON object: {text}"
  );
}

// ─── E10: settings set ───────────────────────────────────────────────────────

// TC-320: no key:: → exit 1
#[ test ]
fn tc320_settings_set_missing_key_exits_1()
{
  let out = run_clm( &[ ".settings.set" ] );
  assert_exit( &out, 1 );
}

// TC-321: key:: but no value:: → exit 1
#[ test ]
fn tc321_settings_set_missing_value_exits_1()
{
  let out = run_clm( &[ ".settings.set", "key::foo" ] );
  assert_exit( &out, 1 );
}

// TC-322: value::true → stores boolean true
#[ test ]
fn tc322_settings_set_stores_boolean_true()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::myBool", "value::true" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"myBool\": true" ), "must store bare true: {content}" );
  assert!( !content.contains( "\"myBool\": \"true\"" ), "must NOT quote true: {content}" );
}

// TC-323: value::false → stores boolean false
#[ test ]
fn tc323_settings_set_stores_boolean_false()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::myBool", "value::false" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"myBool\": false" ), "must store bare false: {content}" );
}

// TC-324: value::0 → stores number 0 (NOT boolean false)
#[ test ]
fn tc324_settings_set_zero_stored_as_number()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::testkey", "value::0" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"testkey\": 0" ), "0 must be stored as number: {content}" );
  assert!( !content.contains( "false" ), "0 must NOT be stored as false: {content}" );
}

// TC-325: value::42 → stores number 42
#[ test ]
fn tc325_settings_set_stores_number()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::num", "value::42" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"num\": 42" ), "must store bare 42: {content}" );
}

// TC-326: value::hello → stores quoted "hello"
#[ test ]
fn tc326_settings_set_stores_string()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::str", "value::hello" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"str\": \"hello\"" ), "must store quoted hello: {content}" );
}

// TC-327: value:: (empty) → exit 1, error must mention "value"
//
// ## Root Cause
//
// `settings_set_routine` used `require_string_arg` (which allows empty strings)
// for the `value::` parameter instead of `require_nonempty_string_arg`.  The
// FR-04 "empty value → exit 1" rule was silently bypassed: `value::` wrote `""`
// into settings.json and exited 0.
//
// ## Why Not Caught
//
// TC-327 was originally written as a POSITIVE test ("stores empty string `""`"),
// which codified the buggy behavior.  No test verified that empty `value::` is
// rejected.
//
// ## Fix Applied
//
// Changed `require_string_arg` to `require_nonempty_string_arg` for `value::` in
// `settings_set_routine`, and removed the now-unused `require_string_arg` helper.
//
// ## Prevention
//
// This TC-327 now locks down that `value::` (empty) is rejected with exit 1 and
// an error message that mentions the parameter name.  TC-334 covers the dry::1
// case to ensure validation precedes the dry-run short-circuit.
//
// ## Pitfall
//
// Without this guard, `cm .settings.set key::k value::` appears to succeed but
// writes a meaningless `""` entry — indistinguishable from "not set" via
// `.settings.get`, silently masking the user typo.
#[ test ]
fn tc327_settings_set_empty_value_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::empty", "value::" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "value" ), "error must mention 'value': {err}" );
  // File must NOT be created — no side effects on error
  assert!(
    !dir.path().join( ".claude/settings.json" ).exists(),
    "settings.json must not be created on empty-value rejection"
  );
}

// TC-328: creates file when absent
#[ test ]
fn tc328_settings_set_creates_file_when_absent()
{
  let dir = TempDir::new().unwrap();
  let settings_path = dir.path().join( ".claude/settings.json" );
  assert!( !settings_path.exists(), "precondition: settings file must not exist" );
  let out = run_clm_with_env(
    &[ ".settings.set", "key::newkey", "value::newval" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  assert!( settings_path.exists(), "settings file must be created" );
}

// TC-329: updates existing key (no duplication)
#[ test ]
fn tc329_settings_set_updates_existing_key()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "mykey", "old" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.set", "key::mykey", "value::new" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"mykey\": \"new\"" ), "must update to new value: {content}" );
  assert_eq!(
    content.matches( "mykey" ).count(), 1,
    "key must appear exactly once (no duplication): {content}"
  );
}

// TC-330: dry::1 → shows preview, no file change
#[ test ]
fn tc330_settings_set_dry_shows_preview_no_write()
{
  let dir = TempDir::new().unwrap();
  let settings_path = dir.path().join( ".claude/settings.json" );
  let out = run_clm_with_env(
    &[ ".settings.set", "key::k", "value::v", "dry::1" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "must contain [dry-run]: {text}" );
  assert!( !settings_path.exists(), "dry-run must not create file" );
}

// TC-331: HOME not set → exit 2
#[ test ]
fn tc331_settings_set_no_home_exits_2()
{
  let out = run_clm_with_env(
    &[ ".settings.set", "key::k", "value::v" ],
    &[ ( "HOME", "" ) ],
  );
  assert_exit( &out, 2 );
}

// TC-332: key::"" (empty key) → exit 1
#[ test ]
fn tc332_settings_set_empty_key_exits_1()
{
  let out = run_clm( &[ ".settings.set", "key::", "value::v" ] );
  assert_exit( &out, 1 );
}

// TC-333: adds new key to existing file preserving existing keys
#[ test ]
fn tc333_settings_set_adds_new_key_preserves_existing()
{
  let dir = TempDir::new().unwrap();
  write_settings( dir.path(), &[ ( "existing", "val" ) ] );
  let out = run_clm_with_env(
    &[ ".settings.set", "key::added", "value::new" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "existing" ), "must preserve existing key: {content}" );
  assert!( content.contains( "added" ), "must contain new key: {content}" );
}

// TC-334: dry::1 + value:: empty → exit 1 (validation before dry-run)
//
// Ensures that the empty-value check (FR-04) is evaluated BEFORE the dry-run
// short-circuit inside `settings_set_routine`.  Without the fix, the dry-run
// branch was reached first and printed "[dry-run] would set k =  (string)"
// with exit 0 — making the user believe the command was valid.
#[ test ]
fn tc334_settings_set_empty_value_with_dry_still_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_clm_with_env(
    &[ ".settings.set", "key::k", "value::", "dry::1" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "value" ), "error must mention 'value': {err}" );
  // No file created, no dry-run output — validation fires before dry-run
  assert!(
    !dir.path().join( ".claude/settings.json" ).exists(),
    "settings.json must not be created on empty-value rejection"
  );
  assert!(
    !crate::helpers::stdout( &out ).contains( "[dry-run]" ),
    "dry-run output must not appear when value:: is empty"
  );
}

// TC-354: version::01.02.03 → leading zeros rejected, exit 1
//
// test_kind: bug_reproducer(leading-zeros)
//
// Root Cause: validate_version_spec only checked that semver parts were non-empty
// ASCII digits, but did not reject leading zeros.  "01" is not valid semver.
//
// Why Not Caught: All existing tests used single-digit or proper multi-digit parts.
//
// Fix Applied: Added `p.len() == 1 || !p.starts_with('0')` check to validation.
//
// Prevention: Include leading-zero variants in version-spec negative test matrix.
//
// Pitfall: The installer accepts and attempts to download leading-zero versions,
// then fails with 404.  By that time, hot_swap_binary has already deleted the
// old binary, leaving the user without any installed version.
#[ test ]
fn tc354_version_install_leading_zeros_exits_1()
{
  let out = run_clm( &[ ".version.install", "version::01.02.03" ] );
  assert_exit( &out, 1 );
  let text = stderr( &out );
  assert!( text.contains( "unknown version" ), "must reject leading zeros: {text}" );
}

// TC-355: version::0.0.0 → valid (single-digit zero parts are fine)
#[ test ]
fn tc355_version_install_zero_parts_valid_dry()
{
  let out = run_clm( &[ ".version.install", "version::0.0.0", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "0.0.0" ), "single-zero parts are valid semver: {text}" );
}

// ─── E5 continued: install preference output ────────────────────────────────

// TC-356: dry::1 output mentions preferred version storage
#[ test ]
fn tc356_version_install_dry_mentions_preferred()
{
  let out = run_clm( &[ ".version.install", "version::stable", "dry::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "preferred version" ),
    "dry-run must mention preferred version storage: {text}"
  );
}

// TC-357: dry::1 does NOT write preference keys to settings
#[ test ]
fn tc357_version_install_dry_no_preference_written()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clm_with_env(
    &[ ".version.install", "version::stable", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // Read settings file and verify no preference keys were written.
  let content = std::fs::read_to_string( dir.path().join( ".claude/settings.json" ) ).unwrap();
  assert!(
    !content.contains( "preferredVersionSpec" ),
    "dry-run must not write preference keys: {content}"
  );
}

// TC-358: idempotent skip ("already at") still stores preference
//
// ## Root Cause
// version_install_handler() returned early at the idempotent guard
// BEFORE calling store_preferred_version(). After a no-op install,
// `version guard` found no preference keys and reported "no preferred version
// set" even though the user explicitly ran `version install`.
//
// ## Why Not Caught Initially
// All install tests used `dry::1` or invalid PATH. No test exercised the
// idempotent early-return path with a writable HOME that could verify settings.
//
// ## Fix Applied
// Store preference in the idempotent early-return path, not only after install.
//
// ## Prevention
// Every successful exit path in install must store the preference.
//
// ## Pitfall
// Early returns that skip post-action bookkeeping silently break downstream
// commands that depend on that bookkeeping.
// test_kind: bug_reproducer(issue-358)
#[ test ]
fn tc358_version_install_idempotent_stores_preference()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  // Run version install with real PATH. If the installed claude matches
  // stable, the idempotent guard fires ("already at ...") and we verify
  // that preference keys were written. If versions differ, install is
  // attempted — it may fail (network), but the test still checks that the
  // idempotent path stores preference when it fires.
  let out = run_clm_with_env(
    &[ ".version.install", "version::stable" ],
    &[ ( "HOME", home ) ],
  );
  let text = stdout( &out );

  // Only assert on the idempotent path — if versions don't match, install
  // runs and may fail; that's not what this test checks.
  if text.contains( "already at" )
  {
    let content = std::fs::read_to_string( dir.path().join( ".claude/settings.json" ) ).unwrap();
    assert!(
      content.contains( "preferredVersionSpec" ),
      "idempotent skip must still store preference: {content}"
    );
  }
}

// ─── E14: version guard ─────────────────────────────────────────────────────

// TC-400: no preference set → defaults to stable, exit 0
#[ test ]
fn tc400_guard_no_preference()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clm_with_env(
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

  let out = run_clm_with_env(
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
  let out = run_clm_with_env(
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

  let out = run_clm_with_env(
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

  let out = run_clm_with_env(
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
// ## Root Cause
//
// After an alias bump (e.g. month 2.1.50 → 2.1.74) the stored
// `preferredVersionResolved` in settings.json becomes stale.
// `guard_once()` must re-resolve the alias through the current table
// rather than blindly trusting the stored resolved value.
//
// ## Why Not Caught
//
// Previous tests never wrote a stale resolved value that diverged
// from the compile-time alias table.
//
// ## Fix Applied
//
// `guard_once()` calls `resolve_version_spec()` on `spec` before
// using it, so the current alias value always wins.
//
// ## Prevention
//
// This test explicitly writes a stale resolved value and asserts the
// output uses the current alias mapping.
//
// ## Pitfall
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

  let out = run_clm_with_env(
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

  let out = run_clm_with_env(
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

  let out = run_clm_with_env(
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
  let out = run_clm( &[ ".version.guard", "bogus::x" ] );
  assert_exit( &out, 1 );
}

// TC-413: version::9.9.9 force::1 dry::1 → dry wins, override shown in output
#[ test ]
fn tc413_guard_version_override_dry_wins_over_force()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[] );

  let out = run_clm_with_env(
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
  let out = run_clm( &[ ".version.guard", "version::" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "version" ),
    "stderr must mention 'version' for empty version:: error: {err}"
  );
}

// TC-415: watch loop continues after install error  —  bug_reproducer(issue-415)
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
// Uses `timeout 2 cm .version.guard interval::1` with empty PATH to force
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

  let bin = env!( "CARGO_BIN_EXE_claude_manager" );
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
  // Second iteration header confirms the loop ran more than once.
  assert!(
    err.contains( "#2" ),
    "stderr must show iteration #2, proving the loop continued past the first error\nstderr: {err}"
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
  let out = run_clm_with_env(
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

  let out0 = run_clm_with_env(
    &[ ".version.guard", "dry::1", "v::0" ],
    &[ ( "HOME", home ) ],
  );
  let out1 = run_clm_with_env(
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
