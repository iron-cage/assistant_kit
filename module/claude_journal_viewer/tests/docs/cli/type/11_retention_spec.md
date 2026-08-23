# Type :: `RetentionSpec`

Validation tests for the `RetentionSpec` semantic type. Tests validate
age-based duration parsing, the floor-to-whole-days rule, and
invalid-format error handling. A size-based mode was considered and
dropped, so size suffixes are rejected rather than parsed.

**Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `7d` -> deletes files dated more than 7 days ago | Age-Based |
| TC-2 | `4w` -> week suffix resolves to 28 days | Age-Based |
| TC-3 | `12h` -> floors to 0 days, only today's file survives | Boundary |
| TC-4 | `100mb` -> exit 1, size suffix rejected | Error Handling |
| TC-5 | `forever` -> exit 1, expected-format error | Error Handling |

## Test Coverage Summary

- Age-Based: 2 tests (TC-1, TC-2)
- Boundary: 1 test (TC-3)
- Error Handling: 2 tests (TC-4, TC-5)

**Total:** 5 test cases

## Test Cases

---

### TC-1: `7d` -> deletes files dated more than 7 days ago

- **Given:** journal directory with `YYYY-MM-DD.jsonl` files both older and newer than 7 days
- **When:** `clj .prune keep::7d`
- **Then:** exit 0; files whose *filename* date is strictly before `today - 7d` (UTC) are deleted; newer files remain. Filesystem mtime is never consulted, and files not matching `YYYY-MM-DD.jsonl` are ignored entirely
- **Exit:** 0
- **Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md) — Behavior

---

### TC-2: `4w` -> week suffix resolves to 28 days

- **Given:** journal directory with files dated 27 and 29 days ago
- **When:** `clj .prune keep::4w`
- **Then:** exit 0; the 29-day-old file is deleted and the 27-day-old file remains — `4w` is exactly 28 days, not a calendar month
- **Exit:** 0
- **Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md) — Format table

---

### TC-3: `12h` -> floors to 0 days, only today's file survives

- **Given:** journal directory with today's file plus files dated earlier
- **When:** `clj .prune keep::12h`
- **Then:** exit 0; the sub-day duration floors to 0 whole days, so every file dated strictly before today is deleted; today's file is structurally never a candidate and survives
- **Exit:** 0
- **Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md) — Format table, Behavior

---

### TC-4: `100mb` -> exit 1, size suffix rejected

- **Given:** clean environment
- **When:** `clj .prune keep::100mb`
- **Then:** exit 1; stderr contains `Error: invalid duration '100mb' (expected e.g. 30s, 5m, 1h, 7d, 2w)`. `b` is not a recognised unit suffix, so size-based retention is unreachable by construction — use `.status` to monitor journal size
- **Exit:** 1
- **Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md) — Validation

---

### TC-5: `forever` -> exit 1, expected-format error

- **Given:** clean environment
- **When:** `clj .prune keep::forever`
- **Then:** exit 1; stderr contains `Error: invalid duration 'forever' (expected e.g. 30s, 5m, 1h, 7d, 2w)` — `r` is not a valid unit suffix and `foreve` is not a number
- **Exit:** 1
- **Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md) — Validation
