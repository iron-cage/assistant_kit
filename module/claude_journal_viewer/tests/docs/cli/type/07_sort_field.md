# Type :: `SortField`

Validation tests for the `SortField` enum. Tests validate all 6 variants
sort by their documented field, case-insensitive matching, and
invalid-variant error handling.

**Source:** [type/07_sort_field.md](../../../../docs/cli/type/07_sort_field.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `time` -> sorted by timestamp, oldest first by default | Parsing |
| TC-2 | `cost` -> sorted by cost, cheapest first by default | Parsing |
| TC-3 | `duration` -> sorted by execution duration | Parsing |
| TC-4 | `exit` -> sorted by exit code, 0 first by default | Parsing |
| TC-5 | `model`, `command` -> sorted alphabetically | Parsing |
| TC-6 | Case-insensitive matching (`COST` = `cost`) | Case Insensitivity |
| TC-7 | Invalid variant -> exit 1 listing valid options | Error Handling |

## Test Coverage Summary

- Parsing: 5 tests (TC-1, TC-2, TC-3, TC-4, TC-5)
- Case Insensitivity: 1 test (TC-6)
- Error Handling: 1 test (TC-7)

**Total:** 7 test cases

## Test Cases

---

### TC-1: `time` -> sorted by timestamp, oldest first by default

- **Given:** journal with events at different timestamps
- **When:** `clj .list sort::time`
- **Then:** exit 0; events are ordered oldest first
- **Exit:** 0
- **Source:** [type/07_sort_field.md](../../../../docs/cli/type/07_sort_field.md)

---

### TC-2: `cost` -> sorted by cost, cheapest first by default

- **Given:** journal with events of varying cost
- **When:** `clj .list sort::cost`
- **Then:** exit 0; events are ordered cheapest first
- **Exit:** 0
- **Source:** [type/07_sort_field.md](../../../../docs/cli/type/07_sort_field.md)

---

### TC-3: `duration` -> sorted by execution duration

- **Given:** journal with events of varying execution duration
- **When:** `clj .list sort::duration`
- **Then:** exit 0; events are ordered fastest first
- **Exit:** 0
- **Source:** [type/07_sort_field.md](../../../../docs/cli/type/07_sort_field.md)

---

### TC-4: `exit` -> sorted by exit code, 0 first by default

- **Given:** journal with events of varying exit codes
- **When:** `clj .list sort::exit`
- **Then:** exit 0; events with exit code 0 appear first
- **Exit:** 0
- **Source:** [type/07_sort_field.md](../../../../docs/cli/type/07_sort_field.md)

---

### TC-5: `model`, `command` -> sorted alphabetically

- **Given:** journal with events across multiple models and commands
- **When:** `clj .list sort::model` and `clj .list sort::command`
- **Then:** exit 0 for both; events ordered alphabetically a-z by the respective field
- **Exit:** 0 for both
- **Source:** [type/07_sort_field.md](../../../../docs/cli/type/07_sort_field.md)

---

### TC-6: Case-insensitive matching (`COST` = `cost`)

- **Given:** journal with events of varying cost
- **When:** `clj .list sort::COST` compared to `clj .list sort::cost`
- **Then:** both produce identical ordering
- **Exit:** 0 for both
- **Source:** [type/07_sort_field.md](../../../../docs/cli/type/07_sort_field.md)

---

### TC-7: Invalid variant -> exit 1 listing valid options

- **Given:** clean environment
- **When:** `clj .list sort::popularity`
- **Then:** exit 1; stderr lists the 6 valid `SortField` variants
- **Exit:** 1
- **Source:** [type/07_sort_field.md](../../../../docs/cli/type/07_sort_field.md)
