# Type :: `Integer`

Validation tests for the `Integer` fundamental type. Tests validate
non-negative parsing, rejection of negative/non-numeric input, and
declared-range acceptance.

**Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Valid non-negative integer accepted | Parsing |
| TC-2 | Negative value -> exit 1 | Error Handling |
| TC-3 | Non-numeric input -> exit 1 | Error Handling |
| TC-4 | `exit::255` (upper bound of declared range) -> accepted | Boundary |

## Test Coverage Summary

- Parsing: 1 test (TC-1)
- Error Handling: 2 tests (TC-2, TC-3)
- Boundary: 1 test (TC-4)

**Total:** 4 test cases

## Test Cases

---

### TC-1: Valid non-negative integer accepted

- **Given:** clean environment
- **When:** `clj .list limit::10`
- **Then:** exit 0; `limit` parses as 10
- **Exit:** 0
- **Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md)

---

### TC-2: Negative value -> exit 1

- **Given:** clean environment
- **When:** `clj .list limit::-5`
- **Then:** exit 1; stderr indicates the value is not a valid non-negative integer
- **Exit:** 1
- **Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md)

---

### TC-3: Non-numeric input -> exit 1

- **Given:** clean environment
- **When:** `clj .list limit::abc`
- **Then:** exit 1; stderr contains `invalid integer 'abc' for parameter 'limit'`
- **Exit:** 1
- **Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md)

---

### TC-4: `exit::255` (upper bound of declared range) -> accepted

- **Given:** journal containing an event with exit code 255
- **When:** `clj .list exit::255`
- **Then:** exit 0; the event with exit code 255 is shown
- **Exit:** 0
- **Source:** [type/04_integer.md](../../../../docs/cli/type/04_integer.md), [param/05_exit.md](../../../../docs/cli/param/05_exit.md)
