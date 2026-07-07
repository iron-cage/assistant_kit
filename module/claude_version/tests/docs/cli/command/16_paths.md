# Test: `.paths`

### Scope

- **Purpose**: Integration test cases for the `.paths` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for show-all, single-key, format, verbosity, and error modes.
- **In Scope**: Mode dispatch, format, verbosity, unresolvable path handling, exit codes.
- **Out of Scope**: `PathKey` unit-level edge cases (→ `../type/09_path_key.md`), `ClaudeVersionPaths` unit tests (→ coverage in `claude_version_core` crate tests).

Integration test planning for `.paths`. See [command/paths.md](../../../../docs/cli/command/paths.md) for specification.

## Test Factor Analysis

### Factor 1: Mode (derived from key:: presence)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| no key:: | show-all mode | Default |
| key::K (known) | single-path mode | Happy path |
| key::K (unknown) | not a valid PathKey | Error: exit 1 |

### Factor 2: `format::` (String, optional, default text)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent / `text` | Human-readable text | Default |
| `json` | Structured JSON object | Alternate valid |

### Factor 3: `v::` (VerbosityLevel, optional, default 1)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| 0 | Plain paths only, unresolved omitted | Valid |
| 1 | Labeled, unresolved shown as placeholder (default) | Default |
| 2 | Labeled + description | Valid |

### Factor 4: `project_settings` resolution

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| found | Ancestor `.claude/settings.json` exists | Happy path |
| not found | No ancestor project config | Unresolvable — verbosity-dependent handling |

### Factor 5: HOME environment variable

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set, non-empty | Valid HOME path | Happy path |
| unset | HOME absent from environment | Error: exit 2 |

---

## Test Matrix

### Positive Tests

| TC | Description | Mode | Exit | Factors |
|----|-------------|------|------|---------|
| IT-1 | No `key::` → show-all with all 5 keys labeled | show-all | 0 | F1=no-key |
| IT-2 | `key::versions_dir` → single resolved path | single | 0 | F1=key-known |
| IT-3 | `key::settings` → single resolved path | single | 0 | F1=key-known |
| IT-4 | `format::json` → valid JSON object with 5 keys | show-all | 0 | F2=json |
| IT-5 | `v::0` → plain unlabeled paths | show-all | 0 | F3=0 |
| IT-6 | `v::0` with `project_settings` unresolved → key omitted from output | show-all | 0 | F3=0, F4=not-found |
| IT-7 | `v::1` with `project_settings` unresolved → "(none found)" placeholder shown | show-all | 0 | F3=1, F4=not-found |
| IT-8 | `v::2` → labeled output with one-line description | show-all | 0 | F3=2 |

### Negative Tests

| TC | Description | Mode | Exit | Factors |
|----|-------------|------|------|---------|
| IT-9 | HOME unset → exit 2 | — | 2 | F5=unset |
| IT-10 | `key::bogus` → exit 1 | — | 1 | F1=key-unknown |
| IT-11 | `key::` (empty) → exit 1 | — | 1 | F1=key-unknown |

### Summary

- **Total:** 11 tests (8 positive, 3 negative)
- **Negative ratio:** 27.3%
- **TC range:** IT-1 to IT-11

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-1 through IT-8 |
| 1 | Invalid `key::` value | IT-10, IT-11 |
| 2 | HOME unset | IT-9 |

### Mode Coverage

| Mode | Tests |
|------|-------|
| show-all | IT-1, IT-4, IT-5, IT-6, IT-7, IT-8 |
| single | IT-2, IT-3 |

---

## Test Case Details

---

### IT-1: No key:: → show-all with all 5 keys

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths`
- **Then:** exit 0; stdout contains labeled lines for all 5 keys
- **Exit:** 0
- **Source:** [command/paths.md](../../../../docs/cli/command/paths.md)

---

### IT-2: key::versions_dir → single path

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::versions_dir`
- **Then:** exit 0; stdout is exactly the resolved versions directory path
- **Exit:** 0

---

### IT-3: key::settings → single path

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::settings`
- **Then:** exit 0; stdout is exactly `<tmp>/.claude/settings.json`
- **Exit:** 0

---

### IT-4: format::json → valid JSON object

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths format::json`
- **Then:** exit 0; stdout is valid JSON parseable as an object; object has all 5 keys
- **Exit:** 0

---

### IT-5: v::0 → plain unlabeled paths

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths v::0`
- **Then:** exit 0; stdout lines contain no label prefixes
- **Exit:** 0

---

### IT-6: v::0 with project_settings unresolved → key omitted

- **Given:** `HOME=<tmp>`; no ancestor `.claude/settings.json`
- **When:** `clv.paths v::0`
- **Then:** exit 0; stdout contains no line corresponding to `project_settings`
- **Exit:** 0

---

### IT-7: v::1 with project_settings unresolved → placeholder shown

- **Given:** `HOME=<tmp>`; no ancestor `.claude/settings.json`
- **When:** `clv.paths`
- **Then:** exit 0; stdout contains `project_settings:` followed by `(none found)`
- **Exit:** 0

---

### IT-8: v::2 → labeled with description

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::binary_symlink v::2`
- **Then:** exit 0; stdout contains the path and a one-line description
- **Exit:** 0

---

### IT-9: HOME unset → exit 2

- **Given:** `HOME` environment variable unset
- **When:** `clv.paths`
- **Then:** exit 2; no path output
- **Exit:** 2

---

### IT-10: key::bogus → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::bogus`
- **Then:** exit 1; stderr names the invalid key
- **Exit:** 1

---

### IT-11: key:: (empty) → exit 1

- **Given:** `HOME=<tmp>`
- **When:** `clv.paths key::`
- **Then:** exit 1; error references `key::` or empty value
- **Exit:** 1

---

## Source Functions Table

| Function | File | Test Cases |
|----------|------|------------|
| `it01_paths_show_all_keys` | `tests/cli/paths_test.rs` | IT-1 |
| `it02_paths_single_versions_dir` | `tests/cli/paths_test.rs` | IT-2 |
| `it03_paths_single_settings` | `tests/cli/paths_test.rs` | IT-3 |
| `it04_paths_json_object_structure` | `tests/cli/paths_test.rs` | IT-4 |
| `it05_paths_v0_unlabeled` | `tests/cli/paths_test.rs` | IT-5 |
| `it06_paths_v0_unresolved_omitted` | `tests/cli/paths_test.rs` | IT-6 |
| `it07_paths_v1_unresolved_placeholder` | `tests/cli/paths_test.rs` | IT-7 |
| `it08_paths_v2_description` | `tests/cli/paths_test.rs` | IT-8 |
| `it09_paths_home_unset_exits_2` | `tests/cli/paths_test.rs` | IT-9 |
| `it10_paths_invalid_key_exits_1` | `tests/cli/paths_test.rs` | IT-10 |
| `it11_paths_empty_key_exits_1` | `tests/cli/paths_test.rs` | IT-11 |
