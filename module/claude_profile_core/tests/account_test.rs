//! Account deletion unit tests.
//!
//! ## Purpose
//!
//! Verify `account::delete()` removes all three files created by `save()`:
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
//! | `ad_delete_also_removes_snapshots` | All 3 files exist → all 3 absent after delete |
//! | `ad_delete_succeeds_when_snapshots_absent` | Only credentials → delete succeeds, no error |

use tempfile::TempDir;
use claude_profile_core::account;

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
