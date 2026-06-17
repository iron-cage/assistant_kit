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
//! - **Prevention:** Structural test confirms `settings_file()` call exists in `account.rs`; four
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
//! | `ad_delete_also_removes_snapshots` | All 3 files exist → all 3 absent after delete |
//! | `ad_delete_succeeds_when_snapshots_absent` | Only credentials → delete succeeds, no error |
//! | `mre_bug_219_switch_account_stale_org_name` | switch_account() overrides org fields from {name}.json |
//! | `bug_mre_bug222_switch_account_reads_settings_snapshot` | structural: `settings_file()` present in account.rs |
//! | `mre_bug222_save_captures_model_to_settings_snapshot` | save() with model in settings.json → {name}.json has model |
//! | `mre_bug222_save_no_model_does_not_write_settings_snapshot` | save() with no model in settings.json → {name}.json not created |
//! | `mre_bug222_switch_account_restores_model_from_settings_snapshot` | switch_account() installs model from {name}.json into live settings |
//! | `mre_bug222_switch_account_clears_model_when_no_snapshot` | switch_account() absent snapshot → removes model from live settings |
//! | `test_ft11_025_other_machines_active_returns_others` | other_machines_active() returns foreign accounts; own marker excluded |
//! | `test_ft12_025_other_machines_active_empty_when_only_own` | other_machines_active() returns empty when only own marker or empty store |
//! | `ft10_set_session_model_preserves_existing_keys` | set_session_model() merges model into existing settings.json without losing other keys |
//! | `ft11_set_session_model_creates_file_when_absent` | set_session_model() creates settings.json when file is absent (dir exists) |
//! | `mre_bug258_set_session_model_creates_parent_dir_when_absent` | BUG-258: set_session_model() creates ~/.claude/ dir + file when dir is absent |
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

  account::save( "alice@test.com", &store, &paths, false, None, None, None, None ).unwrap();

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
/// The BUG-217 fix block (`account.rs` ~line 338) only inserts `emailAddress`. All other
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

  account::save( "alice@acme.com", &store, &paths, true, None, None, None, None ).unwrap();

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
// Fix Applied: structural assertion that account.rs contains at least one settings_file()
//   call, confirming the restore step is present.
// Prevention: if the restore step is removed from account.rs the count drops to 0 and
//   this test goes RED immediately.
// Pitfall: settings_file() appears in both save() and switch_account() after the BUG-222
//   fix; count >= 1 is the correct bound, not == 1.
fn bug_mre_bug222_switch_account_reads_settings_snapshot()
{
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );
  let count = content.matches( "settings_file()" ).count();
  assert!(
    count >= 1,
    "BUG-222: expected >=1 occurrence of 'settings_file()' in account.rs, found {count}",
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
  account::save( "alice@test.com", &store, &paths, false, None, None, None, None ).unwrap();

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
// Fix Applied: save() skips {name}.json creation when model is absent from
//   ~/.claude/settings.json — avoids orphan files for accounts with no model preference.
// Prevention: asserts {name}.json is NOT created when model key absent.
// Pitfall: the skip applies when the model key is absent; other keys in settings.json
//   are not captured — only model is a per-account preference (BUG-222 scope).
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
  account::save( "bob@test.com", &store, &paths, false, None, None, None, None ).unwrap();

  let snap_path = store.join( "bob@test.com.json" );
  assert!(
    !snap_path.exists(),
    "save() must NOT create {{name}}.json when model is absent from ~/.claude/settings.json",
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

/// BUG-225 MRE: `override_session_model_to_opus` upgrades Sonnet→Opus when settings has Sonnet.
///
/// # Root Cause (BUG-225)
/// `switch_account()` restores the snapshot model unconditionally. When the account's Sonnet
/// quota is < 20%, the restored Sonnet model leaves the session on an exhausted tier.
///
/// # Why Not Caught
/// No test covered save-with-Sonnet → deplete-Sonnet → switch → assert-session-model-opus.
///
/// # Fix Applied
/// `override_session_model_to_opus()` reads settings.json and overwrites Sonnet with Opus;
/// returns `true` when the override was applied.
///
/// # Prevention
/// This test asserts the write happens (return `true`) and the model in settings.json
/// changes to "claude-opus-4-6".
///
/// # Pitfall
/// Function is best-effort: if settings.json is missing, it creates a new object with
/// just "model": "claude-opus-4-6" — absence of settings is treated as Sonnet (model empty).
#[ doc = "bug_reproducer(BUG-225)" ]
#[ test ]
fn mre_bug225_override_session_model_to_opus_fires_when_sonnet()
{
  let tmp        = TempDir::new().unwrap();
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"model":"claude-sonnet-4-6","theme":"dark"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  let overrode = account::override_session_model_to_opus( &paths );

  assert!( overrode, "override must return true when model was Sonnet" );
  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let model = account::parse_string_field( &live, "model" );
  assert_eq!( model.as_deref(), Some( "opus" ), "model must be upgraded to opus shorthand" );
}

/// BUG-225 MRE: `override_session_model_to_opus` is a no-op when model is already Opus.
///
/// # Root Cause (BUG-225)
/// Same as above. This test verifies the inverse: when the snapshot already has Opus,
/// the override must not touch settings.json (returns `false`).
///
/// # Prevention
/// Ensures the function skips the write for already-correct models.
///
/// # Pitfall
/// A bug that unconditionally writes would fail this test by writing Opus over Opus
/// unnecessarily, but returning `true` — callers would emit spurious trace lines.
#[ doc = "bug_reproducer(BUG-225)" ]
#[ test ]
fn mre_bug225_override_session_model_to_opus_no_op_when_already_opus()
{
  let tmp        = TempDir::new().unwrap();
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"model":"opus"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  let overrode = account::override_session_model_to_opus( &paths );

  assert!( !overrode, "override must return false when model was already Opus" );
}

/// FT-20 MRE: `override_session_model_to_opus` handles Claude Code shorthand `"sonnet"` input
/// and writes shorthand `"opus"` (not full ID `"claude-opus-4-6"`). Also verifies BUG-286
/// fix: full-ID `"claude-opus-4-6"` is normalized to shorthand `"opus"` when model override fires.
///
/// # Root Cause (BUG-257)
/// `override_session_model_to_opus()` checked `current == "claude-sonnet-4-6"` but Claude Code
/// writes the shorthand `"sonnet"` to `~/.claude/settings.json`. The exact-string check never
/// matched production values — the session remained on Sonnet even when quota was exhausted.
/// Additionally, the write side used `"claude-opus-4-6"` (full ID) instead of `"opus"` shorthand.
///
/// # Root Cause (BUG-286)
/// `set_model::opus` writes `"claude-opus-4-6"` (full ID) to `settings.json`. When
/// `override_session_model_to_opus` ran next, gate `contains("sonnet") || is_empty()`
/// did not match `"claude-opus-4-6"` — full-ID form stayed in `settings.json` unmodified.
///
/// # Why Not Caught
/// BUG-225 tests pre-wrote the full ID `"claude-sonnet-4-6"` — not the shorthand
/// `"sonnet"` that Claude Code actually writes. The test passed while the real-world
/// path was always broken. BUG-286 was introduced when `set_model::opus` write path
/// used full ID; the `override_session_model_to_opus` read path was never updated.
///
/// # Fix Applied
/// BUG-257: read side `current == "claude-sonnet-4-6"` → `current.contains("sonnet")`;
///   write side `"claude-opus-4-6"` → `"opus"` shorthand.
/// BUG-286: gate extended with `|| current == "claude-opus-4-6"` to normalize full-ID opus.
///
/// # Prevention
/// Scenario 1 asserts BOTH return value AND written content. Scenario 2 guards the
/// full-ID sonnet path as a regression guard. Scenario 6 guards full-ID opus normalization.
///
/// # Pitfall
/// `contains("sonnet")` is intentionally broad — matches `"sonnet"`, `"claude-sonnet-4-6"`,
/// and any future sonnet variant. A `"sonnet"` substring in an opus ID would be a naming
/// regression in the Claude API, not a code concern here.
#[ doc = "bug_reproducer(BUG-257)" ]
#[ doc = "bug_reproducer(BUG-286)" ]
#[ test ]
fn mre_bug257_override_shorthand_alias()
{
  let tmp   = TempDir::new().unwrap();
  let paths = ClaudePaths::with_home( tmp.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();

  // Scenario 1: shorthand "sonnet" → must return true + write "opus"
  let settings = paths.settings_file();
  std::fs::write( &settings, r#"{"model":"sonnet"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( overrode, "BUG-257: override must fire for shorthand \"sonnet\" input" );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-6" ),
    "BUG-257: override must write shorthand \"opus\", not full ID; got: {content}",
  );

  // Scenario 2: full ID "claude-sonnet-4-6" still fires (regression guard)
  std::fs::write( &settings, r#"{"model":"claude-sonnet-4-6"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( overrode, "full ID claude-sonnet-4-6 must still fire override" );

  // Scenario 3: non-sonnet model "opus" → must NOT fire
  std::fs::write( &settings, r#"{"model":"opus"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( !overrode, "non-sonnet model must not trigger override" );

  // Scenario 4: absent model → must fire (empty string case)
  std::fs::write( &settings, r"{}" ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( overrode, "absent model field must trigger override (defaults to opus)" );

  // Scenario 5: non-sonnet model "haiku" → must NOT fire (Fix(BUG-286) regression guard)
  std::fs::write( &settings, r#"{"model":"haiku"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( !overrode, "BUG-286: haiku model must not trigger override" );

  // Scenario 6: full-ID "claude-opus-4-6" → must fire; normalize to shorthand "opus" (Fix(BUG-286))
  // BUG: `set_model::opus` writes "claude-opus-4-6" full ID to settings.json; gate
  //   `contains("sonnet") || is_empty()` did not match it, leaving "claude-opus-4-6"
  //   in settings.json rather than normalising to "opus" shorthand on next override call.
  std::fs::write( &settings, r#"{"model":"claude-opus-4-6"}"# ).unwrap();
  let overrode = account::override_session_model_to_opus( &paths );
  assert!( overrode, "BUG-286: full-ID \"claude-opus-4-6\" must trigger override to normalize to shorthand" );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-6" ),
    "BUG-286: override must write shorthand \"opus\", not full ID; got: {content}",
  );
}

/// `set_session_model()` writes the correct model ID or removes the key.
///
/// ## Scenarios
/// - `Some("claude-opus-4-6")` → writes `"model": "claude-opus-4-6"`
/// - `Some("claude-sonnet-4-6")` → writes `"model": "claude-sonnet-4-6"`
/// - `Some("claude-haiku-4-5-20251001")` → writes `"model": "claude-haiku-4-5-20251001"`
/// - `None` (default) → removes the `model` key entirely
///
/// ## Why This Test Exists
/// `set_session_model` is the exclusive mechanism for `set_model::` param — no
/// other code path writes arbitrary model IDs to `settings.json`. Testing the
/// 4 accepted values confirms write correctness and key removal.
#[ test ]
fn it_set_session_model_writes_and_removes()
{
  let tmp   = TempDir::new().unwrap();
  let paths = ClaudePaths::with_home( tmp.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let settings = paths.settings_file();

  // opus
  std::fs::write( &settings, r"{}" ).unwrap();
  account::set_session_model( &paths, Some( "claude-opus-4-6" ) );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( content.contains( "\"claude-opus-4-6\"" ), "set_session_model opus must write full ID; got: {content}" );

  // sonnet
  account::set_session_model( &paths, Some( "claude-sonnet-4-6" ) );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( content.contains( "\"claude-sonnet-4-6\"" ), "set_session_model sonnet must write full ID; got: {content}" );

  // haiku
  account::set_session_model( &paths, Some( "claude-haiku-4-5-20251001" ) );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( content.contains( "\"claude-haiku-4-5-20251001\"" ), "set_session_model haiku must write full ID; got: {content}" );

  // default (None) — removes key
  account::set_session_model( &paths, None );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( !content.contains( "\"model\"" ), "set_session_model None must remove model key; got: {content}" );
}

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

// ── Quota cache (Feature 033) ────────────────────────────────────────────────

/// AC-01: `write_quota_cache` writes `"cache"` key to `{name}.json` preserving existing fields.
///
/// Given: `alice@acme.com.json` containing `{"host":"wbox"}`
/// When: `write_quota_cache` called with `five_hour` utilization 14.0
/// Then: file contains both `"host":"wbox"` and `"cache"` with `fetched_at` + `five_hour.left_pct`
#[ test ]
fn cache_write_preserves_existing_fields()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "alice@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"host":"wbox","role":"dev"}"# ).unwrap();

  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 14.0, Some( "2026-06-07T12:00:00Z" ) ) ),
    Some( ( 25.0, None ) ),
    None,
  );

  let content = std::fs::read_to_string( &meta ).unwrap();
  assert!( content.contains( r#""host": "wbox""# ), "host preserved: {content}" );
  assert!( content.contains( r#""role": "dev""# ), "role preserved: {content}" );
  assert!( content.contains( r#""cache""# ), "cache key present: {content}" );
  assert!( content.contains( r#""fetched_at""# ), "fetched_at present: {content}" );
  assert!( content.contains( r#""left_pct""# ), "left_pct present: {content}" );
  assert!( content.contains( r#""five_hour""# ), "five_hour present: {content}" );
  assert!( content.contains( r#""seven_day""# ), "seven_day present: {content}" );
}

/// AC-02: `read_quota_cache` returns `None` when no cache exists.
///
/// Given: `{name}.json` with `{"host":"wbox"}` but no `"cache"` key
/// When: `read_quota_cache` called
/// Then: returns `None`
#[ test ]
fn cache_read_returns_none_when_absent()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "bob@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"host":"wbox"}"# ).unwrap();

  let result = claude_profile_core::account::read_quota_cache( store.path(), name );
  assert!( result.is_none(), "no cache key must return None" );
}

/// AC-02: `read_quota_cache` returns cached data when valid cache exists.
///
/// Given: `{name}.json` with a fully populated `"cache"` object
/// When: `read_quota_cache` called
/// Then: returns `Some(QuotaCacheEntry)` with all fields matching
#[ test ]
fn cache_read_returns_entry_when_present()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "carol@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"cache":{"fetched_at":"2026-06-07T10:00:00Z","status":"ok","five_hour":{"left_pct":86.0,"resets_at":"2026-06-07T15:00:00Z"},"seven_day":{"left_pct":42.5},"model_override":"opus","last_touch_at":"2026-06-07T09:55:00Z","touch_idle":false}}"# ).unwrap();

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "valid cache must return Some" );
  assert_eq!( entry.fetched_at, "2026-06-07T10:00:00Z" );
  let ( h5_util, h5_reset ) = entry.five_hour.expect( "five_hour must be Some" );
  assert!( ( h5_util - 86.0 ).abs() < f64::EPSILON, "five_hour utilization: {h5_util}" );
  assert_eq!( h5_reset.as_deref(), Some( "2026-06-07T15:00:00Z" ) );
  let ( d7_util, d7_reset ) = entry.seven_day.expect( "seven_day must be Some" );
  assert!( ( d7_util - 42.5 ).abs() < f64::EPSILON, "seven_day utilization: {d7_util}" );
  assert!( d7_reset.is_none(), "seven_day resets_at must be None" );
  assert!( entry.seven_day_sonnet.is_none(), "seven_day_sonnet must be None" );
  assert_eq!( entry.model_override.as_deref(), Some( "opus" ) );
  assert_eq!( entry.last_touch_at.as_deref(), Some( "2026-06-07T09:55:00Z" ) );
  assert_eq!( entry.touch_idle, Some( false ) );
}

/// AC-07: `write_quota_cache` round-trips through `read_quota_cache`.
///
/// Given: empty credential store
/// When: write then read
/// Then: read returns the same values written
#[ test ]
fn cache_write_read_roundtrip()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "rt@test.com";

  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 50.0, Some( "2026-06-07T18:00:00Z" ) ) ),
    None,
    Some( ( 90.0, Some( "2026-06-14T00:00:00Z" ) ) ),
  );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "written cache must be readable" );
  let ( h5, h5r ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( h5 - 50.0 ).abs() < f64::EPSILON );
  assert_eq!( h5r.as_deref(), Some( "2026-06-07T18:00:00Z" ) );
  assert!( entry.seven_day.is_none() );
  let ( sn, snr ) = entry.seven_day_sonnet.expect( "sonnet present" );
  assert!( ( sn - 90.0 ).abs() < f64::EPSILON );
  assert_eq!( snr.as_deref(), Some( "2026-06-14T00:00:00Z" ) );
}

/// `parse_iso_utc_secs` correctly converts known timestamps.
#[ test ]
fn parse_iso_utc_secs_known_values()
{
  // 2026-06-07T12:00:00Z = a known date, verify deterministic output.
  let secs = claude_profile_core::account::parse_iso_utc_secs( "2026-06-07T12:00:00Z" );
  assert!( secs.is_some(), "valid ISO must parse" );
  let s = secs.unwrap();
  // Cross-check: 2026-06-07 is day index from epoch; rough range 1780000000..1790000000
  assert!( s > 1_780_000_000, "must be in 2026 range: {s}" );
  assert!( s < 1_790_000_000, "must be in 2026 range: {s}" );

  // Invalid inputs return None.
  assert!( claude_profile_core::account::parse_iso_utc_secs( "short" ).is_none() );
  assert!( claude_profile_core::account::parse_iso_utc_secs( "not-a-date-at-all!" ).is_none() );
}

/// AC-05: `write_cache_string` persists a field in the cache sub-object.
#[ test ]
fn cache_field_string_persisted()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "field@test.com";
  // Pre-populate with cache.
  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 10.0, None ) ), None, None,
  );
  // Write model_override field.
  claude_profile_core::account::write_cache_string( store.path(), name, "model_override", "opus" );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  assert_eq!( entry.model_override.as_deref(), Some( "opus" ) );
  // Quota data must survive the field write.
  assert!( entry.five_hour.is_some(), "five_hour must survive write_cache_string" );
}

/// AC-06: `write_cache_bool` persists a boolean in the cache sub-object.
#[ test ]
fn cache_field_bool_persisted()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "bool@test.com";
  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 20.0, None ) ), None, None,
  );
  claude_profile_core::account::write_cache_bool( store.path(), name, "touch_idle", false );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  assert_eq!( entry.touch_idle, Some( false ) );
  assert!( entry.five_hour.is_some(), "five_hour must survive write_cache_bool" );
}

// ── Cache corner cases ────────────────────────────────────────────────────────

/// `read_quota_cache` returns `None` when `{name}.json` does not exist.
#[ test ]
fn cache_read_none_when_file_absent()
{
  let store = tempfile::tempdir().unwrap();
  let result = claude_profile_core::account::read_quota_cache( store.path(), "ghost@test.com" );
  assert!( result.is_none(), "absent file must return None" );
}

/// `read_quota_cache` returns `None` when `{name}.json` contains malformed JSON.
#[ test ]
fn cache_read_none_when_json_malformed()
{
  let store = tempfile::tempdir().unwrap();
  let meta  = store.path().join( "bad@test.com.json" );
  std::fs::write( &meta, "{not valid json!!!}" ).unwrap();
  let result = claude_profile_core::account::read_quota_cache( store.path(), "bad@test.com" );
  assert!( result.is_none(), "malformed JSON must return None" );
}

/// `read_quota_cache` returns `None` when cache object has no `fetched_at` key.
#[ test ]
fn cache_read_none_when_fetched_at_missing()
{
  let store = tempfile::tempdir().unwrap();
  let meta  = store.path().join( "notime@test.com.json" );
  std::fs::write( &meta, r#"{"cache":{"status":"ok","five_hour":{"left_pct":50.0}}}"# ).unwrap();
  let result = claude_profile_core::account::read_quota_cache( store.path(), "notime@test.com" );
  assert!( result.is_none(), "cache without fetched_at must return None" );
}

/// `write_quota_cache` preserves `model_override` written by a prior `write_cache_string`.
///
/// The quota write copies side-effect fields (`model_override`, `last_touch_at`, `touch_idle`)
/// from the previous cache object into the new one (lines 1207-1212 in account.rs).
#[ test ]
fn cache_write_preserves_prior_side_effects()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "preserve@test.com";
  // Step 1: write side-effect fields via cache field API.
  claude_profile_core::account::write_cache_string( store.path(), name, "model_override", "opus" );
  claude_profile_core::account::write_cache_string( store.path(), name, "last_touch_at", "2026-06-07T09:00:00Z" );
  claude_profile_core::account::write_cache_bool( store.path(), name, "touch_idle", true );
  // Step 2: write quota cache — must preserve all three side-effect fields.
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 30.0, Some( "2026-06-07T20:00:00Z" ) ) ),
    None,
    None,
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  assert_eq!( entry.model_override.as_deref(), Some( "opus" ), "model_override must survive" );
  assert_eq!( entry.last_touch_at.as_deref(), Some( "2026-06-07T09:00:00Z" ), "last_touch_at must survive" );
  assert_eq!( entry.touch_idle, Some( true ), "touch_idle must survive" );
  assert!( entry.five_hour.is_some(), "quota data must be present" );
}

/// `write_cache_field` creates `{name}.json` from scratch when file is absent.
#[ test ]
fn cache_field_creates_file_from_scratch()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "scratch@test.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  assert!( !meta.exists(), "pre-condition: file must not exist" );
  claude_profile_core::account::write_cache_string( store.path(), name, "model_override", "sonnet" );
  assert!( meta.exists(), "write_cache_string must create file" );
  let content = std::fs::read_to_string( &meta ).unwrap();
  assert!( content.contains( r#""model_override": "sonnet""# ), "field must be in file: {content}" );
  // read_quota_cache returns None because no fetched_at.
  assert!(
    claude_profile_core::account::read_quota_cache( store.path(), name ).is_none(),
    "cache without fetched_at must return None even after write_cache_field",
  );
}

/// Second `write_quota_cache` replaces first period data.
#[ test ]
fn cache_write_second_replaces_first()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "overwrite@test.com";
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 10.0, Some( "2026-06-07T12:00:00Z" ) ) ),
    None,
    None,
  );
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 90.0, Some( "2026-06-07T18:00:00Z" ) ) ),
    Some( ( 50.0, None ) ),
    None,
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  let ( h5, h5r ) = entry.five_hour.expect( "five_hour must be present" );
  assert!( ( h5 - 90.0 ).abs() < f64::EPSILON, "five_hour must be from second write: {h5}" );
  assert_eq!( h5r.as_deref(), Some( "2026-06-07T18:00:00Z" ) );
  assert!( entry.seven_day.is_some(), "seven_day from second write must be present" );
}

/// All three periods written and read back simultaneously.
#[ test ]
fn cache_write_read_all_three_periods()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "all3@test.com";
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 14.0, Some( "2026-06-07T12:00:00Z" ) ) ),
    Some( ( 25.0, Some( "2026-06-14T00:00:00Z" ) ) ),
    Some( ( 100.0, None ) ),
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  let ( h5, _ ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( h5 - 14.0 ).abs() < f64::EPSILON );
  let ( d7, _ ) = entry.seven_day.expect( "seven_day present" );
  assert!( ( d7 - 25.0 ).abs() < f64::EPSILON );
  let ( sn, sn_r ) = entry.seven_day_sonnet.expect( "sonnet present" );
  assert!( ( sn - 100.0 ).abs() < f64::EPSILON, "100.0 utilization boundary" );
  assert!( sn_r.is_none(), "sonnet resets_at must be None" );
}

/// `chrono_now_utc` output is parseable by `parse_iso_utc_secs` (round-trip).
#[ test ]
fn chrono_now_utc_parse_roundtrip()
{
  let ts   = claude_profile_core::account::chrono_now_utc();
  let secs = claude_profile_core::account::parse_iso_utc_secs( &ts )
    .expect( "chrono_now_utc output must be parseable" );
  let now  = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();
  assert!(
    now.abs_diff( secs ) <= 2,
    "round-trip must be within 2 seconds of wall clock: now={now}, parsed={secs}",
  );
}

/// `write_quota_cache` gracefully handles malformed existing `{name}.json`.
///
/// When the file contains invalid JSON, `serde_json::from_str` returns Err
/// and the code falls back to an empty object. The cache is written to a fresh
/// JSON — non-cache fields (host, role) in the malformed file are lost.
#[ test ]
fn cache_write_recovers_from_malformed_json()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "recover@test.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, "NOT VALID JSON AT ALL" ).unwrap();
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 45.0, None ) ),
    None,
    None,
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable after recovery" );
  let ( h5, _ ) = entry.five_hour.expect( "five_hour must be present" );
  assert!( ( h5 - 45.0 ).abs() < f64::EPSILON, "five_hour utilization: {h5}" );
}

// ── set_session_model ─────────────────────────────────────────────────────────

/// FT-10 (AC-10): `set_session_model()` preserves all pre-existing `settings.json` keys.
///
/// A write with `model_id = Some("claude-opus-4-6")` must NOT remove other keys
/// such as `theme` or `autoUpdaterStatus`.
#[ test ]
fn ft10_set_session_model_preserves_existing_keys()
{
  let tmp   = TempDir::new().unwrap();
  let dot   = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write(
    dot.join( "settings.json" ),
    r#"{"theme":"dark","autoUpdaterStatus":"disabled"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_model( &paths, Some( "claude-opus-4-6" ) );

  let content = std::fs::read_to_string( dot.join( "settings.json" ) )
    .expect( "settings.json must exist after set_session_model" );
  assert!(
    content.contains( "\"model\"" ) && content.contains( "claude-opus-4-6" ),
    "settings.json must contain the written model, got: {content}",
  );
  assert!(
    content.contains( "\"theme\"" ) && content.contains( "dark" ),
    "settings.json must preserve `theme` key, got: {content}",
  );
  assert!(
    content.contains( "\"autoUpdaterStatus\"" ) && content.contains( "disabled" ),
    "settings.json must preserve `autoUpdaterStatus` key, got: {content}",
  );
}

/// FT-11 (AC-11): `set_session_model()` creates `settings.json` when the file is absent.
///
/// When `~/.claude/settings.json` does not exist, `set_session_model()` creates it
/// containing only the requested `model` key.
#[ test ]
fn ft11_set_session_model_creates_file_when_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  // settings.json intentionally absent.

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_model( &paths, Some( "claude-opus-4-6" ) );

  let settings = dot.join( "settings.json" );
  assert!( settings.exists(), "set_session_model must create settings.json when absent" );
  let content = std::fs::read_to_string( &settings )
    .expect( "settings.json must be readable" );
  assert!(
    content.contains( "\"model\"" ) && content.contains( "claude-opus-4-6" ),
    "created settings.json must contain the requested model, got: {content}",
  );
}

/// MRE for BUG-258: `set_session_model()` silently failed when `~/.claude/` dir absent.
///
/// ## Root Cause
/// `set_session_model()` called `fs::write(path, ...)` without first ensuring the
/// parent directory existed. When `~/.claude/` was absent, `fs::write` failed with
/// `NotFound`; `let _` silently discarded the error. The model was not written,
/// violating AC-01/AC-02/AC-03 for the `.usage` invocation path.
///
/// ## Why Not Caught
/// FT-11 tests the case where the file is absent but the directory exists (callers
/// always created the dir manually). No test started without `~/.claude/` at all.
///
/// ## Fix Applied
/// `set_session_model()` now calls `create_dir_all(path.parent())` before `fs::write`.
///
/// ## Prevention
/// Precondition `assert!(!dot.exists())` confirms the directory is truly absent —
/// if the fixture accidentally creates it, the test would be a false negative.
///
/// ## Pitfall
/// Unit test callers always pass `ClaudePaths::with_home(tmp.path())` with an explicit
/// `TempDir`, so they must NOT call `create_dir_all` on `~/.claude/` when testing this path.
#[ doc = "bug_reproducer(BUG-258)" ]
#[ test ]
fn mre_bug258_set_session_model_creates_parent_dir_when_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  // Precondition: ~/.claude/ must NOT exist.
  assert!(
    !dot.exists(),
    "test precondition: ~/.claude/ must not exist before calling set_session_model",
  );

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_model( &paths, Some( "claude-opus-4-6" ) );

  let settings = dot.join( "settings.json" );
  assert!(
    settings.exists(),
    "set_session_model must create ~/.claude/ and settings.json when parent dir absent",
  );
  let content = std::fs::read_to_string( &settings )
    .expect( "settings.json must be readable after set_session_model creates parent dir" );
  assert!(
    content.contains( "\"model\"" ) && content.contains( "claude-opus-4-6" ),
    "settings.json must contain the requested model, got: {content}",
  );
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

  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "user@host1" ) ).unwrap();
  let owner = account::read_owner( &store, "alice@test.com" );
  assert_eq!( owner, "user@host1", "FT-01: save() must write owner to {{name}}.json; got: {owner:?}" );

  // Re-save from a different identity — owner must be overwritten.
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "user@host2" ) ).unwrap();
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
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "other@remote" ) ).unwrap();

  // Unclaim: write empty owner.
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "" ) ).unwrap();
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
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "alice@host1" ) ).unwrap();

  // Background save with owner: None — simulates refresh_account_token() path.
  account::save( "alice@test.com", &store, &paths, false, None, None, None, None ).unwrap();

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

  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "" ) ).unwrap();
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

  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "alice@host1" ) ).unwrap();
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "" ) ).unwrap();
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
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( &identity ) ).unwrap();
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
  account::save( "alice@test.com", &store, &paths, false, None, None, None, Some( "" ) ).unwrap();

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
  account::save( "new@test.com", &store, &paths, false, None, None, None, None ).unwrap();

  let owner = account::read_owner( &store, "new@test.com" );
  assert_eq!(
    owner, "",
    "CC-6: background save on new account must not create owner field; got: {owner:?}",
  );
  assert!( account::is_owned( &owner ), "CC-6: absent owner must pass is_owned() gate" );
}
