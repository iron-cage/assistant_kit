# Type :: 7. `ScopeValue`

Type constraint tests for `ScopeValue` — discovery boundary enum.

**Source:** [type/07_scope_value.md](../../../../docs/cli/type/07_scope_value.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | "local" variant accepted | Valid Enum |
| TC-2 | "relevant" variant accepted | Valid Enum |
| TC-3 | "under" variant accepted | Valid Enum |
| TC-4 | "global" variant accepted | Valid Enum |
| TC-5 | "around" variant accepted (default) | Valid Enum |
| TC-6 | Invalid value rejected | Invalid Input |

## Test Coverage Summary

- Valid Enum: 5 tests (TC-1 through TC-5)
- Invalid Input: 1 test (TC-6)

**Total:** 6 cases

## Test Cases

---

### TC-1: "local" variant accepted

- **Given:** Input string `"local"`
- **When:** `ScopeValue` is parsed
- **Then:** Accepted as `ScopeValue::Local`; `requires_path()` returns false

---

### TC-2: "relevant" variant accepted

- **Given:** Input string `"relevant"`
- **When:** `ScopeValue` is parsed
- **Then:** Accepted as `ScopeValue::Relevant`; covers ancestor chain from CWD to `/`

---

### TC-3: "under" variant accepted

- **Given:** Input string `"under"`
- **When:** `ScopeValue` is parsed
- **Then:** Accepted as `ScopeValue::Under`; `requires_path()` returns true

---

### TC-4: "global" variant accepted

- **Given:** Input string `"global"`
- **When:** `ScopeValue` is parsed
- **Then:** Accepted as `ScopeValue::Global`; `ignores_path()` returns true

---

### TC-5: "around" variant accepted (default)

- **Given:** Input string `"around"`
- **When:** `ScopeValue` is parsed
- **Then:** Accepted as `ScopeValue::Around`; `is_default()` returns true; union of relevant + under

---

### TC-6: Invalid value rejected

- **Given:** Input string `"nearby"`
- **When:** `ScopeValue` is parsed
- **Then:** Rejected; error message is `scope must be relevant|local|under|global|around, got nearby`
