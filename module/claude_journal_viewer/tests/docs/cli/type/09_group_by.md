# Type :: `GroupBy`

Validation tests for the `GroupBy` enum. Tests validate all 7 grouping
dimensions, case-insensitive matching, and invalid-variant error handling.

**Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `day` -> grouped by calendar date | Parsing |
| TC-2 | `hour` -> grouped by hour of day | Parsing |
| TC-3 | `model` -> grouped by model name | Parsing |
| TC-4 | `command` -> grouped by CLR command | Parsing |
| TC-5 | `error` -> grouped by error class | Parsing |
| TC-6 | `creds`, `dir` -> grouped by credential/directory | Parsing |
| TC-7 | Case-insensitive matching (`MODEL` = `model`) | Case Insensitivity |
| TC-8 | Invalid variant -> exit 1 listing valid options | Error Handling |

## Test Coverage Summary

- Parsing: 6 tests (TC-1 through TC-6)
- Case Insensitivity: 1 test (TC-7)
- Error Handling: 1 test (TC-8)

**Total:** 8 test cases

## Test Cases

---

### TC-1: `day` -> grouped by calendar date

- **Given:** journal with events spread across multiple days
- **When:** `clj .stats by::day`
- **Then:** exit 0; one row per `YYYY-MM-DD` date, each with count/OK/fail/cost/tokens
- **Exit:** 0
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

---

### TC-2: `hour` -> grouped by hour of day

- **Given:** journal with events spread across multiple hours
- **When:** `clj .stats by::hour`
- **Then:** exit 0; one row per hour (00-23), each with count/OK/fail/avg duration
- **Exit:** 0
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

---

### TC-3: `model` -> grouped by model name

- **Given:** journal with events across multiple models
- **When:** `clj .stats by::model`
- **Then:** exit 0; one row per model, each with count/cost/tokens in/out
- **Exit:** 0
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

---

### TC-4: `command` -> grouped by CLR command

- **Given:** journal with events across multiple commands (run/ask/isolated)
- **When:** `clj .stats by::command`
- **Then:** exit 0; one row per command, each with count/OK/fail/avg duration
- **Exit:** 0
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

---

### TC-5: `error` -> grouped by error class

- **Given:** journal with failed events of varying error classes
- **When:** `clj .stats by::error`
- **Then:** exit 0; one row per error class, each with count/retries/last seen
- **Exit:** 0
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

---

### TC-6: `creds`, `dir` -> grouped by credential/directory

- **Given:** journal with events across multiple credential files and working directories
- **When:** `clj .stats by::creds` and `clj .stats by::dir`
- **Then:** exit 0 for both; one row per credential name (for `creds`) or working directory (for `dir`)
- **Exit:** 0 for both
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

---

### TC-7: Case-insensitive matching (`MODEL` = `model`)

- **Given:** journal with events across multiple models
- **When:** `clj .stats by::MODEL` compared to `clj .stats by::model`
- **Then:** both produce identical grouped output
- **Exit:** 0 for both
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)

---

### TC-8: Invalid variant -> exit 1 listing valid options

- **Given:** clean environment
- **When:** `clj .stats by::region`
- **Then:** exit 1; stderr lists the 7 valid `GroupBy` variants
- **Exit:** 1
- **Source:** [type/09_group_by.md](../../../../docs/cli/type/09_group_by.md)
