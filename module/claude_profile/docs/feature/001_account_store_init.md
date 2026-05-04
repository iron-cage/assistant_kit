# Feature: Account Store Initialization

### Scope

- **Purpose**: Ensure the credential store directory is ready for use on first access without requiring manual setup.
- **Responsibility**: Documents the automatic creation of the credential store directory on first use and its path resolution (FR-6).
- **In Scope**: Credential store path resolution (`$PRO`/`$HOME` chain), directory creation on save when store does not exist.
- **Out of Scope**: Account file creation (→ 002_account_save.md), general persistent storage (→ 010_persistent_storage.md).

### Design

`claude_profile` must create the credential store directory on first use if it does not exist.

**Credential store path resolution:**

1. Try `$PRO` — accepted only if set **and** points to an existing directory on disk.
   - If `$PRO` points to a file or non-existent path → silently skip, continue to step 2.
2. Fall back to `$HOME` (Linux/macOS) or `$USERPROFILE` (Windows).
3. Credential store: `{root}/.persistent/claude/credential/`

**Error condition:** If both `$PRO` and `$HOME`/`$USERPROFILE` are unset → account operations return an error.

**Initialization:**
- Triggered automatically by the first call to any operation that writes to the credential store (e.g., `account::save()`).
- Uses `std::fs::create_dir_all` so that intermediate directories are created without error.
- Idempotent: if the directory already exists, the call is a no-op.

The caller does not need to know whether the store was newly created or pre-existing — both cases succeed identically.

### Acceptance Criteria

- **AC-01**: Saving an account when `{credential_store}` does not exist creates the directory and the credential file successfully (exit 0).
- **AC-02**: Saving an account when `{credential_store}` already exists succeeds without error (idempotency).
- **AC-03**: `$PRO` set to an existing directory → credential store resolves to `$PRO/.persistent/claude/credential/`.
- **AC-04**: `$PRO` unset or not a directory → credential store resolves to `$HOME/.persistent/claude/credential/`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | Account CRUD operations including credential store path resolution and directory initialization |
| source | `src/persist.rs` | `PersistPaths` type — resolves credential store path via `$PRO`/`$HOME` chain (consumed by account operations) |
| test | `tests/account_tests.rs::save_creates_credential_store_when_missing` | Verifies credential store created on first save (AC-01) |
| test | `tests/account_tests.rs::save_copies_credentials_to_named_file` | Verifies idempotency — second save into existing store succeeds (AC-02) |
| test | `tests/cli/persist_test.rs::p01` | `$PRO` set → credential store resolves under `$PRO` (AC-03) |
| test | `tests/cli/persist_test.rs::p02` | `$PRO` unset, `$HOME` set → credential store resolves under `$HOME` (AC-04) |
| test | `tests/cli/persist_test.rs::p16` | `credential_store()` under `$PRO` path starts with `$PRO` (AC-03) |
| test | `tests/cli/persist_test.rs::p17` | `credential_store()` path ends with `.persistent/claude/credential` (AC-03) |
| test | `tests/cli/persist_test.rs::p18` | `credential_store()` path ends with `.persistent/claude/credential` under `$HOME` (AC-04) |
| doc | [010_persistent_storage.md](010_persistent_storage.md) | General persistent storage path (`PersistPaths`); same `$PRO`/`$HOME` resolution chain |
