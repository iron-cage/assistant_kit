# Type :: 14. `StrategyType`

Type constraint tests for `StrategyType` — resume strategy enum.

**Source:** [type/14_strategy_type.md](../../../../docs/cli/type/14_strategy_type.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | "resume" variant accepted | Valid Enum |
| TC-2 | "fresh" variant accepted | Valid Enum |
| TC-3 | Case-insensitive parse accepted | Case Handling |
| TC-4 | Invalid value rejected | Invalid Input |
| TC-5 | Canonical name returned from get() | Method Output |

## Test Coverage Summary

- Valid Enum: 2 tests (TC-1, TC-2)
- Case Handling: 1 test (TC-3)
- Invalid Input: 1 test (TC-4)
- Method Output: 1 test (TC-5)

**Total:** 5 cases

## Test Cases

---

### TC-1: "resume" variant accepted

- **Given:** Input string `"resume"`
- **When:** `StrategyType` is parsed
- **Then:** Accepted as `StrategyType::Resume`; `is_resume()` returns true; `is_fresh()` returns false

---

### TC-2: "fresh" variant accepted

- **Given:** Input string `"fresh"`
- **When:** `StrategyType` is parsed
- **Then:** Accepted as `StrategyType::Fresh`; `is_fresh()` returns true; `is_resume()` returns false

---

### TC-3: Case-insensitive parse accepted

- **Given:** Input string `"RESUME"` (uppercase)
- **When:** `StrategyType` is parsed
- **Then:** Accepted as `StrategyType::Resume` — parsing normalizes case before matching

---

### TC-4: Invalid value rejected

- **Given:** Input string `"continue"`
- **When:** `StrategyType` is parsed
- **Then:** Rejected; error message is `strategy must be resume|fresh, got continue`

---

### TC-5: Canonical name returned from get()

- **Given:** `StrategyType::Fresh` already parsed
- **When:** `get()` is called
- **Then:** Returns `"fresh"` — always lowercase canonical form regardless of input case
