# Parameter :: `--creds`

Edge case test planning for the `--creds` parameter. See [19_creds.md](../../../../docs/cli/param/19_creds.md) for specification.

**Source:** [param/19_creds.md](../../../../docs/cli/param/19_creds.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Valid file path → accepted, subprocess runs | Valid |
| EC-2 | Absolute path → resolved correctly | Valid |
| EC-3 | Relative path → resolved against caller's cwd | Valid |
| EC-4 | File does not exist → exit 1 with file-not-found error | Invalid |
| EC-5 | `--creds` without value → exit 1, argument requires value | Missing Value |
| EC-6 | `--creds` omitted entirely → exit 1, missing required argument | Required |

## Test Coverage Summary

- Valid: 3 tests (EC-1, EC-2, EC-3)
- Invalid: 1 test (EC-4)
- Missing Value: 1 test (EC-5)
- Required Enforcement: 1 test (EC-6)

**Total:** 6 test cases

---

### EC-1: Valid file path → accepted, subprocess runs

- **Given:** credentials JSON at `/tmp/ec1_creds.json`
- **When:** `clr isolated --creds /tmp/ec1_creds.json "test"`
- **Then:** file found; subprocess launched with isolated HOME; no error on startup
- **Exit:** 0 or passthrough from subprocess
- **Source:** [19_creds.md](../../../../docs/cli/param/19_creds.md)

---

### EC-2: Absolute path → resolved correctly

- **Given:** credentials JSON at absolute path `/tmp/ec2_creds.json`
- **When:** `clr isolated --creds /tmp/ec2_creds.json "test"`
- **Then:** file located via absolute path; no "not found" error; subprocess launches
- **Exit:** 0 or passthrough
- **Source:** [19_creds.md](../../../../docs/cli/param/19_creds.md)

---

### EC-3: Relative path → resolved against caller's cwd

- **Given:** credentials file `ec3_creds.json` in caller's cwd
- **When:** `clr isolated --creds ec3_creds.json "test"` (run from that directory)
- **Then:** file found via cwd resolution, not via isolated temp HOME; subprocess launches
- **Exit:** 0 or passthrough
- **Source:** [19_creds.md (Note: path resolved against caller's cwd)](../../../../docs/cli/param/19_creds.md)

---

### EC-4: File does not exist → exit 1 with file-not-found error

- **Given:** `/tmp/ec4_nonexistent.json` does not exist
- **When:** `clr isolated --creds /tmp/ec4_nonexistent.json "test"`
- **Then:** exit 1; stderr contains file-not-found error; no subprocess launched
- **Exit:** 1
- **Source:** [19_creds.md](../../../../docs/cli/param/19_creds.md)

---

### EC-5: `--creds` without value → exit 1, argument requires value

- **Given:** clean environment
- **When:** `clr isolated --creds`
- **Then:** exit 1; stderr says `--creds` requires a value
- **Exit:** 1
- **Source:** [19_creds.md](../../../../docs/cli/param/19_creds.md)

---

### EC-6: `--creds` omitted entirely → exit 1, missing required argument

- **Given:** clean environment
- **When:** `clr isolated "test"`
- **Then:** exit 1; stderr indicates `--creds` is required for `isolated`
- **Exit:** 1
- **Source:** [19_creds.md](../../../../docs/cli/param/19_creds.md)
