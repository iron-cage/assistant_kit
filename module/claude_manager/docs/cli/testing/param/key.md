# Test: `key::`

Edge case coverage for the `key::` parameter. See [params.md](../../params.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-176 | `key::existing` on `.settings.get` → returns value | Valid: existing |
| TC-177 | `key::nonexistent` → exit 2, key not found | Valid: missing |
| TC-174 | No `key::` on `.settings.get` → exit 1 | Absent (required) |
| TC-237 | Without `key::` → error message mentions `key::` | Error Content |
| TC-238 | Without `key::` on `.settings.set` → error mentions `key::` | Absent (required) |
| TC-332 | `key::""` (empty key) on `.settings.set` → exit 1 | Empty Value |
| EC-1 | `key::` (empty value) on `.settings.get` → exit 1 | Empty Value |
| EC-2 | `key::` only accepted by `.settings.get` and `.settings.set` | Command Scope |
| EC-3 | `key::a b c` (key with spaces) → behavior defined | Special Characters |
| EC-4 | `key::foo.bar` (dot in key name) → stored as given | Special Characters |
| EC-5 | `key::foo bar` (space in key) → stored as given | Special Characters |

## Test Coverage Summary

- Valid (existing key): 1 test
- Valid (missing key → exit 2): 1 test
- Absent (required → exit 1): 2 tests
- Error Content: 1 test
- Empty Value: 2 tests
- Command Scope: 1 test
- Special Characters: 3 tests

**Total:** 12 edge cases

---

### TC-176: `key::existing` → value returned

**Goal:** Known key returns its stored value.
**Setup:** `HOME=<tmp>`; settings has `myKey = "myValue"`.
**Command:** `cm .settings.get key::myKey`
**Expected Output:** exit 0; output contains "myValue".
**Pass Criteria:** Exit 0; correct value returned.
**Source:** [commands.md — .settings.get](../../commands.md)

---

### TC-177: `key::nonexistent` → exit 2

**Goal:** Key not present in settings is a runtime error (not usage error).
**Setup:** `HOME=<tmp>`; settings has different key.
**Command:** `cm .settings.get key::nosuchkey`
**Expected Output:** exit code 2.
**Pass Criteria:** Exit 2.
**Source:** [feature/003_settings_management.md](../../../feature/003_settings_management.md)

---

### TC-174: No `key::` → exit 1

**Goal:** `key::` is semantically required.
**Setup:** Valid settings file.
**Command:** `cm .settings.get`
**Expected Output:** exit code 1; error mentions key.
**Pass Criteria:** Exit 1.
**Source:** [feature/003_settings_management.md](../../../feature/003_settings_management.md)

---

### TC-332: `key::""` → exit 1

**Goal:** Empty key string rejected before file access.
**Setup:** None.
**Command:** `cm .settings.set key:: value::x`
**Expected Output:** exit code 1; error mentions empty key.
**Pass Criteria:** Exit 1.
**Source:** [feature/003_settings_management.md](../../../feature/003_settings_management.md)

---

### EC-1: `key::` (empty value) on `.settings.get` → exit 1

**Goal:** Empty string is not a valid key for lookup.
**Setup:** None.
**Command:** `cm .settings.get key::`
**Expected Output:** exit code 1.
**Pass Criteria:** Exit 1.
**Source:** [feature/003_settings_management.md](../../../feature/003_settings_management.md)

---

### EC-2: `key::` only for `.settings.get` and `.settings.set`

**Goal:** Commands that don't declare `key::` reject it.
**Setup:** None.
**Command:** `cm .status key::foo`
**Expected Output:** exit code 1; unknown parameter.
**Pass Criteria:** Exit 1.
**Source:** [feature/005_cli_design.md](../../../feature/005_cli_design.md)

---

### EC-3: `key::foo.bar` (dot in key name)

**Goal:** Dots in key names are stored and retrieved as-is (no path parsing).
**Setup:** `HOME=<tmp>`; no existing settings.
**Command:** `cm .settings.set key::foo.bar value::baz && cm .settings.get key::foo.bar`
**Expected Output:** `baz` returned for key `foo.bar`.
**Pass Criteria:** Exit 0; key round-trips correctly.
**Note:** Tests that the key is treated as an opaque string, not a nested path.
