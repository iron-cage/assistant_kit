# Feature: Account Store Initialization

### Scope

- **Purpose**: Ensure the account store directory is ready for use on first access without requiring manual setup.
- **Responsibility**: Documents the automatic creation of `~/.claude/accounts/` on first use.
- **In Scope**: FR-6 — directory creation on save when accounts/ does not exist.
- **Out of Scope**: Account file creation (→ 002_account_save.md), path resolution (→ 007_file_topology.md).

### Design

`claude_profile` must create `~/.claude/accounts/` on first use if it does not exist.

- Triggered automatically by the first call to any operation that writes to the account store (e.g., `account::save()`).
- Uses `std::fs::create_dir_all` so that intermediate directories are created without error.
- Idempotent: if the directory already exists, the call is a no-op.
- On Linux/macOS: standard filesystem permissions are applied.
- On Windows: standard directory creation semantics apply.

The caller does not need to know whether the store was newly created or pre-existing — both cases succeed identically.

### Acceptance Criteria

- **AC-01**: Saving an account when `~/.claude/accounts/` does not exist creates the directory and the credential file successfully (exit 0).
- **AC-02**: Saving an account when `~/.claude/accounts/` already exists succeeds without error (idempotency).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | Account CRUD operations including directory initialization |
| test | `tests/account_tests.rs::save_creates_accounts_dir_when_missing` | Verifies directory created on first save |
