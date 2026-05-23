# Algorithm Test: Settings Type Inference

### Scope

- **Purpose**: AC- test cases for the 4-step type inference applied to `value::` in `.settings.set`.
- **Responsibility**: Verify that each inference step (bool, int, float, string) produces the correct JSON type and that NaN/inf fall through to string.
- **In Scope**: Step 1 (bool), Step 2 (int), Step 3 (finite float), Step 3 edge case (NaN/inf → string), Step 4 (string fallback).
- **Out of Scope**: Settings write mechanics (-> `../feature/003_settings_management.md`), CLI parameter validation (-> `../feature/005_cli_design.md`).

Algorithm test surface for settings type inference. See [algorithm/001_settings_type_inference.md](../../../../docs/algorithm/001_settings_type_inference.md) for specification.

## Behavioral Divergence Pair

Two string inputs that look similar but resolve to different JSON types:

- **Input A:** `cm .settings.set key::k value::true` → stores `true` (JSON boolean, Step 1)
- **Input B:** `cm .settings.set key::k value::1` → stores `1` (JSON integer, Step 2)

Both are valid values; the stored JSON type differs because `"1"` is not `"true"` or `"false"`.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| AC-1 | `value::true` → JSON boolean `true` (Step 1) | Bool Inference |
| AC-2 | `value::false` → JSON boolean `false` (Step 1) | Bool Inference |
| AC-3 | `value::1` → JSON integer `1`, not boolean (Step 2) | Int Inference |
| AC-4 | `value::3.14` → JSON float `3.14` (Step 3) | Float Inference |
| AC-5 | `value::nan` → JSON string `"nan"` (Step 3 → string fallback) | NaN Edge Case |
| AC-6 | `value::hello` → JSON string `"hello"` (Step 4) | String Fallback |

## Test Coverage Summary

- Bool Inference: 2 tests (AC-1, AC-2)
- Int Inference: 1 test (AC-3)
- Float Inference: 1 test (AC-4)
- NaN Edge Case: 1 test (AC-5)
- String Fallback: 1 test (AC-6)

**Total:** 6 tests

---

### AC-1: `value::true` → JSON boolean `true` (Step 1)

- **Given:** isolated HOME with empty `settings.json`
- **When:** `cm .settings.set key::flag value::true`
- **Then:** `settings.json` contains `"flag": true` (unquoted); exit 0
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md — Step 1](../../../../docs/algorithm/001_settings_type_inference.md)

---

### AC-2: `value::false` → JSON boolean `false` (Step 1)

- **Given:** isolated HOME with empty `settings.json`
- **When:** `cm .settings.set key::flag value::false`
- **Then:** `settings.json` contains `"flag": false` (unquoted); exit 0
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md — Step 1](../../../../docs/algorithm/001_settings_type_inference.md)

---

### AC-3: `value::1` → JSON integer `1`, not boolean (Step 2)

- **Given:** isolated HOME with empty `settings.json`
- **When:** `cm .settings.set key::n value::1`
- **Then:** `settings.json` contains `"n": 1` (unquoted integer, not `true`); exit 0
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md — Step 2 Note](../../../../docs/algorithm/001_settings_type_inference.md)

---

### AC-4: `value::3.14` → JSON float `3.14` (Step 3)

- **Given:** isolated HOME with empty `settings.json`
- **When:** `cm .settings.set key::pi value::3.14`
- **Then:** `settings.json` contains `"pi": 3.14` (unquoted float); exit 0
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md — Step 3](../../../../docs/algorithm/001_settings_type_inference.md)

---

### AC-5: `value::nan` → JSON string `"nan"` (Step 3 → string fallback)

- **Given:** isolated HOME with empty `settings.json`
- **When:** `cm .settings.set key::bad value::nan`
- **Then:** `settings.json` contains `"bad": "nan"` (quoted string); exit 0
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md — Step 3 edge case](../../../../docs/algorithm/001_settings_type_inference.md)

---

### AC-6: `value::hello` → JSON string `"hello"` (Step 4)

- **Given:** isolated HOME with empty `settings.json`
- **When:** `cm .settings.set key::name value::hello`
- **Then:** `settings.json` contains `"name": "hello"` (quoted string); exit 0
- **Exit:** 0
- **Source:** [algorithm/001_settings_type_inference.md — Step 4](../../../../docs/algorithm/001_settings_type_inference.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc322_settings_set_value_true_stores_bool` | `integration/mutation_commands_test.rs` |
| `tc323_settings_set_value_false_stores_bool` | `integration/mutation_commands_test.rs` |
| `tc324_settings_set_value_0_stores_number` | `integration/mutation_commands_test.rs` |
| TBD (ac004_float_inference) | `integration/algorithm_surface_test.rs` |
| TBD (ac005_nan_stores_string) | `integration/algorithm_surface_test.rs` |
| `tc326_settings_set_value_hello_stores_string` | `integration/mutation_commands_test.rs` |
