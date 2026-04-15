# Test: `value::`

Edge case coverage for the `value::` parameter. See [params.md](../../params.md) and [algorithm/001_settings_type_inference.md](../../../algorithm/001_settings_type_inference.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-322 | `value::true` → JSON boolean `true` | Type Inference |
| TC-323 | `value::false` → JSON boolean `false` | Type Inference |
| TC-324 | `value::0` → JSON integer `0` (not boolean false) | Type Inference |
| TC-325 | `value::42` → JSON integer `42` | Type Inference |
| TC-326 | `value::hello` → JSON string `"hello"` | Type Inference |
| TC-327 | `value::""` → JSON string `""` (empty string valid) | Type Inference |
| TC-321 | `key::k` present but no `value::` → exit 1 | Absent (required) |
| TC-239 | Without `value::` → error message mentions `value::` | Error Content |
| EC-1 | `value::1.5` → JSON float (parseable as f64 but not i64) | Type Inference |
| EC-2 | `value::NaN` → JSON string (not number, NaN is not finite) | Type Inference (edge) |
| EC-3 | `value::Infinity` → JSON string (not float, infinite) | Type Inference (edge) |
| EC-4 | `value::true false` (space in value) → JSON string | Type Inference |
| EC-5 | `value::` (empty, no quotes) → stores `""` | Empty Value |
| EC-6 | `value::` only for `.settings.set` | Command Scope |
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

### TC-322: `value::true` → boolean `true`

**Goal:** Canonical boolean literals are type-inferred to JSON boolean.
**Setup:** `HOME=<tmp>`.
**Command:** `cm .settings.set key::flag value::true`
**Expected Output:** exit 0; `settings.json` has `"flag": true` (unquoted).
**Pass Criteria:** Exit 0; native boolean stored.
**Source:** [algorithm/001_settings_type_inference.md](../../../algorithm/001_settings_type_inference.md)

---

### TC-324: `value::0` → integer `0`, NOT boolean

**Goal:** "0" stores as integer 0, not as false. Type check order: i64 before boolean.
**Setup:** `HOME=<tmp>`.
**Command:** `cm .settings.set key::n value::0`
**Expected Output:** exit 0; `settings.json` has `"n": 0` (integer, not `false`).
**Pass Criteria:** Exit 0; integer stored.
**Source:** [algorithm/001_settings_type_inference.md](../../../algorithm/001_settings_type_inference.md)

---

### TC-327: `value::""` → empty string

**Goal:** Empty string value is stored as JSON empty string `""`, not treated as missing.
**Setup:** `HOME=<tmp>`.
**Command:** `cm .settings.set key::s value::`
**Expected Output:** exit 0 (or exit 1 if empty value is rejected — need to verify behavior).
**Note:** If `value::` with empty is treated as absent (missing value), exit 1. If accepted
as empty string, exit 0 with `"s": ""`. Check FR-04 vs FR-07 interaction.
**Pass Criteria:** Consistent with spec; no crash.
**Source:** [feature/003_settings_management.md](../../../feature/003_settings_management.md)

---

### EC-1: `value::1.5` → JSON float

**Goal:** Float-like strings that aren't i64 become JSON floats.
**Setup:** `HOME=<tmp>`.
**Command:** `cm .settings.set key::f value::1.5`
**Expected Output:** exit 0; `settings.json` has `"f": 1.5` (unquoted float).
**Pass Criteria:** Exit 0; float stored.
**Source:** [algorithm/001_settings_type_inference.md](../../../algorithm/001_settings_type_inference.md)

---

### EC-2: `value::NaN` → JSON string

**Goal:** "NaN" is not finite; type check for float fails; stored as string.
**Setup:** `HOME=<tmp>`.
**Command:** `cm .settings.set key::x value::NaN`
**Expected Output:** exit 0; `settings.json` has `"x": "NaN"` (quoted string).
**Pass Criteria:** Exit 0; string stored.
**Source:** [algorithm/001_settings_type_inference.md](../../../algorithm/001_settings_type_inference.md)

---

### EC-6: `value::` only for `.settings.set`

**Goal:** Only `.settings.set` accepts `value::`.
**Setup:** None.
**Command:** `cm .settings.get key::k value::v`
**Expected Output:** exit code 1; unknown parameter.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)
