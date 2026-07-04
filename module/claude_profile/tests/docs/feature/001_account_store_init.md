# Test: Feature 001 — Account Store Initialization

### Scope

- **Purpose**: Test cases for credential store directory creation and `$PRO`/`$HOME` path resolution.
- **Source**: `docs/feature/001_account_store_init.md`
- **Covers**: AC-01 through AC-04

Feature behavioral requirement test cases for `docs/feature/001_account_store_init.md` (FR-6). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Save creates credential store directory on first use | AC-01 |
| FT-02 | Save succeeds idempotently when store already exists | AC-02 |
| FT-03 | `$PRO` set to existing directory → store resolves under `$PRO` | AC-03 |
| FT-04 | `$PRO` unset or not a directory → store resolves under `$HOME` | AC-04 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Save when store absent creates directory and credential file | AC-01 | Store Init |
| FT-02 | Save when store exists succeeds without error | AC-02 | Idempotency |
| FT-03 | `$PRO` existing dir → credential store under `$PRO` | AC-03 | Path Resolution |
| FT-04 | `$PRO` unset → credential store under `$HOME` | AC-04 | Path Resolution |

**Total:** 4 FT cases

---

### FT-01: Save when store absent creates directory and credential file

- **Given:** The credential store directory (`{root}/.persistent/claude/credential/`) does not exist.
- **When:** `account::save(name, credential_store, paths)` is called with a valid name and credentials.
- **Then:** The credential store directory is created. A `{name}.credentials.json` file is written inside it. The call returns success.
- **Exit:** Ok
- **Source fn:** `save_creates_credential_store_when_missing`
- **Source:** [001_account_store_init.md AC-01](../../../docs/feature/001_account_store_init.md)

---

### FT-02: Save when store exists succeeds without error

- **Given:** The credential store directory already contains at least one account file.
- **When:** `account::save(name, credential_store, paths)` is called a second time with the same or a different account.
- **Then:** The call returns success. No error is produced for the pre-existing directory.
- **Exit:** Ok
- **Source fn:** `save_copies_credentials_to_named_file`
- **Source:** [001_account_store_init.md AC-02](../../../docs/feature/001_account_store_init.md)

---

### FT-03: `$PRO` existing dir → credential store under `$PRO`

- **Given:** `$PRO` is set to an existing directory on disk.
- **When:** `PersistPaths::credential_store()` is resolved.
- **Then:** The returned path starts with `$PRO` and ends with `.persistent/claude/credential`.
- **Exit:** Ok
- **Source fn:** `p16_credential_store_under_pro`, `p17_credential_store_path_shape_under_pro`
- **Source:** [001_account_store_init.md AC-03](../../../docs/feature/001_account_store_init.md)

---

### FT-04: `$PRO` unset → credential store under `$HOME`

- **Given:** `$PRO` is not set (or is not an existing directory). `$HOME` is set.
- **When:** `PersistPaths::credential_store()` is resolved.
- **Then:** The returned path starts with `$HOME` and ends with `.persistent/claude/credential`.
- **Exit:** Ok
- **Source fn:** `p18_credential_store_path_shape_under_home`
- **Source:** [001_account_store_init.md AC-04](../../../docs/feature/001_account_store_init.md)
