# Test: `SettingsValue`

Type compliance and validation tests for `SettingsValue`. See [type/05_settings_value.md](../../../../docs/cli/type/05_settings_value.md) for specification.

### Scope

- **Purpose**: Validate SettingsValue type inference rules and required-field enforcement.
- **Responsibility**: JSON type inference for bool, integer, float, and string variants; NaN/inf special-case handling; empty rejection.
- **Commands:** `.settings.set`
- **In Scope**: Type inference for all 5 input categories, required-field validation.
- **Out of Scope**: Key parsing (→ `004_settings_key.md`), settings file I/O (→ `../command/011_settings_set.md`).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `value::true` → JSON boolean true | Inference: bool |
| TC-2 | `value::false` → JSON boolean false | Inference: bool |
| TC-3 | `value::42` → JSON integer 42 | Inference: integer |
| TC-4 | `value::3.14` → JSON float 3.14 | Inference: float |
| TC-5 | `value::dark` → JSON string "dark" | Inference: string |
| TC-6 | `value::NaN` → JSON string "NaN" (not float) | Inference: non-finite float |
| EC-1 | Missing `value::` entirely → exit 1 | Validation: required |
| EC-2 | `value::` (empty) → exit 1 | Validation: empty |

## Test Coverage Summary

- Bool inference: 2 tests (TC-1, TC-2)
- Integer inference: 1 test (TC-3)
- Float inference: 1 test (TC-4)
- String fallback: 1 test (TC-5)
- Non-finite float as string: 1 test (TC-6)
- Required field: 1 test (EC-1)
- Empty value: 1 test (EC-2)

**Total:** 8 tests

**Behavioral Divergence Pair:** TC-1 (`value::true` → settings.json contains `true` unquoted boolean) ↔ TC-5 (`value::dark` → settings.json contains `"dark"` quoted string)

---

### TC-1: `value::true` → JSON boolean

- **Given:** clean isolated HOME
- **When:** `cm .settings.set key::flag value::true` then read `~/.claude/settings.json`
- **Then:** settings.json contains `"flag": true` (unquoted boolean, not `"flag": "true"`)
- **Exit:** 0
- **Source:** [type/05_settings_value.md — Inference: "true"/"false" → Bool](../../../../docs/cli/type/05_settings_value.md)

---

### TC-2: `value::false` → JSON boolean false

- **Given:** clean isolated HOME
- **When:** `cm .settings.set key::flag value::false` then read `~/.claude/settings.json`
- **Then:** settings.json contains `"flag": false` (unquoted boolean)
- **Exit:** 0
- **Source:** [type/05_settings_value.md — Inference: "false" → Bool](../../../../docs/cli/type/05_settings_value.md)

---

### TC-3: `value::42` → JSON integer

- **Given:** clean isolated HOME
- **When:** `cm .settings.set key::count value::42` then read `~/.claude/settings.json`
- **Then:** settings.json contains `"count": 42` (unquoted integer, not `"count": "42"`)
- **Exit:** 0
- **Source:** [type/05_settings_value.md — Inference: integer string → Number (i64)](../../../../docs/cli/type/05_settings_value.md)

---

### TC-4: `value::3.14` → JSON float

- **Given:** clean isolated HOME
- **When:** `cm .settings.set key::rate value::3.14` then read `~/.claude/settings.json`
- **Then:** settings.json contains `"rate": 3.14` (unquoted float)
- **Exit:** 0
- **Source:** [type/05_settings_value.md — Inference: finite float string → Number (f64)](../../../../docs/cli/type/05_settings_value.md)

---

### TC-5: `value::dark` → JSON string

- **Given:** clean isolated HOME
- **When:** `cm .settings.set key::theme value::dark` then read `~/.claude/settings.json`
- **Then:** settings.json contains `"theme": "dark"` (quoted string)
- **Exit:** 0
- **Source:** [type/05_settings_value.md — Inference: everything else → String](../../../../docs/cli/type/05_settings_value.md)

---

### TC-6: `value::NaN` → JSON string (not float)

- **Given:** clean isolated HOME
- **When:** `cm .settings.set key::special value::NaN` then read `~/.claude/settings.json`
- **Then:** settings.json contains `"special": "NaN"` (quoted string, not float NaN which is invalid JSON)
- **Exit:** 0
- **Source:** [type/05_settings_value.md — Inference: NaN/inf/infinity → String](../../../../docs/cli/type/05_settings_value.md)

---

### EC-1: Missing `value::` entirely → exit 1

- **Given:** clean environment
- **When:** `cm .settings.set key::theme` (no `value::` parameter)
- **Then:** exit code 1; error message says "value:: is required" or equivalent
- **Exit:** 1
- **Source:** [type/05_settings_value.md — Validation: value:: is required](../../../../docs/cli/type/05_settings_value.md)

---

### EC-2: `value::` (empty) → exit 1

- **Given:** clean environment
- **When:** `cm .settings.set key::theme value::`
- **Then:** exit code 1; error message says "value:: value cannot be empty" or equivalent
- **Exit:** 1
- **Source:** [type/05_settings_value.md — Validation: value:: value cannot be empty](../../../../docs/cli/type/05_settings_value.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc_settings_value_bool_true_inferred` | `integration/mutation_commands_test.rs` |
| `tc_settings_value_bool_false_inferred` | `integration/mutation_commands_test.rs` |
| `tc_settings_value_integer_inferred` | `integration/mutation_commands_test.rs` |
| `tc_settings_value_float_inferred` | `integration/mutation_commands_test.rs` |
| `tc_settings_value_string_fallback` | `integration/mutation_commands_test.rs` |
| `tc_settings_value_nan_as_string` | `integration/mutation_commands_test.rs` |
| `tc_settings_value_empty_exits_1` | `cli_args_test.rs` |
| `tc_settings_value_absent_exits_1` | `cli_args_test.rs` |
