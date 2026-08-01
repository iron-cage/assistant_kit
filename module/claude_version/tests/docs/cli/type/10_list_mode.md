# Test: `ListMode`

Type compliance and validation tests for `ListMode`. See [type/10_list_mode.md](../../../../docs/cli/type/10_list_mode.md) for specification.

### Scope

- **Purpose**: Validate ListMode parsing, case-sensitivity enforcement, and per-variant listing behavior.
- **Responsibility**: Valid variants, invalid inputs, default behavior, and observable output differences between variant values.
- **Commands:** `.version.list`
- **In Scope**: Variant string parsing, case-sensitive matching, and observable output differences per variant.
- **Out of Scope**: Cross-parameter interaction with `count::` (→ `../param/14_mode.md`), JSON field structure (→ `../command/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `mode::aliases` → alias table (local, deterministic) | Valid: aliases |
| TC-2 | `mode::history` → release history (network-dependent) | Valid: history |
| TC-3 | Absent `mode::` → defaults to aliases | Default |
| TC-4 | `mode::ALIASES` → exit 1 (case-sensitive) | Validation: case |
| TC-5 | `mode::bogus` → exit 1 (unknown variant) | Validation: unknown |
| TC-6 | `mode::` (empty) → exit 1 | Validation: empty |

## Test Coverage Summary

- Valid variant resolution: 2 tests (TC-1, TC-2)
- Default Behavior: 1 test (TC-3)
- Case sensitivity: 1 test (TC-4)
- Unknown variant: 1 test (TC-5)
- Empty value: 1 test (TC-6)

**Total:** 6 tests

**Behavioral Divergence Pair:** TC-1 (`mode::aliases` → local compile-time table, always exit 0) ↔ TC-2 (`mode::history` → network call, always exit 0 — falls back to a compiled-in snapshot with a stderr advisory on network failure)

---

### TC-1: `mode::aliases` → alias table

- **Given:** clean environment
- **When:** `clv .version.list mode::aliases`
- **Then:** exit 0; stdout contains the compile-time alias table (`stable`, `latest`, `month`); behavior is fully deterministic (no network access)
- **Exit:** 0
- **Source:** [type/10_list_mode.md — aliases variant](../../../../docs/cli/type/10_list_mode.md)

---

### TC-2: `mode::history` → release history

- **Given:** Network available.
- **When:** `clv .version.list mode::history`
- **Then:** exit 0; stdout contains release-history entries fetched from the GitHub Releases API, not the alias table
- **Exit:** 0
- **Source:** [type/10_list_mode.md — history variant](../../../../docs/cli/type/10_list_mode.md)

---

### TC-3: Absent `mode::` → defaults to aliases

- **Given:** clean environment
- **When:** `clv .version.list` (no `mode::` parameter)
- **Then:** exit 0; output identical to `mode::aliases` explicit invocation
- **Exit:** 0
- **Source:** [type/10_list_mode.md — Default: aliases](../../../../docs/cli/type/10_list_mode.md)

---

### TC-4: `mode::ALIASES` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list mode::ALIASES`
- **Then:** exit 1; stderr references case-sensitivity or an unknown mode value; `Aliases`/`HISTORY`/`History` are equally rejected
- **Exit:** 1
- **Source:** [type/10_list_mode.md — Case-sensitive matching](../../../../docs/cli/type/10_list_mode.md)

---

### TC-5: `mode::bogus` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list mode::bogus`
- **Then:** exit 1; stderr contains "unknown mode" listing the 2 valid values (`aliases`, `history`)
- **Exit:** 1
- **Source:** [type/10_list_mode.md — Validation errors](../../../../docs/cli/type/10_list_mode.md)

---

### TC-6: `mode::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .version.list mode::`
- **Then:** exit 1; error message references `mode::` or empty value
- **Exit:** 1
- **Source:** [type/10_list_mode.md — Validation errors](../../../../docs/cli/type/10_list_mode.md)

---

### Source Functions

| Function | File |
|----------|------|
| `list_mode_tc1_aliases_shows_table` | `tests/cli/list_mode_test.rs` |
| `list_mode_tc2_history_shows_entries` | `tests/cli/list_mode_test.rs` |
| `list_mode_tc3_absent_defaults_to_aliases` | `tests/cli/list_mode_test.rs` |
| `list_mode_tc4_uppercase_exits_1` | `tests/cli/list_mode_test.rs` |
| `list_mode_tc5_unknown_exits_1` | `tests/cli/list_mode_test.rs` |
| `list_mode_tc6_empty_exits_1` | `tests/cli/list_mode_test.rs` |
