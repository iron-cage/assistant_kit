# Test: Feature 010 — Persistent Storage Path

Feature behavioral requirement test cases for `docs/feature/010_persistent_storage.md` (FR-15). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `$PRO` existing dir → `base()` returns path under `$PRO` | AC-01 |
| FT-02 | `$PRO` set to a file path → falls back to `$HOME` | AC-02 |
| FT-03 | `$PRO` unset → `base()` uses `$HOME` | AC-03 |
| FT-04 | `ensure_exists()` is idempotent — second call is not an error | AC-04 |
| FT-05 | Both `$PRO` and `$HOME` unset → returns error | AC-05 |
| FT-06 | `$PRO` existing → `credential_store()` returns path under `$PRO` | AC-06 |
| FT-07 | `$PRO` unset → `credential_store()` uses `$HOME` | AC-07 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `$PRO` existing dir → `base()` under `$PRO` | AC-01 | Path Resolution |
| FT-02 | `$PRO` is a file path → fallback to `$HOME` | AC-02 | Fallback |
| FT-03 | `$PRO` unset → `base()` under `$HOME` | AC-03 | Path Resolution |
| FT-04 | `ensure_exists()` called twice — no error | AC-04 | Idempotency |
| FT-05 | All env vars unset → `PersistPaths` returns error | AC-05 | Error Condition |
| FT-06 | `$PRO` existing → `credential_store()` under `$PRO` | AC-06 | Path Resolution |
| FT-07 | `$PRO` unset → `credential_store()` under `$HOME` | AC-07 | Path Resolution |

**Total:** 7 FT cases

---

### FT-01: `$PRO` existing dir → `base()` under `$PRO`

- **Given:** `$PRO` is set to an existing directory. `$HOME` is also set.
- **When:** `PersistPaths::new()` is constructed; `base()` is called.
- **Then:** Returns a path starting with `$PRO` and ending with `.persistent/claude_profile`.
- **Exit:** Ok
- **Source fn:** `p01_pro_set_base_under_pro`
- **Source:** [010_persistent_storage.md AC-01](../../../docs/feature/010_persistent_storage.md)

---

### FT-02: `$PRO` is a file path → fallback to `$HOME`

- **Given:** `$PRO` is set but points to an existing FILE (not a directory). `$HOME` is set.
- **When:** `PersistPaths::new()` is constructed; `base()` is called.
- **Then:** Falls back silently to `$HOME`; returns path under `$HOME/.persistent/claude_profile`.
- **Exit:** Ok
- **Source fn:** `p14_pro_set_to_existing_file_falls_back_to_home`
- **Source:** [010_persistent_storage.md AC-02](../../../docs/feature/010_persistent_storage.md)

---

### FT-03: `$PRO` unset → `base()` under `$HOME`

- **Given:** `$PRO` is not set. `$HOME` is set to a known path.
- **When:** `PersistPaths::new()` is constructed; `base()` is called.
- **Then:** Returns path starting with `$HOME` and ending with `.persistent/claude_profile`.
- **Exit:** Ok
- **Source fn:** `p02_pro_unset_home_set_base_under_home`, `p09_path_shape_ends_with_persistent_claude_profile_under_home`
- **Source:** [010_persistent_storage.md AC-03](../../../docs/feature/010_persistent_storage.md)

---

### FT-04: `ensure_exists()` called twice — no error

- **Given:** `PersistPaths` resolved successfully.
- **When:** `ensure_exists()` is called once, creating the directory. `ensure_exists()` is called a second time.
- **Then:** Both calls return `Ok`. No error for pre-existing directory.
- **Exit:** Ok
- **Source fn:** `p10_ensure_exists_is_idempotent`, `p11_ensure_exists_succeeds_when_dir_already_exists`
- **Source:** [010_persistent_storage.md AC-04](../../../docs/feature/010_persistent_storage.md)

---

### FT-05: All env vars unset → `PersistPaths` returns error

- **Given:** Neither `$PRO`, `$HOME`, nor `$USERPROFILE` is set.
- **When:** `PersistPaths::new()` is called.
- **Then:** Returns `Err(…)`. Error message is actionable.
- **Exit:** Err
- **Source fn:** `p03_both_unset_returns_err`, `p12_error_message_is_actionable_when_both_unset`
- **Source:** [010_persistent_storage.md AC-05](../../../docs/feature/010_persistent_storage.md)

---

### FT-06: `$PRO` existing → `credential_store()` under `$PRO`

- **Given:** `$PRO` is set to an existing directory.
- **When:** `PersistPaths::credential_store()` is called.
- **Then:** Returns path starting with `$PRO` and ending with `.persistent/claude/credential`.
- **Exit:** Ok
- **Source fn:** `p16_credential_store_under_pro`, `p17_credential_store_path_shape_under_pro`
- **Source:** [010_persistent_storage.md AC-06](../../../docs/feature/010_persistent_storage.md)

---

### FT-07: `$PRO` unset → `credential_store()` under `$HOME`

- **Given:** `$PRO` is not set. `$HOME` is set.
- **When:** `PersistPaths::credential_store()` is called.
- **Then:** Returns path starting with `$HOME` and ending with `.persistent/claude/credential`.
- **Exit:** Ok
- **Source fn:** `p18_credential_store_path_shape_under_home`
- **Source:** [010_persistent_storage.md AC-07](../../../docs/feature/010_persistent_storage.md)
