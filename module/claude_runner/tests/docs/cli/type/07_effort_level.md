# Type :: `EffortLevel`

Validation tests for the `EffortLevel` semantic type (enum: low/medium/high/max). Tests validate all four valid variants, unknown variant rejection, missing-value handling, and default injection.

**Source:** [type.md](../../../../docs/cli/type.md#type--7-effortlevel)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `low` → accepted | Valid Variant |
| TC-2 | `medium` → accepted | Valid Variant |
| TC-3 | `high` → accepted | Valid Variant |
| TC-4 | `max` → accepted | Valid Variant |
| TC-5 | Unknown value → exit 1 with valid values listed | Invalid Variant |
| TC-6 | `--effort` without value → exit 1 | Missing Value |

## Test Coverage Summary

- Valid Variant: 4 tests (TC-1, TC-2, TC-3, TC-4)
- Invalid Variant: 1 test (TC-5)
- Missing Value: 1 test (TC-6)

**Total:** 6 test cases

## Test Cases

---

### TC-1: `low` → accepted

- **Given:** clean environment
- **When:** `clr --dry-run --effort low "Fix bug"`
- **Then:** Exit 0; assembled command contains `--effort low`
- **Exit:** 0
- **Source:** [type.md — EffortLevel](../../../../docs/cli/type.md#type--7-effortlevel)

---

### TC-2: `medium` → accepted

- **Given:** clean environment
- **When:** `clr --dry-run --effort medium "Fix bug"`
- **Then:** Exit 0; assembled command contains `--effort medium`
- **Exit:** 0
- **Source:** [type.md — EffortLevel](../../../../docs/cli/type.md#type--7-effortlevel)

---

### TC-3: `high` → accepted

- **Given:** clean environment
- **When:** `clr --dry-run --effort high "Fix bug"`
- **Then:** Exit 0; assembled command contains `--effort high`
- **Exit:** 0
- **Source:** [type.md — EffortLevel](../../../../docs/cli/type.md#type--7-effortlevel)

---

### TC-4: `max` → accepted

- **Given:** clean environment
- **When:** `clr --dry-run --effort max "Fix bug"`
- **Then:** Exit 0; assembled command contains `--effort max` (explicit, same as default)
- **Exit:** 0
- **Source:** [type.md — EffortLevel](../../../../docs/cli/type.md#type--7-effortlevel)

---

### TC-5: Unknown value → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --effort ultra`
- **Then:** Exit 1; error: `"unknown effort level: 'ultra' — valid values: low, medium, high, max"`
- **Exit:** 1
- **Source:** [type.md — EffortLevel](../../../../docs/cli/type.md#type--7-effortlevel)

---

### TC-6: `--effort` without value → exit 1

- **Given:** clean environment
- **When:** `clr --effort`
- **Then:** Exit 1; error indicating `--effort` requires a value
- **Exit:** 1
- **Source:** [type.md — EffortLevel](../../../../docs/cli/type.md#type--7-effortlevel)
