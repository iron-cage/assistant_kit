# Test: `SettingsValue`

Type validation cases for the `SettingsValue` type. See [05_settings_value.md](../../../../docs/cli/type/05_settings_value.md) for specification.

### Scope

- **Purpose**: Type validation and inference tests for `SettingsValue` (type-inferred string).
- **Responsibility**: Type inference rules (bool/number/string), non-finite float handling, required presence.
- **Used by:** `value::` parameter
- **In Scope**: Inference from string input, non-empty constraint, required presence.
- **Out of Scope**: Settings persistence (→ `../command/`), key semantics (→ `../param/`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `value::true` → stored as JSON bool `true` | Valid: bool inference |
| TC-2 | `value::false` → stored as JSON bool `false` | Valid: bool inference |
| TC-3 | `value::42` → stored as JSON number `42` | Valid: integer inference |
| TC-4 | `value::3.14` → stored as JSON float `3.14` | Valid: float inference |
| TC-5 | `value::dark` → stored as JSON string `"dark"` | Valid: string fallback |
| TC-6 | `value::NaN` → stored as JSON string `"NaN"` | Valid: non-finite as string |
| TC-7 | `value::` (empty) → exit 1 | Invalid: empty |
| TC-8 | absent `value::` → exit 1, required | Invalid: missing required |

## Test Coverage Summary

- Bool inference (true, false): 2 tests (TC-1, TC-2)
- Integer inference: 1 test (TC-3)
- Float inference: 1 test (TC-4)
- String fallback: 1 test (TC-5)
- Non-finite float as string: 1 test (TC-6)
- Invalid empty: 1 test (TC-7)
- Invalid absent: 1 test (TC-8)

**Total:** 8 type cases

**Behavioral Divergence Pair:** TC-1 (`value::true` → JSON bool `true`) ↔ TC-5 (`value::dark` → JSON string `"dark"`). Both valid, non-empty string inputs; inferred JSON types differ — bool vs string.

---

### TC-1: `value::true` → JSON bool `true`

- **Given:** writable settings file at `~/.claude/settings.json`
- **When:** `cm .settings.set key::flag value::true`; then read back via `cm .settings.get key::flag format::json`
- **Then:** exit 0; stored value is JSON boolean `true`, not the string `"true"`
- **Exit:** 0
- **Source:** [05_settings_value.md — Type Inference: "true"/"false" → Bool](../../../../docs/cli/type/05_settings_value.md)

---

### TC-2: `value::false` → JSON bool `false`

- **Given:** writable settings file
- **When:** `cm .settings.set key::flag value::false`; read back
- **Then:** exit 0; stored value is JSON boolean `false`
- **Exit:** 0
- **Source:** [05_settings_value.md — Type Inference: "true"/"false" → Bool](../../../../docs/cli/type/05_settings_value.md)

---

### TC-3: `value::42` → JSON number `42`

- **Given:** writable settings file
- **When:** `cm .settings.set key::timeout value::42`; read back via JSON format
- **Then:** exit 0; stored value is JSON number `42`, not the string `"42"`
- **Exit:** 0
- **Source:** [05_settings_value.md — Type Inference: Integer string → Number (i64)](../../../../docs/cli/type/05_settings_value.md)

---

### TC-4: `value::3.14` → JSON float `3.14`

- **Given:** writable settings file
- **When:** `cm .settings.set key::rate value::3.14`; read back via JSON format
- **Then:** exit 0; stored value is JSON float `3.14`, not the string `"3.14"`
- **Exit:** 0
- **Source:** [05_settings_value.md — Type Inference: Finite float string → Number (f64)](../../../../docs/cli/type/05_settings_value.md)

---

### TC-5: `value::dark` → JSON string `"dark"`

- **Given:** writable settings file
- **When:** `cm .settings.set key::theme value::dark`; read back via JSON format
- **Then:** exit 0; stored value is JSON string `"dark"` (string fallback for non-bool, non-numeric)
- **Exit:** 0
- **Source:** [05_settings_value.md — Type Inference: Everything else → String](../../../../docs/cli/type/05_settings_value.md)

---

### TC-6: `value::NaN` → JSON string `"NaN"`

- **Given:** writable settings file
- **When:** `cm .settings.set key::x value::NaN`; read back via JSON format
- **Then:** exit 0; stored value is JSON string `"NaN"` — NaN is not a valid JSON number literal, classified as string
- **Exit:** 0
- **Source:** [05_settings_value.md — Type Inference: "NaN", "inf", "infinity" → String](../../../../docs/cli/type/05_settings_value.md)

---

### TC-7: `value::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .settings.set key::theme value::`
- **Then:** exit 1; error message contains "value:: value cannot be empty" or similar
- **Exit:** 1
- **Source:** [05_settings_value.md — validation: "value:: value cannot be empty"](../../../../docs/cli/type/05_settings_value.md)

---

### TC-8: absent `value::` → exit 1

- **Given:** clean environment
- **When:** `cm .settings.set key::theme`
- **Then:** exit 1; error message contains "value:: is required" or similar
- **Exit:** 1
- **Source:** [05_settings_value.md — validation: "value:: is required" if missing](../../../../docs/cli/type/05_settings_value.md)

---

### Source Functions

| Function | File |
|----------|------|
| ⏳ `tc_settings_value_bool_true_inferred` | `integration/mutation_commands_test.rs` |
| ⏳ `tc_settings_value_bool_false_inferred` | `integration/mutation_commands_test.rs` |
| ⏳ `tc_settings_value_integer_inferred` | `integration/mutation_commands_test.rs` |
| ⏳ `tc_settings_value_float_inferred` | `integration/mutation_commands_test.rs` |
| ⏳ `tc_settings_value_string_fallback` | `integration/mutation_commands_test.rs` |
| ⏳ `tc_settings_value_nan_as_string` | `integration/mutation_commands_test.rs` |
| ⏳ `tc_settings_value_empty_exits_1` | `cli_args_test.rs` |
| ⏳ `tc_settings_value_absent_exits_1` | `cli_args_test.rs` |
