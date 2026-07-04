# Type :: `OutputFormat`

Validation tests for the `OutputFormat` enum. Tests validate all 4
variants, case-insensitive matching, and invalid-variant error handling.

**Source:** [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `table` -> aligned columns with headers | Parsing |
| TC-2 | `json` -> single JSON array | Parsing |
| TC-3 | `jsonl` -> one JSON object per line | Parsing |
| TC-4 | `csv` -> header row + comma-separated values | Parsing |
| TC-5 | Case-insensitive matching (`JSON` = `json`) | Case Insensitivity |
| TC-6 | Invalid variant -> exit 1 listing valid options | Error Handling |

## Test Coverage Summary

- Parsing: 4 tests (TC-1, TC-2, TC-3, TC-4)
- Case Insensitivity: 1 test (TC-5)
- Error Handling: 1 test (TC-6)

**Total:** 6 test cases

## Test Cases

---

### TC-1: `table` -> aligned columns with headers

- **Given:** journal with events
- **When:** `clj .list format::table`
- **Then:** exit 0; output is an aligned table with column headers
- **Exit:** 0
- **Source:** [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)

---

### TC-2: `json` -> single JSON array

- **Given:** journal with events
- **When:** `clj .list format::json`
- **Then:** exit 0; output is a single JSON array containing one object per event
- **Exit:** 0
- **Source:** [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)

---

### TC-3: `jsonl` -> one JSON object per line

- **Given:** journal with events
- **When:** `clj .export format::jsonl`
- **Then:** exit 0; each output line is one complete, independently-parseable JSON object
- **Exit:** 0
- **Source:** [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)

---

### TC-4: `csv` -> header row + comma-separated values

- **Given:** journal with events
- **When:** `clj .export format::csv`
- **Then:** exit 0; first line is a header row; subsequent lines are comma-separated values
- **Exit:** 0
- **Source:** [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)

---

### TC-5: Case-insensitive matching (`JSON` = `json`)

- **Given:** journal with events
- **When:** `clj .list format::JSON` compared to `clj .list format::json`
- **Then:** both produce identical JSON output
- **Exit:** 0 for both
- **Source:** [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)

---

### TC-6: Invalid variant -> exit 1 listing valid options

- **Given:** clean environment
- **When:** `clj .list format::xml`
- **Then:** exit 1; stderr lists the 4 valid `OutputFormat` variants
- **Exit:** 1
- **Source:** [type/06_output_format.md](../../../../docs/cli/type/06_output_format.md)
