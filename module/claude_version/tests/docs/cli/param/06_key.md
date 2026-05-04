# Test: `key::`

Edge case coverage for the `key::` parameter. See [params.md](../../../../docs/cli/params.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `key::existing` on `.settings.get` → returns value | Valid: existing |
| EC-2 | `key::nonexistent` → exit 2, key not found | Valid: missing |
| EC-3 | No `key::` on `.settings.get` → exit 1 | Absent (required) |
| TC-237 | Without `key::` → error message mentions `key::` | Error Content |
| TC-238 | Without `key::` on `.settings.set` → error mentions `key::` | Absent (required) |
| EC-4 | `key::""` (empty key) on `.settings.set` → exit 1 | Empty Value |
| EC-1 | `key::` (empty value) on `.settings.get` → exit 1 | Empty Value |
| EC-2 | `key::` only accepted by `.settings.get` and `.settings.set` | Command Scope |
| EC-3 | `key::a b c` (key with spaces) → behavior defined | Special Characters |
| EC-4 | `key::foo.bar` (dot in key name) → stored as given | Special Characters |
| EC-1 | `key::foo bar` (space in key) → stored as given | Special Characters |

## Test Coverage Summary

- Valid (existing key): 1 test
- Valid (missing key → exit 2): 1 test
- Absent (required → exit 1): 2 tests
- Error Content: 1 test
- Empty Value: 2 tests
- Command Scope: 1 test
- Special Characters: 3 tests

**Total:** 12 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

---

### EC-1: `key::existing` → value returned

- **Given:** `HOME=<tmp>`; settings has `myKey = "myValue"`.
- **When:** `cm .settings.get key::myKey`
- **Then:** exit 0; output contains "myValue".; correct value returned
- **Exit:** 0
- **Source:** [commands.md — .settings.get](../../../../docs/cli/commands.md)

---

### EC-2: `key::nonexistent` → exit 2

- **Given:** `HOME=<tmp>`; settings has different key.
- **When:** `cm .settings.get key::nosuchkey`
- **Then:** exit code 2.
- **Exit:** 2
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-3: No `key::` → exit 1

- **Given:** Valid settings file.
- **When:** `cm .settings.get`
- **Then:** exit code 1; error mentions key.
- **Exit:** 1
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-4: `key::""` → exit 1

- **Given:** clean environment
- **When:** `cm .settings.set key:: value::x`
- **Then:** exit code 1; error mentions empty key.
- **Exit:** 1
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-1: `key::` (empty value) on `.settings.get` → exit 1

- **Given:** clean environment
- **When:** `cm .settings.get key::`
- **Then:** exit code 1.
- **Exit:** 1
- **Source:** [feature/003_settings_management.md](../../../../docs/feature/003_settings_management.md)

---

### EC-2: `key::` only for `.settings.get` and `.settings.set`

- **Given:** clean environment
- **When:** `cm .status key::foo`
- **Then:** exit code 1; unknown parameter.
- **Exit:** 1
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### EC-3: `key::foo.bar` (dot in key name)

- **Given:** `HOME=<tmp>`; no existing settings.
- **When:** `cm .settings.set key::foo.bar value::baz && cm .settings.get key::foo.bar`
- **Then:** `baz` returned for key `foo.bar`.; key round-trips correctly.
**Note:** Tests that the key is treated as an opaque string, not a nested path
- **Exit:** 0
