# Type :: `TimeoutSecs`

Validation tests for the `TimeoutSecs` semantic type (u64: non-negative integer seconds). Tests validate boundary values, invalid inputs, and default behavior.

**Source:** [type.md](../../../../docs/cli/type.md#type--9-timeoutsecs)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | `0` → accepted, immediate expiry | Valid Boundary |
| TC-2 | `30` → accepted (default) | Valid |
| TC-3 | `3600` → accepted | Valid |
| TC-4 | `-1` → exit 1, negative rejected | Invalid |
| TC-5 | `abc` → exit 1, non-numeric rejected | Invalid |
| TC-6 | `--timeout` without value → exit 1, requires argument | Missing Value |

## Test Coverage Summary

- Valid Boundary: 1 test (TC-1)
- Valid: 2 tests (TC-2, TC-3)
- Invalid: 2 tests (TC-4, TC-5)
- Missing Value: 1 test (TC-6)

**Total:** 6 test cases

## Test Cases

---

### TC-1: `0` → accepted, immediate expiry

- **Given:** credentials JSON at `/tmp/tc1_ts.json`
- **When:** `clr isolated --creds /tmp/tc1_ts.json --timeout 0 "test"`
- **Then:** no parse error for `0`; subprocess attempted; immediately timed out; exit 0 if creds refreshed, exit 2 otherwise
- **Exit:** 0 or 2
- **Source:** [type.md — TimeoutSecs](../../../../docs/cli/type.md#type--9-timeoutsecs)

---

### TC-2: `30` → accepted (default)

- **Given:** credentials JSON at `/tmp/tc2_ts.json`
- **When:** `clr isolated --creds /tmp/tc2_ts.json --timeout 30 "test"`
- **Then:** no parse error; subprocess runs with 30-second window; same behavior as omitting `--timeout`
- **Exit:** 0 or passthrough
- **Source:** [type.md — TimeoutSecs](../../../../docs/cli/type.md#type--9-timeoutsecs)

---

### TC-3: `3600` → accepted

- **Given:** credentials JSON at `/tmp/tc3_ts.json`
- **When:** `clr isolated --creds /tmp/tc3_ts.json --timeout 3600 "test"`
- **Then:** no parse error; subprocess runs with 1-hour window
- **Exit:** 0 or passthrough
- **Source:** [type.md — TimeoutSecs](../../../../docs/cli/type.md#type--9-timeoutsecs)

---

### TC-4: `-1` → exit 1, negative rejected

- **Given:** credentials JSON at `/tmp/tc4_ts.json`
- **When:** `clr isolated --creds /tmp/tc4_ts.json --timeout -1 "test"`
- **Then:** exit 1; stderr contains `"invalid --timeout value"` and mentions non-negative integer; no subprocess launched
- **Exit:** 1
- **Source:** [type.md — TimeoutSecs (Validation errors)](../../../../docs/cli/type.md#type--9-timeoutsecs)

---

### TC-5: `abc` → exit 1, non-numeric rejected

- **Given:** credentials JSON at `/tmp/tc5_ts.json`
- **When:** `clr isolated --creds /tmp/tc5_ts.json --timeout abc "test"`
- **Then:** exit 1; stderr contains `"invalid --timeout value"` and mentions non-negative integer; no subprocess launched
- **Exit:** 1
- **Source:** [type.md — TimeoutSecs (Validation errors)](../../../../docs/cli/type.md#type--9-timeoutsecs)

---

### TC-6: `--timeout` without value → exit 1, requires argument

- **Given:** credentials JSON at `/tmp/tc6_ts.json`
- **When:** `clr isolated --creds /tmp/tc6_ts.json --timeout`
- **Then:** exit 1; error indicating `--timeout` requires a value
- **Exit:** 1
- **Source:** [type.md — TimeoutSecs](../../../../docs/cli/type.md#type--9-timeoutsecs)
