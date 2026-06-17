# Test: `key::`

Edge case coverage for the `key::` parameter. See [param/readme.md](../../../../docs/cli/param/readme.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `key::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `key::`.
- **Commands:** `.settings.get`, `.settings.set`
- **In Scope**: Single-parameter edge cases, validation errors, type checking.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `key::existing` on `.settings.get` → returns value | Valid: existing |
| EC-2 | `key::nonexistent` → exit 2, key not found | Valid: missing |
| EC-3 | No `key::` on `.settings.get` → exit 1 | Absent (required) |
| EC-10 | Without `key::` → error message mentions `key::` | Error Content |
| EC-11 | Without `key::` on `.settings.set` → error mentions `key::` | Absent (required) |
| EC-4 | `key::""` (empty key) on `.settings.set` → exit 1 | Empty Value |
| EC-5 | `key::` (empty value) on `.settings.get` → exit 1 | Empty Value |
| EC-6 | `key::` only accepted by `.settings.get` and `.settings.set` | Command Scope |
| EC-7 | `key::a b c` (key with spaces) → behavior defined | Special Characters |
| EC-8 | `key::foo.bar` (dot in key name) → stored as given | Special Characters |
| EC-9 | `key::foo bar` (space in key) → stored as given | Special Characters |

## Test Coverage Summary

- Valid (existing key): 1 test
- Valid (missing key → exit 2): 1 test
- Absent (required → exit 1): 2 tests
- Error Content: 1 test
- Empty Value: 2 tests
- Command Scope: 1 test
- Special Characters: 3 tests

**Total:** 12 edge cases

**Behavioral Divergence Pair:** EC-1 (`key::existing` → returns value, exit 0) ↔ EC-2 (`key::nonexistent` → exit 2 key not found)

---

### EC-1: `key::existing` → value returned

- **Given:** `HOME=<tmp>`; settings has `myKey = "myValue"`.
- **When:** `clv .settings.get key::myKey`
- **Then:** exit 0; output contains "myValue".; correct value returned
- **Exit:** 0
- **Source:** [command/readme.md — .settings.get](../../../../docs/cli/command/readme.md)

---

### EC-2: `key::nonexistent` → exit 2

- **Given:** `HOME=<tmp>`; settings has different key.
- **When:** `clv .settings.get key::nosuchkey`
- **Then:** exit code 2.
- **Exit:** 2
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-3: No `key::` → exit 1

- **Given:** Valid settings file.
- **When:** `clv .settings.get`
- **Then:** exit code 1; error mentions key.
- **Exit:** 1
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-4: `key::""` → exit 1

- **Given:** clean environment
- **When:** `clv .settings.set key:: value::x`
- **Then:** exit code 1; error mentions empty key.
- **Exit:** 1
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-5: `key::` (empty value) on `.settings.get` → exit 1

- **Given:** clean environment
- **When:** `clv .settings.get key::`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-6: `key::` only for `.settings.get` and `.settings.set`

- **Given:** clean environment
- **When:** `clv .status key::foo`
- **Then:** exit code 1; unknown parameter.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-7: `key::a b c` (key with spaces) → behavior defined

- **Given:** `HOME=<tmp>`; no existing settings
- **When:** `clv .settings.set "key::a b c" value::x`
- **Then:** behavior is defined by spec — either accepted as opaque key string or rejected; exit code is consistent with spec
- **Exit:** 0
- **Source:** [param/06_key.md](../../../../docs/cli/param/06_key.md)

---

### EC-8: `key::foo.bar` (dot in key name)

- **Given:** `HOME=<tmp>`; no existing settings.
- **When:** `clv .settings.set key::foo.bar value::baz && cm .settings.get key::foo.bar`
- **Then:** `baz` returned for key `foo.bar`.; key round-trips correctly.
**Note:** Tests that the key is treated as an opaque string, not a nested path
- **Exit:** 0

---

### EC-9: `key::foo bar` (space in key) → stored as given

- **Given:** `HOME=<tmp>`; no existing settings
- **When:** `clv .settings.set "key::foo bar" value::baz`
- **Then:** exit 0 (or per-spec); key `foo bar` treated as opaque string; round-trip get returns same value
- **Exit:** 0
- **Source:** [param/06_key.md](../../../../docs/cli/param/06_key.md)

---

### EC-10: Without `key::` → error message mentions `key::`

- **Given:** `HOME=<tmp>` with valid settings.json
- **When:** `clv .settings.get`
- **Then:** exit 1; error message contains the string `key::`
- **Exit:** 1
- **Source:** [param/06_key.md](../../../../docs/cli/param/06_key.md)

---

### EC-11: Without `key::` on `.settings.set` → error mentions `key::`

- **Given:** `HOME=<tmp>` with valid settings.json
- **When:** `clv .settings.set value::dark`
- **Then:** exit 1; error message contains the string `key::`
- **Exit:** 1
- **Source:** [param/06_key.md](../../../../docs/cli/param/06_key.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc320_settings_set_missing_key_exits_1` | `integration/mutation_commands_test.rs` |
| `tc332_settings_set_empty_key_exits_1` | `integration/mutation_commands_test.rs` |
| `tc238_settings_set_missing_key_error_format` | `integration/read_commands_test.rs` |
| `tc505_settings_get_missing_key_error_contains_key` | `integration/error_messages_test.rs` |
| `key_ec1_existing_key_returns_value` | `integration/key_param_test.rs` |
| `key_ec2_nonexistent_key_exits_2` | `integration/key_param_test.rs` |
| `key_ec5_empty_key_on_get_exits_1` | `integration/key_param_test.rs` |
| `key_ec6_command_scope_rejects_on_status` | `integration/key_param_test.rs` |
| `key_ec7_key_with_spaces_behavior` | `integration/key_param_test.rs` |
| `key_ec8_dot_in_key_round_trips` | `integration/key_param_test.rs` |
| `key_ec9_space_in_key_round_trips` | `integration/key_param_test.rs` |
| `key_ec10_missing_key_error_contains_key_token` | `integration/key_param_test.rs` |
| `key_ec11_missing_key_on_set_error_contains_key_token` | `integration/key_param_test.rs` |
