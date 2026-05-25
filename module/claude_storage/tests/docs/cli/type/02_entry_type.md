# Type :: 2. `EntryType`

Type constraint tests for `EntryType` — author filter enum.

**Source:** [type/02_entry_type.md](../../../../docs/cli/type/02_entry_type.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | "user" variant accepted | Valid Enum |
| TC-2 | "assistant" variant accepted | Valid Enum |
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

### TC-1: "user" variant accepted

- **Given:** Input string `"user"`
- **When:** `EntryType` is parsed
- **Then:** Accepted as `EntryType::User`

---

### TC-2: "assistant" variant accepted

- **Given:** Input string `"assistant"`
- **When:** `EntryType` is parsed
- **Then:** Accepted as `EntryType::Assistant`

---

### TC-3: "all" variant accepted (default)

- **Given:** Input string `"all"`
- **When:** `EntryType` is parsed
- **Then:** Accepted as `EntryType::All` — represents no author filter; `is_all()` returns true

---

### TC-4: Case-insensitive parse accepted

- **Given:** Input string `"User"` (mixed case)
- **When:** `EntryType` is parsed
- **Then:** Accepted as `EntryType::User` — parsing normalizes case before matching

---

### TC-5: Invalid value rejected

- **Given:** Input string `"system"`
- **When:** `EntryType` is parsed
- **Then:** Rejected; error message is `entry_type must be user|assistant|all, got system`
