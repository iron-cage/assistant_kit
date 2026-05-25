# Type :: 11. `TargetType`

Type constraint tests for `TargetType` — count target selector enum.

**Source:** [type/11_target_type.md](../../../../docs/cli/type/11_target_type.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | "projects" variant accepted (default) | Valid Enum |
| TC-2 | "sessions" variant accepted | Valid Enum |
| TC-3 | "entries" variant accepted | Valid Enum |
| TC-4 | Case-insensitive parse accepted | Case Handling |
| TC-5 | Invalid value rejected | Invalid Input |

## Test Coverage Summary

- Valid Enum: 3 tests (TC-1, TC-2, TC-3)
- Case Handling: 1 test (TC-4)
- Invalid Input: 1 test (TC-5)

**Total:** 5 cases

## Test Cases

---

### TC-1: "projects" variant accepted (default)

- **Given:** Input string `"projects"`
- **When:** `TargetType` is parsed
- **Then:** Accepted as `TargetType::Projects`; `is_default()` returns true; `requires_project()` returns false

---

### TC-2: "sessions" variant accepted

- **Given:** Input string `"sessions"`
- **When:** `TargetType` is parsed
- **Then:** Accepted as `TargetType::Sessions`; `requires_project()` returns true; `requires_session()` returns false

---

### TC-3: "entries" variant accepted

- **Given:** Input string `"entries"`
- **When:** `TargetType` is parsed
- **Then:** Accepted as `TargetType::Entries`; `requires_project()` returns true; `requires_session()` returns true

---

### TC-4: Case-insensitive parse accepted

- **Given:** Input string `"Sessions"` (mixed case)
- **When:** `TargetType` is parsed
- **Then:** Accepted as `TargetType::Sessions` — parsing normalizes case before matching

---

### TC-5: Invalid value rejected

- **Given:** Input string `"lines"`
- **When:** `TargetType` is parsed
- **Then:** Rejected; error message is `target must be projects|sessions|entries, got lines`
