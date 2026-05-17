# Test: `isolated`

Integration test planning for the `isolated` command. See [command.md](../../../../docs/cli/command.md#command--2-isolated) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `--creds file.json "msg"` → runs with isolated HOME, exit 0 | Happy Path |
| IT-2 | `--creds missing.json` → exit 1, file-not-found error | Error: Missing Creds |
| IT-3 | `--creds file.json --timeout 0 "msg"` → exit 2 (timeout, no creds refresh) | Timeout |
| IT-4 | `--creds file.json --timeout 0` → creds refreshed → exit 0, creds updated | Timeout with Refresh |
| IT-5 | `--creds file.json` (no message) → interactive REPL mode in isolation | Interactive |
| IT-6 | `--creds file.json -- --version` → passes `--version` through to claude | Flag Passthrough |
| IT-7 | `--creds file.json --timeout abc` → exit 1, invalid timeout error | Error: Invalid Timeout |
| IT-8 | Missing `--creds` → exit 1, missing required argument error | Error: Missing Creds Flag |
| IT-9 | `clr isolated --help` → exit 0, prints isolated-specific help | Help |

## Test Coverage Summary

- Happy Path: 1 test (IT-1)
- Error Handling: 3 tests (IT-2, IT-7, IT-8)
- Timeout Behavior: 2 tests (IT-3, IT-4)
- Mode Selection: 2 tests (IT-5, IT-6)
- Help: 1 test (IT-9)

**Total:** 9 test cases

---

### IT-1: `--creds file.json "msg"` → runs in isolated HOME, exit 0

- **Given:** valid credentials JSON at `/tmp/it1_creds.json`; `claude` binary in PATH
- **When:** `clr isolated --creds /tmp/it1_creds.json "What is 2+2?"`
- **Then:** subprocess runs with `HOME=<temp>`; temp HOME contains only `.claude/.credentials.json`; claude produces output; exit 0
- **Exit:** 0
- **Source:** [command.md — isolated](../../../../docs/cli/command.md#command--2-isolated)

---

### IT-2: `--creds missing.json` → exit 1, file-not-found error

- **Given:** `/tmp/it2_missing.json` does not exist
- **When:** `clr isolated --creds /tmp/it2_missing.json "test"`
- **Then:** exit 1; stderr contains "not found" or equivalent; no subprocess launched
- **Exit:** 1
- **Source:** [command.md — isolated](../../../../docs/cli/command.md#command--2-isolated)

---

### IT-3: `--timeout 0 "msg"` → exit 2 (timeout, no creds refresh)

- **Given:** valid credentials JSON at `/tmp/it3_creds.json`; subprocess does not refresh creds before blocking
- **When:** `clr isolated --creds /tmp/it3_creds.json --timeout 0 "Long running task"`
- **Then:** subprocess attempted; wait window expires immediately; creds not refreshed → exit 2
- **Exit:** 2
- **Source:** [command.md — isolated](../../../../docs/cli/command.md#command--2-isolated), [--timeout](../../../../docs/cli/param/20_timeout.md)

---

### IT-4: `--timeout 0` with creds refresh → exit 0, creds updated in-place

- **Given:** expired-token credentials at `/tmp/it4_creds.json`; `claude` performs OAuth refresh at startup before blocking on REPL
- **When:** `clr isolated --creds /tmp/it4_creds.json --timeout 0`
- **Then:** subprocess refreshes token before blocking; `clr isolated` detects refresh → exit 0; `/tmp/it4_creds.json` contains updated token
- **Exit:** 0
- **Source:** [command.md — isolated (Notes)](../../../../docs/cli/command.md#command--2-isolated), [--timeout](../../../../docs/cli/param/20_timeout.md)

---

### IT-5: `--creds file.json` (no message) → interactive REPL in isolation

- **Given:** valid credentials JSON at `/tmp/it5_creds.json`; TTY available
- **When:** `clr isolated --creds /tmp/it5_creds.json` (no message)
- **Then:** Claude starts in interactive REPL mode inside isolated HOME; stdin/stdout connected to subprocess; no `--print` injected
- **Exit:** 0 (when REPL exits)
- **Source:** [command.md — isolated](../../../../docs/cli/command.md#command--2-isolated)

---

### IT-6: `--creds file.json -- --version` → `--version` passed through to claude

- **Given:** valid credentials JSON at `/tmp/it6_creds.json`; `claude --version` exits 0
- **When:** `clr isolated --creds /tmp/it6_creds.json -- --version`
- **Then:** subprocess receives `--version` flag; version string printed to stdout; exit 0
- **Exit:** 0
- **Source:** [command.md — isolated](../../../../docs/cli/command.md#command--2-isolated)

---

### IT-7: `--timeout abc` → exit 1, invalid timeout error

- **Given:** valid credentials JSON at `/tmp/it7_creds.json`
- **When:** `clr isolated --creds /tmp/it7_creds.json --timeout abc "test"`
- **Then:** exit 1; stderr contains invalid `--timeout` error; no subprocess launched
- **Exit:** 1
- **Source:** [type.md — TimeoutSecs validation](../../../../docs/cli/type.md#type--9-timeoutsecs)

---

### IT-8: Missing `--creds` → exit 1, missing required argument

- **Given:** clean environment
- **When:** `clr isolated "test"`
- **Then:** exit 1; stderr contains "missing" or "required" message referencing `--creds`; no subprocess launched
- **Exit:** 1
- **Source:** [command.md — isolated](../../../../docs/cli/command.md#command--2-isolated), [--creds](../../../../docs/cli/param/19_creds.md)

---

### IT-9: `clr isolated --help` → exit 0, prints isolated-specific help

- **Given:** clean environment
- **When:** `clr isolated --help` (also: `clr isolated -h`)
- **Then:** exit 0; stdout contains `--creds`, `--timeout`, and `--help`; no subprocess launched; no error in stderr
- **Exit:** 0
- **Source:** [command.md — isolated](../../../../docs/cli/command.md#command--2-isolated)
