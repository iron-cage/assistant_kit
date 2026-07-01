# Schema 004: Storage Root — `PersistPaths`

SC test cases for `docs/schema/004_storage_root.md`. Verifies the `PersistPaths`
resolution chain ($PRO → $HOME → error), the computed path shapes, and the
`ensure_exists()` idempotent directory creation API.

**Source:** [docs/schema/004_storage_root.md](../../../../docs/schema/004_storage_root.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | `$PRO` set and exists → storage rooted under `$PRO` | Resolution Chain | ✅ |
| SC-2 | `$PRO` set but non-existent → falls through to `$HOME` | Resolution Chain | ✅ |
| SC-3 | `base()` path ends with `.persistent/claude_profile/` | Path Shape | ✅ |
| SC-4 | `ensure_exists()` creates directory idempotently | API | ✅ |

---

### SC-1: `$PRO` set and pointing to existing directory → storage rooted under `$PRO`

- **Given:** `$PRO` env var is set to an existing directory path
- **When:** `PersistPaths::new()` or equivalent is called
- **Then:** `base()` returns `{$PRO}/.persistent/claude_profile/` and `credential_store()` returns `{$PRO}/.persistent/claude/credential/`
- **Source fn:** `p16_credential_store_under_pro` (cli/persist_test.rs)
- **Source:** [docs/schema/004_storage_root.md §Resolution Chain](../../../../docs/schema/004_storage_root.md)

---

### SC-2: `$PRO` set but pointing to non-existent path → silently falls through to `$HOME`

- **Given:** `$PRO` env var is set to a path that does not exist on disk
- **When:** `PersistPaths::new()` is called
- **Then:** Falls through to `$HOME`-based resolution — `base()` returns `{$HOME}/.persistent/claude_profile/`; no error is raised for the non-existent `$PRO` path
- **Source fn:** `p09_path_shape_ends_with_persistent_claude_profile_under_home` (cli/persist_test.rs)
- **Source:** [docs/schema/004_storage_root.md §Resolution Chain](../../../../docs/schema/004_storage_root.md)

---

### SC-3: `base()` path shape ends with `.persistent/claude_profile/`

- **Given:** `PersistPaths` resolved from either `$PRO` or `$HOME`
- **When:** `base()` is called
- **Then:** The returned path ends with `/.persistent/claude_profile/` regardless of the root source
- **Source fn:** `p04_base_path_shape_ends_with_persistent_claude_profile` (cli/persist_test.rs)
- **Source:** [docs/schema/004_storage_root.md §PersistPaths API](../../../../docs/schema/004_storage_root.md)

---

### SC-4: `ensure_exists()` creates the base directory idempotently

- **Given:** The base directory at `{root}/.persistent/claude_profile/` does not yet exist
- **When:** `ensure_exists()` is called
- **Then:** The directory is created; calling `ensure_exists()` again on an already-existing directory succeeds without error (idempotent)
- **Source fn:** `p05_ensure_exists_creates_directory` and `p10_ensure_exists_is_idempotent` (cli/persist_test.rs)
- **Source:** [docs/schema/004_storage_root.md §PersistPaths API](../../../../docs/schema/004_storage_root.md)
