# Feature: Persistent Storage Path

### Scope

- **Purpose**: Give `claude_profile` a user-controlled persistent storage root that survives machine migration.
- **Responsibility**: Documents the `PersistPaths` type and its environment-variable-based path resolution (FR-15).
- **In Scope**: `$PRO`/`$HOME`/`$USERPROFILE` fallback chain, directory creation on first use, `PersistPaths::base()`.
- **Out of Scope**: What is stored under this path (credential data, account snapshots — see FR-7/FR-8).

### Design

`claude_profile` must expose a `PersistPaths` type that resolves a user-controlled persistent storage root from environment variables using stdlib only (no third-party crates).

**Resolution chain:**

1. Try `$PRO` — accepted only if set **and** points to an existing directory on disk.
   - If `$PRO` is set but points to a file or non-existent path → silently skip, continue to step 2.
2. Fall back to `$HOME` (Linux/macOS) or `$USERPROFILE` (Windows).
3. Derive base path: `{root}/persistent/claude_profile/`.
4. Create the directory on first use via `create_dir_all` (idempotent).

**Error condition:** If both `$PRO`/`$HOME`/`$USERPROFILE` are unset → return error.

**`PersistPaths::base()`** returns the resolved `PathBuf` to the persistent storage root.

**`ensure_exists()`** creates the directory (idempotent — no error if already present).

### Acceptance Criteria

- **AC-01**: `$PRO` set to an existing directory → `base()` returns `$PRO/persistent/claude_profile/`.
- **AC-02**: `$PRO` set to a file path → skipped; `base()` falls back to `$HOME/persistent/claude_profile/`.
- **AC-03**: `$PRO` unset → `base()` uses `$HOME/persistent/claude_profile/`.
- **AC-04**: `ensure_exists()` is idempotent — calling twice does not return an error.
- **AC-05**: All `$PRO` and `$HOME`/`$USERPROFILE` unset → returns error.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/persist.rs` | `PersistPaths` struct, resolution chain, `base()`, `ensure_exists()` |
| test | `tests/persist_test.rs::p01–p15` | Full resolution chain, idempotency, error cases |
