# Test: Feature 004 — Switch Account

Feature behavioral requirement test cases for `docs/feature/004_account_use.md` (FR-9). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Atomic credential swap + per-machine active marker update | AC-01 |
| FT-02 | Unknown account exits 2 | AC-02 |
| FT-03 | Crash-safe rename semantics — no partial credential state | AC-03 |
| FT-04 | `dry::1` prints message; no files changed | AC-04 |
| FT-05 | `credentials.status` shows new email after switch | AC-05 |
| FT-06 | Path-unsafe characters in name → exit 1 | AC-06 |
| FT-07 | `oauthAccount.emailAddress` enforced; org fields overridden from `{name}.json` | AC-07 |
| FT-08 | Model preference restored from `{name}.json`; cleared when absent | AC-08 |
| FT-09 | emailAddress patched unconditionally even when `{name}.json` absent | AC-09 |
| FT-10 | Non-owned account: `.account.use` exits 1 with ownership violation message | AC-10 |
| FT-11 | Ownership check fires before `dry::1` — exits 1 even with `dry::1` set | AC-11 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Atomic swap overwrites credentials and updates active marker | AC-01 | Switch |
| FT-02 | Unknown account name exits 2 with error message | AC-02 | Error Handling |
| FT-03 | POSIX rename guarantees no partial credential write on crash | AC-03 | Atomicity |
| FT-04 | Dry-run prints message without modifying any files | AC-04 | Dry Run |
| FT-05 | `.credentials.status` shows new email after switch | AC-05 | Verification |
| FT-06 | Empty name, slash in name, or path-unsafe chars → exit 1 | AC-06 | Validation |
| FT-07 | `emailAddress` enforced as account name; org fields from `{name}.json` | AC-07 | oauthAccount |
| FT-08 | Model preference restored from `{name}.json`; cleared when absent | AC-08 | Model Restore |
| FT-09 | emailAddress patched unconditionally even when `{name}.json` absent | AC-09 | oauthAccount |
| FT-10 | Non-owned account exits 1 with ownership violation message | AC-10 | Ownership Guard |
| FT-11 | Ownership check fires before `dry::1` — exits 1 regardless of dry-run | AC-11 | Ownership Guard |

**Total:** 11 FT cases

---

### FT-01: Atomic swap overwrites credentials and updates active marker

- **Given:** Two accounts saved: `alice@acme.com` (credentials + active marker) and `work@acme.com` (credentials only). The current active account is `alice@acme.com`.
- **When:** `clp .account.use name::work@acme.com`
- **Then:** Exit 0. `~/.claude/.credentials.json` now contains `work@acme.com`'s token. `_active_{hostname}_{user}` marker file contains `work@acme.com`. The active marker is updated atomically via rename; no partial-write state is possible. Switching to the same account (idempotent) also exits 0 with the credentials in place.
- **Exit:** 0
- **Source fn:** `aw01_switch_swaps_credentials`, `aw07_switch_updates_active_marker`, `aw08_switch_same_account_idempotent`, `aw09_switch_copies_credentials`
- **Source:** [004_account_use.md AC-01](../../../docs/feature/004_account_use.md)

---

### FT-02: Unknown account name exits 2 with error message

- **Given:** Only `alice@acme.com` exists in the credential store.
- **When:** `clp .account.use name::ghost@example.com`
- **Then:** Exit 2. Stderr contains an actionable error identifying `ghost@example.com` as not found.
- **When (dry run + not found):** `clp .account.use name::ghost@example.com dry::1`
- **Then:** Exit 2 (not-found guard fires before dry-run message is printed).
- **Exit:** 2
- **Source fn:** `aw03_switch_nonexistent_exits_2`, `aw10_switch_dry_run_nonexistent_exits_2`
- **Source:** [004_account_use.md AC-02](../../../docs/feature/004_account_use.md)

---

### FT-03: POSIX rename guarantees no partial credential write on crash

- **Given:** A switch is in progress between the temp-write (step 2) and rename (step 3).
- **When:** Process terminates between those steps.
- **Then:** `~/.claude/.credentials.json` contains either the old credentials or the new ones — never a partially-written file. The POSIX `rename(2)` syscall is atomic on the same filesystem; the temp file is written to the same directory as the target before renaming.
- **Note:** This invariant has no dedicated integration test — it is a filesystem atomicity guarantee. See `docs/invariant/005_atomic_switching.md` for the invariant specification. The active marker (step 4) and `oauthAccount` patch (step 5) are best-effort; a crash after step 3 always leaves authentication credentials correct.
- **Exit:** 0
- **Source fn:** (no integration test — POSIX rename atomicity; see `docs/invariant/005_atomic_switching.md`)
- **Source:** [004_account_use.md AC-03](../../../docs/feature/004_account_use.md)

---

### FT-04: Dry-run prints message without modifying any files

- **Given:** Account `alice@acme.com` exists. Current active is `work@acme.com`.
- **When:** `clp .account.use name::alice@acme.com dry::1`
- **Then:** Exit 0. Output contains `[dry-run] would switch to 'alice@acme.com'`. `~/.claude/.credentials.json` still contains `work@acme.com`'s token. Active marker file unchanged.
- **Exit:** 0
- **Source fn:** `aw02_switch_dry_run`
- **Source:** [004_account_use.md AC-04](../../../docs/feature/004_account_use.md)

---

### FT-05: `.credentials.status` shows new email after switch

- **Given:** Two accounts saved: `alice@acme.com` and `work@acme.com`. Active is `alice@acme.com`.
- **When:** `clp .account.use name::work@acme.com` followed by `clp .credentials.status`
- **Then:** `.credentials.status` output contains `Email: work@acme.com`. The `~/.claude.json oauthAccount.emailAddress` was patched from the snapshot during the switch, so the status command reads the correct email.
- **Exit:** 0
- **Source fn:** `switch_restores_claude_json`
- **Source:** [004_account_use.md AC-05](../../../docs/feature/004_account_use.md)

---

### FT-06: Empty name, slash in name, or path-unsafe chars → exit 1

- **Given:** Any credential store state.
- **When:** `clp .account.use name::` (empty name)
- **Then:** Exit 1.
- **When:** `clp .account.use name::a/b@c.com` (slash in local part)
- **Then:** Exit 1 — `validate_name()` rejects path-unsafe characters before any filesystem operation.
- **When:** `clp .account.use` (missing name param)
- **Then:** Exit 1.
- **When:** `clp .account.use name::a/b@c.com` (slash specifically in email local part)
- **Then:** Exit 1 — the slash character is path-unsafe even in the domain portion of an email.
- **Exit:** 1
- **Source fn:** `aw04_switch_empty_name_exits_1`, `aw05_switch_slash_name_exits_1`, `aw06_switch_missing_name_param_exits_1`, `aw11_switch_slash_in_email_local_part_exits_1`
- **Source:** [004_account_use.md AC-06](../../../docs/feature/004_account_use.md)

---

### FT-07: `emailAddress` enforced as account name; org fields from `{name}.json`

- **Given:** Account `alice@acme.com` with `alice@acme.com.json` snapshot containing `oauthAccount.emailAddress = "stale@oldco.com"` and `organizationName = "Acme"`, `organizationUuid = "org-123"`.
- **When:** `clp .account.use name::alice@acme.com`
- **Then:** `~/.claude.json oauthAccount.emailAddress` is `"alice@acme.com"` — the account name wins over the stale snapshot value (BUG-217 fix). `~/.claude.json oauthAccount.organizationName` is `"Acme"` and `oauthAccount.organizationUuid` is `"org-123"` (BUG-219 fix — org fields from `{name}.json` override snapshot). All other keys in `~/.claude.json` (e.g., `commands`, `mcpServers`, `projects`) are untouched.
- **Exit:** 0
- **Source fn:** `mre_bug_217_switch_account_enforces_emailaddress`
- **Source:** [004_account_use.md AC-07](../../../docs/feature/004_account_use.md)

---

### FT-08: Model preference restored from `{name}.json`; cleared when absent

- **Given (restore):** Account `alice@acme.com` with `alice@acme.com.json` containing `{"model": "claude-opus-4-6"}`. `~/.claude/settings.json` currently contains `{"model": "claude-sonnet-4-6"}`.
- **When:** `clp .account.use name::alice@acme.com`
- **Then:** `~/.claude/settings.json` `model` field is updated to `"claude-opus-4-6"`. All other keys in `~/.claude/settings.json` are preserved.
- **Given (clear):** Account `bob@acme.com` with no `model` field in `bob@acme.com.json` snapshot. `~/.claude/settings.json` currently has `model` set from a prior account.
- **When:** `clp .account.use name::bob@acme.com`
- **Then:** `~/.claude/settings.json` `model` key is removed (preventing prior account's model from persisting). Other keys preserved.
- **Note:** Coverage in `claude_profile_core` unit tests; no dedicated CLI integration test exists for this AC.
- **Exit:** 0
- **Source fn:** `mre_bug222_switch_account_restores_model_from_settings_snapshot`, `mre_bug222_switch_account_clears_model_when_no_snapshot` (in `module/claude_profile_core/tests/account_test.rs`)
- **Source:** [004_account_use.md AC-08](../../../docs/feature/004_account_use.md)

---

### FT-09: emailAddress patched unconditionally even when `{name}.json` metadata file is absent

- **Given:** Account `bob@acme.com` has `bob@acme.com.credentials.json` but NO `bob@acme.com.json` metadata file. Current active is `alice@acme.com` with `~/.claude.json` containing `oauthAccount.emailAddress = "alice@acme.com"`.
- **When:** `clp .account.use name::bob@acme.com`
- **Then:** Exit 0. `~/.claude.json oauthAccount.emailAddress` is `"bob@acme.com"` — patched unconditionally even without metadata file. All other `oauthAccount` fields retain their previous values from alice's session. `_active_{hostname}_{user}` marker contains `bob@acme.com`.
- **Exit:** 0
- **Source fn:** `mre_bug254_switch_account_patches_email_when_metadata_absent` (core), `aw12_switch_patches_email_when_metadata_absent` (FT)
- **Source:** [004_account_use.md AC-09](../../../docs/feature/004_account_use.md)

---

### FT-10: Non-owned account exits 1 with ownership violation message

- **Given:** Account `alice@corp.com` has `{credential_store}/alice@corp.com.json` with `"owner": "other@remote"`. The current machine's `current_identity()` is `"user1@thishost"` — not equal to `"other@remote"`.
- **When:** `clp .account.use name::alice@corp.com`
- **Then:** Exits 1. Stderr contains `"ownership violation: this account is owned by other@remote"`. `switch_account()` is NOT called — no credential files are modified. The `_active` marker is unchanged.
- **Exit:** 1
- **Source fn:** `ft08_use_exits_1_when_not_owned` (in `tests/cli/account_mutations_test.rs`)
- **Note:** G5 ownership gate from Feature 036 AC-08. Shared with Feature 036 FT-08 — same test function, both specs reference it.
- **Source:** [004_account_use.md AC-10](../../../docs/feature/004_account_use.md)

---

### FT-11: Ownership check fires before `dry::1` — exits 1 even with `dry::1` set

- **Given:** Account `alice@corp.com` is owned by `"other@remote"`. Current identity ≠ `"other@remote"`.
- **When:** `clp .account.use name::alice@corp.com dry::1`
- **Then:** Exits 1. The ownership violation message is printed to stderr. The `[dry-run] would switch to 'alice@corp.com'` message is NOT printed — dry-run output is suppressed when ownership check fails. No files are modified.
- **Exit:** 1
- **Source fn:** `ft13_dry_run_does_not_skip_ownership` (in `tests/cli/account_mutations_test.rs`)
- **Note:** G5 + dry-run ordering gate from Feature 036 AC-13. The ownership guard runs before `dry::1` evaluation — preventing false "would succeed" signals on non-owned accounts.
- **Source:** [004_account_use.md AC-11](../../../docs/feature/004_account_use.md)
