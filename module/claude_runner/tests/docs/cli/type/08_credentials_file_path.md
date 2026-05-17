# Type :: `CredentialsFilePath`

Validation tests for the `CredentialsFilePath` semantic type (String: path to an existing credentials JSON file). Tests validate path acceptance, non-existence rejection, and in-place write-back behavior.

**Source:** [type.md](../../../../docs/cli/type.md#type--8-credentialsfilepath)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Absolute path to existing file → accepted | Valid |
| TC-2 | Relative path to existing file → accepted (resolved from cwd) | Valid |
| TC-3 | Path to non-existent file → exit 1 | Invalid |
| TC-4 | Path points to directory → exit 1 | Invalid |
| TC-5 | File not readable (permission denied) → exit 1 | Invalid |
| TC-6 | OAuth refresh during run → file updated in-place on exit | Write-back |

## Test Coverage Summary

- Valid: 2 tests (TC-1, TC-2)
- Invalid: 3 tests (TC-3, TC-4, TC-5)
- Write-back: 1 test (TC-6)

**Total:** 6 test cases

## Test Cases

---

### TC-1: Absolute path to existing file → accepted

- **Given:** valid JSON at `/tmp/tc1_cfp.json`
- **When:** `clr isolated --creds /tmp/tc1_cfp.json "test"`
- **Then:** file found; isolated HOME seeded; subprocess launches; no error
- **Exit:** 0 or passthrough
- **Source:** [type.md — CredentialsFilePath](../../../../docs/cli/type.md#type--8-credentialsfilepath)

---

### TC-2: Relative path to existing file → accepted (resolved from cwd)

- **Given:** `tc2_cfp.json` in caller's cwd
- **When:** `clr isolated --creds tc2_cfp.json "test"` (run from that directory)
- **Then:** file located via cwd, not via isolated temp HOME; subprocess launches; no error
- **Exit:** 0 or passthrough
- **Source:** [type.md — CredentialsFilePath](../../../../docs/cli/type.md#type--8-credentialsfilepath)

---

### TC-3: Path to non-existent file → exit 1

- **Given:** `/tmp/tc3_cfp_missing.json` does not exist
- **When:** `clr isolated --creds /tmp/tc3_cfp_missing.json "test"`
- **Then:** exit 1; stderr contains "not found" error; no subprocess launched
- **Exit:** 1
- **Source:** [type.md — CredentialsFilePath (Constraints: file must exist)](../../../../docs/cli/type.md#type--8-credentialsfilepath)

---

### TC-4: Path points to directory → exit 1

- **Given:** `/tmp/tc4_cfp_dir/` exists as a directory
- **When:** `clr isolated --creds /tmp/tc4_cfp_dir "test"`
- **Then:** exit 1; stderr contains error (not readable or not a file); no subprocess launched
- **Exit:** 1
- **Source:** [type.md — CredentialsFilePath](../../../../docs/cli/type.md#type--8-credentialsfilepath)

---

### TC-5: File not readable (permission denied) → exit 1

- **Given:** `/tmp/tc5_cfp_noperm.json` exists with mode 000
- **When:** `clr isolated --creds /tmp/tc5_cfp_noperm.json "test"`
- **Then:** exit 1; stderr indicates read/permission error; no subprocess launched
- **Exit:** 1
- **Source:** [type.md — CredentialsFilePath (Constraints: file must be readable)](../../../../docs/cli/type.md#type--8-credentialsfilepath)

---

### TC-6: OAuth refresh during run → file updated in-place on exit

- **Given:** expired-token credentials at `/tmp/tc6_cfp.json`; `claude` performs OAuth refresh at subprocess startup
- **When:** `clr isolated --creds /tmp/tc6_cfp.json --timeout 0`
- **Then:** updated token written back to `/tmp/tc6_cfp.json`; exit 0
- **Exit:** 0
- **Source:** [type.md — CredentialsFilePath (Write-back)](../../../../docs/cli/type.md#type--8-credentialsfilepath)
