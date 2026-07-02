# Test: `SettingsKey`

Type compliance and validation tests for `SettingsKey`. See [type/04_settings_key.md](../../../../docs/cli/type/04_settings_key.md) for specification.

### Scope

- **Purpose**: Validate SettingsKey parsing, dot-as-literal semantics, and required-field enforcement.
- **Responsibility**: Non-empty string acceptance, dot character behavior, missing and empty key rejection.
- **Commands:** `.settings.get`, `.settings.set`
- **In Scope**: Key string parsing, dot-literal behavior, required-field validation.
- **Out of Scope**: Settings file I/O behavior (→ `../command/`), value type inference (→ `05_settings_value.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `key::theme` → simple key accepted | Valid: simple |
| TC-2 | `key::api.endpoint` → dot is literal, not path separator | Valid: dot-literal |
| TC-3 | `key::x` → single-char key accepted | Valid: boundary |
| TC-4 | Missing `key::` entirely → exit 1 | Validation: required |
| TC-5 | `key::` (empty) → exit 1 | Validation: empty |

## Test Coverage Summary

- Valid simple key: 1 test (TC-1)
- Dot-literal behavior: 1 test (TC-2)
- Minimal valid key: 1 test (TC-3)
- Required field: 1 test (TC-4)
- Empty value: 1 test (TC-5)

**Total:** 5 tests

**Behavioral Divergence Pair:** TC-1 (`clv .settings.get key::theme` on settings file → retrieves "theme" entry) ↔ TC-2 (`clv .settings.get key::api.endpoint` on settings file containing "api.endpoint" key → retrieves dot-named entry, NOT nested object path)

---

### TC-1: `key::theme` → simple key accepted

- **Given:** settings file at `~/.claude/settings.json` contains `{"theme": "dark"}`
- **When:** `clv .settings.get key::theme`
- **Then:** exit 0; output contains "dark"
- **Exit:** 0
- **Source:** [type/04_settings_key.md — non-empty UTF-8 string](../../../../docs/cli/type/04_settings_key.md)

---

### TC-2: `key::api.endpoint` → dot is literal

- **Given:** settings file contains `{"api.endpoint": "v1"}` (dot-named key, not nested)
- **When:** `clv .settings.get key::api.endpoint`
- **Then:** exit 0; output contains "v1"; dot treated as part of key name, not path separator
- **Exit:** 0
- **Source:** [type/04_settings_key.md — dot characters are literal](../../../../docs/cli/type/04_settings_key.md)

---

### TC-3: `key::x` → single-char key accepted

- **Given:** settings file contains `{"x": "1"}`
- **When:** `clv .settings.get key::x`
- **Then:** exit 0; output contains "1"
- **Exit:** 0
- **Source:** [type/04_settings_key.md — any non-empty UTF-8 string](../../../../docs/cli/type/04_settings_key.md)

---

### TC-4: Missing `key::` entirely → exit 1

- **Given:** clean environment
- **When:** `clv .settings.get` (no `key::` parameter)
- **Then:** exit code 1; error message says "key:: is required" or equivalent
- **Exit:** 1
- **Source:** [type/04_settings_key.md — Validation: key:: is required](../../../../docs/cli/type/04_settings_key.md)

---

### TC-5: `key::` (empty) → exit 1

- **Given:** clean environment
- **When:** `clv .settings.get key::`
- **Then:** exit code 1; error message says "key:: value cannot be empty" or equivalent
- **Exit:** 1
- **Source:** [type/04_settings_key.md — Validation: key:: value cannot be empty](../../../../docs/cli/type/04_settings_key.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc_settings_key_empty_exits_1` | `cli_args_test/type_surface_test.rs` |
| `tc_settings_key_absent_exits_1` | `cli_args_test/type_surface_test.rs` |
| `tc_settings_key_dot_literal` | `cli_args_test/type_surface_test.rs` |
| `tc_settings_key_valid_accepted` | `cli_args_test/type_surface_test.rs` |
