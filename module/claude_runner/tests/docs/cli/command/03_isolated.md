# Test: `isolated`

Integration test planning for the `isolated` command. See [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md) for specification.

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
| IT-8 | No `--creds`, `CLR_CREDS` unset → defaults to `$HOME/.claude/.credentials.json`; trace confirms | Default Fallback |
| IT-9 | `clr isolated --help` → exit 0, prints isolated-specific help | Help |
| IT-10 | `--creds <f> --trace "msg"` → call details on stderr before execution attempt | Trace |
| IT-11 | Timeout with partial stdout → exit 2, error includes accumulated output (BUG-243) | Timeout Behavior |

## Test Coverage Summary

- Happy Path: 1 test (IT-1)
- Error Handling: 2 tests (IT-2, IT-7)
- Default Fallback: 1 test (IT-8)
- Timeout Behavior: 3 tests (IT-3, IT-4, IT-11)
- Mode Selection: 2 tests (IT-5, IT-6)
- Help: 1 test (IT-9)
- Trace: 1 test (IT-10)

**Total:** 11 test cases

---

### IT-1: `--creds file.json "msg"` → runs in isolated HOME, exit 0

- **Setup:** valid credentials JSON at `/tmp/it1_creds.json`; `claude` binary in PATH
- **Command:** `clr isolated --creds /tmp/it1_creds.json "What is 2+2?"`
- **Expected behavior:** subprocess runs with `HOME=<temp>`; temp HOME contains only `.claude/.credentials.json`; claude produces output; exit 0
- **Exit:** 0
- **Source:** [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)

---

### IT-2: `--creds missing.json` → exit 1, file-not-found error

- **Setup:** `/tmp/it2_missing.json` does not exist
- **Command:** `clr isolated --creds /tmp/it2_missing.json "test"`
- **Expected behavior:** exit 1; stderr contains "not found" or equivalent; no subprocess launched
- **Exit:** 1
- **Source:** [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)

---

### IT-3: `--timeout 0 "msg"` → exit 2 (timeout, no creds refresh)

- **Setup:** valid credentials JSON at `/tmp/it3_creds.json`; subprocess does not refresh creds before blocking
- **Command:** `clr isolated --creds /tmp/it3_creds.json --timeout 0 "Long running task"`
- **Expected behavior:** subprocess attempted; wait window expires immediately; creds not refreshed → exit 2
- **Exit:** 2
- **Source:** [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md), [--timeout](../../../../docs/cli/param/020_timeout.md)

---

### IT-4: `--timeout 0` with creds refresh → exit 0, creds updated in-place

- **Setup:** expired-token credentials at `/tmp/it4_creds.json`; `claude` performs OAuth refresh at startup before blocking on REPL
- **Command:** `clr isolated --creds /tmp/it4_creds.json --timeout 0`
- **Expected behavior:** subprocess refreshes token before blocking; `clr isolated` detects refresh → exit 0; `/tmp/it4_creds.json` contains updated token
- **Exit:** 0
- **Source:** [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md), [--timeout](../../../../docs/cli/param/020_timeout.md)

---

### IT-5: `--creds file.json` (no message) → interactive REPL in isolation

- **Setup:** valid credentials JSON at `/tmp/it5_creds.json`; TTY available
- **Command:** `clr isolated --creds /tmp/it5_creds.json` (no message)
- **Expected behavior:** Claude starts in interactive REPL mode inside isolated HOME; stdin/stdout connected to subprocess; no `--print` injected
- **Exit:** 0 (when REPL exits)
- **Source:** [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)

---

### IT-6: `--creds file.json -- --version` → `--version` passed through to claude

- **Setup:** valid credentials JSON at `/tmp/it6_creds.json`; `claude --version` exits 0
- **Command:** `clr isolated --creds /tmp/it6_creds.json -- --version`
- **Expected behavior:** subprocess receives `--version` flag; version string printed to stdout; exit 0
- **Exit:** 0
- **Source:** [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)

---

### IT-7: `--timeout abc` → exit 1, invalid timeout error

- **Setup:** valid credentials JSON at `/tmp/it7_creds.json`
- **Command:** `clr isolated --creds /tmp/it7_creds.json --timeout abc "test"`
- **Expected behavior:** exit 1; stderr contains invalid `--timeout` error; no subprocess launched
- **Exit:** 1
- **Source:** [type/09_timeout_secs.md](../../../../docs/cli/type/09_timeout_secs.md)

---

### IT-8: No `--creds` → defaults to `$HOME/.claude/.credentials.json`; trace confirms path

- **Setup:** `$HOME/.claude/.credentials.json` exists (readable; content `{}`; no live credentials needed — trace fires before subprocess attempt); `CLR_CREDS` unset
- **Command:** `clr isolated --trace "test"`
- **Expected behavior:** trace stderr contains `# creds: <HOME>/.claude/.credentials.json`; subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md), [--creds](../../../../docs/cli/param/019_creds.md)

---

### IT-9: `clr isolated --help` → exit 0, prints isolated-specific help

- **Command:** `clr isolated --help` (also: `clr isolated -h`)
- **Expected behavior:** exit 0; stdout contains `--creds`, `--timeout`, and `--help`; no subprocess launched; no error in stderr
- **Exit:** 0
- **Source:** [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)

---

### IT-10: `--creds <f> --trace "msg"` → call details on stderr before execution attempt

- **Setup:** credentials JSON written to a temp file `<f>` (file is readable); claude binary absent in test environment
- **Command:** `clr isolated --creds <f> --trace "Fix bug"` (no `--dry-run`; trace fires before subprocess attempt)
- **Expected behavior:** stderr contains `# clr isolated`, `# creds: <path>`, `# timeout: 30s`, env var block (including `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`), and `claude --chrome --model claude-sonnet-4-6 --print "Fix bug"` before any subprocess attempt; subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [--trace](../../../../docs/cli/param/013_trace.md)

---

### IT-11: Timeout with partial stdout → exit 2, error includes accumulated output (BUG-243)

- **Setup:** fake-claude script that emits one line of output then blocks indefinitely (e.g. `echo "partial output"; sleep 60`); credentials JSON at `/tmp/it11_creds.json`; script injected via PATH or `CLR_CLAUDE_BIN`
- **Command:** `clr isolated --creds /tmp/it11_creds.json --timeout 1 "test"`
- **Expected behavior:** subprocess is killed after 1 second; exit 2 (timeout without credentials refresh); the partial stdout emitted before the timeout is included in the error output — diagnostic context is not discarded
- **Exit:** 2
- **Source:** [--timeout](../../../../docs/cli/param/020_timeout.md), [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)
- **Note:** Implemented in TSK-196 (BUG-243); test function `timeout_includes_partial_stdout` in `tests/bug_reproducers_239_244_test.rs`; also covered by EC-7 in [tests/docs/cli/param/20_timeout.md](../param/20_timeout.md)
