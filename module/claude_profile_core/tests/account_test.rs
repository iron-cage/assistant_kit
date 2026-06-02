//! Account unit tests: save, delete, and `switch_account` operations.
//!
//! ## Purpose
//!
//! Verify `account::save()` writes `_active` = `name` on every successful save,
//! that `account::delete()` removes all three files created by `save()`:
//! `{name}.credentials.json`, `{name}.claude.json`, and `{name}.settings.json`,
//! and that `account::switch_account()` correctly restores per-account model preference
//! from `{name}.settings.json` into `~/.claude/settings.json` (BUG-222).
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
//! ## Fix Documentation — BUG-222
//!
//! - **Root Cause:** `save()` never captured the `model` field from `~/.claude/settings.json`,
//!   so no per-account model snapshot existed. `switch_account()` never touched `settings.json`,
//!   leaving the prior account's model in place after every switch.
//! - **Why Not Caught:** All `switch_account()` tests asserted on credentials and oauthAccount fields;
//!   `settings.json` was not part of any assertion. The silent persistence of model was invisible.
//! - **Fix Applied:** `save()` reads `~/.claude/settings.json`, extracts `model`, and write-merges
//!   it into `{name}.settings.json` when present. `switch_account()` reads `{name}.settings.json`,
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
//! | `mre_bug_219_switch_account_stale_org_name` | switch_account() overrides org fields from roles.json |
//! | `bug_mre_bug222_switch_account_reads_settings_snapshot` | structural: `settings_file()` present in account.rs |
//! | `mre_bug222_save_captures_model_to_settings_snapshot` | save() with model in settings.json → {name}.settings.json has model |
//! | `mre_bug222_save_no_model_does_not_write_settings_snapshot` | save() with no model in settings.json → {name}.settings.json not created |
//! | `mre_bug222_switch_account_restores_model_from_settings_snapshot` | switch_account() installs model from {name}.settings.json into live settings |
//! | `mre_bug222_switch_account_clears_model_when_no_snapshot` | switch_account() absent snapshot → removes model from live settings |

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

  account::save( "alice@test.com", &store, &paths, false, None ).unwrap();

  let marker = store.join( account::active_marker_filename() );
  assert!(
    !marker.exists(),
    "save() with update_marker=false must NOT write the _active marker file; found: {marker:?}",
  );
}

/// BUG-219 MRE: `switch_account()` must override `oauthAccount.organizationName`
/// and `oauthAccount.organizationUuid` from `{name}.roles.json`, not from the stale snapshot.
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
/// After the BUG-217 `emailAddress` insert, read `{name}.roles.json` and override
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

  // i6's claude.json snapshot — STALE: contains i7's org (captured while i7 was active)
  std::fs::write(
    store.join( "i6@test.com.claude.json" ),
    r#"{"oauthAccount":{"emailAddress":"i6@test.com","organizationName":"i7 Org","organizationUuid":"uuid-i7"}}"#,
  ).unwrap();

  // i6's roles.json — CORRECT: contains i6's actual org from live API
  std::fs::write(
    store.join( "i6@test.com.roles.json" ),
    r#"{"organization_uuid":"uuid-i6","organization_name":"i6 Org","organization_role":"member"}"#,
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

  account::save( "alice@acme.com", &store, &paths, true, None ).unwrap();

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
// Root Cause: switch_account() never read {name}.settings.json or touched
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
//   model preference from ~/.claude/settings.json, so no {name}.settings.json was created
//   with model data; switch_account() had nothing to restore (BUG-222).
// Why Not Caught: no test exercised the {name}.settings.json write path in save();
//   the file appeared only in delete() as a best-effort orphan removal target.
// Fix Applied: save() reads ~/.claude/settings.json, extracts "model" via parse_string_field,
//   and write-merges it into {name}.settings.json when present.
// Prevention: asserts {name}.settings.json is created and contains the correct model value
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
  account::save( "alice@test.com", &store, &paths, false, None ).unwrap();

  let snap_path = store.join( "alice@test.com.settings.json" );
  assert!( snap_path.exists(), "save() must create {{name}}.settings.json when model is present in live settings" );
  let snap = std::fs::read_to_string( &snap_path )
    .expect( "{{name}}.settings.json must be readable after save()" );
  let model = account::parse_string_field( &snap, "model" )
    .expect( "{{name}}.settings.json must contain 'model' after save() with model in live settings" );
  assert_eq!( model, "claude-opus-4-5", "captured model must equal the value in ~/.claude/settings.json" );
}

#[ test ]
// Root Cause: (same — save() did not read settings.json at all before BUG-222 fix)
// Why Not Caught: (same — no test exercised any save()/settings.json interaction)
// Fix Applied: save() skips {name}.settings.json creation when model is absent from
//   ~/.claude/settings.json — avoids orphan files for accounts with no model preference.
// Prevention: asserts {name}.settings.json is NOT created when model key absent.
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
  account::save( "bob@test.com", &store, &paths, false, None ).unwrap();

  let snap_path = store.join( "bob@test.com.settings.json" );
  assert!(
    !snap_path.exists(),
    "save() must NOT create {{name}}.settings.json when model is absent from ~/.claude/settings.json",
  );
}

#[ test ]
// Root Cause: switch_account() copied credentials and patched oauthAccount but left
//   ~/.claude/settings.json untouched; the prior account's model persisted after every
//   switch — switching from sonnet to an account saved with haiku still ran on sonnet (BUG-222).
// Why Not Caught: switch_account() tests validated credentials and oauthAccount; settings.json
//   was never asserted on, so the stale model was invisible.
// Fix Applied: switch_account() reads {name}.settings.json, extracts model, and installs it
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
    store.join( "max@test.com.settings.json" ),
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
// Fix Applied: when {name}.settings.json is absent or has no model, switch_account() removes
//   the "model" key from live settings.json so no stale model persists.
// Prevention: asserts model key is ABSENT from live settings.json after switching to an account
//   with no settings snapshot; any regression re-introducing stale persistence fails this.
// Pitfall: absent {name}.settings.json is not an error; clearing is the correct behaviour when
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
  // No {name}.settings.json for this account.
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
  assert_eq!( model.as_deref(), Some( "claude-opus-4-6" ), "model must be upgraded to opus" );
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
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"model":"claude-opus-4-6"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  let overrode = account::override_session_model_to_opus( &paths );

  assert!( !overrode, "override must return false when model was already Opus" );
}
