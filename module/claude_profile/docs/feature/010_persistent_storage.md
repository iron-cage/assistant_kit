# Feature: Persistent Storage Path

### Scope

- **Purpose**: Give `claude_profile` a user-controlled persistent storage root that survives machine migration.
- **Responsibility**: Documents the `PersistPaths` type and its environment-variable-based path resolution (FR-15).
- **In Scope**: `$PRO`/`$HOME`/`$USERPROFILE` fallback chain, directory creation on first use, `PersistPaths::base()`, `PersistPaths::credential_store()`.
- **Out of Scope**: What is stored under these paths (credential data, account snapshots — see FR-6/FR-7/FR-8).

### Design

`claude_profile` must expose a `PersistPaths` type that resolves a user-controlled persistent storage root from environment variables using stdlib only (no third-party crates). The resolution chain, path methods, and error conditions are documented in [schema/004_storage_root.md](../schema/004_storage_root.md).

### Acceptance Criteria

- **AC-01**: `$PRO` set to an existing directory → `base()` returns `$PRO/.persistent/claude_profile/`.
- **AC-02**: `$PRO` set to a file path → skipped; `base()` falls back to `$HOME/.persistent/claude_profile/`.
- **AC-03**: `$PRO` unset → `base()` uses `$HOME/.persistent/claude_profile/`.
- **AC-04**: `ensure_exists()` is idempotent — calling twice does not return an error.
- **AC-05**: All `$PRO` and `$HOME`/`$USERPROFILE` unset → returns error.
- **AC-06**: `$PRO` set to an existing directory → `credential_store()` returns `$PRO/.persistent/claude/credential/`.
- **AC-07**: `$PRO` unset → `credential_store()` uses `$HOME/.persistent/claude/credential/`.

### Invariants

| File | Relationship |
|------|--------------|
| [007_json_storage_format.md](../invariant/007_json_storage_format.md) | Credential snapshot files under `credential_store()` must be written as 2-space pretty-printed JSON with trailing newline |

### Features

| File | Relationship |
|------|--------------|
| [001_account_store_init.md](001_account_store_init.md) | Persistent storage path is the store location initialized by first-save |

### Sources

| File | Relationship |
|------|--------------|
| `src/persist.rs` | `PersistPaths` struct, resolution chain, `base()`, `credential_store()`, `ensure_exists()` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/persist_test.rs::p01–p15` | Resolution chain (base()), idempotency, error cases (AC-01 through AC-05) |
| `tests/cli/persist_test.rs::p16` | `credential_store()` under `$PRO` — path starts with `$PRO` (AC-06) |
| `tests/cli/persist_test.rs::p17` | `credential_store()` path ends with `.persistent/claude/credential` under `$PRO` (AC-06) |
| `tests/cli/persist_test.rs::p18` | `credential_store()` path ends with `.persistent/claude/credential` under `$HOME` (AC-07) |

### Schema

| File | Relationship |
|------|-------------|
| [schema/004_storage_root.md](../schema/004_storage_root.md) | Resolution chain and `PersistPaths` API reference — extracted from this feature |
