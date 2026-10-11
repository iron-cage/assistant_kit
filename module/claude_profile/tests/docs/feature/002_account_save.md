# Test: Feature 002 — Save Account

### Scope

- **Purpose**: Test cases for saving current credentials as a named account, including metadata snapshotting and name inference.
- **Source**: `docs/feature/002_account_save.md`
- **Covers**: AC-01 through AC-22

Feature behavioral requirement test cases for `docs/feature/002_account_save.md`. Each FT case maps to one or more acceptance criteria. Name inference cases (AC-08, AC-09) are expanded in [feature/025_per_machine_active_marker.md](025_per_machine_active_marker.md) (FT-09) and the command spec [cli/command/004_account_save.md](../cli/command/04_account_save.md) (IT-10, IT-14).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|----|-------|
| FT-01 | `clp .account.save name::alice@acme.com` exits 0 and creates credential file | AC-01 | Integration |
| FT-02 | `dry::1` prints preview message; no file created | AC-04 | Integration |
| FT-03 | `oauthAccount` + model snapshot created alongside credential file | AC-05, AC-18 | Integration |
| FT-04 | No `name::` with valid `_active` marker — name inferred, save succeeds | AC-08 | Integration (BUG-209) |
| FT-05 | No `name::` with no `_active` marker — exits 1 with clear error | AC-09 | Integration (BUG-209) |
| FT-06 | Active marker written after save — `.credentials.status` shows account | AC-10 | Integration |
| FT-07 | Path-unsafe chars (`/`) in email local part exits 1 | AC-11 | Integration |
| FT-08 | Stale top-level `emailAddress` ignored; `oauthAccount.emailAddress` names the save (BUG-209) | AC-08 | Integration (BUG-209) |
| FT-09 | `save(update_marker=false)` does not write `_active`; background refresh callers pass `false` | AC-15 | BUG-211 MRE |
| FT-10 | Stale `_active` marker overridden by `oauthAccount.emailAddress` (BUG-212) | AC-16 | Name Inference / Regression |
| FT-11 | Re-running `.account.save` preserves `_renewal_at` key (read-merge, not overwrite) | AC-17 | Read-Merge |
| FT-12 | `.account.save` does NOT modify `owner` field — `account_save_routine()` passes `owner: None` | AC-19 | Ownership Neutral |
| FT-13 | `name::` (empty) exits 1 with `account name must not be empty` | AC-02 | Validation |
| FT-14 | `name::notanemail` (no `@`) exits 1 — `validate_name()` rejects before dry-run | AC-03 | Validation |
| FT-15 | `~/.claude.json` absent — save succeeds; no `oauthAccount` snapshot written | AC-07 | Best-Effort |
| FT-16 | `settings.json` absent — save succeeds; no `model` field written to `{name}.json` | AC-18 | Best-Effort |
| FT-17 | Bare save after a switch, `~/.claude.json` naming the previous account — saved under the slot holding the live session | AC-20 | Integration (BUG-582) |
| FT-18 | Saving a live session another slot holds exits 2 and writes no account file; own-slot and fresh-login saves succeed | AC-21 | Integration + Unit (BUG-582) |
| FT-19 | Live redirect key, `~/.claude.json` naming alice — bare save names the redirect slot and skips; save as alice exits 2 | AC-22 | Integration + Unit (BUG-582) |
| — | Endpoint 005 success → org identity fields written to `{name}.json` | AC-12 | Live-only (requires real credentials + network) |
| — | Endpoint 005 failure → no org identity; save still exits 0 | AC-13 | Structural (feature-gated; offline builds never call endpoint 005) |
| — | Re-save overwrites org identity fields with fresh endpoint 005 response | AC-14 | Live-only (idempotency mechanism; structurally covered by FT-03) |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Core save creates credential file | AC-01 | Basic Invocation |
| FT-02 | dry::1 previews without writing | AC-04 | Dry Run |
| FT-03 | `oauthAccount` + model snapshot created in `{name}.json` | AC-05, AC-18 | Metadata Snapshot |
| FT-04 | Name inferred from per-machine active marker | AC-08 | Name Inference |
| FT-05 | Missing marker exits 1 with actionable error | AC-09 | Inference Failure |
| FT-06 | Active marker written after save | AC-10 | Active Marker |
| FT-07 | Path-unsafe chars in local part exit 1 | AC-11 | Validation |
| FT-08 | Stale top-level `emailAddress` ignored; `oauthAccount.emailAddress` names the save (BUG-209) | AC-08 | Name Inference / Regression |
| FT-09 | `save(update_marker=false)` does not write `_active` | AC-15 | BUG-211 MRE |
| FT-10 | Stale `_active` marker overridden by `oauthAccount.emailAddress` (BUG-212) | AC-16 | Name Inference / Regression |
| FT-11 | Re-running `.account.save` preserves `_renewal_at` (read-merge) | AC-17 | Read-Merge |
| FT-12 | `.account.save` does NOT modify `owner` in `{name}.json` — passes `owner: None` | AC-19 | Ownership Neutral |
| FT-13 | `name::` (empty) exits 1 | AC-02 | Validation |
| FT-14 | `name::notanemail` (no `@`) exits 1 — validation before dry-run | AC-03 | Validation |
| FT-15 | `~/.claude.json` absent — save succeeds, no `oauthAccount` snapshot | AC-07 | Best-Effort |
| FT-16 | `settings.json` absent — save succeeds, no `model` field written | AC-18 | Best-Effort |
| FT-17 | Holding slot names a bare save over a stale `~/.claude.json` (BUG-582) | AC-20 | Name Inference / Regression |
| FT-18 | Save of a live session another slot holds is refused (BUG-582) | AC-21 | Session Guard / Regression |
| FT-19 | A live redirect key is never filed under another slot (BUG-582) | AC-22 | Session Guard / Regression |

**Total:** 19 FT cases

---

### FT-01: Core save creates credential file

- **Given:** `~/.claude/.credentials.json` exists with valid credential content.
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exits 0. stdout contains `saved current credentials as 'alice@acme.com'`. `{credential_store}/alice@acme.com.credentials.json` exists with content identical to source credentials.
- **Exit:** 0
- **Source fn:** `as01_save_creates_file` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/002_account_save.md AC-01](../../../docs/feature/002_account_save.md)

---

### FT-02: `dry::1` previews without writing

- **Given:** `~/.claude/.credentials.json` exists with valid credential content. `{credential_store}` is empty.
- **When:** `clp .account.save name::alice@acme.com dry::1`
- **Then:** Exits 0. stdout contains `[dry-run] would save current credentials as 'alice@acme.com'`. No credential file created in `{credential_store}`.
- **Exit:** 0
- **Source fn:** `as02_save_dry_run` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/002_account_save.md AC-04](../../../docs/feature/002_account_save.md)

---

### FT-03: `oauthAccount` + model snapshot created

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` exists with an `oauthAccount` subtree containing account identity fields. `~/.claude/settings.json` exists with `{"model": "claude-sonnet"}`.
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exits 0. `{credential_store}/alice@acme.com.json` is created containing `{"oauthAccount": {...}, "model": "claude-sonnet"}` with the extracted subtree and `model` field from `~/.claude/settings.json` (BUG-222 fix).
- **Exit:** 0
- **Source fn:** `acc26_save_creates_snapshot_files` (in `accounts_list_test_b.rs`)
- **Source:** [feature/002_account_save.md AC-05](../../../docs/feature/002_account_save.md)

---

### FT-04: Name inferred from per-machine active marker (fallback path)

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` is absent (no `oauthAccount.emailAddress` available — fallback path exercised). The per-machine active marker `{credential_store}/_active_{hostname}_{user}` contains `"alice@acme.com"`. No `name::` argument is passed.
- **When:** `clp .account.save`
- **Then:** Exits 0. stdout contains `saved current credentials as 'alice@acme.com'`. `{credential_store}/alice@acme.com.credentials.json` created. Behaves identically to `clp .account.save name::alice@acme.com`.
- **Exit:** 0
- **Source fn:** `as15_save_infers_name_from_active_marker` (in `tests/cli/account_mutations_test.rs`)
- **Note:** Tests the `_active` marker FALLBACK path only. The slot-match path is covered by FT-17, and the `oauthAccount.emailAddress` path by FT-10.
- **Source:** [feature/002_account_save.md AC-08](../../../docs/feature/002_account_save.md)

---

### FT-05: Missing marker exits 1 with actionable error

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. The credential store is absent, so no saved slot holds the session and no
  `_active_{hostname}_{user}` marker exists; `~/.claude.json` is absent too, so there's no `oauthAccount.emailAddress`.
- **When:** `clp .account.save`
- **Then:** Exits 1. stderr contains `cannot infer account name: no active account set — pass name:: explicitly`. No credential file created.
- **Exit:** 1
- **Source fn:** `as10_save_infer_absent_email_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/002_account_save.md AC-09](../../../docs/feature/002_account_save.md)

---

### FT-06: Active marker written after save

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `{credential_store}` has no `_active_{hostname}_{user}` file before the command.
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exits 0. `{credential_store}/_active_{hostname}_{user}` contains `"alice@acme.com"`. A subsequent `clp .credentials.status` shows `Account: alice@acme.com`.
- **Exit:** 0
- **Source fn:** `as16_save_writes_active_marker` (in `account_mutations_test_b.rs`)
- **Source:** [feature/002_account_save.md AC-10](../../../docs/feature/002_account_save.md)

---

### FT-07: Path-unsafe chars in local part exit 1

- **Given:** `~/.claude/.credentials.json` exists with valid credentials.
- **When:** `clp .account.save name::a/b@c.com`
- **Then:** Exits 1. stderr indicates path-unsafe characters in account name. No file created in `{credential_store}` — validation occurs before any filesystem operation.
- **Exit:** 1
- **Source fn:** `as17_save_slash_in_email_local_part_exits_1` (in `account_mutations_test_b.rs`)
- **Source:** [feature/002_account_save.md AC-11](../../../docs/feature/002_account_save.md)

---

### FT-08: Stale top-level `emailAddress` ignored (BUG-209 regression)

- **Given:** `~/.claude/.credentials.json` exists with credentials no saved slot holds. `~/.claude.json` has a stale top-level
  `emailAddress = "a@test.com"` and `oauthAccount.emailAddress = "b@test.com"`. The per-machine active marker `_active_{hostname}_{user}` contains
  `"b@test.com"`.
- **When:** `clp .account.save` (no `name::`)
- **Then:** Exits 0. stdout contains `b@test.com` and not `a@test.com`.
  The inference: (1) no saved slot holds the live session → None; (2) `oauthAccount.emailAddress` → `b@test.com`.
  Top-level `emailAddress` is never read by the inference logic. The per-machine marker still reads `b@test.com` after save.
- **Exit:** 0
- **Source fn:** `mre_bug_209_account_save_uses_active_marker_not_stale_email` (in `account_relogin_test_b.rs`)
- **Note:** `oauthAccount.emailAddress` and the marker both name `b@test.com`, so this case can't tell those two sources apart; it pins only that the
  top-level field is ignored. The marker fallback is covered by FT-04, `oauthAccount.emailAddress` over a stale marker by FT-10, and the slot-match path
  ahead of both by FT-17.
- **Source:** [feature/002_account_save.md AC-08](../../../docs/feature/002_account_save.md)

---

### FT-09: `save(update_marker=false)` does not write `_active`; background callers pass `false`

- **Given:** Empty credential store (no `_active` marker file). Valid credentials at `~/.claude/.credentials.json`.
- **When:** `account::save("alice@test.com", &store, &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None)` is called (unit test — simulates `refresh_account_token` context).
- **Then:** The credential file `alice@test.com.credentials.json` is written. The `_active_{hostname}_{user}` marker file does NOT exist — `update_marker=false` suppresses the write. A concurrent `.account.use` switch would be unaffected.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug211_save_false_leaves_marker_unchanged` (in `claude_profile_core/tests/account_test.rs`)
- **Note:** BUG-211 MRE — verifies the `update_marker` guard in `save()`. Background refresh calls (`refresh_account_token`) pass `false`; user CLI calls (`.account.save`, `.account.relogin`) pass `true`. All callers pass `owner: None` (preserves existing owner) per Feature 036 AC-14 — see Feature 036 FT-14 for the ownership side of the same requirement.
- **Source:** [feature/002_account_save.md AC-15](../../../docs/feature/002_account_save.md)

---

### FT-10: Stale `_active` marker overridden by `oauthAccount.emailAddress` (BUG-212 regression)

- **Given:** `~/.claude/.credentials.json` exists with live credentials. `~/.claude.json` has `oauthAccount.emailAddress = "i5@example.com"` (fresh — written by external OAuth login). The per-machine active marker `_active_{hostname}_{user}` contains `"i2@example.com"` (stale — last written by a prior clp session). No `name::` argument is passed.
- **When:** `clp .account.save` (no `name::`)
- **Then:** Exits 0. stdout contains `saved current credentials as 'i5@example.com'`. `{credential_store}/i5@example.com.credentials.json` is created. `{credential_store}/i2@example.com.credentials.json` is NOT created or modified — the stale marker is not used when `oauthAccount.emailAddress` provides a valid name.
- **Exit:** 0
- **Source fn:** `mre_bug_212_account_save_stale_marker_uses_oauth_email` (in `account_relogin_test_b.rs`)
- **Note:** BUG-212 MRE — verifies that when no saved slot holds the live session (none does here), `oauthAccount.emailAddress` from `~/.claude.json`
  names the save ahead of the `_active` marker, which is only used as a fallback when `oauthAccount.emailAddress` is absent or empty.
  External OAuth login updates `oauthAccount.emailAddress` but not the `_active` marker.
  <!-- BUG-582 — no FT covered a bare save whose ~/.claude.json named a different saved account than the live session's; FT-17 does -->
- **Source:** [feature/002_account_save.md AC-16](../../../docs/feature/002_account_save.md)

---

### FT-11: Re-running `.account.save` preserves `_renewal_at` key (read-merge, not overwrite)

- **Given:** Account `test@example.com` already has `{credential_store}/test@example.com.json` with `_renewal_at: "2026-06-29T21:00:00Z"` and a current `oauthAccount` snapshot. Fresh credentials available at `~/.claude/.credentials.json` with updated `oauthAccount` content.
- **When:** `clp .account.save name::test@example.com` is run a second time.
- **Then:** Exits 0. `test@example.com.json` contains:
  - `_renewal_at: "2026-06-29T21:00:00Z"` — unchanged (preserved from prior run)
  - `oauthAccount` — updated with fresh content from `~/.claude.json`, which still names `test@example.com` (so step 6a doesn't apply)
  - No other pre-existing top-level keys are removed.
- **Exit:** 0
- **Source fn:** `as22_save_preserves_renewal_at` (in `account_renewal_test.rs`)
- **Source:** [feature/002_account_save.md AC-17](../../../docs/feature/002_account_save.md)

---

### FT-12: `.account.save` does NOT modify `owner` field — ownership-neutral

- **Given:** Account `alice@acme.com` exists. `{credential_store}/alice@acme.com.json` contains `"owner": "old@host"`. Current identity = `"testuser@testmachine"` — deliberately different from the stored owner, to prove preservation does not depend on an identity match.
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exits 0. `{credential_store}/alice@acme.com.json` still contains `"owner": "old@host"` — unchanged even though it differs from current identity. `account_save_routine()` passes `owner: None` to `save()`, preserving the existing value via read-merge. Credentials re-saved; all other fields updated normally.
- **Exit:** 0
- **Source fn:** `ft12_save_does_not_stamp_owner` (in `account_ownership_test.rs`)
- **Note:** AC-19 — `.account.save` is ownership-neutral. Existing `owner` field is preserved unchanged on every save. An account with no `owner` field retains no `owner` field after save (read-merge: absent key stays absent).
- **Source:** [feature/002_account_save.md AC-19](../../../docs/feature/002_account_save.md)

---

### FT-13: `name::` (empty) exits 1 with `account name must not be empty`

- **Given:** `~/.claude/.credentials.json` exists with valid credentials.
- **When:** `clp .account.save name::`
- **Then:** Exits 1. stderr contains `account name must not be empty`. No credential file created.
- **Exit:** 1
- **Source fn:** `as06_save_empty_name_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/002_account_save.md AC-02](../../../docs/feature/002_account_save.md)

---

### FT-14: `name::notanemail` (no `@`) exits 1 — `validate_name()` rejects before dry-run

- **Given:** `~/.claude/.credentials.json` exists with valid credentials.
- **When:** `clp .account.save name::not-an-email dry::1`
- **Then:** Exits 1. stderr contains `not a valid email address`. The `[dry-run]` message is NOT printed — `validate_name()` fires before dry-run evaluation. No files created.
- **Exit:** 1
- **Source fn:** `as35_save_dry_run_rejects_invalid_name` (in `account_renewal_test_b.rs`)
- **Source:** [feature/002_account_save.md AC-03](../../../docs/feature/002_account_save.md)

---

### FT-15: `~/.claude.json` absent — save succeeds; no `oauthAccount` snapshot written

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` does NOT exist (no `oauthAccount` data available).
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exits 0. `{credential_store}/alice@acme.com.credentials.json` is created. `{credential_store}/alice@acme.com.json` is created but does NOT contain `oauthAccount` key (no source data).
- **Exit:** 0
- **Source fn:** `acc27_save_succeeds_without_claude_json` (in `accounts_list_test_b.rs`)
- **Source:** [feature/002_account_save.md AC-07](../../../docs/feature/002_account_save.md)

---

### FT-16: `settings.json` absent — save succeeds; no `model` field written to `{name}.json`

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` exists with `oauthAccount` data. `~/.claude/settings.json` does NOT exist (no model preference source).
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exits 0. `{credential_store}/alice@acme.com.json` is created with `oauthAccount` content but no `model` field. Save still succeeds when model source is absent.
- **Exit:** 0
- **Source fn:** `acc28_save_succeeds_without_settings_json` (in `accounts_list_test_b.rs`)
- **Source:** [feature/002_account_save.md AC-18](../../../docs/feature/002_account_save.md)

---

### FT-17: Holding slot names a bare save over a stale `~/.claude.json` (BUG-582 regression)

- **Given:** Saved slots `alice@example.com` and `bob@example.com`, each holding its own session (`refreshToken` `rt-alice`, `rt-bob`).
  `clp .account.use name::bob@example.com touch::0` has made bob's session live, and `~/.claude.json` was then rewritten to
  `oauthAccount.emailAddress = "alice@example.com"`, as a Claude process still running under alice does.
- **When:** `clp .account.save` (no `name::`)
- **Then:** Exits 0. stdout contains `saved current credentials as 'bob@example.com'`: the slot holding the live `refreshToken` names the save ahead of
  `~/.claude.json`. `{credential_store}/alice@example.com.credentials.json` is byte-identical to before, and the per-machine marker reads `bob@example.com`.
  `bob@example.com.json` still names bob, not alice: a slot re-saving the session it holds keeps its stored `oauthAccount` (step 6a).
- **Exit:** 0
- **Source fn:** `mre_bug_582_bare_save_files_live_session_under_its_own_slot` (in `account_relogin_test_b.rs`)
- **Note:** BUG-582 MRE. `.account.use` rewrites `~/.claude.json` itself, so the stale name is written after the switch. A live session no slot holds
  (a fresh login, or one refreshed outside clp since its last save) still takes its name from `~/.claude.json` — FT-10 covers that path.
  `bug_582_resave_keeps_slot_identity_when_claude_json_names_another` (`claude_profile_core/tests/account_test.rs`) checks step 6a on `save()`, and
  that a fresh login no slot holds still takes `~/.claude.json`'s identity.
- **Source:** [feature/002_account_save.md AC-20](../../../docs/feature/002_account_save.md)

---

### FT-18: Save of a live session another slot holds is refused (BUG-582)

- **Given:** Saved slots `alice@example.com` and `bob@example.com`, each holding its own session. Bob's session is live and the marker names bob.
- **When:** `clp .account.save name::alice@example.com`
- **Then:** Exits 2. stderr contains `live session is already saved as 'bob@example.com'`. Alice's slot and the marker are unchanged.
- **Exit:** 2
- **Source fn:** `bug_582_explicit_save_refuses_session_another_slot_holds` (in `account_relogin_test_b.rs`)
- **Note:** Also covered, in the same file: `bug_582_bare_save_refuses_when_two_slots_hold_live_session`. Two slots already hold the live session and
  `~/.claude.json` names one of them, so a bare save resolves that name and exits 2 with
  `already saved as 'i7@example.com' — refusing to also save it as 'i4@example.com'`. Also `bug_582_refused_rebackend_save_keeps_redirect_record`:
  a `backend::anthropic` re-save of a redirect slot exits 2 before its `{name}.json` is reset, so the record and key stay. In `claude_profile_core/tests/account_test.rs`:
  `mre_bug_582_save_refuses_session_another_slot_holds` (`save()` errs naming bob and writes no slot, metadata or marker),
  `bug_582_guard_allows_own_slot_and_unheld_session` (re-saving the holder's own slot and saving a fresh login both succeed) and
  `slots_holding_session_names_every_holder_in_name_order` (every holder in name order; none for an unheld session, an empty token, an access token
  no slot carries, or a missing store).
- **Source:** [feature/002_account_save.md AC-21](../../../docs/feature/002_account_save.md)

---

### FT-19: A live redirect key is never filed under another slot (BUG-582)

- **Given:** Saved slot `alice@example.com` holds its own session, and redirect slot `kimi` (`backend::redirect`, key `key-kimi`) is saved.
  `clp .account.use name::kimi touch::0` has made kimi's key live, and `~/.claude.json` was then rewritten to
  `oauthAccount.emailAddress = "alice@example.com"`.
- **When:** `clp .account.save` (no `name::`), then `clp .account.save name::alice@example.com`
- **Then:** The bare save exits 0 with `'kimi' is a redirect account (static credentials) — nothing to snapshot; save skipped`: kimi's slot holds the
  live `accessToken`, so it names the save and step 1a skips it. The explicit save exits 2 with
  `already saved as 'kimi' — refusing to also save it as 'alice@example.com'`. Alice's slot is byte-identical after both.
- **Exit:** 0, then 2
- **Source fn:** `bug_582_live_redirect_key_never_filed_under_another_slot` (in `account_relogin_test_b.rs`)
- **Note:** `CLAUDE_QUOTA_BASE_URL` points at a closed loopback port, so a save that wrongly files the key as alice can't send it anywhere when it
  looks up her org. In `claude_profile_core/tests/account_test.rs`, `bug_582_live_redirect_key_matched_by_access_token` checks the same match on
  `slots_holding_session()` and `save()` directly.
- **Source:** [feature/002_account_save.md AC-22](../../../docs/feature/002_account_save.md)
