# Test: Feature 002 — Save Account

Feature behavioral requirement test cases for `docs/feature/002_account_save.md`. Each FT case maps to one or more acceptance criteria. Name inference cases (AC-08, AC-09) are expanded in [feature/025_per_machine_active_marker.md](025_per_machine_active_marker.md) (FT-09) and the command spec [cli/command/004_account_save.md](../cli/command/04_account_save.md) (IT-10, IT-14).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|----|-------|
| FT-01 | `clp .account.save name::alice@acme.com` exits 0 and creates credential file | AC-01 | Integration |
| FT-02 | `dry::1` prints preview message; no file created | AC-04 | Integration |
| FT-03 | `oauthAccount` + model snapshot created alongside credential file | AC-05 | Integration |
| FT-04 | No `name::` with valid `_active` marker — name inferred, save succeeds | AC-08 | Integration (BUG-209) |
| FT-05 | No `name::` with no `_active` marker — exits 1 with clear error | AC-09 | Integration (BUG-209) |
| FT-06 | Active marker written after save — `.credentials.status` shows account | AC-10 | Integration |
| FT-07 | Path-unsafe chars (`/`) in email local part exits 1 | AC-11 | Integration |
| FT-08 | Stale top-level `emailAddress` ignored; `oauthAccount.emailAddress` absent → `_active` marker fallback (BUG-209) | AC-08 | Integration (BUG-209) |
| FT-09 | `save(update_marker=false)` does not write `_active`; background refresh callers pass `false` | AC-15 | BUG-211 MRE |
| FT-10 | Stale `_active` marker overridden by `oauthAccount.emailAddress` (BUG-212) | AC-16 | Name Inference / Regression |
| FT-11 | Re-running `.account.save` preserves `_renewal_at` key (read-merge, not overwrite) | AC-17 | Read-Merge |
| FT-12 | `.account.save` writes `current_identity()` as `owner` in `{name}.json` | AC-19 | Ownership Capture |
| FT-13 | `.account.save unclaim::1` writes `owner: ""` — all G1–G7 gates pass | AC-20 | Ownership Unclaim |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Core save creates credential file | AC-01 | Basic Invocation |
| FT-02 | dry::1 previews without writing | AC-04 | Dry Run |
| FT-03 | `oauthAccount` + model snapshot created in `{name}.json` | AC-05 | Metadata Snapshot |
| FT-04 | Name inferred from per-machine active marker | AC-08 | Name Inference |
| FT-05 | Missing marker exits 1 with actionable error | AC-09 | Inference Failure |
| FT-06 | Active marker written after save | AC-10 | Active Marker |
| FT-07 | Path-unsafe chars in local part exit 1 | AC-11 | Validation |
| FT-08 | Stale top-level `emailAddress` ignored; `oauthAccount.emailAddress` absent → `_active` fallback (BUG-209) | AC-08 | Name Inference / Regression |
| FT-09 | `save(update_marker=false)` does not write `_active` | AC-15 | BUG-211 MRE |
| FT-10 | Stale `_active` marker overridden by `oauthAccount.emailAddress` (BUG-212) | AC-16 | Name Inference / Regression |
| FT-11 | Re-running `.account.save` preserves `_renewal_at` (read-merge) | AC-17 | Read-Merge |
| FT-12 | `.account.save` writes `current_identity()` as `owner` in `{name}.json` | AC-19 | Ownership Capture |
| FT-13 | `.account.save unclaim::1` writes `owner: ""` — all G1–G7 gates pass | AC-20 | Ownership Unclaim |

**Total:** 13 FT cases

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
- **Source fn:** `acc26_save_creates_snapshot_files` (in `tests/cli/accounts_test.rs`)
- **Source:** [feature/002_account_save.md AC-05](../../../docs/feature/002_account_save.md)

---

### FT-04: Name inferred from per-machine active marker (fallback path)

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` is absent (no `oauthAccount.emailAddress` available — fallback path exercised). The per-machine active marker `{credential_store}/_active_{hostname}_{user}` contains `"alice@acme.com"`. No `name::` argument is passed.
- **When:** `clp .account.save`
- **Then:** Exits 0. stdout contains `saved current credentials as 'alice@acme.com'`. `{credential_store}/alice@acme.com.credentials.json` created. Behaves identically to `clp .account.save name::alice@acme.com`.
- **Exit:** 0
- **Source fn:** `as15_save_infers_name_from_active_marker` (in `tests/cli/account_mutations_test.rs`)
- **Note:** Tests the `_active` marker FALLBACK path only. Primary path (`oauthAccount.emailAddress` present) is covered by FT-10.
- **Source:** [feature/002_account_save.md AC-08](../../../docs/feature/002_account_save.md)

---

### FT-05: Missing marker exits 1 with actionable error

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. No `_active_{hostname}_{user}` marker file exists in `{credential_store}` (or the credential store directory is absent entirely).
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
- **Source fn:** `as16_save_writes_active_marker` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/002_account_save.md AC-10](../../../docs/feature/002_account_save.md)

---

### FT-07: Path-unsafe chars in local part exit 1

- **Given:** `~/.claude/.credentials.json` exists with valid credentials.
- **When:** `clp .account.save name::a/b@c.com`
- **Then:** Exits 1. stderr indicates path-unsafe characters in account name. No file created in `{credential_store}` — validation occurs before any filesystem operation.
- **Exit:** 1
- **Source fn:** `as17_save_slash_in_email_local_part_exits_1` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/002_account_save.md AC-11](../../../docs/feature/002_account_save.md)

---

### FT-08: Stale top-level `emailAddress` ignored — fallback to `_active` marker (BUG-209 regression)

- **Given:** `~/.claude/.credentials.json` exists with credentials for `b@test.com`. `~/.claude.json` has top-level `emailAddress = "a@test.com"` (stale, no `oauthAccount.emailAddress` field). The per-machine active marker `_active_{hostname}_{user}` contains `"b@test.com"`.
- **When:** `clp .account.save` (no `name::`)
- **Then:** Exits 0. stdout contains `saved current credentials as 'b@test.com'`. The two-level inference: (1) `oauthAccount.emailAddress` absent from the JSON → None; (2) fallback to `_active` marker → `b@test.com`. Top-level `emailAddress` is never read by the inference logic. The per-machine marker still reads `b@test.com` after save.
- **Exit:** 0
- **Source fn:** `mre_bug_209_account_save_uses_active_marker_not_stale_email` (in `tests/cli/account_mutations_test.rs`)
- **Note:** Tests the fallback path — exercises the case where `oauthAccount.emailAddress` is absent, so the `_active` marker is used. The primary path (oauthAccount.emailAddress wins over a stale marker) is covered by FT-10.
- **Source:** [feature/002_account_save.md AC-08](../../../docs/feature/002_account_save.md)

---

### FT-09: `save(update_marker=false)` does not write `_active`; background callers pass `false`

- **Given:** Empty credential store (no `_active` marker file). Valid credentials at `~/.claude/.credentials.json`.
- **When:** `account::save("alice@test.com", store.path(), &paths, false, None, None, None)` is called (unit test — simulates `refresh_account_token` context).
- **Then:** The credential file `alice@test.com.credentials.json` is written. The `_active_{hostname}_{user}` marker file does NOT exist — `update_marker=false` suppresses the write. A concurrent `.account.use` switch would be unaffected.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug211_save_false_leaves_marker_unchanged` (in `claude_profile_core/tests/account_test.rs`)
- **Note:** BUG-211 MRE — verifies the `update_marker` guard in `save()`. Background refresh calls (`refresh_account_token`) pass `false`; user CLI calls (`.account.save`, `.account.relogin`) pass `true`. Background callers must also pass `owner: None` per Feature 036 AC-14 — see Feature 036 FT-14 for the ownership side of the same requirement.
- **Source:** [feature/002_account_save.md AC-15](../../../docs/feature/002_account_save.md)

---

### FT-10: Stale `_active` marker overridden by `oauthAccount.emailAddress` (BUG-212 regression)

- **Given:** `~/.claude/.credentials.json` exists with live credentials. `~/.claude.json` has `oauthAccount.emailAddress = "i5@wbox.pro"` (fresh — written by external OAuth login). The per-machine active marker `_active_{hostname}_{user}` contains `"i2@wbox.pro"` (stale — last written by a prior clp session). No `name::` argument is passed.
- **When:** `clp .account.save` (no `name::`)
- **Then:** Exits 0. stdout contains `saved current credentials as 'i5@wbox.pro'`. `{credential_store}/i5@wbox.pro.credentials.json` is created. `{credential_store}/i2@wbox.pro.credentials.json` is NOT created or modified — the stale marker is not used when `oauthAccount.emailAddress` provides a valid name.
- **Exit:** 0
- **Source fn:** `mre_bug_212_account_save_stale_marker_uses_oauth_email` (in `tests/cli/account_mutations_test.rs`)
- **Note:** BUG-212 MRE — verifies that `oauthAccount.emailAddress` from `~/.claude.json` is the primary name inference source; the stale `_active` marker is only used as a fallback when `oauthAccount.emailAddress` is absent or empty. External OAuth login updates `oauthAccount.emailAddress` but not the `_active` marker.
- **Source:** [feature/002_account_save.md AC-16](../../../docs/feature/002_account_save.md)

---

### FT-11: Re-running `.account.save` preserves `_renewal_at` key (read-merge, not overwrite)

- **Given:** Account `test@example.com` already has `{credential_store}/test@example.com.json` with `_renewal_at: "2026-06-29T21:00:00Z"` and a current `oauthAccount` snapshot. Fresh credentials available at `~/.claude/.credentials.json` with updated `oauthAccount` content.
- **When:** `clp .account.save name::test@example.com` is run a second time.
- **Then:** Exits 0. `test@example.com.json` contains:
  - `_renewal_at: "2026-06-29T21:00:00Z"` — unchanged (preserved from prior run)
  - `oauthAccount` — updated with fresh content from `~/.claude.json`
  - No other pre-existing top-level keys are removed.
- **Exit:** 0
- **Source fn:** `as22_save_preserves_renewal_at` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/002_account_save.md AC-17](../../../docs/feature/002_account_save.md)

---

### FT-12: `.account.save` auto-captures `current_identity()` as `owner` in `{name}.json`

- **Given:** Account `alice@acme.com` does not yet exist in the credential store. The machine running the command has `$USER=user1` and hostname resolves to `host1` via `resolve_hostname()`, making `current_identity() = "user1@host1"`.
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exits 0. `{credential_store}/alice@acme.com.json` contains `"owner": "user1@host1"`. When the save is re-run from a machine where `current_identity() = "user2@host2"`, `owner` is overwritten to `"user2@host2"` — the last saver owns the account.
- **Exit:** 0
- **Source fn:** `ft01_save_captures_owner` (in `claude_profile_core/tests/account_test.rs`)
- **Note:** See also Feature 036 FT-01 for the unit-test counterpart. This FT covers the CLI integration path (account_save_routine → save() with owner: Some(current_identity())).
- **Source:** [feature/002_account_save.md AC-19](../../../docs/feature/002_account_save.md)

---

### FT-13: `.account.save unclaim::1` writes `owner: ""` — all G1–G7 gates pass after

- **Given:** Account `alice@acme.com` has `{credential_store}/alice@acme.com.json` with `"owner": "user1@host1"`. The current machine's identity is different (`current_identity() ≠ "user1@host1"`), so the account is currently non-owned.
- **When:** `clp .account.save name::alice@acme.com unclaim::1`
- **Then:** Exits 0. `{credential_store}/alice@acme.com.json` contains `"owner": ""`. `is_owned("", current_identity())` returns `true` for any caller — all G1–G7 gates pass. Other fields in `alice@acme.com.json` (e.g., `_renewal_at`, `oauthAccount`) are preserved via read-merge.
- **Exit:** 0
- **Source fn:** `ft02_unclaim_clears_owner` (in `claude_profile_core/tests/account_test.rs`)
- **Note:** See also Feature 036 FT-02 for the unit-test counterpart. This FT covers the CLI integration path (account_save_routine → save() with owner: Some("") when unclaim::1).
- **Source:** [feature/002_account_save.md AC-20](../../../docs/feature/002_account_save.md)
