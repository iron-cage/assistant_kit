# Test: `name::`

Edge case coverage for the `name::` parameter. See [param/16_name.md](../../../../docs/cli/param/16_name.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `name::` parameter.
- **Responsibility**: Validation constraints, required-field absence, pattern violations, reserved-name collisions, and length limits for `name::`.
- **Commands:** `.version.mark`
- **In Scope**: Required-field validation, pattern `[a-z][a-z0-9-]*`, max-32-char limit, reserved-name guard.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `name::team-pin` (valid) → success | Valid: standard name |
| EC-2 | Absent `name::` → exit 1 (required) | Invalid: absent |
| EC-3 | `name::` (empty) → exit 1 | Invalid: empty |
| EC-4 | `name::Team-Pin` (uppercase) → exit 1 | Invalid: pattern — uppercase |
| EC-5 | `name::1bad` (starts with digit) → exit 1 | Invalid: pattern — leading digit |
| EC-6 | `name::` with 33-char value → exit 1 | Invalid: too long |
| EC-7 | `name::stable` (reserved) → exit 1 | Invalid: reserved alias |
| EC-8 | `name::latest` (reserved) → exit 1 | Invalid: reserved alias |

## Test Coverage Summary

- Valid name: 1 test (EC-1)
- Invalid absent: 1 test (EC-2)
- Invalid empty: 1 test (EC-3)
- Invalid pattern — uppercase: 1 test (EC-4)
- Invalid pattern — leading digit: 1 test (EC-5)
- Invalid too long: 1 test (EC-6)
- Invalid reserved alias: 2 tests (EC-7, EC-8)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (`name::team-pin` → success, exit 0) ↔ EC-7 (`name::stable` → exit 1, reserved alias collision)

---

### EC-1: `name::team-pin` (valid) → success

- **Given:** clean environment with `dry::1` to prevent actual marker write
- **When:** `clv .version.mark name::team-pin version::stable dry::1`
- **Then:** exit 0; no error output; name accepted
- **Exit:** 0
- **Source:** [param/16_name.md](../../../../docs/cli/param/16_name.md)

---

### EC-2: Absent `name::` → exit 1 (required)

- **Given:** clean environment
- **When:** `clv .version.mark version::stable`
- **Then:** exit 1; error message references `name::` as a required parameter
- **Exit:** 1
- **Source:** [param/16_name.md](../../../../docs/cli/param/16_name.md)

---

### EC-3: `name::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name:: version::stable`
- **Then:** exit 1; error message references `name::` or empty value
- **Exit:** 1
- **Source:** [param/16_name.md](../../../../docs/cli/param/16_name.md)

---

### EC-4: `name::Team-Pin` (uppercase) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::Team-Pin version::stable`
- **Then:** exit 1; error indicates `name::` value violates pattern `[a-z][a-z0-9-]*` (uppercase letters rejected)
- **Exit:** 1
- **Source:** [param/16_name.md](../../../../docs/cli/param/16_name.md)

---

### EC-5: `name::1bad` (starts with digit) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::1bad version::stable`
- **Then:** exit 1; error indicates `name::` value must start with a lowercase letter
- **Exit:** 1
- **Source:** [param/16_name.md](../../../../docs/cli/param/16_name.md)

---

### EC-6: 33-character `name::` value → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa version::stable` (33 chars)
- **Then:** exit 1; error indicates `name::` value exceeds maximum 32-character length
- **Exit:** 1
- **Source:** [param/16_name.md](../../../../docs/cli/param/16_name.md)

---

### EC-7: `name::stable` (reserved alias) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::stable version::2.1.220`
- **Then:** exit 1; error indicates `stable` is a reserved built-in alias and cannot be used as a custom marker name
- **Exit:** 1
- **Source:** [param/16_name.md](../../../../docs/cli/param/16_name.md)

---

### EC-8: `name::latest` (reserved alias) → exit 1

- **Given:** clean environment
- **When:** `clv .version.mark name::latest version::2.1.220`
- **Then:** exit 1; error indicates `latest` is a reserved built-in alias and cannot be used as a custom marker name
- **Exit:** 1
- **Source:** [param/16_name.md](../../../../docs/cli/param/16_name.md)

---

### Source Functions

| Function | File |
|----------|------|
| `ft010_1_create_marker_appears_in_list` | `tests/cli/mutation_version_mark_test.rs` |
| `it11_mark_name_absent_exits_1` | `tests/cli/mutation_version_mark_test.rs` |
| `it12_mark_name_uppercase_exits_1` | `tests/cli/mutation_version_mark_test.rs` |
| `it13_mark_name_digit_start_exits_1` | `tests/cli/mutation_version_mark_test.rs` |
| `it14_mark_name_shadows_stable_exits_1` | `tests/cli/mutation_version_mark_test.rs` |
| `it15_mark_name_shadows_latest_exits_1` | `tests/cli/mutation_version_mark_test.rs` |
| `it19_mark_name_empty_exits_1` | `tests/cli/mutation_version_mark_test.rs` |
| `it20_mark_name_too_long_exits_1` | `tests/cli/mutation_version_mark_test.rs` |
