# Test: Path Discovery

Acceptance tests for User Story 008. See [user_story/008_path_discovery.md](../../../../docs/cli/user_story/008_path_discovery.md) for specification.

### Scope

- **Purpose**: Verify `.paths` reports all clv-managed filesystem paths with labels, single-key lookup, and scriptable output.
- **Responsibility**: Acceptance criteria coverage for the path discovery workflow.
- **Commands:** `.paths`
- **In Scope**: Show-all mode, single-key mode, plain/labeled/described verbosity tiers, JSON output, unresolvable path handling.
- **Out of Scope**: Unlabeled pipeline enumeration (-> `../command/15_runtime_files.md`), key edge cases (-> `../type/09_path_key.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.paths` shows all 5 keys, each labeled | Acceptance: show-all |
| US-2 | `.paths key::versions_dir` shows single resolved path | Acceptance: single-key |
| US-3 | `.paths v::0` outputs plain unlabeled paths | Acceptance: scripting |
| US-4 | `.paths format::json` returns valid JSON object with all keys | Acceptance: JSON output |
| US-5 | Unresolvable `project_settings` shown as "(none found)" at v::1 | Acceptance: unresolvable handling |
| US-6 | `.paths key::bogus` exits 1 | Acceptance: error handling |
| US-7 | `.paths v::2` shows descriptions | Acceptance: verbose detail |

## Test Coverage Summary

- Show-all: 1 test (US-1)
- Single-key: 1 test (US-2)
- Scripting output: 1 test (US-3)
- JSON output: 1 test (US-4)
- Unresolvable handling: 1 test (US-5)
- Error paths: 1 test (US-6)
- Verbose detail: 1 test (US-7)

**Total:** 7 tests

## Test Case Details

---

### US-1: `.paths` shows all 5 keys, each labeled

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths`
- **Then:** exit 0; stdout contains a labeled line for each of `settings`, `project_settings`, `versions_dir`, `binary_symlink`, `version_history_cache`
- **Exit:** 0

---

### US-2: `.paths key::versions_dir` shows single resolved path

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::versions_dir`
- **Then:** exit 0; stdout contains exactly the resolved versions directory path
- **Exit:** 0

---

### US-3: `.paths v::0` outputs plain unlabeled paths

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths v::0`
- **Then:** exit 0; stdout lines contain no labels, only raw paths, suitable for `xargs`/`while read` consumption
- **Exit:** 0

---

### US-4: `.paths format::json` returns valid JSON object with all keys

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths format::json`
- **Then:** exit 0; stdout is valid JSON parseable as an object; object has keys `settings`, `project_settings`, `versions_dir`, `binary_symlink`, `version_history_cache`
- **Exit:** 0

---

### US-5: Unresolvable project_settings shown as "(none found)" at v::1

- **Given:** `HOME=<tmp>`; no ancestor `.claude/settings.json` from current directory
- **When:** `clv.paths`
- **Then:** exit 0; stdout contains `project_settings:` followed by a `(none found)` placeholder
- **Exit:** 0

---

### US-6: `.paths key::bogus` exits 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::bogus`
- **Then:** exit 1; stderr names the invalid key and lists valid values
- **Exit:** 1

---

### US-7: `.paths v::2` shows descriptions

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::binary_symlink v::2`
- **Then:** exit 0; stdout contains the path plus a one-line description of the path's purpose
- **Exit:** 0

---

## Source Functions Table

| Function | File | Test Cases |
|----------|------|------------|
| `us01_008_paths_show_all_keys` | `tests/cli/user_story_test.rs` | US-1 |
| `us02_008_paths_single_key` | `tests/cli/user_story_test.rs` | US-2 |
| `us03_008_paths_v0_unlabeled` | `tests/cli/user_story_test.rs` | US-3 |
| `us04_008_paths_json_object` | `tests/cli/user_story_test.rs` | US-4 |
| `us05_008_paths_unresolvable_none_found` | `tests/cli/user_story_test.rs` | US-5 |
| `us06_008_paths_invalid_key_exits_1` | `tests/cli/user_story_test.rs` | US-6 |
| `us07_008_paths_v2_descriptions` | `tests/cli/user_story_test.rs` | US-7 |
