# Type :: 1. `EntryCount`

Type constraint tests for `EntryCount` — non-negative integer entry threshold.

**Source:** [type/01_entry_count.md](../../../../docs/cli/type/01_entry_count.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Zero accepted (minimum) | Valid Boundary |
| TC-2 | Positive integer accepted | Valid Boundary |
| TC-3 | Large value accepted | Valid Boundary |
| TC-4 | Negative integer rejected | Invalid Input |
| TC-5 | Non-integer string rejected | Type Error |

## Test Coverage Summary

- Valid Boundary: 3 tests (TC-1, TC-2, TC-3)
- Invalid Input: 1 test (TC-4)
- Type Error: 1 test (TC-5)

**Total:** 5 cases

## Test Cases

---

### TC-1: Zero accepted (minimum)

- **Given:** Input string `"0"`
- **When:** `EntryCount` is parsed
- **Then:** Accepted as `EntryCount(0)` — value 0 is valid (no minimum enforced)

---

### TC-2: Positive integer accepted

- **Given:** Input string `"10"`
- **When:** `EntryCount` is parsed
- **Then:** Accepted as `EntryCount(10)` — positive integer within valid range

---

### TC-3: Large value accepted

- **Given:** Input string `"1000000"`
- **When:** `EntryCount` is parsed
- **Then:** Accepted as `EntryCount(1000000)` — any non-negative integer up to i64::MAX is valid

---

### TC-4: Negative integer rejected

- **Given:** Input string `"-1"`
- **When:** `EntryCount` is parsed
- **Then:** Rejected; error message is `min_entries must be ≥ 0, got -1`

---

### TC-5: Non-integer string rejected

- **Given:** Input string `"abc"`
- **When:** `EntryCount` is parsed
- **Then:** Rejected; error message is `min_entries must be a non-negative integer, got abc`
