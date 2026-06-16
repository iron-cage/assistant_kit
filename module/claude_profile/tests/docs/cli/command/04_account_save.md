# Test: `.account.save`

Integration test planning for the `.account.save` command. See [command/namespace.md](../../../../docs/cli/command/001_account.md#command--4-accountsave) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Save creates credential file in `~/.persistent/claude/credential/` | Basic Invocation |
| IT-2 | Save creates credential store if missing | Directory Init |
| IT-3 | Save with existing name overwrites silently | Overwrite |
| IT-4 | Save with non-email name exits 1 | Validation |
| IT-5 | Save with empty `name::` exits 1 | Validation |
| IT-6 | Save when `~/.claude/.credentials.json` missing exits 2 | Error Handling |
| IT-7 | `dry::1` prints action without creating file | Dry Run |
| IT-8 | `dry::1` then `dry::0` creates file as previewed | Dry Run Fidelity |
| IT-9 | Saved file content matches active credentials exactly | Data Integrity |
| IT-10 | Missing `name::` and no active marker exits 1 | Inference Failure |
| IT-14 | Missing `name::` — infers from `oauthAccount.emailAddress`; falls back to active marker | Name Inference |
| IT-11 | Save creates `{name}.json` with `oauthAccount` subtree and model (BUG-222 fix) | Metadata Snapshot |
| IT-12 | Save succeeds when `~/.claude.json` absent — `{name}.json` created without `oauthAccount` field | Metadata Snapshot / Best-Effort |
| IT-13 | Save succeeds when `~/.claude.json` has no `oauthAccount` key — `{name}.json` created without `oauthAccount` | Metadata Snapshot / Best-Effort |
| IT-15 | Save writes active marker — `.credentials.status` shows `Account: {name}` immediately after save | Active Marker |
| IT-16 | Save with path-unsafe chars in email local part (`/`, `\`) exits 1 | Validation |
| IT-17 | Save writes org identity to `{name}.json` when endpoint 005 returns org identity | Org Identity Snapshot |
| IT-18 | Save succeeds even when endpoint 005 call fails — no org fields in `{name}.json`, no error | Org Identity Snapshot / Best-Effort |
| IT-19 | Stale `_active` marker overridden by `oauthAccount.emailAddress` (BUG-212) | Name Inference / Regression |
| IT-20 | Save does NOT modify `owner` field — passes `owner: None`; existing value preserved via read-merge | Ownership Neutral |

### Test Coverage Summary

- Basic Invocation: 1 test
- Directory Init: 1 test
- Overwrite: 1 test
- Validation: 3 tests
- Error Handling: 1 test
- Dry Run: 2 tests
- Data Integrity: 1 test
- Inference Failure: 1 test
- Name Inference: 1 test
- Metadata Snapshot: 1 test
- Metadata Snapshot / Best-Effort: 2 tests
- Active Marker: 1 test
- Org Identity Snapshot: 1 test
- Org Identity Snapshot / Best-Effort: 1 test
- Name Inference / Regression: 1 test
- Ownership Neutral: 1 test

**Total:** 20 integration tests

---

### IT-1: Save creates credential file in `~/.persistent/claude/credential/`

- **Given:** Create `~/.claude/.credentials.json` with valid credential content. Create `~/.persistent/claude/credential/` directory (empty).
- **When:** `clp .account.save name::work@acme.com`
- **Then:** stdout: `saved current credentials as 'work@acme.com'`. File `~/.persistent/claude/credential/work@acme.com.credentials.json` now exists.; credential file created at expected path
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-2: Save creates credential store if missing

- **Given:** Create `~/.claude/.credentials.json` with valid credential content. Ensure `~/.persistent/claude/credential/` does not exist.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** stdout: `saved current credentials as 'work@acme.com'`. Both `~/.persistent/claude/credential/` directory and `~/.persistent/claude/credential/work@acme.com.credentials.json` file now exist.; directory auto-created; credential file created
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-3: Save with existing name overwrites silently

- **Given:** Create `~/.claude/.credentials.json` with credential content V2. Create `~/.persistent/claude/credential/work@acme.com.credentials.json` with older credential content V1.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** stdout: `saved current credentials as 'work@acme.com'`. File `~/.persistent/claude/credential/work@acme.com.credentials.json` now contains V2 content.; file overwritten with current credentials; no error output
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-4: Save with non-email name exits 1

- **Given:** Create `~/.claude/.credentials.json` with valid credential content.
- **When:** `clp .account.save name::notanemail`
- **Then:** Error message on stderr indicating the name must be a valid email address. No file created.; no file created; error message references email format
- **Exit:** 1
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-5: Save with empty `name::` exits 1

- **Given:** Create `~/.claude/.credentials.json` with valid credential content.
- **When:** `clp .account.save name::`
- **Then:** Error message indicating the account name must not be empty. No file created.; no file created; error message about empty name
- **Exit:** 1
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-6: Save when `~/.claude/.credentials.json` missing exits 2

- **Given:** Ensure `~/.claude/.credentials.json` does not exist. Create `~/.persistent/claude/credential/` directory.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** Error message indicating credentials file is unreadable or missing. No account file created.; no file created; error message about missing credentials
- **Exit:** 2
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-7: `dry::1` prints action without creating file

- **Given:** Create `~/.claude/.credentials.json` with valid credential content. Create `~/.persistent/claude/credential/` directory (empty).
- **When:** `clp .account.save name::work@acme.com dry::1`
- **Then:** stdout: `[dry-run] would save current credentials as 'work@acme.com'`. No file created.; dry-run message printed; no file created on disk
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-8: `dry::1` then `dry::0` creates file as previewed

- **Given:** Create `~/.claude/.credentials.json` with valid credential content. Create `~/.persistent/claude/credential/` directory (empty).
- **When:** `clp .account.save name::work@acme.com dry::1` followed by `clp .account.save name::work@acme.com`
- **Then:** First command: `[dry-run] would save current credentials as 'work@acme.com'` (no file). Second command: `saved current credentials as 'work@acme.com'` (file created).; on both; dry run creates nothing; real run creates exactly the previewed file
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-9: Saved file content matches active credentials exactly

- **Given:** Create `~/.claude/.credentials.json` with known credential content (e.g., specific JSON with `accessToken`, `refreshToken`, `expiresAt` fields).
- **When:** `clp .account.save name::work@acme.com`
- **Then:** `~/.persistent/claude/credential/work@acme.com.credentials.json` is a byte-identical copy of `~/.claude/.credentials.json`.; saved file is byte-identical to source credentials
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-10: Missing `name::` — no active marker exits 1

- **Given:** Create `~/.claude/.credentials.json` with valid credential content. Ensure no `_active_{hostname}_{user}` marker file exists in `{credential_store}` (or credential store itself is absent).
- **When:** `clp .account.save`
- **Then:** Error message: `cannot infer account name: no active account set — pass name:: explicitly`. No file created.
- **Exit:** 1
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-11: Save creates `oauthAccount` + settings snapshots

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` exists with `oauthAccount.displayName = "alice"`. `~/.claude/settings.json` exists with `model = "sonnet"`.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** `{credential_store}/work@acme.com.json` created containing `{"oauthAccount": {...}, "model": "sonnet"}` (BUG-222 fix: model preference captured for restore by `switch_account()`).
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-12: Save succeeds when `~/.claude.json` absent — best-effort snapshot

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` does NOT exist.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** Credential file created. `{name}.json` created but contains no `oauthAccount` field (source absent — best-effort skipped). No error emitted; save completes successfully; no error for absent optional source.
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-13: Save extracts `oauthAccount` only; `~/.claude.json` present with no `oauthAccount` key → no `oauthAccount` in snapshot

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` exists but contains only `{"commands": {"foo": 1}}` (no `oauthAccount` key).
- **When:** `clp .account.save name::work@acme.com`
- **Then:** Credential file created. `{name}.json` created but contains no `oauthAccount` key (absent in source — extraction skipped). No error emitted; save completes successfully.
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-14: Missing `name::` — name inferred from `oauthAccount.emailAddress`; falls back to active marker

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `~/.claude.json` contains `oauthAccount.emailAddress = "alice@acme.com"`. Per-machine active marker `{credential_store}/_active_{hostname}_{user}` also contains `"alice@acme.com"`.
- **When:** `clp .account.save`
- **Then:** stdout: `saved current credentials as 'alice@acme.com'`. Credential file `{credential_store}/alice@acme.com.credentials.json` created; `name::` inferred from `oauthAccount.emailAddress`; behaves identically to explicit `name::alice@acme.com`.
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave), [002_account_save.md AC-08](../../../../docs/feature/002_account_save.md)

---

### IT-15: Save writes active marker — `.credentials.status` shows account immediately

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. No active marker (`_active_{hostname}_{user}`) exists in `{credential_store}`.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** `{credential_store}/_active_{hostname}_{user}` contains the text `work@acme.com`. Subsequent `clp .credentials.status` shows `Account: work@acme.com` (not `N/A`).
- **Exit:** 0
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave)

---

### IT-16: Save with path-unsafe chars in email local part exits 1

- **Given:** `~/.claude/.credentials.json` exists with valid credentials.
- **When:** `clp .account.save name::a/b@c.com`
- **Then:** Error message on stderr indicating path-unsafe characters in account name, exit 1. No credential file created in the store.
- **Exit:** 1
- **Source:** [command/001_account.md — .account.save](../../../../docs/cli/command/001_account.md#command--4-accountsave), [002_account_save.md AC-11](../../../../docs/feature/002_account_save.md), [as17/as18 in account_mutations_test.rs]

---

### IT-17: Save writes org identity to `{name}.json` when endpoint 005 returns org identity

- **Given:** `~/.claude/.credentials.json` exists with valid credentials that allow endpoint 005 to return `{"organization_uuid":"org-xyz","organization_name":"Acme Corp"}`.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** `{credential_store}/work@acme.com.json` contains `organization_uuid` and `organization_name` fields with the returned values. Credential file and metadata snapshot also created. Exit 0 with normal success message.
- **Exit:** 0
- **Source:** [022_org_identity_snapshot.md AC-01](../../../../docs/feature/022_org_identity_snapshot.md), [002_account_save.md AC-12](../../../../docs/feature/002_account_save.md)

---

### IT-18: Save succeeds even when endpoint 005 call fails — best-effort

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. Endpoint 005 is unreachable or returns an error (simulate with invalid/expired credentials or network mock if available).
- **When:** `clp .account.save name::work@acme.com`
- **Then:** Exit 0 with normal success message. Credential file and metadata snapshot created. `{credential_store}/work@acme.com.json` does NOT contain org identity fields. No error message on stderr referencing endpoint 005 failure.
- **Exit:** 0
- **Source:** [022_org_identity_snapshot.md AC-02](../../../../docs/feature/022_org_identity_snapshot.md), [002_account_save.md AC-13](../../../../docs/feature/002_account_save.md)

---

### IT-19: Stale `_active` marker — `oauthAccount.emailAddress` wins (BUG-212 regression)

- **Given:** `~/.claude/.credentials.json` exists with live credentials. `~/.claude.json` contains `oauthAccount.emailAddress = "i5@wbox.pro"` (fresh — written by external OAuth login). Per-machine active marker `{credential_store}/_active_{hostname}_{user}` contains `"i2@wbox.pro"` (stale — from prior clp session). No `name::` passed.
- **When:** `clp .account.save`
- **Then:** Exit 0. stdout: `saved current credentials as 'i5@wbox.pro'`. `{credential_store}/i5@wbox.pro.credentials.json` created. `{credential_store}/i2@wbox.pro.credentials.json` NOT created or modified — stale marker ignored when `oauthAccount.emailAddress` is available.
- **Exit:** 0
- **Source fn:** `mre_bug_212_account_save_stale_marker_uses_oauth_email` (in `tests/cli/account_mutations_test.rs`)
- **Source:** [002_account_save.md AC-16](../../../../docs/feature/002_account_save.md)

---

### IT-20: Save does NOT modify `owner` field — `owner: None` passed to `save()`

- **Given:** `~/.claude/.credentials.json` exists with valid credentials. `{credential_store}/work@acme.com.json` already exists and contains `"owner": "user1@host1"`.
- **When:** `clp .account.save name::work@acme.com`
- **Then:** Exit 0. stdout: `saved current credentials as 'work@acme.com'`. `{credential_store}/work@acme.com.json` still contains `"owner": "user1@host1"` — unchanged. `account_save_routine()` passes `owner: None` to `save()`; the `owner` field is preserved via read-merge.
- **Exit:** 0
- **Source fn:** `as_save_does_not_modify_owner`
- **Source:** [002_account_save.md AC-19](../../../../docs/feature/002_account_save.md)
