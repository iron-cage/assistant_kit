# Type :: `VerbosityLevel`

Validation tests for the `VerbosityLevel` semantic type (u8, 0–5, default 3). Tests validate range enforcement, error messages, and default behavior.

**Source:** [type.md](../../../../docs/cli/type.md#type--5-verbositylevel)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `0` → accepted (minimum, silent) | Boundary |
| TC-2 | `5` → accepted (maximum, debug) | Boundary |
| TC-3 | `6` → exit 1 (out of range) | Boundary |
| TC-4 | Non-integer value → exit 1 | Type Validation |
| TC-5 | Default level 3 applied when unset | Default |

## Test Coverage Summary

- Boundary: 3 tests (TC-1, TC-2, TC-3)
- Type Validation: 1 test (TC-4)
- Default: 1 test (TC-5)

**Total:** 5 test cases

## Test Cases

---

### TC-1: `0` → accepted (silent)

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 0 "Fix bug"`
- **Then:** Exit 0; runner suppresses diagnostic output (dry-run preview still shown)
- **Exit:** 0
- **Source:** [type.md — VerbosityLevel](../../../../docs/cli/type.md#type--5-verbositylevel)

---

### TC-2: `5` → accepted (debug)

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 5 "Fix bug"`
- **Then:** Exit 0; maximum diagnostic output; no error
- **Exit:** 0
- **Source:** [type.md — VerbosityLevel](../../../../docs/cli/type.md#type--5-verbositylevel)

---

### TC-3: `6` → exit 1 (out of range)

- **Given:** clean environment
- **When:** `clr "Fix bug" --verbosity 6`
- **Then:** Exit 1; error: `"verbosity level out of range: 6\nExpected 0–5"`
- **Exit:** 1
- **Source:** [type.md — VerbosityLevel](../../../../docs/cli/type.md#type--5-verbositylevel)

---

### TC-4: Non-integer value → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --verbosity high`
- **Then:** Exit 1; error: `"invalid verbosity level: high\nExpected integer 0–5"`
- **Exit:** 1
- **Source:** [type.md — VerbosityLevel](../../../../docs/cli/type.md#type--5-verbositylevel)

---

### TC-5: Default level 3 applied when unset

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"` (no `--verbosity` given)
- **Then:** Exit 0; runner behaves at level 3 (normal); no error
- **Exit:** 0
- **Source:** [type.md — VerbosityLevel](../../../../docs/cli/type.md#type--5-verbositylevel)
