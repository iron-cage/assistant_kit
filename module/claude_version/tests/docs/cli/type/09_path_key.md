# Test: `PathKey`

Type compliance and validation tests for `PathKey`. See [type/09_path_key.md](../../../../docs/cli/type/09_path_key.md) for specification.

### Scope

- **Purpose**: Validate PathKey parsing, case-sensitivity enforcement, and per-variant path resolution.
- **Responsibility**: Valid variants, invalid inputs, default behavior, and observable output differences between key values.
- **Commands:** `.paths`
- **In Scope**: Key string parsing, case-sensitive matching, and observable output differences per variant.
- **Out of Scope**: Per-command JSON schema structure (→ `../command/`), parameter interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `key::settings` → single path (settings.json) | Valid: settings |
| TC-2 | `key::versions_dir` → single path (versions dir) | Valid: versions_dir |
| TC-3 | `key::binary_symlink` → single path (symlink) | Valid: binary_symlink |
| TC-4 | `key::version_history_cache` → single path (cache) | Valid: version_history_cache |
| TC-5 | `key::project_settings` → single path or "(none found)" | Valid: project_settings |
| TC-6 | Absent `key::` → all 5 paths shown | Default |
| TC-7 | `key::Settings` → exit 1 (case-sensitive) | Validation: case |
| TC-8 | `key::bogus` → exit 1 (unknown variant) | Validation: unknown |
| TC-9 | `key::` (empty) → exit 1 | Validation: empty |

## Test Coverage Summary

- Valid variant resolution: 5 tests (TC-1 through TC-5)
- Default Behavior: 1 test (TC-6)
- Case sensitivity: 1 test (TC-7)
- Unknown variant: 1 test (TC-8)
- Empty value: 1 test (TC-9)

**Total:** 9 tests

**Behavioral Divergence Pair:** TC-2 (`key::versions_dir` → always resolves, exit 0) ↔ TC-5 (`key::project_settings` → may resolve to "(none found)", exit 0)

---

### TC-1: `key::settings` → settings.json path

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::settings`
- **Then:** exit 0; output is exactly `<tmp>/.claude/settings.json`
- **Exit:** 0
- **Source:** [type/09_path_key.md — settings variant](../../../../docs/cli/type/09_path_key.md)

---

### TC-2: `key::versions_dir` → versions directory path

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::versions_dir`
- **Then:** exit 0; output is exactly `<tmp>/.local/share/claude/versions`
- **Exit:** 0
- **Source:** [type/09_path_key.md — versions_dir variant](../../../../docs/cli/type/09_path_key.md)

---

### TC-3: `key::binary_symlink` → symlink path

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::binary_symlink`
- **Then:** exit 0; output is exactly `<tmp>/.local/bin/claude`
- **Exit:** 0
- **Source:** [type/09_path_key.md — binary_symlink variant](../../../../docs/cli/type/09_path_key.md)

---

### TC-4: `key::version_history_cache` → cache path

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::version_history_cache`
- **Then:** exit 0; output is exactly `<tmp>/.claude/.transient/version_history_cache.json`
- **Exit:** 0
- **Source:** [type/09_path_key.md — version_history_cache variant](../../../../docs/cli/type/09_path_key.md)

---

### TC-5: `key::project_settings` → resolved path or placeholder

- **Given:** `HOME=<tmp>`; current directory has an ancestor `.claude/settings.json` at `<proj>/.claude/settings.json`
- **When:** `clv.paths key::project_settings`
- **Then:** exit 0; output is exactly `<proj>/.claude/settings.json`
- **Exit:** 0
- **Source:** [type/09_path_key.md — project_settings variant](../../../../docs/cli/type/09_path_key.md)

---

### TC-6: Absent `key::` → all 5 paths

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths` (no `key::`)
- **Then:** exit 0; output contains all 5 keys: `settings`, `project_settings`, `versions_dir`, `binary_symlink`, `version_history_cache`
- **Exit:** 0
- **Source:** [type/09_path_key.md — Default: absent (no filter)](../../../../docs/cli/type/09_path_key.md)

---

### TC-7: `key::Settings` → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::Settings`
- **Then:** exit 1; stderr contains an error message referencing case-sensitivity or unknown key
- **Exit:** 1
- **Source:** [type/09_path_key.md — Parsing: exact string match](../../../../docs/cli/type/09_path_key.md)

---

### TC-8: `key::bogus` → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::bogus`
- **Then:** exit 1; stderr contains "unknown key" or similar message listing the 5 valid values
- **Exit:** 1
- **Source:** [type/09_path_key.md — Validation errors](../../../../docs/cli/type/09_path_key.md)

---

### TC-9: `key::` (empty) → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::`
- **Then:** exit 1; error message references `key::` or empty value
- **Exit:** 1
- **Source:** [type/09_path_key.md — Validation errors](../../../../docs/cli/type/09_path_key.md)

---

### Source Functions

| Function | File |
|----------|------|
| `path_key_tc1_settings_resolves` | `tests/cli/path_key_test.rs` |
| `path_key_tc2_versions_dir_resolves` | `tests/cli/path_key_test.rs` |
| `path_key_tc3_binary_symlink_resolves` | `tests/cli/path_key_test.rs` |
| `path_key_tc4_version_history_cache_resolves` | `tests/cli/path_key_test.rs` |
| `path_key_tc5_project_settings_resolves_or_placeholder` | `tests/cli/path_key_test.rs` |
| `path_key_tc6_absent_shows_all_keys` | `tests/cli/path_key_test.rs` |
| `path_key_tc7_mixed_case_exits_1` | `tests/cli/path_key_test.rs` |
| `path_key_tc8_unknown_variant_exits_1` | `tests/cli/path_key_test.rs` |
| `path_key_tc9_empty_exits_1` | `tests/cli/path_key_test.rs` |
