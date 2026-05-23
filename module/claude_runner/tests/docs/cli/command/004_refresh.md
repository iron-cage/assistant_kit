# Test: `refresh`

Integration test planning for the `refresh` command. See [001_command.md](../../../../docs/cli/001_command.md#command--3-refresh) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `--creds file.json` → refresh succeeds, exit 0, creds updated in-place | Happy Path |
| IT-2 | `--creds missing.json` → exit 1, file-not-found error | Error: Missing Creds |
| IT-3 | `--creds file.json --timeout 90` → explicit timeout applied | Custom Timeout |
| IT-4 | `--creds file.json --timeout 0` → immediate expiry, exit 2 or 0 (if refresh at startup) | Timeout |
| IT-5 | Missing `--creds` → exit 1, missing required argument error | Error: Missing Creds Flag |
| IT-6 | `--creds file.json --timeout abc` → exit 1, invalid timeout error | Error: Invalid Timeout |
| IT-7 | `--creds file.json --trace` → call details printed to stderr before execution | Trace |
| IT-8 | `clr refresh --help` → exit 0, prints refresh-specific help | Help |

## Test Coverage Summary

- Happy Path: 1 test (IT-1)
- Error Handling: 3 tests (IT-2, IT-5, IT-6)
- Timeout Behavior: 2 tests (IT-3, IT-4)
- Trace: 1 test (IT-7)
- Help: 1 test (IT-8)

**Total:** 8 test cases

---

### IT-1: `--creds file.json` → refresh succeeds, exit 0, creds updated in-place

- **Given:** valid but expired credentials JSON at `/tmp/it1_refresh_creds.json`; `claude` binary in PATH
- **When:** `clr refresh --creds /tmp/it1_refresh_creds.json`
- **Then:** subprocess spawned with `HOME=<temp>` and `["--print", "."]`; OAuth token refreshed; `/tmp/it1_refresh_creds.json` overwritten with updated token; exit 0
- **Exit:** 0
- **Note:** lim_it — requires valid but expiring credentials
- **Source:** [001_command.md — refresh](../../../../docs/cli/001_command.md#command--3-refresh)

---

### IT-2: `--creds missing.json` → exit 1, file-not-found error

- **Given:** `/tmp/it2_refresh_missing.json` does not exist
- **When:** `clr refresh --creds /tmp/it2_refresh_missing.json`
- **Then:** exit 1; stderr contains "not found" or equivalent; no subprocess launched
- **Exit:** 1
- **Source:** [001_command.md — refresh](../../../../docs/cli/001_command.md#command--3-refresh)

---

### IT-3: `--creds file.json --timeout 90` → explicit timeout applied

- **Given:** valid credentials JSON at `/tmp/it3_refresh_creds.json`
- **When:** `clr refresh --creds /tmp/it3_refresh_creds.json --timeout 90`
- **Then:** subprocess launched with 90-second deadline (not the 45-second default); behavior identical to an explicit `--timeout 90`
- **Exit:** 0 or 1
- **Note:** lim_it — requires valid credentials; validates explicit timeout overrides default
- **Source:** [001_command.md — refresh](../../../../docs/cli/001_command.md#command--3-refresh), [--timeout](../../../../docs/cli/param/020_timeout.md)

---

### IT-4: `--timeout 0` → immediate expiry, exit 2 or 0 if refresh at startup

- **Given:** valid credentials JSON at `/tmp/it4_refresh_creds.json`
- **When:** `clr refresh --creds /tmp/it4_refresh_creds.json --timeout 0`
- **Then:** subprocess killed immediately (0-second deadline); exit 2 when no token refresh occurred; exit 0 and creds written back when OAuth refresh completed at subprocess startup before the kill
- **Exit:** 2 (or 0 if creds refreshed at startup)
- **Note:** lim_it — fast-path credential check; matches `isolated --timeout 0` semantics
- **Source:** [001_command.md — refresh](../../../../docs/cli/001_command.md#command--3-refresh)

---

### IT-5: Missing `--creds` → exit 1, missing required argument

- **Given:** clean environment
- **When:** `clr refresh`
- **Then:** exit 1; stderr contains "missing" or "required" message referencing `--creds`; no subprocess launched
- **Exit:** 1
- **Source:** [001_command.md — refresh](../../../../docs/cli/001_command.md#command--3-refresh), [--creds](../../../../docs/cli/param/019_creds.md)

---

### IT-6: `--timeout abc` → exit 1, invalid timeout error

- **Given:** valid credentials JSON at `/tmp/it6_refresh_creds.json`
- **When:** `clr refresh --creds /tmp/it6_refresh_creds.json --timeout abc`
- **Then:** exit 1; stderr contains invalid `--timeout` error; no subprocess launched
- **Exit:** 1
- **Source:** [005_type.md — TimeoutSecs validation](../../../../docs/cli/005_type.md#type--9-timeoutsecs)

---

### IT-7: `--trace` → call details printed to stderr before execution

- **Given:** valid credentials JSON at `/tmp/it7_refresh_creds.json`
- **When:** `clr refresh --creds /tmp/it7_refresh_creds.json --trace`
- **Then:** stderr contains the credentials path, temp HOME path, timeout value (45), and forwarded args (`["--print", "."]`) before subprocess launch
- **Exit:** 0 or 1
- **Note:** lim_it — requires claude binary accessible; verifies `--trace` honoured for `refresh` command specifically
- **Source:** [001_command.md — refresh](../../../../docs/cli/001_command.md#command--3-refresh), [--trace](../../../../docs/cli/param/013_trace.md)

---

### IT-8: `clr refresh --help` → exit 0, prints refresh-specific help

- **Given:** clean environment
- **When:** `clr refresh --help` (also: `clr refresh -h`)
- **Then:** exit 0; stdout contains `--creds`, `--timeout`, `--trace`, and `--help`; no subprocess launched; no error in stderr
- **Exit:** 0
- **Source:** [001_command.md — refresh](../../../../docs/cli/001_command.md#command--3-refresh)
