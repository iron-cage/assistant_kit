# Type :: `Boolean`

Validation tests for the `Boolean` fundamental type. Tests validate the
0/1 integer convention and rejection of any other value.

**Source:** [type/08_boolean.md](../../../../docs/cli/type/08_boolean.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `0` -> false/disabled | Parsing |
| TC-2 | `1` -> true/enabled | Parsing |
| TC-3 | Any other value (`true`, `2`, `yes`) -> exit 1 | Error Handling |

## Test Coverage Summary

- Parsing: 2 tests (TC-1, TC-2)
- Error Handling: 1 test (TC-3)

**Total:** 3 test cases

## Test Cases

---

### TC-1: `0` -> false/disabled

- **Given:** clean environment
- **When:** `clj .list reverse::0`
- **Then:** exit 0; reverse sort is disabled
- **Exit:** 0
- **Source:** [type/08_boolean.md](../../../../docs/cli/type/08_boolean.md), [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md)

---

### TC-2: `1` -> true/enabled

- **Given:** clean environment
- **When:** `clj .list reverse::1`
- **Then:** exit 0; reverse sort is enabled
- **Exit:** 0
- **Source:** [type/08_boolean.md](../../../../docs/cli/type/08_boolean.md), [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md)

---

### TC-3: Any other value (`true`, `2`, `yes`) -> exit 1

- **Given:** clean environment
- **When:** `clj .list reverse::true`
- **Then:** exit 1; stderr contains `invalid boolean 'true' for parameter 'reverse' — expected 0 or 1`
- **Exit:** 1
- **Source:** [type/08_boolean.md](../../../../docs/cli/type/08_boolean.md)
