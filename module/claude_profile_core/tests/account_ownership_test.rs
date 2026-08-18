//! Ownership tests: owner/claim fields, per-machine `_active_*` markers, and
//! `other_machines_active()` (Feature 025/036).
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `test_ft11_025_other_machines_active_returns_others` | other_machines_active() returns foreign accounts; own marker excluded |
//! | `test_ft12_025_other_machines_active_empty_when_only_own` | other_machines_active() returns empty when only own marker or empty store |
//! | `ft01_save_captures_owner` | save() with owner:Some("user@host1") writes owner to {name}.json; re-save overwrites |
//! | `ft02_unclaim_clears_owner` | save() with owner:Some("") writes empty owner; is_owned() returns true |
//! | `ft11_no_owner_field_backward_compat` | absent owner key → read_owner() returns ""; is_owned() returns true |
//! | `ft14_background_save_preserves_owner` | save() with owner:None preserves existing owner field (background path) |
//! | `ec1_unclaim_writes_empty_owner` | unclaim::1 writes owner:"" on freshly saved account |
//! | `ec2_unclaim_overwrites_existing_owner` | unclaim::1 overwrites existing non-empty owner |
//! | `ec3_default_sets_owner_to_current_identity` | default save writes current_identity() as owner |
//! | `ec4_unclaim_preserves_other_fields` | unclaim::1 clears owner only; host/role preserved via read-merge |
//! | `ec5_unclaim_dry_run_no_write` | dry-run: without save() call, existing owner is unchanged |
//! | `cc1_read_owner_missing_file` | read_owner on missing {name}.json → "" (safe fallback) |
//! | `cc2_read_owner_empty_file` | read_owner on empty file → "" |
//! | `cc3_read_owner_corrupt_content` | read_owner on non-JSON content → "" |
//! | `cc4_read_owner_null_value` | read_owner with "owner": null → "" |
//! | `cc5_read_owner_numeric_value` | read_owner with "owner": 42 → "" |
//! | `cc6_background_save_new_account_no_owner` | background save on new account (owner:None) → no owner field |

use tempfile::TempDir;
use claude_profile_core::account;
use claude_core::ClaudePaths;

// BUG-347 task/bug/347_orphaned_marker_after_cross_machine_delete.md — this test's dual-HOSTNAME
// simulation pattern is the prior art a cross-machine `delete()` regression test should follow.
/// FT-11/025 — `other_machines_active()` returns other machines' account names,
/// excludes own marker.
///
/// ## Root Cause (AC-05 coverage)
/// `other_machines_active()` filters by `starts_with("_active_")` then excludes
/// the file whose name equals `active_marker_filename()`. Without this test, a
/// refactor removing the exclusion filter would silently include the own marker.
///
/// ## Setup
/// `TempDir` with own marker + 2 foreign markers. Foreign names are hard-coded to
/// `_active_machine2_user1` and `_active_machine3_user2` — guaranteed to differ
/// from `active_marker_filename()` on any real machine (those strings would require
/// `$HOSTNAME=machine2` + `$USER=user1` or `$HOSTNAME=machine3` + `$USER=user2`).
///
/// ## Assert
/// Set size = 2; contains "alice@test.com" and "bob@test.com"; does NOT contain
/// "own@test.com".
///
/// Spec: [`tests/docs/feature/025_per_machine_active_marker.md` FT-11]
#[ test ]
fn test_ft11_025_other_machines_active_returns_others()
{
  use std::collections::HashSet;

  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Own machine's marker — excluded by the function under test
  let own_name = account::active_marker_filename();
  std::fs::write( store.join( &own_name ), "own@test.com" ).unwrap();

  // Two foreign markers with names that cannot match active_marker_filename()
  // on any realistic CI machine ($HOSTNAME≠"machine2" or $USER≠"user1", etc.)
  std::fs::write( store.join( "_active_machine2_user1" ), "alice@test.com" ).unwrap();
  std::fs::write( store.join( "_active_machine3_user2" ), "bob@test.com"   ).unwrap();

  // Sanity guard: own_name must differ from the chosen hard-coded names
  assert!(
    own_name != "_active_machine2_user1" && own_name != "_active_machine3_user2",
    "FT-11: own_name '{own_name}' collides with a hard-coded foreign filename — \
     update the test to use different foreign names",
  );

  let result : HashSet< String > = account::other_machines_active( store );

  assert_eq!(
    result.len(), 2,
    "FT-11: expected exactly 2 foreign accounts; got {result:?}",
  );
  assert!(
    result.contains( "alice@test.com" ),
    "FT-11: 'alice@test.com' must be in the result; got {result:?}",
  );
  assert!(
    result.contains( "bob@test.com" ),
    "FT-11: 'bob@test.com' must be in the result; got {result:?}",
  );
  assert!(
    !result.contains( "own@test.com" ),
    "FT-11: own marker content must be excluded from the result; got {result:?}",
  );
}

// test_kind: bug_reproducer(BUG-347)
/// FT-14/025 — `delete()` clears a foreign-machine marker naming the deleted
/// account, not only the calling machine's own marker.
///
/// ## Root Cause (AC-06 coverage)
/// `delete()`'s marker-clear guard checked only `read_active_marker(store)`,
/// which resolves via `active_marker_filename()` — bound to the CALLING
/// machine's own hostname+user. A marker belonging to a different machine,
/// even one naming the exact account being deleted, was structurally
/// invisible to that check and survived the delete untouched.
///
/// ## Setup
/// `TempDir` with the target account's `.credentials.json`, plus a
/// hard-coded foreign marker (`_active_machine2_user1` — the same
/// collision-free fixture name FT-11 already relies on) containing the
/// target account's name.
///
/// ## Assert
/// After `delete()`, the foreign marker no longer names the deleted account.
///
/// Spec: [`tests/docs/feature/025_per_machine_active_marker.md` FT-14]
#[ test ]
fn test_ft14_025_delete_clears_foreign_machine_marker()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  let name           = "ghost@obox.systems";
  let foreign_marker = "_active_machine2_user1";

  // Sanity guard: foreign_marker must differ from this machine's own marker name
  // (same guarantee FT-11 relies on), or this test would exercise the already-
  // correct same-machine path instead of the cross-machine path under test.
  let own_name = account::active_marker_filename();
  assert!(
    own_name != foreign_marker,
    "FT-14: own_name '{own_name}' collides with the hard-coded foreign filename — \
     update the test to use a different foreign name",
  );

  std::fs::write( store.join( format!( "{name}.credentials.json" ) ), "{}" ).unwrap();
  std::fs::write( store.join( foreign_marker ), name ).unwrap();

  account::delete( name, store ).unwrap();

  let still_points_at_deleted = std::fs::read_to_string( store.join( foreign_marker ) )
    .is_ok_and( | s | s.trim() == name );
  assert!(
    !still_points_at_deleted,
    "FT-14: foreign marker '{foreign_marker}' must no longer name the deleted \
     account '{name}' after delete(); marker survived unchanged",
  );
}

/// FT-12/025 — `other_machines_active()` returns empty `HashSet` when only own
/// marker exists, or when the store contains no `_active_*` files.
///
/// ## Root Cause (AC-05 coverage)
/// Case A tests the own-marker exclusion filter (own file present but excluded).
/// Case B tests the empty-directory path (no files → no iteration → empty result).
///
/// Spec: [`tests/docs/feature/025_per_machine_active_marker.md` FT-12]
#[ test ]
fn test_ft12_025_other_machines_active_empty_when_only_own()
{
  use std::collections::HashSet;

  // Case A: only own marker present — must be excluded → empty result
  {
    let tmp   = TempDir::new().unwrap();
    let store = tmp.path();
    let own_name = account::active_marker_filename();
    std::fs::write( store.join( &own_name ), "own@test.com" ).unwrap();

    let result : HashSet< String > = account::other_machines_active( store );
    assert!(
      result.is_empty(),
      "FT-12 Case A: only own marker → must return empty HashSet; got {result:?}",
    );
  }

  // Case B: empty store — no _active_* files at all
  {
    let tmp   = TempDir::new().unwrap();
    let store = tmp.path();

    let result : HashSet< String > = account::other_machines_active( store );
    assert!(
      result.is_empty(),
      "FT-12 Case B: empty store → must return empty HashSet; got {result:?}",
    );
  }
}

// ── Ownership: Feature 036 (FT-01, FT-02, FT-11, FT-14) ──────────────────────

/// Unit: `save()` with `Some(identity)` writes owner; re-save with different identity overwrites.
///
/// Tests the `save()` primitive API: `owner: Some(s)` always writes the given string.
/// (FT-01 in the integration test suite covers the command-level ownership-neutral behavior.)
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-01 (unit-level API contract)]
#[ test ]
fn ft01_save_captures_owner()
{
  let tmp   = TempDir::new().unwrap();
  let home  = tmp.path();
  let dot   = home.join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write( dot.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  let paths = ClaudePaths::with_home( home );
  let store = home.join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "user@host1" ), account::AccountBackend::Anthropic, None, None, None ).unwrap();
  let owner = account::read_owner( &store, "alice@test.com" );
  assert_eq!( owner, "user@host1", "FT-01: save() must write owner to {{name}}.json; got: {owner:?}" );

  // Re-save from a different identity — owner must be overwritten.
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "user@host2" ), account::AccountBackend::Anthropic, None, None, None ).unwrap();
  let owner2 = account::read_owner( &store, "alice@test.com" );
  assert_eq!( owner2, "user@host2", "FT-01: re-save must overwrite owner field; got: {owner2:?}" );
}

/// FT-02 (AC-02): `save()` with `owner: Some("")` writes empty owner string.
/// After unclaim, `is_owned()` returns `true` (empty owner disables all gates).
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-02]
#[ test ]
fn ft02_unclaim_clears_owner()
{
  let tmp   = TempDir::new().unwrap();
  let home  = tmp.path();
  let dot   = home.join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write( dot.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  let paths = ClaudePaths::with_home( home );
  let store = home.join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  // Set a non-local owner first.
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "other@remote" ), account::AccountBackend::Anthropic, None, None, None ).unwrap();

  // Unclaim: write empty owner.
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "" ), account::AccountBackend::Anthropic, None, None, None ).unwrap();
  let owner = account::read_owner( &store, "alice@test.com" );
  assert_eq!( owner, "", "FT-02: unclaim must write empty string as owner; got: {owner:?}" );
  assert!(
    account::is_owned( &owner ),
    "FT-02: is_owned() must return true for empty owner (G1–G7 gates pass)",
  );
}

/// FT-11 (AC-11): Account without `owner` key in `{name}.json` is backward compatible.
/// `read_owner()` returns `""` and `is_owned()` returns `true` — pre-feature behavior preserved.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-11]
#[ test ]
fn ft11_no_owner_field_backward_compat()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Write a legacy {name}.json with no `owner` key.
  std::fs::write(
    store.join( "legacy@test.com.json" ),
    r#"{"emailAddress":"legacy@test.com","model":"claude-opus-4-6"}"#,
  ).unwrap();

  let owner = account::read_owner( store, "legacy@test.com" );
  assert_eq!( owner, "", "FT-11: absent owner key must read as empty string; got: {owner:?}" );
  assert!(
    account::is_owned( &owner ),
    "FT-11: is_owned() must return true when owner key absent (backward compat — G1–G7 pass)",
  );
}

/// FT-14 (AC-14): Background `save()` calls with `owner: None` preserve existing owner field.
/// Simulates the `refresh_account_token()` path which must not mutate ownership.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-14]
#[ test ]
fn ft14_background_save_preserves_owner()
{
  let tmp   = TempDir::new().unwrap();
  let home  = tmp.path();
  let dot   = home.join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write( dot.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  let paths = ClaudePaths::with_home( home );
  let store = home.join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  // Initial CLI save: set owner.
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "alice@host1" ), account::AccountBackend::Anthropic, None, None, None ).unwrap();

  // Background save with owner: None — simulates refresh_account_token() path.
  account::save( "alice@test.com", &store, &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None ).unwrap();

  let owner = account::read_owner( &store, "alice@test.com" );
  assert_eq!(
    owner, "alice@host1",
    "FT-14: background save with owner:None must preserve existing owner; got: {owner:?}",
  );
}

// ── Ownership: param/057 unclaim EC cases ─────────────────────────────────────

/// EC-1: `unclaim::1` writes `owner: ""` on a freshly saved account.
///
/// Spec: [`tests/docs/cli/param/57_unclaim.md` EC-1]
#[ test ]
fn ec1_unclaim_writes_empty_owner()
{
  let tmp   = TempDir::new().unwrap();
  let home  = tmp.path();
  let dot   = home.join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write( dot.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  let paths = ClaudePaths::with_home( home );
  let store = home.join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "" ), account::AccountBackend::Anthropic, None, None, None ).unwrap();
  let owner = account::read_owner( &store, "alice@test.com" );
  assert_eq!( owner, "", "EC-1: unclaim must write empty string as owner; got: {owner:?}" );
  assert!( account::is_owned( &owner ), "EC-1: empty owner must pass all enforcement gates" );
}

/// EC-2: `unclaim::1` overwrites an existing non-empty `owner` value.
///
/// Spec: [`tests/docs/cli/param/57_unclaim.md` EC-2]
#[ test ]
fn ec2_unclaim_overwrites_existing_owner()
{
  let tmp   = TempDir::new().unwrap();
  let home  = tmp.path();
  let dot   = home.join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write( dot.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  let paths = ClaudePaths::with_home( home );
  let store = home.join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "alice@host1" ), account::AccountBackend::Anthropic, None, None, None ).unwrap();
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "" ), account::AccountBackend::Anthropic, None, None, None ).unwrap();
  let owner = account::read_owner( &store, "alice@test.com" );
  assert_eq!( owner, "", "EC-2: unclaim must overwrite existing non-empty owner; got: {owner:?}" );
}

/// EC-3: `save()` with `Some(identity)` writes the provided identity as `owner`.
///
/// Unit test of the `save()` primitive: when called with `owner: Some(identity)`,
/// the exact string is written to `{name}.json`. (The command handler passes
/// `owner: None` in production — see `account_ops.rs` `account_save_routine()`.)
///
/// Spec: [`tests/docs/cli/param/57_unclaim.md` EC-3]
#[ test ]
fn ec3_default_sets_owner_to_current_identity()
{
  let tmp   = TempDir::new().unwrap();
  let home  = tmp.path();
  let dot   = home.join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write( dot.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  let paths = ClaudePaths::with_home( home );
  let store = home.join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let identity = account::current_identity();
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( &identity ), account::AccountBackend::Anthropic, None, None, None ).unwrap();
  let owner = account::read_owner( &store, "alice@test.com" );
  assert_eq!(
    owner, identity,
    "EC-3: default save must write current_identity() as owner; got: {owner:?}",
  );
  assert!( account::is_owned( &owner ), "EC-3: current identity must pass is_owned() gate" );
}

/// EC-4: `unclaim::1` clears only `owner`; all other `{name}.json` fields are preserved via read-merge.
///
/// Spec: [`tests/docs/cli/param/57_unclaim.md` EC-4]
#[ test ]
fn ec4_unclaim_preserves_other_fields()
{
  let tmp   = TempDir::new().unwrap();
  let home  = tmp.path();
  let dot   = home.join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write( dot.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  let paths = ClaudePaths::with_home( home );
  let store = home.join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  // Pre-populate {name}.json with host, role, and owner.
  let meta = store.join( "alice@test.com.json" );
  std::fs::write(
    &meta,
    r#"{"host":"workstation","role":"work","owner":"alice@host1"}"#,
  ).unwrap();

  // Unclaim — only owner should change.
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "" ), account::AccountBackend::Anthropic, None, None, None ).unwrap();

  let content = std::fs::read_to_string( &meta ).unwrap();
  assert!(
    content.contains( "\"workstation\"" ),
    "EC-4: unclaim must preserve host field; got: {content}",
  );
  assert!(
    content.contains( "\"work\"" ),
    "EC-4: unclaim must preserve role field; got: {content}",
  );
  let owner = account::read_owner( &store, "alice@test.com" );
  assert_eq!( owner, "", "EC-4: owner field must be cleared by unclaim; got: {owner:?}" );
}

/// EC-5: Dry-run mode — when `is_dry()` is active, the command handler does NOT call `save()`.
/// Without a `save()` call, `{name}.json` retains its pre-existing `owner` value.
///
/// Design: `is_dry()` causes the command handler to return early. `save()` is never invoked.
/// This test verifies the expected end-state: pre-existing owner survives a dry-run pass.
///
/// Spec: [`tests/docs/cli/param/57_unclaim.md` EC-5]
#[ test ]
fn ec5_unclaim_dry_run_no_write()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Pre-populate {name}.json with a non-empty owner.
  std::fs::write(
    store.join( "alice@test.com.json" ),
    r#"{"owner":"alice@host1"}"#,
  ).unwrap();

  // Dry-run: do NOT call save() — command handler returns early on is_dry().
  // No write occurs; read_owner() must return the pre-existing value.
  let owner = account::read_owner( store, "alice@test.com" );
  assert_eq!(
    owner, "alice@host1",
    "EC-5: dry-run must not change owner; without save() call owner is preserved; got: {owner:?}",
  );
}

// ── Ownership: write_owner() unit tests ───────────────────────────────────────

/// `write_owner` read-merge: updates owner, preserves all other fields, does not
/// touch credentials file.
#[ test ]
fn test_write_owner_read_merge_preserves_fields()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Pre-populate {name}.json with multiple fields including owner.
  std::fs::write(
    store.join( "alice@test.com.json" ),
    r#"{"oauthAccount":{"email":"a@b.com"},"_renewal_at":"2026-01-01T00:00:00Z","owner":"old@host"}"#,
  ).unwrap();

  account::write_owner( "alice@test.com", store, "new@host2" ).unwrap();

  let content = std::fs::read_to_string( store.join( "alice@test.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &content ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap(), "new@host2",
    "write_owner must update owner field; got: {content}",
  );
  assert_eq!(
    val[ "_renewal_at" ].as_str().unwrap(), "2026-01-01T00:00:00Z",
    "write_owner must preserve _renewal_at; got: {content}",
  );
  assert_eq!(
    val[ "oauthAccount" ][ "email" ].as_str().unwrap(), "a@b.com",
    "write_owner must preserve oauthAccount; got: {content}",
  );

  // credentials file must NOT be created.
  assert!(
    !store.join( "alice@test.com.credentials.json" ).exists(),
    "write_owner must not create or touch credentials file",
  );
}

// ── Ownership: corner-case resilience ─────────────────────────────────────────

/// CC-1: `read_owner` with missing `{name}.json` file → returns "".
///
/// When the metadata file does not exist, `read_owner` must return an empty
/// string so that `is_owned()` returns `true` (all gates pass). This prevents
/// a missing file from blocking operations on legacy accounts that predate
/// the ownership feature.
#[ test ]
fn cc1_read_owner_missing_file()
{
  let tmp = TempDir::new().unwrap();
  // No file created — store is empty.
  let owner = account::read_owner( tmp.path(), "nonexistent@test.com" );
  assert_eq!( owner, "", "CC-1: read_owner on missing file must return empty string; got: {owner:?}" );
  assert!( account::is_owned( &owner ), "CC-1: missing file must pass is_owned() gate" );
}

/// CC-2: `read_owner` with empty file → returns "".
///
/// An empty `{name}.json` has no parseable `owner` field. `parse_string_field`
/// returns `None` and `read_owner` falls through to the default empty string.
#[ test ]
fn cc2_read_owner_empty_file()
{
  let tmp = TempDir::new().unwrap();
  std::fs::write( tmp.path().join( "alice@test.com.json" ), "" ).unwrap();
  let owner = account::read_owner( tmp.path(), "alice@test.com" );
  assert_eq!( owner, "", "CC-2: read_owner on empty file must return empty string; got: {owner:?}" );
  assert!( account::is_owned( &owner ), "CC-2: empty file must pass is_owned() gate" );
}

/// CC-3: `read_owner` with corrupt (non-JSON) content → returns "".
///
/// Binary/garbage content must not panic; `parse_string_field` finds no match
/// and returns `None`, producing the safe default.
#[ test ]
fn cc3_read_owner_corrupt_content()
{
  let tmp = TempDir::new().unwrap();
  std::fs::write( tmp.path().join( "alice@test.com.json" ), "<<<not json at all>>>" ).unwrap();
  let owner = account::read_owner( tmp.path(), "alice@test.com" );
  assert_eq!( owner, "", "CC-3: read_owner on corrupt content must return empty string; got: {owner:?}" );
  assert!( account::is_owned( &owner ), "CC-3: corrupt content must pass is_owned() gate" );
}

/// CC-4: `read_owner` with `"owner": null` (JSON null) → returns "".
///
/// `parse_string_field` checks for a leading `"` after the colon; `null` does
/// not start with `"`, so it returns `None` → safe default.
#[ test ]
fn cc4_read_owner_null_value()
{
  let tmp = TempDir::new().unwrap();
  std::fs::write(
    tmp.path().join( "alice@test.com.json" ),
    r#"{"owner": null}"#,
  ).unwrap();
  let owner = account::read_owner( tmp.path(), "alice@test.com" );
  assert_eq!( owner, "", "CC-4: read_owner with null owner must return empty string; got: {owner:?}" );
  assert!( account::is_owned( &owner ), "CC-4: null owner must pass is_owned() gate" );
}

/// CC-5: `read_owner` with `"owner": 42` (numeric) → returns "".
///
/// A numeric value lacks the leading `"` that `parse_string_field` requires.
#[ test ]
fn cc5_read_owner_numeric_value()
{
  let tmp = TempDir::new().unwrap();
  std::fs::write(
    tmp.path().join( "alice@test.com.json" ),
    r#"{"owner": 42}"#,
  ).unwrap();
  let owner = account::read_owner( tmp.path(), "alice@test.com" );
  assert_eq!( owner, "", "CC-5: read_owner with numeric owner must return empty string; got: {owner:?}" );
  assert!( account::is_owned( &owner ), "CC-5: numeric owner must pass is_owned() gate" );
}

/// CC-6: `save()` with `owner: None` on new account (no pre-existing `{name}.json`).
///
/// Background callers (`refresh_account_token`) pass `owner: None` and may be
/// the first caller to create `{name}.json` for a given account. Since there is
/// no pre-existing file to read-merge from, the `owner` key must be absent.
/// `read_owner()` must then return "" → `is_owned()` returns `true`.
#[ test ]
fn cc6_background_save_new_account_no_owner()
{
  let tmp   = TempDir::new().unwrap();
  let home  = tmp.path();
  let dot   = home.join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write( dot.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  let paths = ClaudePaths::with_home( home );
  let store = home.join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  // Background save: owner: None, no pre-existing {name}.json.
  account::save( "new@test.com", &store, &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None ).unwrap();

  let owner = account::read_owner( &store, "new@test.com" );
  assert_eq!(
    owner, "",
    "CC-6: background save on new account must not create owner field; got: {owner:?}",
  );
  assert!( account::is_owned( &owner ), "CC-6: absent owner must pass is_owned() gate" );
}

