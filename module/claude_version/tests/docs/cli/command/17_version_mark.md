# Test: `.version.mark`

### Scope

- **Purpose**: Integration test cases for the `.version.mark` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for marker creation, update, removal, validation, dry-run, and format modes.
- **In Scope**: Set path, unset path, name validation, version validation, dry-run, JSON format, `.version.list` integration.
- **Out of Scope**: `CustomMarker` unit-level tests (→ `claude_version_core` crate), parameter edge cases (→ `../param/`).

Integration test planning for `.version.mark`. See [command/version.md](../../../../docs/cli/command/version.md#command-17-versionmark) for specification.

## Test Factor Analysis

### Factor 1: Path (derived from unset:: presence)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| no `unset::` | set path: create or update marker | Default |
| `unset::1` | unset path: remove marker | Alternate |

### Factor 2: `name::` validity

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| valid name (e.g. `team-pin`) | matches `[a-z][a-z0-9-]*`, ≤32 chars, no shadow | Happy path |
| absent | `name::` not provided | Error: exit 1 |
| uppercase start | `MyPin` — fails regex | Error: exit 1 |
| starts with digit | `1pin` — fails regex | Error: exit 1 |
| too long (>32 chars) | exceeds max length | Error: exit 1 |
| shadows `stable` | collides with built-in | Error: exit 1 |
| shadows `latest` | collides with built-in | Error: exit 1 |

### Factor 3: `version::` validity (set path only)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| valid semver (e.g. `2.1.220`) | accepted | Happy path |
| built-in alias (`stable`) | accepted | Happy path |
| invalid / absent | rejected | Error: exit 1 |

### Factor 4: `dry::` flag

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | writes `version-markers.json` | Default |
| `dry::1` | preview only, no write | Dry Run |

### Factor 5: `format::` output

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent / `text` | Human-readable text | Default |
| `json` | Structured JSON object | Alternate valid |

---

## Test Matrix

### Positive Tests

| TC | Description | Path | Exit | Factors |
|----|-------------|------|------|---------|
| IT-1 | Create new marker → `version-markers.json` written | set | 0 | F1=set, F2=valid, F3=semver |
| IT-2 | Update existing marker (upsert) → value replaced | set | 0 | F1=set, F2=valid, F3=semver |
| IT-3 | Remove existing marker | unset | 0 | F1=unset, F2=valid |
| IT-4 | Remove absent marker → no-op, exit 0 | unset | 0 | F1=unset, F2=valid |
| IT-5 | `dry::1` set path → preview, no write | set | 0 | F1=set, F4=dry |
| IT-6 | `dry::1` unset path → preview, no write | unset | 0 | F1=unset, F4=dry |
| IT-7 | `version::stable` (built-in alias) accepted as marker value | set | 0 | F1=set, F3=alias |
| IT-8 | Created marker appears in `.version.list` output | set | 0 | F1=set, integration |
| IT-9 | Created marker accepted by `.version.install version::name dry::1` | set | 0 | F1=set, resolution |
| IT-10 | `format::json dry::1` → JSON output, exit 0 | set | 0 | F5=json, F4=dry |

### Negative Tests

| TC | Description | Path | Exit | Factors |
|----|-------------|------|------|---------|
| IT-11 | `name::` absent → exit 1 | set | 1 | F2=absent |
| IT-12 | `name::MyPin` (uppercase start) → exit 1 | set | 1 | F2=uppercase |
| IT-13 | `name::1pin` (digit start) → exit 1 | set | 1 | F2=digit-start |
| IT-14 | `name::stable` (shadows built-in) → exit 1 | set | 1 | F2=shadows-stable |
| IT-15 | `name::latest` (shadows built-in) → exit 1 | set | 1 | F2=shadows-latest |
| IT-16 | `version::` absent on set path → exit 1 | set | 1 | F3=absent |
| IT-17 | `version::x` (invalid spec) → exit 1 | set | 1 | F3=invalid |

### Resilience Tests

| TC | Description | Path | Exit | Factors |
|----|-------------|------|------|---------|
| IT-18 | Malformed `version-markers.json` → graceful degradation, exit 0 | — | 0 | resilience |

### Summary

- **Total:** 18 tests (11 positive, 7 negative)
- **Negative ratio:** 38.9%
- **TC range:** IT-1 to IT-18

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-1 through IT-10, IT-18 |
| 1 | Validation error | IT-11 through IT-17 |

### Path Coverage

| Path | Tests |
|------|-------|
| set | IT-1, IT-2, IT-5, IT-7, IT-8, IT-9, IT-10, IT-11 through IT-17 |
| unset | IT-3, IT-4, IT-6 |
| — (resilience, list path) | IT-18 |

---

## Test Case Details

---

### IT-1: Create new marker

- **Given:** isolated HOME; no `version-markers.json`
- **When:** `clv .version.mark name::team-pin version::2.1.220`
- **Then:** exit 0; `~/.claude/version-markers.json` exists; contains entry for `team-pin` with value `2.1.220`
- **Exit:** 0
- **Source:** [command/version.md](../../../../docs/cli/command/version.md#command-17-versionmark)

---

### IT-2: Update existing marker (upsert)

- **Given:** isolated HOME; `version-markers.json` with `team-pin → 2.1.200`
- **When:** `clv .version.mark name::team-pin version::2.1.220`
- **Then:** exit 0; `team-pin` entry value is now `2.1.220`, not `2.1.200`
- **Exit:** 0

---

### IT-3: Remove existing marker

- **Given:** isolated HOME; `version-markers.json` with `team-pin → 2.1.220`
- **When:** `clv .version.mark name::team-pin unset::1`
- **Then:** exit 0; `team-pin` entry absent from `version-markers.json`
- **Exit:** 0

---

### IT-4: Remove absent marker → no-op

- **Given:** isolated HOME; `version-markers.json` with no `team-pin` entry
- **When:** `clv .version.mark name::team-pin unset::1`
- **Then:** exit 0; no error; `version-markers.json` unchanged
- **Exit:** 0

---

### IT-5: `dry::1` set path → preview, no write

- **Given:** isolated HOME; no `version-markers.json`
- **When:** `clv .version.mark name::team-pin version::2.1.220 dry::1`
- **Then:** exit 0; `version-markers.json` does not exist after the call; stdout contains preview text
- **Exit:** 0

---

### IT-6: `dry::1` unset path → preview, no write

- **Given:** isolated HOME; `version-markers.json` with `team-pin → 2.1.220`
- **When:** `clv .version.mark name::team-pin unset::1 dry::1`
- **Then:** exit 0; `team-pin` still present in `version-markers.json`; stdout contains preview text
- **Exit:** 0

---

### IT-7: `version::stable` (built-in alias) accepted as marker value

- **Given:** isolated HOME
- **When:** `clv .version.mark name::team-pin version::stable`
- **Then:** exit 0; `version-markers.json` entry for `team-pin` has value `stable`
- **Exit:** 0

---

### IT-8: Created marker appears in `.version.list`

- **Given:** isolated HOME; `version-markers.json` with `team-pin → 2.1.220`
- **When:** `clv .version.list`
- **Then:** exit 0; stdout contains `team-pin`
- **Exit:** 0

---

### IT-9: Created marker accepted by `.version.install`

- **Given:** isolated HOME; `version-markers.json` with `team-pin → 2.1.220`
- **When:** `clv .version.install version::team-pin dry::1`
- **Then:** exit 0; stdout contains `2.1.220`
- **Exit:** 0

---

### IT-10: `format::json dry::1` → JSON output

- **Given:** isolated HOME
- **When:** `clv .version.mark name::team-pin version::2.1.220 format::json dry::1`
- **Then:** exit 0; stdout starts with `{`; parseable JSON object
- **Exit:** 0

---

### IT-11: `name::` absent → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark version::2.1.220`
- **Then:** exit 1; stderr references missing `name` parameter
- **Exit:** 1

---

### IT-12: `name::MyPin` (uppercase start) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::MyPin version::2.1.220`
- **Then:** exit 1; stderr references invalid marker name
- **Exit:** 1

---

### IT-13: `name::1pin` (digit start) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::1pin version::2.1.220`
- **Then:** exit 1; stderr references invalid marker name
- **Exit:** 1

---

### IT-14: `name::stable` (shadows built-in) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::stable version::2.1.220`
- **Then:** exit 1; stderr references name collision with built-in alias
- **Exit:** 1

---

### IT-15: `name::latest` (shadows built-in) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::latest version::2.1.220`
- **Then:** exit 1; stderr references name collision with built-in alias
- **Exit:** 1

---

### IT-16: `version::` absent on set path → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::team-pin`
- **Then:** exit 1; stderr references missing `version` parameter
- **Exit:** 1

---

### IT-17: `version::x` (invalid spec) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::team-pin version::x`
- **Then:** exit 1; stderr references invalid version spec
- **Exit:** 1

---

### IT-18: Malformed `version-markers.json` → graceful degradation

- **Given:** isolated HOME; `~/.claude/version-markers.json` contains `"not valid json {{{"` (invalid JSON)
- **When:** `clv .version.list`
- **Then:** exit 0; stdout contains `stable`; built-in aliases visible despite malformed markers file; no crash or error exit
- **Exit:** 0

---

## Source Functions Table

| Function | File | Test Cases |
|----------|------|------------|
| `it01_mark_create_new` | `tests/cli/mutation_version_mark_test.rs` | IT-1 |
| `it02_mark_update_existing` | `tests/cli/mutation_version_mark_test.rs` | IT-2 |
| `it03_mark_remove_existing` | `tests/cli/mutation_version_mark_test.rs` | IT-3 |
| `it04_mark_remove_absent_noop` | `tests/cli/mutation_version_mark_test.rs` | IT-4 |
| `it05_mark_dry_set_no_write` | `tests/cli/mutation_version_mark_test.rs` | IT-5 |
| `it06_mark_dry_unset_no_write` | `tests/cli/mutation_version_mark_test.rs` | IT-6 |
| `it07_mark_version_builtin_alias` | `tests/cli/mutation_version_mark_test.rs` | IT-7 |
| `it08_mark_appears_in_list` | `tests/cli/mutation_version_mark_test.rs` | IT-8 |
| `it09_mark_accepted_by_install` | `tests/cli/mutation_version_mark_test.rs` | IT-9 |
| `it10_mark_json_format_dry` | `tests/cli/mutation_version_mark_test.rs` | IT-10 |
| `it11_mark_name_absent_exits_1` | `tests/cli/mutation_version_mark_test.rs` | IT-11 |
| `it12_mark_name_uppercase_exits_1` | `tests/cli/mutation_version_mark_test.rs` | IT-12 |
| `it13_mark_name_digit_start_exits_1` | `tests/cli/mutation_version_mark_test.rs` | IT-13 |
| `it14_mark_name_shadows_stable_exits_1` | `tests/cli/mutation_version_mark_test.rs` | IT-14 |
| `it15_mark_name_shadows_latest_exits_1` | `tests/cli/mutation_version_mark_test.rs` | IT-15 |
| `it16_mark_version_absent_exits_1` | `tests/cli/mutation_version_mark_test.rs` | IT-16 |
| `it17_mark_version_invalid_exits_1` | `tests/cli/mutation_version_mark_test.rs` | IT-17 |
| `it18_mark_malformed_json_graceful` | `tests/cli/mutation_version_mark_test.rs` | IT-18 |
