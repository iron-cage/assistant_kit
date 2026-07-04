# Type :: `Duration`

Validation tests for the `Duration` semantic type. Tests validate suffix
parsing, case sensitivity, and invalid-format error handling.

**Source:** [type/01_duration.md](../../../../docs/cli/type/01_duration.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `2h` -> parses as 7200 seconds | Parsing |
| TC-2 | `4w` -> parses as 2419200 seconds | Parsing |
| TC-3 | `3M` (months) vs `3m` (minutes) -> case-sensitive, different values | Case Sensitivity |
| TC-4 | `30` (no suffix) -> exit 1, expected-format error | Error Handling |
| TC-5 | `0h` (zero numeric part) -> exit 1 | Boundary |

## Test Coverage Summary

- Parsing: 2 tests (TC-1, TC-2)
- Case Sensitivity: 1 test (TC-3)
- Error Handling: 1 test (TC-4)
- Boundary: 1 test (TC-5)

**Total:** 5 test cases

## Test Cases

---

### TC-1: `2h` -> parses as 7200 seconds

- **Given:** clean environment
- **When:** `clj .list since::2h`
- **Then:** filter window start is 7200 seconds before now
- **Exit:** 0
- **Source:** [type/01_duration.md](../../../../docs/cli/type/01_duration.md)

---

### TC-2: `4w` -> parses as 2419200 seconds

- **Given:** clean environment
- **When:** `clj .prune keep::4w dry_run::1`
- **Then:** retention threshold is 2419200 seconds (4 weeks) before now
- **Exit:** 0
- **Source:** [type/01_duration.md](../../../../docs/cli/type/01_duration.md)

---

### TC-3: `3M` (months) vs `3m` (minutes) -> case-sensitive, different values

- **Given:** clean environment
- **When:** `clj .list since::3M` compared to `clj .list since::3m`
- **Then:** `3M` resolves to 3 months (7776000s); `3m` resolves to 3 minutes (180s); the two produce different filter windows
- **Exit:** 0 for both
- **Source:** [type/01_duration.md](../../../../docs/cli/type/01_duration.md)

---

### TC-4: `30` (no suffix) -> exit 1, expected-format error

- **Given:** clean environment
- **When:** `clj .list since::30`
- **Then:** exit 1; stderr contains `expected format: <number><s|m|h|d|w|M>`
- **Exit:** 1
- **Source:** [type/01_duration.md](../../../../docs/cli/type/01_duration.md)

---

### TC-5: `0h` (zero numeric part) -> exit 1

- **Given:** clean environment
- **When:** `clj .list since::0h`
- **Then:** exit 1; numeric part must be greater than 0
- **Exit:** 1
- **Source:** [type/01_duration.md](../../../../docs/cli/type/01_duration.md)
