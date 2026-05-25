# Type :: 6. `ProjectType`

Type constraint tests for `ProjectType` — project naming scheme enum.

**Source:** [type/06_project_type.md](../../../../docs/cli/type/06_project_type.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | "path" variant accepted | Valid Enum |
| TC-2 | "uuid" variant accepted | Valid Enum |
| TC-3 | "all" variant accepted (default) | Valid Enum |
| TC-4 | Case-insensitive parse accepted | Case Handling |
| TC-5 | Invalid value rejected | Invalid Input |

## Test Coverage Summary

- Valid Enum: 3 tests (TC-1, TC-2, TC-3)
- Case Handling: 1 test (TC-4)
- Invalid Input: 1 test (TC-5)

**Total:** 5 cases

## Test Cases

---

### TC-1: "path" variant accepted

- **Given:** Input string `"path"`
- **When:** `ProjectType` is parsed
- **Then:** Accepted as `ProjectType::Path`; `matches()` returns true for path-encoded projects

---

### TC-2: "uuid" variant accepted

- **Given:** Input string `"uuid"`
- **When:** `ProjectType` is parsed
- **Then:** Accepted as `ProjectType::Uuid`; `matches()` returns true for UUID-named projects

---

### TC-3: "all" variant accepted (default)

- **Given:** Input string `"all"`
- **When:** `ProjectType` is parsed
- **Then:** Accepted as `ProjectType::All`; `is_all()` returns true — no naming scheme filter applied

---

### TC-4: Case-insensitive parse accepted

- **Given:** Input string `"UUID"` (uppercase)
- **When:** `ProjectType` is parsed
- **Then:** Accepted as `ProjectType::Uuid` — parsing normalizes case before matching

---

### TC-5: Invalid value rejected

- **Given:** Input string `"git"`
- **When:** `ProjectType` is parsed
- **Then:** Rejected; error message is `type must be uuid|path|all, got git`
