# Type :: `RetentionSpec`

Validation tests for the `RetentionSpec` semantic type. Tests validate
age-based and size-based formats, case-insensitive size suffixes, and
invalid-format error handling.

**Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `7d` (age-based) -> deletes files older than 7 days | Age-Based |
| TC-2 | `100mb` (size-based) -> deletes oldest until under threshold | Size-Based |
| TC-3 | `100MB` = `100mb` -> case-insensitive size suffix | Case Insensitivity |
| TC-4 | Invalid format -> exit 1, expected-format error | Error Handling |

## Test Coverage Summary

- Age-Based: 1 test (TC-1)
- Size-Based: 1 test (TC-2)
- Case Insensitivity: 1 test (TC-3)
- Error Handling: 1 test (TC-4)

**Total:** 4 test cases

## Test Cases

---

### TC-1: `7d` (age-based) -> deletes files older than 7 days

- **Given:** journal directory with files both older and newer than 7 days
- **When:** `clj .prune keep::7d confirm::1`
- **Then:** exit 0; files with a date older than 7 days in their filename are deleted; newer files remain
- **Exit:** 0
- **Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)

---

### TC-2: `100mb` (size-based) -> deletes oldest until under threshold

- **Given:** journal directory totaling more than 100MB
- **When:** `clj .prune keep::100mb confirm::1`
- **Then:** exit 0; oldest files are deleted first until total directory size is under 100MB
- **Exit:** 0
- **Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)

---

### TC-3: `100MB` = `100mb` -> case-insensitive size suffix

- **Given:** journal directory totaling more than 100MB
- **When:** `clj .prune keep::100MB dry_run::1` compared to `clj .prune keep::100mb dry_run::1`
- **Then:** both produce an identical candidate-deletion list
- **Exit:** 0 for both
- **Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)

---

### TC-4: Invalid format -> exit 1, expected-format error

- **Given:** clean environment
- **When:** `clj .prune keep::forever`
- **Then:** exit 1; stderr contains `expected duration (7d, 4w) or size (100mb, 1gb)`
- **Exit:** 1
- **Source:** [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)
