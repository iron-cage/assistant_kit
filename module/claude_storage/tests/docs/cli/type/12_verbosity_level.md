# Type :: 12. `VerbosityLevel`

Type constraint tests for `VerbosityLevel` — output detail level integer (0-5).

**Source:** [type/12_verbosity_level.md](../../../../docs/cli/type/12_verbosity_level.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Value 0 accepted (minimum, machine-readable) | Valid Boundary |
| TC-2 | Value 1 accepted (default, normal) | Valid Boundary |
| TC-3 | Value 2 accepted (detailed) | Valid Boundary |
| TC-4 | Value 5 accepted (maximum) | Valid Boundary |
| TC-5 | Value 6 rejected (above range) | Invalid Input |
| TC-6 | Non-integer string rejected | Type Error |

## Test Coverage Summary

- Valid Boundary: 4 tests (TC-1, TC-2, TC-3, TC-4)
- Invalid Input: 1 test (TC-5)
- Type Error: 1 test (TC-6)

**Total:** 6 cases

## Test Cases

---

### TC-1: Value 0 accepted (minimum, machine-readable)

- **Given:** Input string `"0"`
- **When:** `VerbosityLevel` is parsed
- **Then:** Accepted as `VerbosityLevel(0)`; `is_silent()` returns true; `get()` returns 0

---

### TC-2: Value 1 accepted (default, normal)

- **Given:** Input string `"1"`
- **When:** `VerbosityLevel` is parsed
- **Then:** Accepted as `VerbosityLevel(1)`; `is_normal()` returns true; matches `default()`

---

### TC-3: Value 2 accepted (detailed)

- **Given:** Input string `"2"`
- **When:** `VerbosityLevel` is parsed
- **Then:** Accepted as `VerbosityLevel(2)`; `is_detailed()` returns true

---

### TC-4: Value 5 accepted (maximum)

- **Given:** Input string `"5"`
- **When:** `VerbosityLevel` is parsed
- **Then:** Accepted as `VerbosityLevel(5)`; `is_verbose()` returns true; `get()` returns 5

---

### TC-5: Value 6 rejected (above range)

- **Given:** Input string `"6"`
- **When:** `VerbosityLevel` is parsed
- **Then:** Rejected; error message is `verbosity must be 0-5, got 6`

---

### TC-6: Non-integer string rejected

- **Given:** Input string `"high"`
- **When:** `VerbosityLevel` is parsed
- **Then:** Rejected; error message is `verbosity must be an integer 0-5, got high`
