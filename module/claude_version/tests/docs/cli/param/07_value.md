# Test: `value::`

Edge case coverage for the `value::` parameter. See [005_params.md](../../../../docs/cli/param/readme.md) and [algorithm/001_settings_type_inference.md](../../../../docs/algorithm/001_settings_type_inference.md) for specification.

### Scope

- **Purpose**: Edge case tests for the `value::` parameter.
- **Responsibility**: Boundary values, invalid inputs, type violations, and default behavior for `value::`.
- **Commands:** `.settings.set`
- **In Scope**: Single-parameter edge cases, validation errors, type inference integration.
- **Out of Scope**: Command integration (→ `../command/`), group interactions (→ `../param_group/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `value::true` → JSON boolean `true` | Type Inference |
| EC-11 | `value::false` → JSON boolean `false` | Type Inference |
| EC-2 | `value::0` → JSON integer `0` (not boolean false) | Type Inference |
| EC-12 | `value::42` → JSON integer `42` | Type Inference |
| EC-13 | `value::hello` → JSON string `"hello"` | Type Inference |
| EC-3 | `value::""` → JSON string `""` (empty string valid) | Type Inference |
| EC-14 | `key::k` present but no `value::` → exit 1 | Absent (required) |
| EC-15 | Without `value::` → error message mentions `value::` | Error Content |
| EC-5 | `value::1.5` → JSON float (parseable as f64 but not i64) | Type Inference |
| EC-6 | `value::NaN` → JSON string (not number, NaN is not finite) | Type Inference (edge) |
| EC-8 | `value::Infinity` → JSON string (not float, infinite) | Type Inference (edge) |
| EC-9 | `value::true false` (space in value) → JSON string | Type Inference |
| EC-10 | `value::` (empty) → exit 1 (empty value rejected) | Empty Value |
| EC-4 | `value::` only for `.settings.set` | Command Scope |
| EC-7 | Round-trip: set then get returns identical value | Persistence |

## Test Coverage Summary

- Type Inference (boolean): 2 tests
- Type Inference (integer): 2 tests
- Type Inference (string): 2 tests
- Type Inference (float): 1 test
- Type Inference (edge: NaN/Infinity): 2 tests
- Absent (required): 1 test
- Error Content: 1 test
- Empty Value: 1 test
- Command Scope: 1 test
- Persistence: 1 test

**Total:** 15 edge cases

**Behavioral Divergence Pair:** EC-1 (`value::true` → JSON boolean `true`, exit 0) ↔ EC-2 (`value::0` → JSON integer `0`, not boolean, exit 0)

---

### Type Inference Priority (FR-07)

The type inference chain processes in this order:
1. `"true"` / `"false"` → JSON boolean
2. Any string parseable as `i64` → JSON integer (includes `"0"`, `"1"`)
3. Parseable as finite `f64` but not `i64` → JSON float
4. All other strings (including NaN/inf variants) → JSON string

**Critical distinction:** `"0"` and `"1"` parse as integers (step 2), NOT as booleans
(step 1 only matches exact "true"/"false"). This is intentional for settings values.

---

### EC-1: `value::true` → boolean `true`

- **Given:** `HOME=<tmp>`.
- **When:** `clv .settings.set key::flag value::true`
- **Then:** exit 0; `settings.json` has `"flag": true` (unquoted).; native boolean stored
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md](../../../../docs/algorithm/001_settings_type_inference.md)

---

### EC-2: `value::0` → integer `0`, NOT boolean

- **Given:** `HOME=<tmp>`.
- **When:** `clv .settings.set key::n value::0`
- **Then:** exit 0; `settings.json` has `"n": 0` (integer, not `false`).; integer stored
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md](../../../../docs/algorithm/001_settings_type_inference.md)

---

### EC-3: `value::""` → empty string

- **Given:** `HOME=<tmp>`.
- **When:** `clv .settings.set key::s value::`
- **Then:** exit 0 (or exit 1 if empty value is rejected — need to verify behavior).
**Note:** If `value::` with empty is treated as absent (missing value), exit 1. If accepted
as empty string, exit 0 with `"s": ""`. Check FR-04 vs FR-07 interaction.; Consistent with spec; no crash
- **Exit:** 0
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-5: `value::1.5` → JSON float

- **Given:** `HOME=<tmp>`.
- **When:** `clv .settings.set key::f value::1.5`
- **Then:** exit 0; `settings.json` has `"f": 1.5` (unquoted float).; float stored
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md](../../../../docs/algorithm/001_settings_type_inference.md)

---

### EC-6: `value::NaN` → JSON string

- **Given:** `HOME=<tmp>`.
- **When:** `clv .settings.set key::x value::NaN`
- **Then:** exit 0; `settings.json` has `"x": "NaN"` (quoted string).; string stored
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md](../../../../docs/algorithm/001_settings_type_inference.md)

---

### EC-4: `value::` only for `.settings.set`

- **Given:** clean environment
- **When:** `clv .settings.get key::k value::v`
- **Then:** exit code 1; unknown parameter.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-7: Round-trip: set then get returns identical value

- **Given:** `HOME=<tmp>` with no settings.json
- **When:** `clv .settings.set key::roundtrip value::hello` followed by `clv .settings.get key::roundtrip`
- **Then:** `.settings.get` output contains `hello`; the value stored and retrieved is identical
- **Exit:** 0
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc321_settings_set_missing_value_exits_1` | `integration/mutation_commands_test.rs` |
| `tc327_settings_set_empty_value_rejected` | `integration/mutation_commands_test.rs` |
| `tc239_settings_set_missing_value_error_format` | `integration/read_commands_test.rs` |
| `tc506_settings_set_missing_value_error_contains_value` | `integration/error_messages_test.rs` |
