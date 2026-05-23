# Test: `SettingsKey`

Type validation cases for the `SettingsKey` type. See [04_settings_key.md](../../../../docs/cli/type/04_settings_key.md) for specification.

### Scope

- **Purpose**: Type validation tests for `SettingsKey` (non-empty UTF-8 string).
- **Responsibility**: Acceptance of valid keys, rejection of empty/absent values, dot-literal behavior.
- **Used by:** `key::` parameter
- **In Scope**: Required presence, non-empty constraint, dot character semantics, UTF-8 acceptance.
- **Out of Scope**: Settings read/write behavior (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `key::theme` → accepted short alphanumeric key | Valid: minimum viable |
| TC-2 | `key::api.endpoint` → accepted, dot is literal | Valid: dot as literal |
| TC-3 | `key::autoUpdate` → accepted camelCase key | Valid: camelCase |
| TC-4 | `key::` (empty value) → exit 1 | Invalid: empty |
| TC-5 | absent `key::` → exit 1, required | Invalid: missing required |
| TC-6 | `key::` with Unicode value → accepted | Valid: UTF-8 |

## Test Coverage Summary

- Valid non-empty keys: 3 tests (TC-1, TC-2, TC-3)
- Valid Unicode: 1 test (TC-6)
- Invalid empty: 1 test (TC-4)
- Invalid absent: 1 test (TC-5)

**Total:** 6 type cases

**Behavioral Divergence Pair:** TC-1 (`key::theme` → exit 0, key found or key-not-found path) ↔ TC-4 (`key::` empty → exit 1, validation fails before lookup). Valid non-empty key reaches lookup; empty key fails validation immediately.

---

### TC-1: `key::theme` → accepted

- **Given:** clean environment with any settings state
- **When:** `cm .settings.get key::theme`
- **Then:** exit 0 or exit 2 (key not found); `key::` itself is valid; no validation error for key format
- **Exit:** 0 or 2
- **Source:** [04_settings_key.md — Constraints: non-empty, any UTF-8 string](../../../../docs/cli/type/04_settings_key.md)

---

### TC-2: `key::api.endpoint` → accepted, dot is literal

- **Given:** clean environment
- **When:** `cm .settings.get key::api.endpoint`
- **Then:** exit 0 or 2; dot character treated as literal key character, not path separator
- **Exit:** 0 or 2
- **Source:** [04_settings_key.md — dot characters are literal](../../../../docs/cli/type/04_settings_key.md)

---

### TC-3: `key::autoUpdate` → accepted camelCase

- **Given:** clean environment
- **When:** `cm .settings.get key::autoUpdate`
- **Then:** exit 0 or 2; camelCase key accepted without transformation
- **Exit:** 0 or 2
- **Source:** [04_settings_key.md — any UTF-8 string](../../../../docs/cli/type/04_settings_key.md)

---

### TC-4: `key::` (empty value) → exit 1

- **Given:** clean environment
- **When:** `cm .settings.get key::`
- **Then:** exit 1; error message contains "key:: value cannot be empty" or similar
- **Exit:** 1
- **Source:** [04_settings_key.md — validation: "key:: value cannot be empty"](../../../../docs/cli/type/04_settings_key.md)

---

### TC-5: absent `key::` → exit 1

- **Given:** clean environment
- **When:** `cm .settings.get`
- **Then:** exit 1; error message contains "key:: is required" or similar
- **Exit:** 1
- **Source:** [04_settings_key.md — validation: "key:: is required" if missing](../../../../docs/cli/type/04_settings_key.md)

---

### TC-6: `key::` with non-ASCII UTF-8 character → accepted

- **Given:** clean environment
- **When:** `cm .settings.get key::config_ñ`
- **Then:** exit 0 or 2; UTF-8 non-ASCII characters accepted in key name
- **Exit:** 0 or 2
- **Source:** [04_settings_key.md — any UTF-8 string](../../../../docs/cli/type/04_settings_key.md)

---

### Source Functions

| Function | File |
|----------|------|
| ⏳ `tc_settings_key_empty_exits_1` | `cli_args_test.rs` |
| ⏳ `tc_settings_key_absent_exits_1` | `cli_args_test.rs` |
| ⏳ `tc_settings_key_dot_literal` | `cli_args_test.rs` |
| ⏳ `tc_settings_key_valid_accepted` | `cli_args_test.rs` |
