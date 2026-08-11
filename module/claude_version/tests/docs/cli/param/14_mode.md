# Test: `mode::`

Edge case coverage for the `mode::` parameter. See [param/14_mode.md](../../../../docs/cli/param/14_mode.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `mode::` parameter.
- **Responsibility**: Boundary values, invalid inputs, case-sensitivity, and default behavior for `mode::`.
- **Commands:** `.version.list`
- **In Scope**: Single-parameter edge cases, validation errors, case-sensitivity, `count::` interaction.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `mode::aliases` → alias listing shown | Valid: aliases |
| EC-2 | `mode::history` → release history shown | Valid: history |
| EC-3 | Absent `mode::` → defaults to aliases listing | Default Behavior |
| EC-4 | `mode::invalid` → exit 1, message about valid values | Invalid: unknown |
| EC-5 | `mode::` (empty) → exit 1 | Invalid: empty |
| EC-6 | `mode::ALIASES` (uppercase) → exit 1 | Invalid: case |
| EC-7 | `count::` accepted but inert under `mode::aliases` | Cross-Param Interaction |

## Test Coverage Summary

- Valid mode: 2 tests (EC-1, EC-2)
- Default Behavior: 1 test (EC-3)
- Invalid unknown: 1 test (EC-4)
- Invalid empty: 1 test (EC-5)
- Invalid case: 1 test (EC-6)
- Cross-Param Interaction: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (`mode::aliases` → alias listing, exit 0) ↔ EC-4 (`mode::invalid` → exit 1 with valid values message)

---

### EC-1: `mode::aliases` → alias listing shown

- **Given:** clean environment
- **When:** `clv .version.list mode::aliases`
- **Then:** exit 0; stdout contains the compile-time alias table (`stable`, `latest`)
- **Exit:** 0
- **Source:** [param/14_mode.md](../../../../docs/cli/param/14_mode.md)

---

### EC-2: `mode::history` → release history shown

- **Given:** Network available.
- **When:** `clv .version.list mode::history`
- **Then:** exit 0; stdout contains release-history entries, not the alias table (falls back to a compiled-in snapshot with a stderr advisory if network is unavailable)
- **Exit:** 0
- **Source:** [param/14_mode.md](../../../../docs/cli/param/14_mode.md)

---

### EC-3: Absent `mode::` → defaults to aliases

- **Given:** clean environment
- **When:** `clv .version.list` (no `mode::` parameter)
- **Then:** exit 0; output identical to `mode::aliases` explicit invocation
- **Exit:** 0
- **Source:** [param/14_mode.md](../../../../docs/cli/param/14_mode.md)

---

### EC-4: `mode::invalid` → exit 1

- **Given:** clean environment
- **When:** `clv .version.list mode::invalid`
- **Then:** exit 1; stderr contains a message indicating valid values are `aliases` or `history`
- **Exit:** 1
- **Source:** [param/14_mode.md](../../../../docs/cli/param/14_mode.md)

---

### EC-5: `mode::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .version.list mode::`
- **Then:** exit 1; error message references `mode::` or empty value
- **Exit:** 1
- **Source:** [param/14_mode.md](../../../../docs/cli/param/14_mode.md)

---

### EC-6: `mode::ALIASES` (uppercase) → exit 1

- **Given:** clean environment
- **When:** `clv .version.list mode::ALIASES`
- **Then:** exit 1; `mode::` is case-sensitive; `ALIASES` is not a valid variant
- **Exit:** 1
- **Source:** [type/10_list_mode.md](../../../../docs/cli/type/10_list_mode.md)

---

### EC-7: `count::` accepted but inert under `mode::aliases`

- **Given:** clean environment
- **When:** `clv .version.list mode::aliases count::5`
- **Then:** exit 0; output identical to `mode::aliases` without `count::` — `count::` is not rejected as unknown, but has no truncating effect
- **Exit:** 0
- **Source:** [param_group/01_output_control.md](../../../../docs/cli/param_group/01_output_control.md)

---

### Source Functions

| Function | File |
|----------|------|
| `mode_ec1_aliases_shows_alias_table` | `tests/cli/mode_param_test.rs` |
| `mode_ec2_history_shows_release_history` | `tests/cli/mode_param_test.rs` |
| `mode_ec3_absent_defaults_to_aliases` | `tests/cli/mode_param_test.rs` |
| `mode_ec4_invalid_exits_1` | `tests/cli/mode_param_test.rs` |
| `mode_ec5_empty_exits_1` | `tests/cli/mode_param_test.rs` |
| `mode_ec6_uppercase_exits_1` | `tests/cli/mode_param_test.rs` |
| `mode_ec7_count_inert_under_aliases` | `tests/cli/mode_param_test.rs` |
