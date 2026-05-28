//! Account unit tests: save and delete operations.
//!
//! ## Purpose
//!
//! Verify `account::save()` writes `_active` = `name` on every successful save,
//! and that `account::delete()` removes all three files created by `save()`:
//! `{name}.credentials.json`, `{name}.claude.json`, and `{name}.settings.json`.
//!
//! ## Fix Documentation — issue-snapshot-orphan
//!
//! - **Root Cause:** `save()` creates 3 files but `delete()` only removed `.credentials.json`,
//!   leaving `.claude.json` and `.settings.json` as orphans after every deletion.
//! - **Why Not Caught:** No test verified that snapshot files are absent after `delete()`; the
//!   orphan files accumulated silently over every `save` / `delete` call pair.
//! - **Fix Applied:** After the mandatory `remove_file(credentials)`, two best-effort
//!   `let _ = remove_file(...)` calls clean up `.claude.json` and `.settings.json`.
//! - **Prevention:** `ad_delete_also_removes_snapshots` asserts all 3 files absent post-delete.
//! - **Pitfall:** Snapshot removal must be best-effort (`let _ = ...`) — accounts saved before
//!   snapshot support was added have no snapshot files; a strict `remove_file` would fail them.
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `as_save_writes_active_marker` | save() with update_marker=true → `_active` written |
//! | `test_mre_bug211_save_false_leaves_marker_unchanged` | save() with update_marker=false → `_active` not written |
//! | `ad_delete_also_removes_snapshots` | All 3 files exist → all 3 absent after delete |
//! | `ad_delete_succeeds_when_snapshots_absent` | Only credentials → delete succeeds, no error |

use tempfile::TempDir;
use claude_profile_core::account;
use claude_core::ClaudePaths;

// ── helpers ───────────────────────────────────────────────────────────────────

fn write_credentials_file( store : &std::path::Path, name : &str )
{
  std::fs::write(
    store.join( format!( "{name}.credentials.json" ) ),
    r#"{"accessToken":"tok","expiresAt":9999999999999,"subscriptionType":"pro"}"#,
  ).unwrap();
}

fn write_active( store : &std::path::Path, active_name : &str )
{
  std::fs::write( store.join( "_active" ), active_name ).unwrap();
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[ test ]
fn ad_delete_also_removes_snapshots()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Active account is different — allows deletion of old@archive.com
  write_active( store, "work@acme.com" );
  write_credentials_file( store, "old@archive.com" );
  std::fs::write( store.join( "old@archive.com.claude.json" ),    r#"{"emailAddress":"old@archive.com"}"# ).unwrap();
  std::fs::write( store.join( "old@archive.com.settings.json" ),  "{}" ).unwrap();

  let result = account::delete( "old@archive.com", store );
  assert!( result.is_ok(), "delete must succeed when all 3 files exist: {result:?}" );

  assert!(
    !store.join( "old@archive.com.credentials.json" ).exists(),
    "credentials file must be absent after delete",
  );
  assert!(
    !store.join( "old@archive.com.claude.json" ).exists(),
    "claude.json snapshot must be absent after delete",
  );
  assert!(
    !store.join( "old@archive.com.settings.json" ).exists(),
    "settings.json snapshot must be absent after delete",
  );
}

#[ test ]
fn ad_delete_succeeds_when_snapshots_absent()
{
  // Guard: accounts saved before snapshot support have no .claude.json / .settings.json;
  // delete() must still succeed.
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  write_active( store, "work@acme.com" );
  write_credentials_file( store, "old@archive.com" );
  // No snapshot files — pre-snapshot-support account

  let result = account::delete( "old@archive.com", store );
  assert!(
    result.is_ok(),
    "delete must succeed when snapshot files were never created: {result:?}",
  );
  assert!(
    !store.join( "old@archive.com.credentials.json" ).exists(),
    "credentials file must be absent after delete",
  );
}

// ── AS: Account Save ──────────────────────────────────────────────────────────

/// BUG-211 MRE: `save()` with `update_marker=false` must NOT write the `_active` marker file.
///
/// # Root Cause
/// `save()` unconditionally wrote `_active` on every call, including background refresh
/// calls from `refresh_account_token`. Each per-account refresh clobbered `_active` with
/// the refreshed account's name, and the subsequent `switch_account` restore in
/// `apply_refresh`/`apply_touch` then overwrote any concurrent `.account.use` switch.
/// See `bug/211_apply_refresh_touch_restore_clobbers_active_marker_race.md`.
///
/// # Why Not Caught
/// `save()` had no mechanism to suppress the `_active` write; background callers had no
/// opt-out. The TOCTOU race window is ~35s (subprocess timeout), making it rare in unit
/// tests that run serially. Only a two-session command chain revealed the symptom.
///
/// # Fix Applied
/// Added `update_marker: bool` as the 4th parameter to `save()`. The `_active` write is
/// guarded by `if update_marker { ... }`. CLI callers (`.account.save`, `.account.relogin`)
/// pass `true`; `refresh_account_token` passes `false`.
///
/// # Prevention
/// This test is a compile-gate in Phase 1 (wrong arity → compile error) and a runtime
/// guard in Phase 2+ (marker absent when `update_marker=false`). Regressions that remove
/// the guard will fail this test.
///
/// # Pitfall
/// `update_marker=false` must only be used from background/internal callers. Any user-facing
/// path that omits the write leaves `.credentials.status` showing `Account: N/A` until the
/// next explicit `.account.save` or `.account.use`.
// test_kind: bug_reproducer(BUG-211)
#[ test ]
fn test_mre_bug211_save_false_leaves_marker_unchanged()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );

  account::save( "alice@test.com", &store, &paths, false ).unwrap();

  let marker = store.join( account::active_marker_filename() );
  assert!(
    !marker.exists(),
    "save() with update_marker=false must NOT write the _active marker file; found: {marker:?}",
  );
}

#[ test ]
fn as_save_writes_active_marker()
{
  // Confirm that save() writes _active = name so credentials_status can
  // display the account without a separate switch call.
  //
  // Fix(issue-active-marker): Root cause was save() never writing _active.
  // Prevention: this test will catch any regression that drops the write.
  // Pitfall: use ClaudePaths::with_home() — not set_var("HOME") — to avoid
  // mutating the process environment across parallel nextest processes.
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  // credentials_file must exist for the copy inside save() to succeed.
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );

  account::save( "alice@acme.com", &store, &paths, true ).unwrap();

  let marker_name = account::active_marker_filename();
  let active = std::fs::read_to_string( store.join( &marker_name ) )
    .expect( "_active must exist after save()" );
  assert_eq!(
    active.trim(),
    "alice@acme.com",
    "_active must contain the saved account name",
  );
}
