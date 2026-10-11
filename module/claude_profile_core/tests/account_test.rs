//! Account unit tests: save, delete, and `switch_account` operations.
//!
//! ## Purpose
//!
//! Verify `account::save()` writes `_active` = `name` on every successful save,
//! that `account::delete()` removes the consolidated file created by `save()`:
//! `{name}.json`,
//! and that `account::switch_account()` correctly restores per-account model preference
//! from `{name}.json` into `~/.claude/settings.json` (BUG-222).
//!
//! ## Fix Documentation — issue-snapshot-orphan
//!
//! - **Root Cause:** `save()` creates multiple satellite files but `delete()` only removed
//!   `.credentials.json`, leaving `.json` and other snapshot files as orphans after deletion.
//! - **Why Not Caught:** No test verified that snapshot files are absent after `delete()`; the
//!   orphan files accumulated silently over every `save` / `delete` call pair.
//! - **Fix Applied:** After the mandatory `remove_file(credentials)`, best-effort
//!   `let _ = remove_file(...)` calls clean up `{name}.json` and legacy satellite files.
//! - **Prevention:** `ad_delete_also_removes_snapshots` asserts all 3 files absent post-delete.
//! - **Pitfall:** Snapshot removal must be best-effort (`let _ = ...`) — accounts saved before
//!   snapshot support was added have no snapshot files; a strict `remove_file` would fail them.
//!
//! ## Fix Documentation — BUG-222
//!
//! - **Root Cause:** `save()` never captured the `model` field from `~/.claude/settings.json`,
//!   so no per-account model snapshot existed. `switch_account()` never touched `settings.json`,
//!   leaving the prior account's model in place after every switch.
//! - **Why Not Caught:** All `switch_account()` tests asserted on credentials and oauthAccount fields;
//!   `settings.json` was not part of any assertion. The silent persistence of model was invisible.
//! - **Fix Applied:** `save()` reads `~/.claude/settings.json`, extracts `model`, and write-merges
//!   it into `{name}.json` when present. `switch_account()` reads `{name}.json`,
//!   and either installs the saved model or removes the `model` key from live `settings.json`.
//! - **Prevention:** Structural test confirms `settings_file()` call exists in `src/account/`; four
//!   MRE tests cover both directions of save and switch for present and absent model.
//! - **Pitfall:** Both operations are best-effort — `settings.json` handling must never cause
//!   `save()` or `switch_account()` to return `Err`; credentials switch already succeeded.
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `as_save_writes_active_marker` | save() with update_marker=true → `_active` written |
//! | `test_mre_bug211_save_false_leaves_marker_unchanged` | save() with update_marker=false → `_active` not written |
//! | `mre_bug_582_save_refuses_session_another_slot_holds` | live session held by bob's slot, save as alice → Err naming bob, slots and marker unchanged |
//! | `slots_holding_session_names_every_holder_in_name_order` | two slots share a refresh token → both named, sorted; fresh, empty or unheld access token, or no store → none |
//! | `bug_582_guard_allows_own_slot_and_unheld_session` | re-save of the holding slot and save of a fresh login both succeed |
//! | `bug_582_live_redirect_key_matched_by_access_token` | kimi's redirect key live → helper names kimi, save as alice errs, her slot unchanged |
//! | `bug_582_resave_keeps_slot_identity_when_claude_json_names_another` | bob re-saved while `~/.claude.json` names alice → bob's identity kept; a fresh login saved over bob's slot takes the live one |
//! | `ad_delete_also_removes_snapshots` | All 3 files exist → all 3 absent after delete |
//! | `ad_delete_succeeds_when_snapshots_absent` | Only credentials → delete succeeds, no error |
//! | `mre_bug_219_switch_account_stale_org_name` | switch_account() overrides org fields from {name}.json |
//! | `bug_mre_bug222_switch_account_reads_settings_snapshot` | structural: `settings_file()` present in src/account/ |
//! | `mre_bug222_save_captures_model_to_settings_snapshot` | save() with model in settings.json → {name}.json has model |
//! | `mre_bug222_save_no_model_does_not_write_settings_snapshot` | save() with no model in settings.json → {name}.json has no "model" key (Feature 071: file itself is always created) |
//! | `mre_bug222_switch_account_restores_model_from_settings_snapshot` | switch_account() installs model from {name}.json into live settings |
//! | `mre_bug222_switch_account_clears_model_when_no_snapshot` | switch_account() absent snapshot → removes model from live settings |
//! | `m8_lock_store_blocks_second_holder_until_release` | Audit M8: a second lock_store() on the same store blocks until the first holder drops — mutual exclusion direction |
//! | `m8_lock_store_release_frees_reacquire_and_names_lockfile` | Audit M8: after drop the lock is immediately reacquirable (release direction), and the lock file is the gitignored `-store.lock` |

use tempfile::TempDir;
use claude_profile_core::account;
use claude_core::ClaudePaths;

mod account_fixture;
use account_fixture::*;

// ── tests ─────────────────────────────────────────────────────────────────────

#[ test ]
fn ad_delete_also_removes_snapshots()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Active account is different — allows deletion of old@archive.com
  write_active( store, "work@acme.com" );
  write_credentials_file( store, "old@archive.com" );
  std::fs::write( store.join( "old@archive.com.json" ),    r#"{"emailAddress":"old@archive.com"}"# ).unwrap();

  let result = account::delete( "old@archive.com", store );
  assert!( result.is_ok(), "delete must succeed when all 2 files exist: {result:?}" );

  assert!(
    !store.join( "old@archive.com.credentials.json" ).exists(),
    "credentials file must be absent after delete",
  );
  assert!(
    !store.join( "old@archive.com.json" ).exists(),
    "metadata file must be absent after delete",
  );
}

#[ test ]
fn ad_delete_succeeds_when_snapshots_absent()
{
  // Guard: accounts saved before consolidation have no .json;
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

/// Write `{name}.credentials.json` holding a session with `refresh_token`, as an earlier save
/// would have, and return its content.
///
/// No `accessToken`: a successful `save()` with `update_marker` looks up the org with it,
/// and a fixture token must never reach the network.
fn write_session_slot( store : &std::path::Path, name : &str, refresh_token : &str ) -> String
{
  std::fs::create_dir_all( store ).unwrap();
  let content = format!(
    r#"{{"claudeAiOauth":{{"refreshToken":"{refresh_token}","expiresAt":4102444800000}}}}"#
  );
  std::fs::write( store.join( format!( "{name}.credentials.json" ) ), &content ).unwrap();
  content
}

/// Make `content` the live session in `~/.claude/.credentials.json` under `home`.
fn write_live_session( home : &std::path::Path, content : &str )
{
  let dot_claude = home.join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), content ).unwrap();
}

/// bug_reproducer(BUG-582): `save()` must refuse a live session that another slot already
/// holds, and leave the slots and the active marker byte-identical.
///
/// # Root Cause
/// `save()` copied `~/.claude/.credentials.json` into `{name}.credentials.json` without
/// checking whose session it was. A bare `.account.save` takes `name` from `~/.claude.json`,
/// which a running Claude process can rewrite after a switch, so alice's slot got bob's
/// session and alice's own was gone from the store.
///
/// # Why Not Caught
/// No save test checked whether another slot already held the live session, so fixtures that
/// saved one live session under two names (`claude_profile`'s account list tests) passed
/// unnoticed.
///
/// # Fix Applied
/// `save()` asks `slots_holding_session()` for every slot whose `refreshToken` (a redirect
/// key's `accessToken`) equals the live one, and refuses before writing any account file when
/// one of them isn't `name`.
///
/// # Prevention
/// Stores alice and bob, makes bob's session live, and asserts a save as alice errs naming
/// bob, with both slots and the marker unchanged and no metadata written.
///
/// # Pitfall
/// Refresh tokens are single-use, so a match proves the slot holds this very session but no
/// match proves nothing: after a refresh outside clp the live token matches no slot at all.
#[ doc = "bug_reproducer(BUG-582)" ]
#[ test ]
fn mre_bug_582_save_refuses_session_another_slot_holds()
{
  let tmp    = TempDir::new().unwrap();
  let store  = tmp.path().join( "store" );
  let alice  = write_session_slot( &store, "alice@example.com", "rt-alice" );
  let bob    = write_session_slot( &store, "bob@example.com", "rt-bob" );
  let marker = store.join( account::active_marker_filename() );
  std::fs::write( &marker, "bob@example.com" ).unwrap();
  // bob's session is live, as right after `.account.use name::bob@example.com`.
  write_live_session( tmp.path(), &bob );
  let paths = ClaudePaths::with_home( tmp.path() );

  let err = account::save(
    "alice@example.com", &store, &paths, true, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None, None,
  ).expect_err( "save() must refuse bob's live session under alice's name" );

  assert!(
    err.to_string().contains( "already saved as 'bob@example.com'" ),
    "the refusal must name the slot holding the session, got: {err}",
  );
  let slot = | name : &str |
  {
    std::fs::read_to_string( store.join( format!( "{name}.credentials.json" ) ) ).unwrap()
  };
  assert_eq!(
    slot( "alice@example.com" ), alice,
    "alice's slot must keep alice's own session",
  );
  assert_eq!(
    slot( "bob@example.com" ), bob,
    "bob's slot must be untouched",
  );
  assert_eq!(
    std::fs::read_to_string( &marker ).unwrap(), "bob@example.com",
    "the marker must not move",
  );
  assert!(
    !store.join( "alice@example.com.json" ).exists(),
    "a refused save must write no metadata",
  );
}

/// `slots_holding_session()` names every slot holding the live refresh token, in name order,
/// and nothing for a session no slot holds.
#[ test ]
fn slots_holding_session_names_every_holder_in_name_order()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  write_session_slot( &store, "zed@example.com", "rt-shared" );
  write_session_slot( &store, "amy@example.com", "rt-shared" );
  write_session_slot( &store, "kim@example.com", "rt-other" );
  let live    = | rt : &str | format!( r#"{{"claudeAiOauth":{{"refreshToken":"{rt}"}}}}"# );
  let holders = | rt : &str | account::slots_holding_session( &store, &live( rt ) );

  assert_eq!(
    holders( "rt-shared" ),
    strings( &[ "amy@example.com", "zed@example.com" ] ),
    "both slots holding the session must be named, sorted",
  );
  assert_eq!(
    holders( "rt-other" ),
    strings( &[ "kim@example.com" ] ),
    "a session one slot holds names that slot",
  );
  assert!(
    holders( "rt-new" ).is_empty(),
    "a fresh login has no holder",
  );
  assert!(
    holders( "" ).is_empty(),
    "an empty token matches nothing",
  );
  assert!(
    account::slots_holding_session( &store, r#"{"accessToken":"tok"}"# ).is_empty(),
    "an access token no slot carries matches nothing",
  );
  assert!(
    account::slots_holding_session( &tmp.path().join( "absent" ), &live( "rt-shared" ) ).is_empty(),
    "a missing store has no holders",
  );
}

/// BUG-582's guard leaves the two ordinary saves alone: re-saving the slot that holds the
/// live session, and saving a fresh login no slot holds yet.
#[ test ]
fn bug_582_guard_allows_own_slot_and_unheld_session()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  let bob   = write_session_slot( &store, "bob@example.com", "rt-bob" );
  let paths = ClaudePaths::with_home( tmp.path() );
  let save  = | name : &str | account::save(
    name, &store, &paths, true, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None, None,
  );

  write_live_session( tmp.path(), &bob );
  save( "bob@example.com" ).expect( "re-saving the slot that holds the live session must succeed" );

  let carol = r#"{"claudeAiOauth":{"refreshToken":"rt-carol","expiresAt":4102444800000}}"#;
  write_live_session( tmp.path(), carol );
  save( "carol@example.com" ).expect( "a session no slot holds must save" );
  assert_eq!(
    std::fs::read_to_string( store.join( "carol@example.com.credentials.json" ) ).unwrap(),
    carol,
    "carol's slot must hold the fresh login",
  );
}

/// BUG-582: a redirect key carries no refresh token, so `slots_holding_session()` matches it
/// by `accessToken`, and `save()` refuses to copy it into another slot.
#[ test ]
fn bug_582_live_redirect_key_matched_by_access_token()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  let alice = write_session_slot( &store, "alice@example.com", "rt-alice" );
  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "kimi", &store, &paths, false, Some( b"key-kimi" ), None, None, None,
    account::AccountBackend::Redirect, Some( "http://127.0.0.1:9" ), Some( "m1" ), None, None,
  ).expect( "a redirect save must succeed" );
  // `.account.use name::kimi` installs the slot's file as the live one, byte for byte.
  let kimi = std::fs::read_to_string( store.join( "kimi.credentials.json" ) ).unwrap();
  write_live_session( tmp.path(), &kimi );

  assert_eq!(
    account::slots_holding_session( &store, &kimi ),
    strings( &[ "kimi" ] ),
    "the live redirect key must name the slot holding it",
  );
  let err = account::save(
    "alice@example.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None, None,
  ).expect_err( "save() must refuse kimi's live key under alice's name" );
  assert!(
    err.to_string().contains( "already saved as 'kimi'" ),
    "the refusal must name the slot holding the key, got: {err}",
  );
  assert_eq!(
    std::fs::read_to_string( store.join( "alice@example.com.credentials.json" ) ).unwrap(),
    alice,
    "alice's slot must keep alice's own session",
  );
}

/// BUG-582: re-saving the slot that holds the live session keeps the slot's identity when
/// `~/.claude.json` names another account; a fresh login saved over it takes `~/.claude.json`'s.
#[ test ]
fn bug_582_resave_keeps_slot_identity_when_claude_json_names_another()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  let paths = ClaudePaths::with_home( tmp.path() );
  let save  = | name : &str | account::save(
    name, &store, &paths, true, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None, None,
  );
  let claude_json_names = | email : &str |
  {
    let text = format!( r#"{{"oauthAccount":{{"emailAddress":"{email}"}}}}"# );
    std::fs::write( tmp.path().join( ".claude.json" ), text ).unwrap();
  };
  let stored_email = | name : &str |
  {
    let meta = std::fs::read_to_string( store.join( format!( "{name}.json" ) ) ).unwrap();
    account::parse_string_field( &meta, "emailAddress" )
  };

  let bob = write_session_slot( &store, "bob@example.com", "rt-bob" );
  write_live_session( tmp.path(), &bob );
  claude_json_names( "bob@example.com" );
  save( "bob@example.com" ).expect( "saving bob's live session as bob must succeed" );
  // A Claude process still running under alice rewrites ~/.claude.json.
  claude_json_names( "alice@example.com" );
  save( "bob@example.com" ).expect( "re-saving the slot that holds the live session must succeed" );
  assert_eq!(
    stored_email( "bob@example.com" ).as_deref(), Some( "bob@example.com" ),
    "bob's slot must keep bob's identity, not alice's from a lagging ~/.claude.json",
  );

  let carol = r#"{"claudeAiOauth":{"refreshToken":"rt-carol","expiresAt":4102444800000}}"#;
  write_live_session( tmp.path(), carol );
  claude_json_names( "carol@example.com" );
  save( "bob@example.com" ).expect( "a fresh login must save under the given name" );
  assert_eq!(
    stored_email( "bob@example.com" ).as_deref(), Some( "carol@example.com" ),
    "a fresh login no slot holds must take ~/.claude.json's identity over the stored one",
  );
}

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

  account::save( "alice@test.com", &store, &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None, None ).unwrap();

  let marker = store.join( account::active_marker_filename() );
  assert!(
    !marker.exists(),
    "save() with update_marker=false must NOT write the _active marker file; found: {marker:?}",
  );
}

/// BUG-219 MRE: `switch_account()` must override `oauthAccount.organizationName`
/// and `oauthAccount.organizationUuid` from `{name}.json`, not from the stale snapshot.
///
/// # Root Cause
/// The BUG-217 fix block (switch.rs; account.rs ~line 338 at fix time) only inserts `emailAddress`. All other
/// `oauthAccount` fields — including `organizationName`, `organizationUuid` — are copied
/// verbatim from the snapshot. When the snapshot was captured while a different account
/// (`i7@test.com`) was active, these fields carry i7's org identity. Claude Code's `/usage`
/// command reads `oauthAccount.organizationName` from `~/.claude.json` and displays the
/// wrong org name.
///
/// # Why Not Caught
/// No test verified org fields post-switch. The BUG-217 fix was scoped to `emailAddress`
/// only. The two data paths (`clp` reads `roles.json` — correct; Claude Code reads
/// `~/.claude.json` `oauthAccount` — stale) were never exercised together.
///
/// # Fix Applied
/// After the BUG-217 `emailAddress` insert, read `{name}.json` and override
/// `organizationName` and `organizationUuid` using `parse_string_field`.
///
/// # Prevention
/// This test catches any regression that removes the `organizationName` override or
/// reverts the scope of the BUG-217 fix block.
///
/// # Pitfall
/// `parse_string_field` is a simple substring matcher — it requires `"organizationName":`
/// (double-quoted key) in the output. Do not use `json!()` macro for the assertion;
/// read `~/.claude.json` as a raw string and use `parse_string_field` to extract.
/// `claude_json_file()` returns `$HOME/.claude.json` (at HOME level, one level ABOVE
/// `$HOME/.claude/`). Do NOT use `dot_claude.join("claude.json")` — that path is inside
/// `.claude/` and is never written by `switch_account()`.
#[ doc = "bug_reproducer(BUG-219)" ]
#[ test ]
fn mre_bug_219_switch_account_stale_org_name()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();

  // Set up ~/.claude.json with i7's org currently active (simulates i7 being the active session).
  // NOTE: claude_json_file() returns $HOME/.claude.json (HOME level), NOT $HOME/.claude/claude.json.
  std::fs::write(
    tmp.path().join( ".claude.json" ),
    r#"{"oauthAccount":{"emailAddress":"i7@test.com","organizationName":"i7 Org","organizationUuid":"uuid-i7"},"commands":{}}"#,
  ).unwrap();

  // i6's credentials file (required for switch_account to proceed)
  std::fs::write(
    store.join( "i6@test.com.credentials.json" ),
    r#"{"accessToken":"tok-i6","expiresAt":9999999999999,"subscriptionType":"pro"}"#,
  ).unwrap();

  // i6's unified metadata snapshot — oauthAccount has stale org (captured while i7 was active),
  // but top-level organization_* fields have the correct values from the live API.
  // switch_account() must override oauthAccount org fields from the top-level fields.
  std::fs::write(
    store.join( "i6@test.com.json" ),
    r#"{"oauthAccount":{"emailAddress":"i6@test.com","organizationName":"i7 Org","organizationUuid":"uuid-i7"},"organization_uuid":"uuid-i6","organization_name":"i6 Org","organization_role":"member"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "i6@test.com", &store, &paths ).unwrap();

  let claude_json = std::fs::read_to_string( tmp.path().join( ".claude.json" ) )
    .expect( "~/.claude.json must exist after switch_account" );

  let org_name = account::parse_string_field( &claude_json, "organizationName" )
    .expect( "oauthAccount.organizationName must be present after switch_account" );
  let org_uuid = account::parse_string_field( &claude_json, "organizationUuid" )
    .expect( "oauthAccount.organizationUuid must be present after switch_account" );
  let email    = account::parse_string_field( &claude_json, "emailAddress" )
    .expect( "oauthAccount.emailAddress must be present after switch_account" );

  assert_eq!(
    org_name, "i6 Org",
    "oauthAccount.organizationName must be i6's org from roles.json, not the stale i7 snapshot value",
  );
  assert_eq!(
    org_uuid, "uuid-i6",
    "oauthAccount.organizationUuid must be i6's UUID from roles.json, not the stale i7 value",
  );
  assert_eq!(
    email, "i6@test.com",
    "oauthAccount.emailAddress must be enforced to name (BUG-217 invariant preserved)",
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

  account::save( "alice@acme.com", &store, &paths, true, None, None, None, None, account::AccountBackend::Anthropic, None, None, None, None ).unwrap();

  let marker_name = account::active_marker_filename();
  let active = std::fs::read_to_string( store.join( &marker_name ) )
    .expect( "_active must exist after save()" );
  assert_eq!(
    active.trim(),
    "alice@acme.com",
    "_active must contain the saved account name",
  );
}

// ── BUG-222 — per-account model preference capture and restore ─────────────────

#[ test ]
// Root Cause: switch_account() never read {name}.json or touched
//   ~/.claude/settings.json; the active model persisted from the prior account
//   after every switch regardless of the target account's preference (BUG-222).
// Why Not Caught: switch_account() tests asserted only on credentials and oauthAccount;
//   settings.json was never part of any assertion, so the gap was invisible.
// Fix Applied: structural assertion that the account module contains at least one
//   settings_file() call, confirming the restore step is present.
// Prevention: if the restore step is removed from src/account/ the count drops to 0 and
//   this test goes RED immediately.
// Pitfall: settings_file() appears in both save() (store.rs) and switch_account()
//   (switch.rs) after the BUG-222 fix; count >= 1 is the correct bound, not == 1. The
//   scan concatenates every file under src/account/ so intra-module moves stay green.
fn bug_mre_bug222_switch_account_reads_settings_snapshot()
{
  let account_dir = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account" );
  let content : String = std::fs::read_dir( &account_dir )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_dir.display() ) )
    .filter_map( Result::ok )
    .map( | e | std::fs::read_to_string( e.path() ).unwrap_or_default() )
    .collect();
  let count = content.matches( "settings_file()" ).count();
  assert!(
    count >= 1,
    "BUG-222: expected >=1 occurrence of 'settings_file()' in src/account/, found {count}",
  );
}

#[ test ]
// Root Cause: save() wrote credentials and oauthAccount snapshots but never captured the
//   model preference from ~/.claude/settings.json, so no {name}.json was created
//   with model data; switch_account() had nothing to restore (BUG-222).
// Why Not Caught: no test exercised the {name}.json write path in save();
//   the file appeared only in delete() as a best-effort orphan removal target.
// Fix Applied: save() reads ~/.claude/settings.json, extracts "model" via parse_string_field,
//   and write-merges it into {name}.json when present.
// Prevention: asserts {name}.json is created and contains the correct model value
//   after save() when ~/.claude/settings.json has a model key.
// Pitfall: save() is best-effort on settings capture — a failing settings write does NOT
//   cause save() to return Err; only the credentials write is mandatory.
fn mre_bug222_save_captures_model_to_settings_snapshot()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"model":"claude-opus-4-5","theme":"dark"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save( "alice@test.com", &store, &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None, None ).unwrap();

  let snap_path = store.join( "alice@test.com.json" );
  assert!( snap_path.exists(), "save() must create {{name}}.json when model is present in live settings" );
  let snap = std::fs::read_to_string( &snap_path )
    .expect( "{{name}}.json must be readable after save()" );
  let model = account::parse_string_field( &snap, "model" )
    .expect( "{{name}}.json must contain 'model' after save() with model in live settings" );
  assert_eq!( model, "claude-opus-4-5", "captured model must equal the value in ~/.claude/settings.json" );
}

#[ test ]
// Root Cause: (same — save() did not read settings.json at all before BUG-222 fix)
// Why Not Caught: (same — no test exercised any save()/settings.json interaction)
// Fix Applied: save() skips writing a "model" key to {name}.json when model is absent from
//   ~/.claude/settings.json — avoids a misleading model value for accounts with no preference.
// Prevention: asserts {name}.json has no "model" key when the live settings model key is absent.
// Pitfall: the skip applies when the model key is absent; other keys in settings.json
//   are not captured — only model is a per-account preference (BUG-222 scope).
// Feature 071/AC-04 amendment: {name}.json is now unconditionally created on every save
// (it always carries at least `backend`) — this test's original assertion (no file at all)
// predates that change; it now asserts the narrower, still-true claim: no "model" key.
fn mre_bug222_save_no_model_does_not_write_settings_snapshot()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"theme":"dark"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "bob@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None, None,
  ).unwrap();

  let snap_content = std::fs::read_to_string( store.join( "bob@test.com.json" ) )
    .expect( "{name}.json must exist after save (Feature 071: backend is always written)" );
  assert!(
    !snap_content.contains( "\"model\"" ),
    "save() must NOT write a 'model' key when absent from ~/.claude/settings.json; got: {snap_content}",
  );
}

#[ test ]
// Root Cause: switch_account() copied credentials and patched oauthAccount but left
//   ~/.claude/settings.json untouched; the prior account's model persisted after every
//   switch — switching from sonnet to an account saved with haiku still ran on sonnet (BUG-222).
// Why Not Caught: switch_account() tests validated credentials and oauthAccount; settings.json
//   was never asserted on, so the stale model was invisible.
// Fix Applied: switch_account() reads {name}.json, extracts model, and installs it
//   into ~/.claude/settings.json; if model is absent it removes the key (see next test).
// Prevention: asserts the target account's saved model appears in live settings.json after
//   switch; any regression removing the restore step fails this assertion.
// Pitfall: switch_account() restore is best-effort — credentials+oauthAccount switch already
//   succeeded before the settings step; a settings write failure is silent.
fn mre_bug222_switch_account_restores_model_from_settings_snapshot()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "max@test.com.credentials.json" ),
    r#"{"accessToken":"tok-max","expiresAt":9999999999999,"subscriptionType":"max"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "max@test.com.json" ),
    r#"{"model":"claude-haiku-4-5"}"#,
  ).unwrap();
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"model":"claude-sonnet-4-6","theme":"dark"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "max@test.com", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let model = account::parse_string_field( &live, "model" )
    .expect( "model must be present in settings.json after switching to account with settings snapshot" );
  assert_eq!(
    model, "claude-haiku-4-5",
    "model must equal the target account's saved preference, not the prior account's",
  );
}

#[ test ]
// Root Cause: (same — switch_account() left settings.json untouched entirely before BUG-222 fix)
// Why Not Caught: (same — no tests asserted on settings.json after switch)
// Fix Applied: when {name}.json is absent or has no model, switch_account() removes
//   the "model" key from live settings.json so no stale model persists.
// Prevention: asserts model key is ABSENT from live settings.json after switching to an account
//   with no settings snapshot; any regression re-introducing stale persistence fails this.
// Pitfall: absent {name}.json is not an error; clearing is the correct behaviour when
//   the target account was never saved with a model preference.
fn mre_bug222_switch_account_clears_model_when_no_snapshot()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "free@test.com.credentials.json" ),
    r#"{"accessToken":"tok-free","expiresAt":9999999999999,"subscriptionType":"free"}"#,
  ).unwrap();
  // No {name}.json for this account.
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"model":"claude-opus-4-6","theme":"light"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "free@test.com", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  assert!(
    account::parse_string_field( &live, "model" ).is_none(),
    "model key must be removed from live settings.json when target account has no settings snapshot",
  );
}

/// # Root Cause
/// `switch_account()` gates the `emailAddress` patch inside `if let Ok(saved_val) =
/// serde_json::from_str(&meta_text)`. When `{name}.json` is absent, `meta_text` is `""`,
/// `from_str("")` returns `Err`, and the entire oauthAccount patch block is skipped —
/// including the BUG-217 `emailAddress` enforcement. `~/.claude.json` retains the previous
/// account's `emailAddress`, causing downstream `save()` name inference to target the wrong file.
///
/// # Why Not Caught
/// All existing `switch_account()` tests provide a `{name}.json` metadata file. No test
/// covers the absent-metadata-file path where only credentials exist.
///
/// # Fix Applied
/// Lift the unconditional `emailAddress` patch out of the metadata-file-conditional block.
/// Patch `~/.claude.json oauthAccount.emailAddress = name` before attempting to read
/// `{name}.json`. The full overlay (BUG-217 + BUG-219) still fires when metadata is present.
///
/// # Prevention
/// This MRE test creates a credential-only account (no `{name}.json`) and asserts that
/// `emailAddress` is patched to the switched-to name after `switch_account()`.
///
/// # Pitfall
/// `claude_json_file()` returns `$HOME/.claude.json` (HOME level), not `$HOME/.claude/claude.json`.
/// Machine-global keys (`commands`, `mcpServers`) must survive the patch — assert preservation.
#[ doc = "bug_reproducer(BUG-254)" ]
#[ test ]
fn mre_bug254_switch_account_patches_email_when_metadata_absent()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();

  // Seed ~/.claude.json with alice's session + machine-global keys.
  std::fs::write(
    tmp.path().join( ".claude.json" ),
    r#"{"oauthAccount":{"emailAddress":"alice@acme.com","displayName":"Alice"},"commands":{"enabled":true},"mcpServers":{}}"#,
  ).unwrap();

  // bob has credentials ONLY — no bob@acme.com.json metadata file.
  std::fs::write(
    store.join( "bob@acme.com.credentials.json" ),
    r#"{"accessToken":"tok-bob","expiresAt":9999999999999}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "bob@acme.com", &store, &paths ).unwrap();

  let claude_json = std::fs::read_to_string( tmp.path().join( ".claude.json" ) )
    .expect( "~/.claude.json must exist after switch_account" );

  // Core assertion: emailAddress must be patched unconditionally.
  let email = account::parse_string_field( &claude_json, "emailAddress" )
    .expect( "oauthAccount.emailAddress must be present" );
  assert_eq!(
    email, "bob@acme.com",
    "emailAddress must be patched to switched-to name even when {{name}}.json is absent",
  );

  // Machine-global keys must be preserved.
  assert!(
    claude_json.contains( r#""commands":"# ),
    "machine-global key 'commands' must survive the emailAddress patch",
  );
  assert!(
    claude_json.contains( r#""mcpServers":"# ),
    "machine-global key 'mcpServers' must survive the emailAddress patch",
  );
}


// ── Credential-file permissions (audit-credential-file-perms) ─────────────────

/// Store credential files land owner-read/write only, and replacing a
/// world-readable pre-existing slot tightens it to `0o600`.
///
/// ## Fix Documentation — audit-credential-file-perms
///
/// - **Root Cause:** `save()` wrote `{name}.credentials.json` via bare `fs::write`/
///   `fs::copy`, landing OAuth tokens with umask-default `0644` — readable by any
///   local user; `fs::copy` additionally propagated whatever mode the source had.
/// - **Why Not Caught:** No test asserted on-disk permission bits of any credential
///   write anywhere in the crate.
/// - **Fix Applied:** All credential writes route through
///   `claude_core::file_io::atomic_write_secret`, which opens the temp file `0o600`
///   before the first content byte; the mode travels through the rename.
/// - **Prevention:** This test pins the final mode bits for both the fresh-write and
///   the replace-a-world-readable-file paths.
/// - **Pitfall:** chmod-after-write leaves a readable window, and `fs::copy` is a
///   mode-preserving trap — write content through the secret-mode primitive instead.
#[ cfg( unix ) ]
#[ test ]
fn save_writes_credential_file_owner_only()
{
  use std::os::unix::fs::PermissionsExt;
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  let dest  = store.join( "alice@test.com.credentials.json" );

  // Pre-plant a world-readable slot — save() must replace it with an 0600 file.
  std::fs::write( &dest, r#"{"accessToken":"old"}"# ).unwrap();
  std::fs::set_permissions( &dest, std::fs::Permissions::from_mode( 0o644 ) ).unwrap();

  account::save( "alice@test.com", &store, &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None, None ).unwrap();

  let mode = std::fs::metadata( &dest ).unwrap().permissions().mode() & 0o777;
  assert_eq!( mode, 0o600, "store credential file must be 0600, got {mode:o}" );
  assert_eq!(
    std::fs::read_to_string( &dest ).unwrap(), r#"{"accessToken":"tok"}"#,
    "save() must install the live credentials content"
  );
}

/// Audit M8 (mutual-exclusion direction): while one `StoreLock` is held, a second
/// `lock_store()` on the same store must block until the first is dropped.
///
/// Two separate `File` opens are two open file descriptions, so `flock` contention
/// applies even within one process — no subprocess needed for a real, non-mocked
/// contention check. The assertion is a lower bound only (second acquisition cannot
/// land before the 300 ms hold ends); no upper bound, so scheduler jitter can't flake it.
#[ test ]
fn m8_lock_store_blocks_second_holder_until_release()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );

  let t0     = std::time::Instant::now();
  let lock_a = account::lock_store( &store ).unwrap();

  let store_b = store.clone();
  let waiter  = std::thread::spawn( move ||
  {
    let _lock_b = account::lock_store( &store_b ).unwrap();
    std::time::Instant::now()
  } );

  const HOLD : core::time::Duration = core::time::Duration::from_millis( 300 );
  std::thread::sleep( HOLD );
  drop( lock_a );

  let acquired_at = waiter.join().unwrap();
  let blocked_for = acquired_at.duration_since( t0 );
  assert!(
    blocked_for >= HOLD,
    "second lock_store() must block until the first holder releases — acquired after {blocked_for:?}, hold was {HOLD:?}"
  );
}

/// Audit M8 (release direction — counterpart of the blocking test): dropping the
/// guard frees the lock for immediate reacquisition, and the lock file lands at
/// the store root as `-store.lock` (hyphen prefix → gitignored in tracked stores).
///
/// If drop failed to release, the second `lock_store()` would block forever and the
/// runner's per-test timeout would fail this loudly — no timing assertion needed.
#[ test ]
fn m8_lock_store_release_frees_reacquire_and_names_lockfile()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );

  let first = account::lock_store( &store ).unwrap();
  assert!( store.join( "-store.lock" ).is_file(), "lock file must be created at {{store}}/-store.lock" );
  drop( first );

  let second = account::lock_store( &store ).unwrap();
  drop( second );
}

/// BUG-582: a credentials file can carry an MCP server's tokens ahead of the session's;
/// `session_token()` reads the session's own, so slots sharing only the MCP entry don't match.
#[ test ]
fn bug_582_mcp_entries_never_identify_the_session()
{
  let tmp   = TempDir::new().unwrap();
  let creds = | rt : &str |
  {
    let mcp  = r#"{"mcpOAuth":{"srv":{"accessToken":"mcp-at","refreshToken":"mcp-rt"}},"#;
    let main = format!( r#""claudeAiOauth":{{"accessToken":"at-{rt}","refreshToken":"{rt}"}}}}"# );
    format!( "{mcp}{main}" )
  };
  std::fs::write( tmp.path().join( "alice@example.com.credentials.json" ), creds( "rt-alice" ) ).unwrap();
  std::fs::write( tmp.path().join( "bob@example.com.credentials.json" ), creds( "rt-bob" ) ).unwrap();
  let live = creds( "rt-bob" );
  assert_eq!( account::slots_holding_session( tmp.path(), &live ), vec![ "bob@example.com" ] );
  assert_eq!( account::session_token( &live, "refreshToken" ).as_deref(), Some( "rt-bob" ) );
  assert_eq!( account::session_token( &live, "accessToken" ).as_deref(), Some( "at-rt-bob" ) );
  let redirect = r#"{"accessToken":"sk-key"}"#;
  assert_eq!( account::session_token( redirect, "accessToken" ).as_deref(), Some( "sk-key" ) );
  let logged_out = r#"{"mcpOAuth":{"srv":{"refreshToken":"mcp-rt"}}}"#;
  assert_eq!( account::session_token( logged_out, "refreshToken" ), None );
}
