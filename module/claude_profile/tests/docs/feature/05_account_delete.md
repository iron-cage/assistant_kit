# Test: Feature 005 — Delete Account

Feature behavioral requirement test cases for `docs/feature/005_account_delete.md` (FR-10). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Delete inactive account removes credential file, exits 0 | AC-01 |
| FT-02 | Delete active account removes credential file + per-machine active marker | AC-02 |
| FT-03 | Non-existent account exits 2 | AC-03 |
| FT-04 | Dry-run prints message without removing any files | AC-04 |
| FT-05 | Snapshot files also removed; absent snapshots cause no error | AC-05 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Inactive account: credential file removed, exits 0 | AC-01 | Delete |
| FT-02 | Active account: credential file + active marker both removed | AC-02 | Delete Active |
| FT-03 | Non-existent account exits 2 with not-found error | AC-03 | Error Handling |
| FT-04 | Dry-run prints message; no files removed | AC-04 | Dry Run |
| FT-05 | Snapshot files removed with credentials; absent snapshots are no-ops | AC-05 | Snapshot Cleanup |

**Total:** 5 FT cases

---

### FT-01: Inactive account: credential file removed, exits 0

- **Given:** Two accounts in the store: `alice@acme.com` (inactive) and `work@acme.com` (active). `alice@acme.com.credentials.json` exists.
- **When:** `clp .account.delete name::alice@acme.com`
- **Then:** Exit 0. `{credential_store}/alice@acme.com.credentials.json` is removed. The active account (`work@acme.com`) and its active marker are unaffected.
- **Exit:** 0
- **Source fn:** `ad01_delete_inactive_removes_file`
- **Source:** [005_account_delete.md AC-01](../../../docs/feature/005_account_delete.md)

---

### FT-02: Active account: credential file + active marker both removed

- **Given:** One account `alice@acme.com` which is the active account (per-machine active marker `_active_{hostname}_{user}` contains `alice@acme.com`).
- **When:** `clp .account.delete name::alice@acme.com`
- **Then:** Exit 0. `{credential_store}/alice@acme.com.credentials.json` is removed. `{credential_store}/_active_{hostname}_{user}` is also removed (leaving no active account). The system is now in a "no active account" state.
- **Exit:** 0
- **Source fn:** `ad03_delete_active_exits_0`
- **Source:** [005_account_delete.md AC-02](../../../docs/feature/005_account_delete.md)

---

### FT-03: Non-existent account exits 2 with not-found error

- **Given:** Credential store exists but `ghost@example.com.credentials.json` is not present.
- **When:** `clp .account.delete name::ghost@example.com`
- **Then:** Exit 2. Stderr contains a not-found message identifying `ghost@example.com`.
- **Exit:** 2
- **Source fn:** `ad04_delete_nonexistent_exits_2`
- **Source:** [005_account_delete.md AC-03](../../../docs/feature/005_account_delete.md)

---

### FT-04: Dry-run prints message; no files removed

- **Given:** Three scenarios: inactive account, active account, non-existent account.
- **When (inactive):** `clp .account.delete name::alice@acme.com dry::1`
- **Then:** Exit 0. Output contains `[dry-run] would delete account 'alice@acme.com'`. Credential file is still present.
- **When (active):** `clp .account.delete name::active@acme.com dry::1`
- **Then:** Exit 0. `[dry-run]` message printed; credential file and active marker file unchanged.
- **When (not found):** `clp .account.delete name::ghost@example.com dry::1`
- **Then:** Exit 2. Not-found guard fires before the dry-run action.
- **Exit:** 0 / 2
- **Source fn:** `ad02_delete_dry_run_keeps_file`, `ad10_delete_dry_run_active_exits_0`, `ad11_delete_dry_run_nonexistent_exits_2`
- **Source:** [005_account_delete.md AC-04](../../../docs/feature/005_account_delete.md)

---

### FT-05: Snapshot files removed with credentials; absent snapshots are no-ops

- **Given:** Account `alice@acme.com` with `alice@acme.com.credentials.json` and `alice@acme.com.json` both present in the credential store.
- **When:** `clp .account.delete name::alice@acme.com`
- **Then:** Exit 0. Both files are removed: `.credentials.json` and `.json`. Deletion is best-effort per file; an absent snapshot file does not cause a non-zero exit.
- **Exit:** 0
- **Source fn:** `ad12_delete_removes_snapshot_files`, `ad15_delete_removes_roles_json`
- **Source:** [005_account_delete.md AC-05](../../../docs/feature/005_account_delete.md)
