# Type :: `ModelName`

Validation tests for the `ModelName` semantic type (any non-empty string). Tests validate pass-through and missing-value handling.

**Source:** [type.md](../../../../docs/cli/type.md#type--4-modelname)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Valid model name → forwarded to claude | Valid Input |
| TC-2 | Model name with hyphens → accepted | Valid Input |
| TC-3 | `--model` without value → exit 1 | Missing Value |
| TC-4 | `--model` absent → claude uses its own default | Default |

## Test Coverage Summary

- Valid Input: 2 tests (TC-1, TC-2)
- Missing Value: 1 test (TC-3)
- Default: 1 test (TC-4)

**Total:** 4 test cases

## Test Cases

---

### TC-1: Valid model name → forwarded

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug" --model sonnet`
- **Then:** Assembled command contains `--model sonnet`; value forwarded verbatim
- **Exit:** 0
- **Source:** [type.md — ModelName](../../../../docs/cli/type.md#type--4-modelname)

---

### TC-2: Model name with hyphens → accepted

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug" --model claude-sonnet-4-6`
- **Then:** Exit 0; hyphenated model name accepted and forwarded intact
- **Exit:** 0
- **Source:** [type.md — ModelName](../../../../docs/cli/type.md#type--4-modelname)

---

### TC-3: `--model` without value → exit 1

- **Given:** clean environment
- **When:** `clr --model`
- **Then:** Exit 1; error indicating `--model` requires a value
- **Exit:** 1
- **Source:** [type.md — ModelName](../../../../docs/cli/type.md#type--4-modelname)

---

### TC-4: `--model` absent → no model flag in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--model`; claude uses its own default model
- **Exit:** 0
- **Source:** [type.md — ModelName](../../../../docs/cli/type.md#type--4-modelname)
