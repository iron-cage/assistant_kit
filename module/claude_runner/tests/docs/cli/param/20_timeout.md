# Parameter :: `--timeout`

Edge case test planning for the `--timeout` parameter. See [20_timeout.md](../../../../docs/cli/param/20_timeout.md) for specification.

**Source:** [param/20_timeout.md](../../../../docs/cli/param/20_timeout.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--timeout 30` → accepted (default value) | Valid |
| EC-2 | `--timeout 0` → immediate expiry | Valid Edge |
| EC-3 | `--timeout 3600` → large value accepted | Valid |
| EC-4 | `--timeout -1` → exit 1, negative not accepted | Invalid |
| EC-5 | `--timeout abc` → exit 1, non-numeric rejected | Invalid |
| EC-6 | `--timeout` without value → exit 1, requires argument | Missing Value |

## Test Coverage Summary

- Valid: 3 tests (EC-1, EC-2, EC-3)
- Invalid: 2 tests (EC-4, EC-5)
- Missing Value: 1 test (EC-6)

**Total:** 6 test cases

---

### EC-1: `--timeout 30` → accepted (default value)

- **Given:** credentials JSON at `/tmp/ec1_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec1_to_creds.json --timeout 30 "test"`
- **Then:** no parse error; subprocess runs with 30-second window (same as omitting `--timeout`)
- **Exit:** 0 or passthrough
- **Source:** [20_timeout.md](../../../../docs/cli/param/20_timeout.md)

---

### EC-2: `--timeout 0` → immediate expiry

- **Given:** credentials JSON at `/tmp/ec2_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec2_to_creds.json --timeout 0 "test"`
- **Then:** no parse error; subprocess attempted then immediately timed out; exit 2 if no creds refresh, exit 0 if creds refreshed
- **Exit:** 0 or 2
- **Source:** [20_timeout.md (Note: timeout 0 causes immediate expiry)](../../../../docs/cli/param/20_timeout.md)

---

### EC-3: `--timeout 3600` → large value accepted

- **Given:** credentials JSON at `/tmp/ec3_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec3_to_creds.json --timeout 3600 "test"`
- **Then:** no parse error; subprocess runs with 1-hour window
- **Exit:** 0 or passthrough
- **Source:** [20_timeout.md](../../../../docs/cli/param/20_timeout.md)

---

### EC-4: `--timeout -1` → exit 1, negative rejected

- **Given:** credentials JSON at `/tmp/ec4_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec4_to_creds.json --timeout -1 "test"`
- **Then:** exit 1; stderr contains invalid `--timeout` error; no subprocess launched
- **Exit:** 1
- **Source:** [type.md — TimeoutSecs validation](../../../../docs/cli/type.md#type--9-timeoutsecs)

---

### EC-5: `--timeout abc` → exit 1, non-numeric rejected

- **Given:** credentials JSON at `/tmp/ec5_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec5_to_creds.json --timeout abc "test"`
- **Then:** exit 1; stderr contains invalid `--timeout` error; no subprocess launched
- **Exit:** 1
- **Source:** [type.md — TimeoutSecs validation](../../../../docs/cli/type.md#type--9-timeoutsecs)

---

### EC-6: `--timeout` without value → exit 1, requires argument

- **Given:** credentials JSON at `/tmp/ec6_to_creds.json`
- **When:** `clr isolated --creds /tmp/ec6_to_creds.json --timeout`
- **Then:** exit 1; stderr indicates `--timeout` requires a value
- **Exit:** 1
- **Source:** [20_timeout.md](../../../../docs/cli/param/20_timeout.md)
