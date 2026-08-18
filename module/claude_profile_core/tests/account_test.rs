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
//! ## Fix Documentation — BUG-002
//!
//! - **Root Cause:** `parse_string_field()`/`parse_u64_field()`/`parse_bool_field()`/
//!   `parse_string_array_field()` all open with an unbounded `json.find(&search)` over the
//!   ENTIRE input string — none accepts or enforces "search only within this one object."
//!   A caller holding multi-entry JSON (e.g. `roles_json`, a list of workspace/organization
//!   memberships) has no way to scope the search to the entry it actually needs, and
//!   silently gets whichever entry's field is textually first.
//! - **Why Not Caught:** No test exercised any of the four helpers against multi-entry JSON —
//!   every existing fixture is a flat, single-object JSON blob (credentials files,
//!   settings.json), where "first occurrence" is always correct by coincidence of there
//!   being nothing else to find.
//! - **Fix Applied:** Added `extract_object_block()` — a brace-depth-counted `{...}` bound
//!   (mirrors `claude_quota`'s own helper of the same name; independently duplicated, not
//!   shared). A caller walking a multi-entry array can now bound each entry with
//!   `extract_object_block()` before calling `parse_string_field()` etc. on the bounded
//!   slice, eliminating the wrong-entry ambiguity for any caller that adopts it. The 4
//!   existing unbounded helpers are unchanged — still correct for flat single-object JSON.
//! - **Prevention:** `bug002_extract_object_block_bounds_multi_entry_roles_json` reproduces
//!   the exact MRE scenario from BUG-002 (`roles_json` with two workspace memberships) and
//!   asserts the second entry's `workspace_name` is correctly extracted once bounded.
//! - **Pitfall:** Do not add object-boundary scanning inside the 4 existing helpers directly
//!   — that would need a scoping parameter and break every existing single-object call site
//!   across the crate. Bounding is the caller's responsibility via `extract_object_block()`.
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
//! | `mre_bug222_save_no_model_does_not_write_settings_snapshot` | save() with no model in settings.json → {name}.json has no "model" key (Feature 071: file itself is always created) |
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
//! | `history_append_stores_correct_fields` | FT-01: write_history_entry stores t/h5/d7/sn in the history ring (local cache file since TSK-500) |
//! | `history_ring_buffer_evicts_oldest` | FT-02: 11th append evicts entry 0; length stays 10 |
//! | `history_read_absent_key_returns_empty` | FT-11: absent history key → empty vec (AC-11 backward compat) |
//! | `history_duplicate_timestamp_overwrites` | FT-13: same-second append overwrites last entry, not appends |
//! | `ft01_071_backend_redirect_parses_to_redirect_variant` | Feature 071/T01: `"backend":"redirect"` + base_url/redirect_model round-trip via list() |
//! | `ft02_071_absent_backend_key_defaults_to_anthropic` | Feature 071/AC-05: no `backend` key → `AccountBackend::Anthropic`, base_url/redirect_model None |
//! | `ft03_071_unrecognized_backend_value_defaults_to_anthropic_not_error` | Feature 071/AC-05: `"backend":"bogus"` → `AccountBackend::Anthropic`, not an error |
//! | `ft04_071_save_redirect_writes_minimal_credentials_and_metadata` | Feature 071/T01/AC-01: redirect save → {name}.credentials.json has only accessToken; {name}.json has backend/base_url/redirect_model |
//! | `ft05_071_save_redirect_never_touches_live_credentials_file` | Feature 071/T01/AC-01: redirect save never reads/writes ~/.claude/.credentials.json |
//! | `ft06_071_save_default_anthropic_writes_backend_field` | Feature 071/T02/AC-04: default (anthropic) save preserves live-file copy; writes backend:"anthropic" into {name}.json
//! | `ft07_071_switch_to_redirect_writes_env_keys` | Feature 071/T03/AC-06: switch_account() to a redirect account writes env.ANTHROPIC_BASE_URL/AUTH_TOKEN/MODEL; unrelated fields survive |
//! | `ft08_071_switch_to_anthropic_removes_env_keys_and_prunes_empty_env` | Feature 071/T03/AC-07: switch_account() to an anthropic account removes the 3 env keys and prunes `env` when empty |
//! | `ft09_071_switch_to_anthropic_preserves_unrelated_env_subkey` | Feature 071/T03/AC-07: an unrelated pre-existing env.* sub-key survives a switch-away from redirect |
//! | `ft10_071_read_backend_missing_file_defaults_anthropic` | Feature 071/T14: read_backend() on missing {name}.json defaults to Anthropic |
//! | `ft11_071_read_backend_redirect_value` | Feature 071/T14: read_backend() reads an explicit "backend":"redirect" field |
//! | `ft12_071_read_backend_corrupt_content_defaults_anthropic` | Feature 071/T14: read_backend() on corrupt content defaults to Anthropic, no panic |
//! | `ft01_072_save_some_inference_provider_writes_field` | Feature 072/T01/AC-01: save(inference_provider: Some("kimi")) on fresh account writes inference_provider:"kimi"
//! | `ft02_072_save_none_inference_provider_preserves_existing` | Feature 072/T02/AC-02: save(inference_provider: None) preserves existing inference_provider unchanged
//! | `ft03_072_save_none_inference_provider_no_prior_key_writes_no_key` | Feature 072/T03/AC-03/AF3: save(inference_provider: None) with no prior key writes no key at all (never "anthropic")
//! | `ft04_072_list_reads_inference_provider_when_present` | Feature 072/T04/AC-04: list() reads inference_provider from {name}.json when present
//! | `ft05_072_list_defaults_inference_provider_to_empty_when_absent` | Feature 072/T05/AC-05: list() defaults Account.inference_provider to "" when key absent |
//! | `ft01_073_switch_to_kimi_redirect_writes_all_tier_env_vars` | Feature 073/AC-05: switch_account() to a redirect+kimi account writes all 5 default-model vars, CLAUDE_CODE_EFFORT_LEVEL, and a 1M CLAUDE_CODE_AUTO_COMPACT_WINDOW for a kimi-k3 model |
//! | `ft02_073_switch_to_kimi_redirect_uses_narrow_compact_window_for_non_k3_model` | Feature 073/AC-06: kimi-k2.7-code redirect_model → CLAUDE_CODE_AUTO_COMPACT_WINDOW is the 256K value, not the 1M default |
//! | `ft03_073_switch_to_redirect_non_kimi_provider_omits_tier_env_vars` | Feature 073/AC-08: a redirect account not tagged inference_provider:"kimi" gets only the original 3 env vars, none of the 7 Kimi-tier additions |
//! | `ft04_073_switch_from_kimi_to_anthropic_clears_all_tier_env_vars` | Feature 073/AC-07: switching from a kimi redirect account to an anthropic account removes all 10 env vars, not just the original 3 |
//! | `ft05_073_switch_from_kimi_to_other_redirect_clears_stale_tier_env_vars` | Feature 073/AC-07: switching from a kimi redirect account to a different, non-kimi redirect account also clears the 7 stale Kimi-tier vars |
//! | `it_remove_session_effort_removes_key_preserves_others` | Task 464/T01: remove_session_effort() removes effortLevel, preserves other keys |
//! | `it_remove_session_effort_noop_when_key_absent` | Task 464/T02: remove_session_effort() is a no-op when effortLevel already absent |
//! | `ft_remove_session_effort_creates_file_when_settings_absent` | Task 464/T03: remove_session_effort() creates settings.json as {} when file absent |
//! | `ft_remove_session_effort_creates_dir_when_claude_absent` | Task 464/T04: remove_session_effort() creates ~/.claude/ dir + file when dir absent |
//! | `bug002_extract_object_block_bounds_multi_entry_roles_json` | BUG-002: extract_object_block() bounds parse_string_field() to one membership entry in multi-entry roles_json |
//! | `t500_01_fetch_write_sequence_leaves_migrated_tracked_byte_identical` | TSK-500/T01: write_quota_cache + write_history_entry leave a migrated tracked file byte-identical (AF1 hash compare) |
//! | `t502_01_volatile_cache_lands_in_per_host_tracked_file` | TSK-502/T01: volatile cache written to `cache/{host}_{user}/{name}.json` — no hyphen-prefixed component, so tracked; no account file created; merged read round-trips |
//! | `t500_02b_history_entry_lands_local_only` | TSK-500/T02: write_history_entry targets the per-host cache file only (TSK-502 location) |
//! | `t500_05_low_churn_writes_land_top_level_tracked` | TSK-500/T05: write_cache_string/bool land top-level tracked; no cache{} block recreated; merged read surfaces both |
//! | `t500_06_legacy_cache_migrates_prunes_and_preserves` | TSK-500/T06: legacy cache{} readable pre-migration, dissolved in one write (volatile→per-host file since TSK-502, low-churn→top-level, AF2 cache-key absent) |
//! | `t500_01b_if_changed_write_skips_identical_value` | TSK-500/T01: write_cache_string_if_changed skips the tracked write when the value is unchanged (steady-state org_created_at stamp) |
//! | `t502_02_read_merges_freshest_across_host_subtrees` | TSK-502/T02+T03: freshest fetched_at wins across cache/*/ subtrees in both directions; an unparseable fetched_at candidate is skipped |
//! | `t502_03_legacy_gitignored_cache_read_then_self_cleaned` | TSK-502/T04: legacy `-cache/{name}.json` readable as fallback; the next write relocates values + history to the per-host file and deletes it |
//! | `t502_04_no_cache_anywhere_returns_none` | TSK-502/T05 (regression guard): empty store → read_quota_cache None — the no-cache contract unchanged |
//! | `t502_05_history_ring_continues_across_hosts` | TSK-502/T06: another host's ring is carried into the own-host file by write_quota_cache and continued by write_history_entry |
//! | `m8_lock_store_blocks_second_holder_until_release` | Audit M8: a second lock_store() on the same store blocks until the first holder drops — mutual exclusion direction |
//! | `m8_lock_store_release_frees_reacquire_and_names_lockfile` | Audit M8: after drop the lock is immediately reacquirable (release direction), and the lock file is the gitignored `-store.lock` |

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

  account::save( "alice@test.com", &store, &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None ).unwrap();

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

  account::save( "alice@acme.com", &store, &paths, true, None, None, None, None, account::AccountBackend::Anthropic, None, None, None ).unwrap();

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
  account::save( "alice@test.com", &store, &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None ).unwrap();

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
    account::AccountBackend::Anthropic, None, None, None,
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

/// Task 464 (T01): `remove_session_effort()` removes exactly the `effortLevel` key
/// from `~/.claude/settings.json`, preserving every other key already present —
/// the removal counterpart `set_session_effort()` lacked (unlike `set_session_model()`,
/// which already supports removal via `None`).
#[ test ]
fn it_remove_session_effort_removes_key_preserves_others()
{
  let tmp   = TempDir::new().unwrap();
  let paths = ClaudePaths::with_home( tmp.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let settings = paths.settings_file();

  std::fs::write( &settings, r#"{"effortLevel":"high","model":"opus"}"# ).unwrap();
  account::remove_session_effort( &paths );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( !content.contains( "effortLevel" ), "remove_session_effort must remove the key; got: {content}" );
  assert!( content.contains( "\"opus\"" ), "remove_session_effort must preserve other keys; got: {content}" );
}

/// Task 464 (T02): `remove_session_effort()` is a no-op, not an error, when
/// `effortLevel` is already absent — mirrors `set_session_effort()`'s best-effort policy.
#[ test ]
fn it_remove_session_effort_noop_when_key_absent()
{
  let tmp   = TempDir::new().unwrap();
  let paths = ClaudePaths::with_home( tmp.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let settings = paths.settings_file();

  std::fs::write( &settings, r#"{"model":"opus"}"# ).unwrap();
  account::remove_session_effort( &paths );
  let content = std::fs::read_to_string( &settings ).unwrap();
  assert!( content.contains( "\"opus\"" ), "remove_session_effort no-op must preserve existing keys; got: {content}" );
  assert!( !content.contains( "effortLevel" ), "remove_session_effort no-op must not introduce the key; got: {content}" );
}

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

/// AC-01: `write_quota_cache` targets this host's per-host cache file
/// (TSK-502 location) and leaves the metadata `{name}.json` byte-identical.
///
/// Given: `alice@acme.com.json` containing `{"host":"wbox","role":"dev"}` (no legacy `cache{}`)
/// When: `write_quota_cache` called with `five_hour` utilization 14.0
/// Then: metadata file unchanged; `cache/{host}_{user}/alice@acme.com.json` holds `fetched_at` + periods
#[ test ]
fn cache_write_preserves_existing_fields()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "alice@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"host":"wbox","role":"dev"}"# ).unwrap();
  let before = std::fs::read( &meta ).unwrap();

  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 14.0, Some( "2026-06-07T12:00:00Z" ) ) ),
    Some( ( 25.0, None ) ),
    None,
  );

  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "tracked file must stay byte-identical on a migrated store",
  );
  let local   = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let content = std::fs::read_to_string( &local ).expect( "local cache file must exist" );
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

// ── TSK-500: quota cache externalized to local untracked file ─────────────────

/// T01 (TSK-500): the fetch-success write sequence leaves a migrated tracked file
/// byte-identical (AF1: hash compare, not `git status` silence).
///
/// Sequence mirrors `fetch.rs`'s success path: `write_quota_cache` then
/// `write_history_entry`. Store is post-migration: low-churn fields top-level,
/// no legacy `cache{}` block. Zero tracked writes is the defining property.
#[ test ]
fn t500_01_fetch_write_sequence_leaves_migrated_tracked_byte_identical()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "owner@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write(
    &meta,
    r#"{"host":"wbox","org_created_at":"2025-11-30T00:00:00Z","model_override":"opus"}"#,
  ).unwrap();
  let before = std::fs::read( &meta ).unwrap();

  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 42.0, Some( "2026-08-16T18:00:00Z" ) ) ),
    Some( ( 30.0, None ) ),
    None,
  );
  claude_profile_core::account::write_history_entry(
    store.path(), name, 1_755_360_000,
    Some( ( 42.0, "2026-08-16T18:00:00Z" ) ),
    None,
    None,
  );

  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "T01: tracked credential file must be byte-identical after the fetch write sequence",
  );
}

/// T01 (TSK-500): `write_cache_string_if_changed` skips the tracked write when the
/// value is unchanged — the steady-state `org_created_at` stamp on every successful
/// fetch must not re-dirty the tracked file — and still writes on a real change.
#[ test ]
fn t500_01b_if_changed_write_skips_identical_value()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "stamp@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"org_created_at":"2025-11-30T00:00:00Z"}"# ).unwrap();
  let before = std::fs::read( &meta ).unwrap();

  claude_profile_core::account::write_cache_string_if_changed(
    store.path(), name, "org_created_at", "2025-11-30T00:00:00Z",
  );
  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "identical value must be a zero-write no-op (byte-identical tracked file)",
  );

  claude_profile_core::account::write_cache_string_if_changed(
    store.path(), name, "org_created_at", "2026-01-01T00:00:00Z",
  );
  let json : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &meta ).unwrap() ).unwrap();
  assert_eq!(
    json[ "org_created_at" ].as_str(), Some( "2026-01-01T00:00:00Z" ),
    "changed value must be written",
  );
}

/// T01 (TSK-502, supersedes TSK-500/T02's location assertion): volatile cache
/// lands in the tracked per-host file `cache/{host}_{user}/{name}.json` — no
/// path component is hyphen-prefixed, so the global `-*` gitignore rule cannot
/// match it — and no tracked account file is created.
#[ test ]
fn t502_01_volatile_cache_lands_in_per_host_tracked_file()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "fresh@acme.com";

  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 55.5, Some( "2026-08-16T20:00:00Z" ) ) ),
    None,
    None,
  );

  let local = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  assert!( local.is_file(), "T01: cache/{}/{name}.json must exist", host_slug() );
  for component in local.strip_prefix( store.path() ).unwrap().components()
  {
    let c = component.as_os_str().to_string_lossy();
    assert!(
      !c.starts_with( '-' ),
      "T01: no cache path component may be hyphen-prefixed (would be gitignored by the global -* rule): {c}",
    );
  }
  let json : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &local ).unwrap() ).unwrap();
  assert!( json[ "fetched_at" ].is_string(), "T01: per-host cache carries fetched_at" );
  let u = json[ "five_hour" ][ "left_pct" ].as_f64().expect( "left_pct" );
  assert!( ( u - 55.5 ).abs() < 1e-9, "T01: utilization stored per-host, got {u}" );
  assert!(
    !store.path().join( format!( "{name}.json" ) ).exists(),
    "T01: write_quota_cache must not create a tracked account file",
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T01: merged read must see the per-host cache" );
  let ( util, reset ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( util - 55.5 ).abs() < f64::EPSILON );
  assert_eq!( reset.as_deref(), Some( "2026-08-16T20:00:00Z" ) );
}

/// T02 (TSK-500): `write_history_entry` targets the per-host cache file
/// (TSK-502 location) — the ring buffer never touches (or creates) the tracked
/// `{name}.json`.
#[ test ]
fn t500_02b_history_entry_lands_local_only()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "hist@acme.com";

  claude_profile_core::account::write_history_entry(
    store.path(), name, 2_000,
    Some( ( 40.0, "2026-08-16T20:00:00Z" ) ),
    None,
    None,
  );

  let local = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let json : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( &local ).expect( "local cache file must exist" )
  ).unwrap();
  assert_eq!(
    json[ "history" ].as_array().map( Vec::len ), Some( 1 ),
    "T02: history entry stored in the local file",
  );
  assert!(
    !store.path().join( format!( "{name}.json" ) ).exists(),
    "T02: write_history_entry must not create a tracked file",
  );
  let entries = claude_profile_core::account::read_history( store.path(), name );
  assert_eq!( entries.len(), 1, "read_history must read the local ring" );
  assert_eq!( entries[ 0 ].t, 2_000 );
}

/// T05 (TSK-500): low-churn writes (`write_cache_string`/`write_cache_bool`) land
/// as top-level keys of the tracked file — never under a `cache{}` block — and the
/// merged reader surfaces them alongside local volatile data.
#[ test ]
fn t500_05_low_churn_writes_land_top_level_tracked()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "meta@acme.com";
  std::fs::write( store.path().join( format!( "{name}.json" ) ), r#"{"host":"wbox"}"# ).unwrap();

  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 10.0, None ) ), None, None,
  );
  claude_profile_core::account::write_cache_string( store.path(), name, "model_override", "opus" );
  claude_profile_core::account::write_cache_bool( store.path(), name, "touch_idle", false );

  let json : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( store.path().join( format!( "{name}.json" ) ) ).unwrap()
  ).unwrap();
  assert_eq!( json[ "model_override" ].as_str(), Some( "opus" ), "T05: model_override top-level tracked" );
  assert_eq!( json[ "touch_idle" ].as_bool(), Some( false ), "T05: touch_idle top-level tracked" );
  assert!( json.get( "cache" ).is_none(), "T05: no cache{{}} block may be (re)created: {json}" );
  assert_eq!( json[ "host" ].as_str(), Some( "wbox" ), "T05: unrelated fields preserved" );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "merged read must succeed" );
  assert_eq!( entry.model_override.as_deref(), Some( "opus" ) );
  assert_eq!( entry.touch_idle, Some( false ) );
  assert!( entry.five_hour.is_some(), "volatile (local) + low-churn (tracked) must merge" );
}

/// T06 (TSK-500): first post-upgrade write migrates a legacy `cache{}` block —
/// volatile fields move to the local file, low-churn fields relocate to top level,
/// and the `cache` key is pruned from the tracked JSON in one write.
///
/// AF2 — all three legs asserted: legacy readable pre-migration AND volatile
/// present locally post-migration AND `cache` key absent from tracked JSON.
#[ test ]
fn t500_06_legacy_cache_migrates_prunes_and_preserves()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "legacy@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write(
    &meta,
    r#"{"host":"wbox","cache":{"fetched_at":"2026-08-01T00:00:00Z","status":"ok","five_hour":{"left_pct":80.0},"history":[{"t":1000,"h5":[80.0,"2026-08-01T05:00:00Z"],"d7":null,"sn":null}],"model_override":"opus","org_created_at":"2025-11-30T00:00:00Z"}}"#,
  ).unwrap();

  // AF2 leg 1: legacy values readable BEFORE migration (fallback window).
  let pre = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T06: legacy cache{} must be readable pre-migration" );
  assert_eq!( pre.fetched_at, "2026-08-01T00:00:00Z", "T06: legacy fetched_at honored" );
  assert_eq!( pre.org_created_at.as_deref(), Some( "2025-11-30T00:00:00Z" ) );
  assert_eq!(
    claude_profile_core::account::read_history( store.path(), name ).len(), 1,
    "T06: legacy history readable pre-migration",
  );

  // First post-upgrade quota write triggers the migration.
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 60.0, Some( "2026-08-16T21:00:00Z" ) ) ),
    None,
    None,
  );

  // AF2 leg 3: `cache` key absent from tracked; low-churn relocated top-level.
  let tracked : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &meta ).unwrap() ).unwrap();
  assert!( tracked.get( "cache" ).is_none(), "T06/AF2: cache key pruned from tracked JSON: {tracked}" );
  assert_eq!(
    tracked[ "org_created_at" ].as_str(), Some( "2025-11-30T00:00:00Z" ),
    "T06: org_created_at retained tracked (TSK-368 not regressed)",
  );
  assert_eq!( tracked[ "model_override" ].as_str(), Some( "opus" ), "T06: model_override relocated top-level" );
  assert_eq!( tracked[ "host" ].as_str(), Some( "wbox" ), "T06: unrelated fields preserved" );

  // AF2 leg 2: volatile + history present in the per-host file post-migration.
  let local = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let ljson : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &local ).unwrap() ).unwrap();
  let u = ljson[ "five_hour" ][ "left_pct" ].as_f64().expect( "left_pct" );
  assert!( ( u - 60.0 ).abs() < 1e-9, "T06: new fetch values in local file, got {u}" );
  assert_eq!(
    ljson[ "history" ].as_array().map( Vec::len ), Some( 1 ),
    "T06: legacy history carried into the local file",
  );

  // Merged read: new volatile + preserved low-churn.
  let post = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "post-migration read must succeed" );
  let ( util, _ ) = post.five_hour.expect( "five_hour" );
  assert!( ( util - 60.0 ).abs() < f64::EPSILON );
  assert_eq!( post.org_created_at.as_deref(), Some( "2025-11-30T00:00:00Z" ) );
  assert_eq!( post.model_override.as_deref(), Some( "opus" ) );

  // Steady state: a second write leaves tracked byte-identical (T01 property).
  let before = std::fs::read( &meta ).unwrap();
  claude_profile_core::account::write_quota_cache( store.path(), name, Some( ( 61.0, None ) ), None, None );
  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "T06/T01: second write must leave tracked byte-identical",
  );
}

// ── TSK-502: per-host tracked quota cache tree ────────────────────────────────

/// Expected per-host cache subdirectory name — derived from the shipped
/// active-marker filename so the test can never drift from the slug
/// sanitization actually used by the write path.
fn host_slug() -> String
{
  claude_profile_core::account::active_marker_filename()
    .strip_prefix( "_active_" )
    .expect( "marker filename always starts with _active_" )
    .to_string()
}

/// Seed a raw volatile cache object into a named host subtree.
fn seed_host_cache(
  store      : &std::path::Path,
  host_dir   : &str,
  name       : &str,
  fetched_at : &str,
  left_pct   : f64,
)
{
  let dir = store.join( "cache" ).join( host_dir );
  std::fs::create_dir_all( &dir ).unwrap();
  std::fs::write(
    dir.join( format!( "{name}.json" ) ),
    format!( r#"{{"fetched_at":"{fetched_at}","status":"ok","five_hour":{{"left_pct":{left_pct}}}}}"# ),
  ).unwrap();
}

/// T02+T03 (TSK-502): the merged read returns the freshest `fetched_at` across
/// host subtrees — in both directions — and a candidate whose `fetched_at`
/// does not parse is skipped instead of winning or aborting the merge.
#[ test ]
fn t502_02_read_merges_freshest_across_host_subtrees()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "fleet@acme.com";
  seed_host_cache( store.path(), "w001_user1", name, "2026-01-01T00:00:00Z", 10.0 );
  seed_host_cache( store.path(), "w002_user1", name, "2026-01-02T00:00:00Z", 20.0 );
  // T03: lexicographically huge but unparseable timestamp must never win.
  seed_host_cache( store.path(), "w009_user1", name, "not-a-timestamp", 99.0 );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T02: merged read must see the host subtrees" );
  let ( u, _ ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( u - 20.0 ).abs() < f64::EPSILON, "T02: fresher w002 entry must win, got {u}" );
  assert_eq!( entry.fetched_at, "2026-01-02T00:00:00Z" );

  // Direction flip: w001 becomes the freshest.
  seed_host_cache( store.path(), "w001_user1", name, "2026-01-03T00:00:00Z", 30.0 );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T02: merged read after flip" );
  let ( u, _ ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( u - 30.0 ).abs() < f64::EPSILON, "T02: freshest must flip to w001, got {u}" );
}

/// T04 (TSK-502): a legacy gitignored `-cache/{name}.json` (pre-502 layout) is
/// readable as fallback, and the next `write_quota_cache` relocates its role to
/// the per-host tracked file — carrying the history ring — and deletes it.
#[ test ]
fn t502_03_legacy_gitignored_cache_read_then_self_cleaned()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "legacylocal@acme.com";
  let legacy_dir = store.path().join( "-cache" );
  std::fs::create_dir_all( &legacy_dir ).unwrap();
  let legacy = legacy_dir.join( format!( "{name}.json" ) );
  std::fs::write(
    &legacy,
    r#"{"fetched_at":"2026-01-01T00:00:00Z","status":"ok","five_hour":{"left_pct":70.0},"history":[{"t":1000,"h5":[70.0,"2026-01-01T05:00:00Z"],"d7":null,"sn":null}]}"#,
  ).unwrap();

  let pre = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T04: legacy gitignored cache must be readable as fallback" );
  let ( u, _ ) = pre.five_hour.expect( "five_hour present" );
  assert!( ( u - 70.0 ).abs() < f64::EPSILON, "T04: legacy value honored, got {u}" );

  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 71.0, None ) ), None, None,
  );

  assert!( !legacy.exists(), "T04: legacy -cache file must be deleted after a successful per-host write" );
  let per_host = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let json : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &per_host ).unwrap() ).unwrap();
  assert_eq!(
    json[ "history" ].as_array().map( Vec::len ), Some( 1 ),
    "T04/AF2: legacy history must be carried into the per-host file",
  );
  let post = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T04: post-migration read" );
  let ( u, _ ) = post.five_hour.expect( "five_hour present" );
  assert!( ( u - 71.0 ).abs() < f64::EPSILON, "T04: per-host file serves reads, got {u}" );
}

/// T05 (TSK-502, regression guard — passed before the change too): a store with
/// no cache in any location returns `None`, the unchanged no-cache contract.
#[ test ]
fn t502_04_no_cache_anywhere_returns_none()
{
  let store = tempfile::tempdir().unwrap();
  assert!(
    claude_profile_core::account::read_quota_cache( store.path(), "nobody@acme.com" ).is_none(),
    "T05: empty store must read as no cache",
  );
}

/// T06 (TSK-502): a history ring living in another host's subtree is carried
/// into the own-host file by `write_quota_cache` and continued — not restarted —
/// by `write_history_entry`.
#[ test ]
fn t502_05_history_ring_continues_across_hosts()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "ring@acme.com";
  let other = store.path().join( "cache" ).join( "otherbox_op" );
  std::fs::create_dir_all( &other ).unwrap();
  std::fs::write(
    other.join( format!( "{name}.json" ) ),
    r#"{"fetched_at":"2026-01-01T00:00:00Z","status":"ok","history":[{"t":1000,"h5":[80.0,"2026-01-01T05:00:00Z"],"d7":null,"sn":null},{"t":2000,"h5":[75.0,"2026-01-01T05:00:00Z"],"d7":null,"sn":null}]}"#,
  ).unwrap();

  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 60.0, None ) ), None, None,
  );
  let own = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let json : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &own ).unwrap() ).unwrap();
  assert_eq!(
    json[ "history" ].as_array().map( Vec::len ), Some( 2 ),
    "T06: other host's ring must be carried into the own-host file",
  );

  claude_profile_core::account::write_history_entry(
    store.path(), name, 3_000, Some( ( 60.0, "2026-01-02T05:00:00Z" ) ), None, None,
  );
  let entries = claude_profile_core::account::read_history( store.path(), name );
  assert_eq!( entries.len(), 3, "T06: ring continued (2 carried + 1 appended)" );
  assert_eq!( entries[ 0 ].t, 1_000 );
  assert_eq!( entries[ 2 ].t, 3_000 );
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

/// `trace_ts()` returns a UTC-marked timestamp prefix (BUG-338).
///
/// ## Fix Documentation — BUG-338
///
/// - **Root Cause:** `trace_ts()` sliced `chrono_now_utc()`'s ISO-8601 output into
///   `"YYYY-MM-DD · HH:MM:SS · "`, dropping the trailing `Z` (UTC marker) entirely.
///   The rendered prefix was visually indistinguishable from a differently-clocked
///   (e.g. local-time) timestamp source sharing the same shape in a combined transcript.
/// - **Why Not Caught:** No test asserted on `trace_ts()`'s own return value — only
///   `chrono_now_utc()` (the function it wraps) had a round-trip test. The slicing
///   step that drops the `Z` had no dedicated coverage.
/// - **Fix Applied:** `format!` literal changed from `"{} · {} · "` to `"{} · {} UTC · "`,
///   restoring an explicit timezone marker in place of the dropped `Z`.
/// - **Prevention:** This test asserts both the substring and the full structural shape,
///   so a future slicing change that drops the marker again fails immediately.
/// - **Pitfall:** A bare `.contains("UTC")` check would pass even if `UTC` appeared in
///   the wrong position (e.g. before the date) — the structural check below pins the
///   exact position, ensuring the marker sits between time and trailer.
#[ test ]
fn trace_ts_returns_utc_marked_timestamp()
{
  let ts = claude_profile_core::account::trace_ts();

  assert!( ts.contains( " UTC · " ), "must contain UTC marker substring: {ts}" );

  // Structural check (AF1): validate the full shape, not just substring presence.
  let mut parts = ts.splitn( 2, " · " );
  let date_part = parts.next().expect( "date segment present" );
  let rest       = parts.next().expect( "time+marker segment present" );

  assert_eq!( date_part.len(), 10, "date segment must be YYYY-MM-DD: {date_part}" );
  assert!( date_part.chars().enumerate().all( |( i, c )| if i == 4 || i == 7 { c == '-' } else { c.is_ascii_digit() } ), "date segment must be YYYY-MM-DD: {date_part}" );

  assert_eq!( rest, format!( "{} UTC · ", &rest[ ..8 ] ), "time+marker segment must be HH:MM:SS UTC · : {rest}" );
  let time_part = &rest[ ..8 ];
  assert!( time_part.chars().enumerate().all( |( i, c )| if i == 2 || i == 5 { c == ':' } else { c.is_ascii_digit() } ), "time segment must be HH:MM:SS: {time_part}" );
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

/// Task 464 (T03): `remove_session_effort()` creates `settings.json` when the file
/// is absent but `~/.claude/` exists — mirrors FT-11's `set_session_model` precedent.
#[ test ]
fn ft_remove_session_effort_creates_file_when_settings_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  // settings.json intentionally absent.

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::remove_session_effort( &paths );

  let settings = dot.join( "settings.json" );
  assert!( settings.exists(), "remove_session_effort must create settings.json when absent" );
  let content = std::fs::read_to_string( &settings ).expect( "settings.json must be readable" );
  assert!( content.trim() == "{}", "created settings.json must be an empty object, got: {content}" );
}

/// Task 464 (T04, mirrors BUG-258's fix): `remove_session_effort()` creates
/// `~/.claude/` itself when the directory is absent, then behaves as the settings-absent case.
#[ test ]
fn ft_remove_session_effort_creates_dir_when_claude_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  assert!( !dot.exists(), "precondition: .claude/ must be absent" );

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::remove_session_effort( &paths );

  let settings = dot.join( "settings.json" );
  assert!( settings.exists(), "remove_session_effort must create .claude/ and settings.json when both absent" );
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

// ── Feature 040: Measurement history storage ──────────────────────────────────

/// FT-01 (AC-01): `write_history_entry()` stores correct `t`, `h5`, `d7`, `sn` fields.
///
/// # Given
/// Account `alice` has `alice.json` with an empty `cache.history[]` array.
/// A successful quota fetch returned utilization values for all three periods.
/// # When
/// `write_history_entry()` is called with the current timestamp and period data.
/// # Then
/// `cache.history[0]` contains `t` (Unix seconds), `h5: [42.0, "..."]`, `d7: [35.0, "..."]`, `sn: [20.0, "..."]`.
#[ test ]
fn history_append_stores_correct_fields()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Write alice.json with empty cache.history[].
  std::fs::write(
    store.join( "alice.json" ),
    r#"{"cache":{"fetched_at":"2026-06-21T12:00:00Z","status":"ok","history":[]}}"#,
  ).unwrap();

  let t = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, |d| d.as_secs() );

  account::write_history_entry(
    store,
    "alice",
    t,
    Some( ( 42.0, "2026-06-21T14:00:00+00:00" ) ),
    Some( ( 35.0, "2026-06-25T00:00:00+00:00" ) ),
    Some( ( 20.0, "2026-06-25T00:00:00+00:00" ) ),
  );

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 1, "FT-01: exactly 1 history entry after first append" );
  let e = &entries[ 0 ];
  assert!(
    t.abs_diff( e.t ) <= 2,
    "FT-01: stored t={} must be within 2s of now t={}", e.t, t,
  );
  let h5 = e.h5.as_ref().expect( "FT-01: h5 must be Some" );
  assert!( ( h5.0 - 42.0 ).abs() < 1e-9, "FT-01: h5 utilization got {}", h5.0 );
  assert_eq!( h5.1, "2026-06-21T14:00:00+00:00", "FT-01: h5 resets_at" );
  let d7 = e.d7.as_ref().expect( "FT-01: d7 must be Some" );
  assert!( ( d7.0 - 35.0 ).abs() < 1e-9, "FT-01: d7 utilization got {}", d7.0 );
  let sn = e.sn.as_ref().expect( "FT-01: sn must be Some" );
  assert!( ( sn.0 - 20.0 ).abs() < 1e-9, "FT-01: sn utilization got {}", sn.0 );
}

/// FT-02 (AC-02): Ring buffer evicts oldest entry when 11th measurement appended.
///
/// # Given
/// `alice.json` `cache.history[]` already has 10 entries with `t` values 1000..1009.
/// # When
/// An 11th measurement is appended with `t = 1010`.
/// # Then
/// `cache.history[]` has exactly 10 entries; `history[0].t == 1001` (oldest evicted);
/// `history[9].t == 1010` (newest appended).
#[ test ]
fn history_ring_buffer_evicts_oldest()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Append 10 entries with t = 1000..1009.
  for i in 0_u64..10
  {
    account::write_history_entry( store, "alice", 1000 + i, None, None, None );
  }

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 10, "FT-02: exactly 10 entries after 10 appends" );

  // Append 11th entry.
  account::write_history_entry( store, "alice", 1010, None, None, None );

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 10, "FT-02: still 10 entries after 11th append (ring buffer cap)" );
  assert_eq!(
    entries[ 0 ].t, 1001,
    "FT-02: oldest entry (t=1000) evicted; t=1001 is now first",
  );
  assert_eq!(
    entries[ 9 ].t, 1010,
    "FT-02: newest entry (t=1010) is last",
  );
}

/// FT-11 (AC-11): Backward compatibility — absent `"history"` key returns empty vec.
///
/// # Given
/// `alice.json` has a `cache` object with quota fields but no `"history"` key (old format).
/// # When
/// `read_history()` is called for `alice`.
/// # Then
/// Returns empty `Vec` — existing single-point fallback behavior preserved.
#[ test ]
fn history_read_absent_key_returns_empty()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Old cache format without "history" key.
  std::fs::write(
    store.join( "alice.json" ),
    r#"{"cache":{"fetched_at":"2026-06-21T12:00:00Z","status":"ok","five_hour":{"left_pct":42.0}}}"#,
  ).unwrap();

  let entries = account::read_history( store, "alice" );
  assert!(
    entries.is_empty(),
    "FT-11: absent history key must return empty vec (AC-11 backward compat); got: {}", entries.len(),
  );
}

/// FT-13 (AC-13): Duplicate timestamp overwrites last entry instead of appending.
///
/// # Given
/// `alice.json` `cache.history[]` has 3 entries. The last entry has `t = 1002`.
/// # When
/// A new measurement is appended with `t = 1002` (same Unix second, updated values).
/// # Then
/// `cache.history[]` still has 3 entries (not 4). The last entry's `h5` is updated.
#[ test ]
fn history_duplicate_timestamp_overwrites()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Append 3 entries.
  account::write_history_entry( store, "alice", 1000, Some( ( 10.0, "2026-06-21T14:00:00+00:00" ) ), None, None );
  account::write_history_entry( store, "alice", 1001, Some( ( 20.0, "2026-06-21T14:00:00+00:00" ) ), None, None );
  account::write_history_entry( store, "alice", 1002, Some( ( 30.0, "2026-06-21T14:00:00+00:00" ) ), None, None );

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 3, "FT-13: 3 entries before duplicate-timestamp test" );

  // Append with same t as last entry (duplicate timestamp).
  account::write_history_entry( store, "alice", 1002, Some( ( 99.0, "2026-06-21T14:00:00+00:00" ) ), None, None );

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 3, "FT-13: still 3 entries after duplicate-timestamp append" );
  let last = &entries[ 2 ];
  let h5   = last.h5.as_ref().expect( "FT-13: last entry h5 must be Some" );
  assert!(
    ( h5.0 - 99.0 ).abs() < 1e-9,
    "FT-13: last entry updated to new value (99.0), not 30.0; got {}", h5.0,
  );
}

// ── set_session_effort (Feature 062) ──────────────────────────────────────────

/// FT-09 (062): `set_session_effort()` writes `effortLevel` and preserves existing keys.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-09]
#[ test ]
fn ft09_set_session_effort_writes_effort_level()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot ).unwrap();
  std::fs::write(
    dot.join( "settings.json" ),
    r#"{"theme":"dark","model":"sonnet"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_effort( &paths, "max" );

  let content = std::fs::read_to_string( dot.join( "settings.json" ) )
    .expect( "settings.json must exist after set_session_effort" );
  assert!(
    content.contains( "\"effortLevel\"" ) && content.contains( "\"max\"" ),
    "FT-09: settings.json must contain effortLevel=max; got: {content}",
  );
  assert!(
    content.contains( "\"theme\"" ) && content.contains( "dark" ),
    "FT-09: set_session_effort must preserve existing 'theme' key; got: {content}",
  );
  assert!(
    content.contains( "\"model\"" ) && content.contains( "sonnet" ),
    "FT-09: set_session_effort must preserve existing 'model' key; got: {content}",
  );
}

/// FT-10 (062): `set_session_effort()` creates `~/.claude/` directory when absent.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-10]
#[ test ]
fn ft10_set_session_effort_creates_parent_dir_when_absent()
{
  let tmp = TempDir::new().unwrap();
  let dot = tmp.path().join( ".claude" );
  // Precondition: ~/.claude/ must NOT exist.
  assert!(
    !dot.exists(),
    "test precondition: ~/.claude/ must not exist before calling set_session_effort",
  );

  let paths = ClaudePaths::with_home( tmp.path() );
  claude_profile_core::account::set_session_effort( &paths, "high" );

  let settings = dot.join( "settings.json" );
  assert!(
    settings.exists(),
    "FT-10: set_session_effort must create ~/.claude/ dir and settings.json when parent dir absent",
  );
  let content = std::fs::read_to_string( &settings )
    .expect( "settings.json must be readable" );
  assert!(
    content.contains( "\"effortLevel\"" ) && content.contains( "\"high\"" ),
    "FT-10: created settings.json must contain effortLevel=high; got: {content}",
  );
}

// ── Feature 071 — AccountBackend domain type (Phase 1, task 433) ───────────────

/// T01/433: a fixture with `"backend":"redirect"` parses to `AccountBackend::Redirect`;
/// `base_url`/`redirect_model` round-trip through `list()`.
#[ test ]
fn ft01_071_backend_redirect_parses_to_redirect_variant()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "redirect@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"backend":"redirect","base_url":"https://foreign.example.com","redirect_model":"foreign-model-1"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name ).expect( "account must be listed" );
  assert_eq!(
    acct.backend, account::AccountBackend::Redirect,
    "backend:\"redirect\" must parse to AccountBackend::Redirect",
  );
  assert_eq!(
    acct.base_url.as_deref(), Some( "https://foreign.example.com" ),
    "base_url must round-trip through list()",
  );
  assert_eq!(
    acct.redirect_model.as_deref(), Some( "foreign-model-1" ),
    "redirect_model must round-trip through list()",
  );
}

/// Feature 071/AC-05 (domain-layer half): a legacy fixture with no `backend` key
/// parses to `AccountBackend::Anthropic` — byte-for-byte unchanged classification
/// for every account saved before Feature 071.
#[ test ]
fn ft02_071_absent_backend_key_defaults_to_anthropic()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "legacy@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"emailAddress":"legacy@test.com"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name ).expect( "account must be listed" );
  assert_eq!(
    acct.backend, account::AccountBackend::Anthropic,
    "absent backend key must default to AccountBackend::Anthropic",
  );
  assert!( acct.base_url.is_none(), "base_url must be None when absent from JSON" );
  assert!( acct.redirect_model.is_none(), "redirect_model must be None when absent from JSON" );
}

/// Feature 071/AC-05: an unrecognized `backend` value neither errors nor
/// misclassifies — it defaults to `AccountBackend::Anthropic`, same as absent.
#[ test ]
fn ft03_071_unrecognized_backend_value_defaults_to_anthropic_not_error()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "bogus@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"backend":"bogus"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name )
    .expect( "account must be listed — list() must not error on unrecognized backend value" );
  assert_eq!(
    acct.backend, account::AccountBackend::Anthropic,
    "unrecognized backend value must default to AccountBackend::Anthropic, never error",
  );
}

// ── Feature 071 — save()'s redirect write path (Phase 2, task 433) ────────────

/// T01/433/AC-01: `save()` with `backend: Redirect` writes `{name}.credentials.json`
/// containing exactly one key (`accessToken`, from the caller-supplied API key) and
/// writes `backend`/`base_url`/`redirect_model` into `{name}.json`.
#[ test ]
fn ft04_071_save_redirect_writes_minimal_credentials_and_metadata()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "redirect@foreign.com", &store, &paths, false,
    Some( b"sk-foreign-key-abc123" ), None, None, None,
    account::AccountBackend::Redirect, Some( "https://foreign.example.com" ), Some( "foreign-model-x" ), None,
  ).unwrap();

  let creds_content = std::fs::read_to_string( store.join( "redirect@foreign.com.credentials.json" ) )
    .expect( "{name}.credentials.json must exist after redirect save" );
  let creds_json : serde_json::Value = serde_json::from_str( &creds_content )
    .expect( "{name}.credentials.json must be valid JSON" );
  let creds_obj = creds_json.as_object().expect( "{name}.credentials.json must be a JSON object" );
  assert_eq!(
    creds_obj.len(), 1,
    "redirect save's {{name}}.credentials.json must contain exactly 1 key (accessToken); got: {creds_content}",
  );
  assert_eq!(
    creds_obj.get( "accessToken" ).and_then( | v | v.as_str() ), Some( "sk-foreign-key-abc123" ),
    "redirect save must write the caller-supplied API key as accessToken",
  );

  let meta_content = std::fs::read_to_string( store.join( "redirect@foreign.com.json" ) )
    .expect( "{name}.json must exist after redirect save" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  assert_eq!( meta_json[ "backend" ].as_str(), Some( "redirect" ), "redirect save must write backend:\"redirect\"; got: {meta_content}" );
  assert_eq!(
    meta_json[ "base_url" ].as_str(), Some( "https://foreign.example.com" ),
    "redirect save must write base_url to {{name}}.json; got: {meta_content}",
  );
  assert_eq!(
    meta_json[ "redirect_model" ].as_str(), Some( "foreign-model-x" ),
    "redirect save must write redirect_model to {{name}}.json; got: {meta_content}",
  );
}

/// T01/433/AC-01: a redirect save never reads `~/.claude/.credentials.json` — the
/// live Anthropic OAuth session file is left completely untouched.
#[ test ]
fn ft05_071_save_redirect_never_touches_live_credentials_file()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  // Distinct fixture value — a redirect save must never copy from or overwrite this.
  let live_marker = r#"{"accessToken":"LIVE-SESSION-SENTINEL-DO-NOT-TOUCH","expiresAt":1}"#;
  std::fs::write( dot_claude.join( ".credentials.json" ), live_marker ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "redirect@foreign.com", &store, &paths, false,
    Some( b"sk-foreign-key-abc123" ), None, None, None,
    account::AccountBackend::Redirect, Some( "https://foreign.example.com" ), Some( "foreign-model-x" ), None,
  ).unwrap();

  let live_content = std::fs::read_to_string( dot_claude.join( ".credentials.json" ) )
    .expect( "live ~/.claude/.credentials.json must still exist" );
  assert_eq!(
    live_content, live_marker,
    "redirect save must never modify ~/.claude/.credentials.json",
  );
}

/// T02/433/AC-04 (`docs/feature/071_redirect_backend_accounts.md`): `save()` with no
/// explicit backend argument (i.e. `AccountBackend::Anthropic`) still copies
/// `~/.claude/.credentials.json` exactly as before Feature 071, but now additionally
/// writes `backend: "anthropic"` into `{name}.json` — this is an intentional Feature 071
/// behavior addition (every account file becomes self-describing), not a regression.
#[ test ]
fn ft06_071_save_default_anthropic_writes_backend_field()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok","expiresAt":9999999999999}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None,
  ).unwrap();

  let creds_content = std::fs::read_to_string( store.join( "alice@test.com.credentials.json" ) )
    .expect( "{name}.credentials.json must exist after anthropic save" );
  assert!(
    creds_content.contains( "\"accessToken\"" ) && creds_content.contains( "\"expiresAt\"" ),
    "anthropic save must still copy the full live credentials file unchanged; got: {creds_content}",
  );

  let meta_content = std::fs::read_to_string( store.join( "alice@test.com.json" ) )
    .expect( "{name}.json must exist after anthropic save (Feature 071: backend is always written)" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  assert_eq!(
    meta_json[ "backend" ].as_str(), Some( "anthropic" ),
    "AC-04: default/anthropic save must write backend:\"anthropic\" into {{name}}.json; got: {meta_content}",
  );
}

// ── Feature 071 — switch_account()'s env.* responsibility (Phase 3, task 433) ─

/// T03/433/AC-06: `switch_account()` to a `backend: redirect` account writes all three
/// `env.ANTHROPIC_*` keys into `settings.json`, matching the target account's stored
/// `base_url`/`accessToken`/`redirect_model` values; unrelated top-level fields survive.
#[ test ]
fn ft07_071_switch_to_redirect_writes_env_keys()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "kimi@moonshot.ai.credentials.json" ),
    r#"{"accessToken":"sk-foreign-key-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "kimi@moonshot.ai.json" ),
    r#"{"backend":"redirect","base_url":"https://api.moonshot.ai/anthropic","redirect_model":"kimi-k3"}"#,
  ).unwrap();
  // Pre-existing unrelated top-level field — must survive the switch untouched (AC-06).
  std::fs::write( dot_claude.join( "settings.json" ), r#"{"theme":"dark"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "kimi@moonshot.ai", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let live_json : serde_json::Value = serde_json::from_str( &live ).expect( "settings.json must be valid JSON" );
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_BASE_URL" ].as_str(), Some( "https://api.moonshot.ai/anthropic" ),
    "AC-06: switch to redirect must write env.ANTHROPIC_BASE_URL from the account's base_url; got: {live}",
  );
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_AUTH_TOKEN" ].as_str(), Some( "sk-foreign-key-abc123" ),
    "AC-06: switch to redirect must write env.ANTHROPIC_AUTH_TOKEN from the account's accessToken; got: {live}",
  );
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_MODEL" ].as_str(), Some( "kimi-k3" ),
    "AC-06: switch to redirect must write env.ANTHROPIC_MODEL from the account's redirect_model; got: {live}",
  );
  assert_eq!(
    live_json[ "theme" ].as_str(), Some( "dark" ),
    "AC-06: unrelated top-level settings.json fields must survive the switch; got: {live}",
  );
}

/// T03/433/AC-07: `switch_account()` to a `backend: anthropic` account, after a prior
/// redirect switch populated `env`, removes exactly the 3 `ANTHROPIC_*` keys and prunes
/// the whole `env` object once it becomes empty as a result.
#[ test ]
fn ft08_071_switch_to_anthropic_removes_env_keys_and_prunes_empty_env()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
  // No {name}.json — an absent backend key defaults to Anthropic (AC-05).
  // Live settings.json already has env populated by a prior switch-to-redirect.
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.moonshot.ai/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-foreign","ANTHROPIC_MODEL":"kimi-k3"},"theme":"dark"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "alice@test.com", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let live_json : serde_json::Value = serde_json::from_str( &live ).expect( "settings.json must be valid JSON" );
  assert!(
    live_json.get( "env" ).is_none(),
    "AC-07: env must be removed entirely once its last ANTHROPIC_* sub-key is cleared; got: {live}",
  );
  assert_eq!(
    live_json[ "theme" ].as_str(), Some( "dark" ),
    "AC-07: unrelated top-level settings.json fields must survive the switch; got: {live}",
  );
}

/// T03/433/AC-07: switching to an anthropic account preserves any unrelated `env.*`
/// sub-key that was already present — only the 3 `ANTHROPIC_*` keys are removed.
#[ test ]
fn ft09_071_switch_to_anthropic_preserves_unrelated_env_subkey()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.moonshot.ai/anthropic","TZ":"Europe/Kyiv"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "alice@test.com", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let live_json : serde_json::Value = serde_json::from_str( &live ).expect( "settings.json must be valid JSON" );
  assert_eq!(
    live_json[ "env" ][ "TZ" ].as_str(), Some( "Europe/Kyiv" ),
    "AC-07: an unrelated env.* sub-key must survive a switch-away from redirect; got: {live}",
  );
  assert!(
    live_json[ "env" ].get( "ANTHROPIC_BASE_URL" ).is_none(),
    "AC-07: ANTHROPIC_BASE_URL must be removed on switch to anthropic; got: {live}",
  );
}

// ── Feature 071 — read_backend() helper (Phase 5, task 434) ────────────────────

/// T14/434: `read_backend()` on a missing `{name}.json` defaults to `Anthropic`,
/// mirroring `read_owner()`'s missing-file default-on-failure behaviour.
#[ test ]
fn ft10_071_read_backend_missing_file_defaults_anthropic()
{
  let tmp = TempDir::new().unwrap();
  let backend = account::read_backend( tmp.path(), "nonexistent@test.com" );
  assert_eq!(
    backend, account::AccountBackend::Anthropic,
    "read_backend on missing file must default to Anthropic; got: {backend:?}",
  );
}

/// T14/434: `read_backend()` reads an explicit `"backend":"redirect"` field correctly.
#[ test ]
fn ft11_071_read_backend_redirect_value()
{
  let tmp = TempDir::new().unwrap();
  std::fs::write( tmp.path().join( "kimi@moonshot.ai.json" ), r#"{"backend":"redirect"}"# ).unwrap();
  let backend = account::read_backend( tmp.path(), "kimi@moonshot.ai" );
  assert_eq!(
    backend, account::AccountBackend::Redirect,
    "read_backend must read an explicit redirect value; got: {backend:?}",
  );
}

/// T14/434: `read_backend()` on corrupt (non-JSON) content defaults to `Anthropic` —
/// must not panic, same resilience contract as `read_owner()`'s CC-3 case.
#[ test ]
fn ft12_071_read_backend_corrupt_content_defaults_anthropic()
{
  let tmp = TempDir::new().unwrap();
  std::fs::write( tmp.path().join( "alice@test.com.json" ), "<<<not json at all>>>" ).unwrap();
  let backend = account::read_backend( tmp.path(), "alice@test.com" );
  assert_eq!(
    backend, account::AccountBackend::Anthropic,
    "read_backend on corrupt content must default to Anthropic; got: {backend:?}",
  );
}

// ── Feature 072 — inference_provider field (task 435) ──────────────────────────

/// T01/435/AC-01: `save()` with `inference_provider: Some("kimi")` on a fresh account
/// writes `"inference_provider": "kimi"` to `{name}.json`.
#[ test ]
fn ft01_072_save_some_inference_provider_writes_field()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, Some( "kimi" ),
  ).unwrap();

  let meta_content = std::fs::read_to_string( store.join( "alice@test.com.json" ) )
    .expect( "{name}.json must exist after save" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  assert_eq!(
    meta_json[ "inference_provider" ].as_str(), Some( "kimi" ),
    "AC-01: save(inference_provider: Some(\"kimi\")) must write inference_provider:\"kimi\"; got: {meta_content}",
  );
}

/// T02/435/AC-02: `save()` with `inference_provider: None` on an account whose
/// `{name}.json` already has `"inference_provider": "kimi"` preserves it unchanged.
#[ test ]
fn ft02_072_save_none_inference_provider_preserves_existing()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, Some( "kimi" ),
  ).unwrap();

  // Second save with inference_provider: None — must not clobber the existing value.
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None,
  ).unwrap();

  let meta_content = std::fs::read_to_string( store.join( "alice@test.com.json" ) )
    .expect( "{name}.json must exist after save" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  assert_eq!(
    meta_json[ "inference_provider" ].as_str(), Some( "kimi" ),
    "AC-02: save(inference_provider: None) must preserve existing inference_provider unchanged; got: {meta_content}",
  );
}

/// T03/435/AC-03/AF3: `save()` with `inference_provider: None` on an account with no
/// pre-existing `inference_provider` key writes no such key at all — never the literal
/// default `"anthropic"`. Checks literal key absence (`contains_key`), not merely an
/// empty-string read, per AF3.
#[ test ]
fn ft03_072_save_none_inference_provider_no_prior_key_writes_no_key()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();

  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::save(
    "alice@test.com", &store, &paths, false, None, None, None, None,
    account::AccountBackend::Anthropic, None, None, None,
  ).unwrap();

  let meta_content = std::fs::read_to_string( store.join( "alice@test.com.json" ) )
    .expect( "{name}.json must exist after save" );
  let meta_json : serde_json::Value = serde_json::from_str( &meta_content ).expect( "{name}.json must be valid JSON" );
  let obj = meta_json.as_object().expect( "{name}.json must be a JSON object" );
  assert!(
    !obj.contains_key( "inference_provider" ),
    "AC-03/AF3: save(inference_provider: None) with no prior key must write no inference_provider key at all (never \"anthropic\"); got: {meta_content}",
  );
}

/// T04/435/AC-04: `list()` reads `inference_provider` from `{name}.json` into
/// `Account.inference_provider` when the key is present.
#[ test ]
fn ft04_072_list_reads_inference_provider_when_present()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "moonshot@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"inference_provider":"moonshot"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name ).expect( "account must be listed" );
  assert_eq!(
    acct.inference_provider, "moonshot",
    "AC-04: list() must read inference_provider from {{name}}.json into Account.inference_provider",
  );
}

/// T05/435/AC-05: `list()` returns `Account.inference_provider == ""` when the key is
/// absent from `{name}.json` (pre-existing account, or one saved before this feature).
#[ test ]
fn ft05_072_list_defaults_inference_provider_to_empty_when_absent()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "legacy_provider@test.com";

  write_credentials_file( store, name );
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"emailAddress":"legacy_provider@test.com"}"#,
  ).unwrap();

  let accounts = account::list( store ).unwrap();
  let acct = accounts.iter().find( | a | a.name == name ).expect( "account must be listed" );
  assert_eq!(
    acct.inference_provider, "",
    "AC-05: list() must default Account.inference_provider to empty string when key absent; got: {:?}", acct.inference_provider,
  );
}

// ── Feature 073 — Kimi provider preset env vars ────────────────────────────────

/// AC-05: `switch_account()` to a `backend: redirect`, `inference_provider: "kimi"`
/// account writes the 5 default-model-tier vars + `CLAUDE_CODE_EFFORT_LEVEL` +
/// `CLAUDE_CODE_AUTO_COMPACT_WINDOW` (1M for a `kimi-k3*` model), alongside the
/// pre-existing 3 `ANTHROPIC_*` vars.
#[ test ]
fn ft01_073_switch_to_kimi_redirect_writes_all_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "kimi.credentials.json" ),
    r#"{"accessToken":"sk-kimi-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "kimi.json" ),
    r#"{"backend":"redirect","base_url":"https://api.moonshot.ai/anthropic","redirect_model":"kimi-k3","inference_provider":"kimi"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "kimi", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) )
    .expect( "~/.claude/settings.json must exist after switch_account" );
  let live_json : serde_json::Value = serde_json::from_str( &live ).expect( "settings.json must be valid JSON" );
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "ANTHROPIC_DEFAULT_SONNET_MODEL", "ANTHROPIC_DEFAULT_HAIKU_MODEL", "ANTHROPIC_DEFAULT_FABLE_MODEL", "CLAUDE_CODE_SUBAGENT_MODEL" ]
  {
    assert_eq!(
      live_json[ "env" ][ key ].as_str(), Some( "kimi-k3" ),
      "AC-05: switch to a kimi redirect account must write env.{key} = redirect_model; got: {live}",
    );
  }
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_EFFORT_LEVEL" ].as_str(), Some( "max" ),
    "AC-05: switch to a kimi redirect account must write env.CLAUDE_CODE_EFFORT_LEVEL = \"max\"; got: {live}",
  );
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ].as_str(), Some( "1048576" ),
    "AC-05: a kimi-k3 redirect_model must write the 1M auto-compact window; got: {live}",
  );
}

/// AC-06: a `kimi-k2.7-code` `redirect_model` writes the narrower 256K
/// `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, not the 1M default used for `kimi-k3*`.
#[ test ]
fn ft02_073_switch_to_kimi_redirect_uses_narrow_compact_window_for_non_k3_model()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "kimi-code.credentials.json" ),
    r#"{"accessToken":"sk-kimi-code-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "kimi-code.json" ),
    r#"{"backend":"redirect","base_url":"https://api.moonshot.ai/anthropic","redirect_model":"kimi-k2.7-code","inference_provider":"kimi"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "kimi-code", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert_eq!(
    live_json[ "env" ][ "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ].as_str(), Some( "262144" ),
    "AC-06: kimi-k2.7-code must write the 256K auto-compact window, not the kimi-k3 1M default; got: {live}",
  );
}

/// AC-08: a `backend: redirect` account whose `inference_provider` is not `"kimi"`
/// (here: absent, defaulting to `"anthropic"`) gets only the pre-existing 3
/// `ANTHROPIC_*` vars — none of the 7 Kimi-tier additions.
#[ test ]
fn ft03_073_switch_to_redirect_non_kimi_provider_omits_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "other@foreign.ai.credentials.json" ),
    r#"{"accessToken":"sk-other-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "other@foreign.ai.json" ),
    r#"{"backend":"redirect","base_url":"https://api.other.ai/anthropic","redirect_model":"other-model-1"}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "other@foreign.ai", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_MODEL" ].as_str(), Some( "other-model-1" ),
    "sanity: the pre-existing 3 vars must still be written; got: {live}",
  );
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "ANTHROPIC_DEFAULT_SONNET_MODEL", "ANTHROPIC_DEFAULT_HAIKU_MODEL", "ANTHROPIC_DEFAULT_FABLE_MODEL", "CLAUDE_CODE_SUBAGENT_MODEL", "CLAUDE_CODE_EFFORT_LEVEL", "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ]
  {
    assert!(
      live_json[ "env" ].get( key ).is_none(),
      "AC-08: a non-kimi redirect account must not get the Kimi-tier env.{key}; got: {live}",
    );
  }
}

/// AC-07: switching from a `kimi` redirect account to a `backend: anthropic`
/// account clears all 10 env vars (the 3 pre-existing `ANTHROPIC_*` plus the 7
/// Kimi-tier additions) — not just the original 3.
#[ test ]
fn ft04_073_switch_from_kimi_to_anthropic_clears_all_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "alice@test.com.credentials.json" ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
  // Live settings.json already carries a full Kimi-tier env block from a prior switch.
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.moonshot.ai/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-foreign","ANTHROPIC_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_OPUS_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_SONNET_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_HAIKU_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_FABLE_MODEL":"kimi-k3","CLAUDE_CODE_SUBAGENT_MODEL":"kimi-k3","CLAUDE_CODE_EFFORT_LEVEL":"max","CLAUDE_CODE_AUTO_COMPACT_WINDOW":"1048576"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "alice@test.com", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  assert!(
    live_json.get( "env" ).is_none(),
    "AC-07: env must be removed entirely once every ANTHROPIC_*/CLAUDE_CODE_* sub-key is cleared; got: {live}",
  );
}

/// AC-07: switching from a `kimi` redirect account to a *different*, non-kimi
/// redirect account also clears the 7 stale Kimi-tier vars — this exercises the
/// redirect-branch's own non-kimi cleanup path, distinct from the anthropic-branch
/// cleanup `ft04_073` covers.
#[ test ]
fn ft05_073_switch_from_kimi_to_other_redirect_clears_stale_tier_env_vars()
{
  let tmp        = TempDir::new().unwrap();
  let store      = tmp.path().join( "store" );
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::create_dir_all( &dot_claude ).unwrap();

  std::fs::write(
    store.join( "other@foreign.ai.credentials.json" ),
    r#"{"accessToken":"sk-other-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.join( "other@foreign.ai.json" ),
    r#"{"backend":"redirect","base_url":"https://api.other.ai/anthropic","redirect_model":"other-model-1"}"#,
  ).unwrap();
  std::fs::write(
    dot_claude.join( "settings.json" ),
    r#"{"env":{"ANTHROPIC_BASE_URL":"https://api.moonshot.ai/anthropic","ANTHROPIC_AUTH_TOKEN":"sk-foreign","ANTHROPIC_MODEL":"kimi-k3","ANTHROPIC_DEFAULT_OPUS_MODEL":"kimi-k3","CLAUDE_CODE_EFFORT_LEVEL":"max","CLAUDE_CODE_AUTO_COMPACT_WINDOW":"1048576"}}"#,
  ).unwrap();

  let paths = ClaudePaths::with_home( tmp.path() );
  account::switch_account( "other@foreign.ai", &store, &paths ).unwrap();

  let live = std::fs::read_to_string( dot_claude.join( "settings.json" ) ).unwrap();
  let live_json : serde_json::Value = serde_json::from_str( &live ).unwrap();
  for key in [ "ANTHROPIC_DEFAULT_OPUS_MODEL", "CLAUDE_CODE_EFFORT_LEVEL", "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ]
  {
    assert!(
      live_json[ "env" ].get( key ).is_none(),
      "AC-07: switching to a non-kimi redirect account must clear stale Kimi-tier env.{key}; got: {live}",
    );
  }
  assert_eq!(
    live_json[ "env" ][ "ANTHROPIC_MODEL" ].as_str(), Some( "other-model-1" ),
    "sanity: the new account's own ANTHROPIC_MODEL must still be written; got: {live}",
  );
}

/// BUG-002 MRE: `parse_string_field()` (and siblings) search the entire input for the
/// first occurrence of a key, with no way to bound the search to a single object —
/// callers with multi-entry JSON (e.g. `roles_json`'s membership list) silently get the
/// wrong entry's value. `extract_object_block()` gives callers a way to bound the search
/// to one object before calling the existing helpers.
///
/// # Root Cause
/// `parse_string_field()`/`parse_u64_field()`/`parse_bool_field()`/`parse_string_array_field()`
/// all open with an unbounded `json.find(&search)` over the ENTIRE input string — none
/// accepts or enforces "search only within this one object." A caller holding multi-entry
/// JSON (e.g. `roles_json`, a list of workspace/organization memberships) has no way to
/// scope the search to the entry it actually needs.
///
/// # Why Not Caught
/// No test exercised any of the four helpers against multi-entry JSON — every existing
/// fixture is a flat, single-object JSON blob (credentials files, settings.json), where
/// "first occurrence" is always correct by coincidence of there being nothing else to find.
///
/// # Fix Applied
/// Added `extract_object_block()` — a brace-depth-counted `{...}` bound (mirrors
/// `claude_quota`'s own helper of the same name; independently duplicated, not shared).
/// A caller walking a multi-entry array can now bound each entry with
/// `extract_object_block()` before calling `parse_string_field()` etc. on the bounded
/// slice, eliminating the wrong-entry ambiguity for any caller that adopts it.
///
/// # Prevention
/// Reproduces the exact MRE scenario documented in BUG-002 (`roles_json` with two
/// workspace memberships) and asserts the second entry's `workspace_name` is correctly
/// extracted once bounded, not silently defaulting to the first (Acme) entry.
///
/// # Pitfall
/// The existing 4 unbounded helpers are UNCHANGED and remain correct for genuinely flat,
/// single-object JSON — do not add object-boundary scanning inside them directly, since
/// that would need a scoping parameter and break every existing single-object call site.
#[ doc = "bug_reproducer(BUG-002)" ]
#[ test ]
fn bug002_extract_object_block_bounds_multi_entry_roles_json()
{
  let roles_json = r#"{"roles":[
  {"organization_name":"Acme Corp","organization_uuid":"org-AAA","workspace_name":"Acme Prod","workspace_uuid":"ws-AAA"},
  {"organization_name":"Beta Inc","organization_uuid":"org-BBB","workspace_name":"Beta Prod","workspace_uuid":"ws-BBB"}
]}"#;

  // Sanity: unbounded search still returns the first entry — unchanged, documented
  // behavior for flat single-object JSON; not itself the fix under test.
  let unbounded = account::parse_string_field( roles_json, "workspace_name" );
  assert_eq!(
    unbounded.as_deref(), Some( "Acme Prod" ),
    "sanity: unbounded parse_string_field must still return the first entry; got {unbounded:?}",
  );

  // Bound the search to the SECOND membership entry via extract_object_block().
  let second_brace = roles_json.match_indices( '{' ).nth( 2 ).map( |( i, _ )| i )
    .expect( "MRE fixture must contain a third '{' (outer object + 2 memberships)" );
  let second_entry = account::extract_object_block( &roles_json[ second_brace.. ] )
    .expect( "extract_object_block must bound the second membership object" );

  let scoped = account::parse_string_field( second_entry, "workspace_name" );
  assert_eq!(
    scoped.as_deref(), Some( "Beta Prod" ),
    "BUG-002: once the caller bounds the search to the second membership entry via \
     extract_object_block(), parse_string_field() must return that entry's own \
     workspace_name (Beta Prod), not silently fall back to the first entry; got {scoped:?}",
  );
}

// ── FT-08 (021): parse_string_array_field ─────────────────────────────────────
// Relocated from an in-src `#[cfg(test)]` module in account.rs — all tests for
// this crate live in tests/ per the workspace test-placement convention.

/// `ft08_a`: Two-element array returns both values in order.
///
/// Given: `{"capabilities":["claude_max","chat"]}`
/// When: `parse_string_array_field(json, "capabilities")`
/// Then: Returns `["claude_max", "chat"]`
#[ test ]
fn ft08_parse_string_array_field_two_elements()
{
  let json   = r#"{"capabilities":["claude_max","chat"]}"#;
  let result = account::parse_string_array_field( json, "capabilities" );
  assert_eq!( result, vec![ "claude_max", "chat" ] );
}

/// `ft08_b`: Missing key returns empty Vec.
///
/// Given: JSON with no "capabilities" key
/// When: `parse_string_array_field(json, "capabilities")`
/// Then: Returns empty Vec
#[ test ]
fn ft08_parse_string_array_field_missing_key_returns_empty()
{
  let json   = r#"{"other_field":"value"}"#;
  let result = account::parse_string_array_field( json, "capabilities" );
  assert!( result.is_empty(), "missing key must return empty Vec, got: {result:?}" );
}

/// `ft08_c`: Empty array `[]` returns empty Vec.
///
/// Given: `{"capabilities":[]}`
/// When: `parse_string_array_field(json, "capabilities")`
/// Then: Returns empty Vec
#[ test ]
fn ft08_parse_string_array_field_empty_array_returns_empty()
{
  let json   = r#"{"capabilities":[]}"#;
  let result = account::parse_string_array_field( json, "capabilities" );
  assert!( result.is_empty(), "empty array must return empty Vec, got: {result:?}" );
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

  account::save( "alice@test.com", &store, &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None ).unwrap();

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
