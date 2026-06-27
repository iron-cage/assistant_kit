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
| IT-12 | `--dry-run` exits 0, prints command preview without spawning subprocess | Dry-run |
| IT-13 | `--dry-run "msg"` preview contains `--print` and message text | Dry-run |
| IT-14 | `--dry-run --dir /tmp` preview contains `--dir` | Dry-run |
| IT-15 | `--dry-run --add-dir /tmp` preview contains `--add-dir` | Dry-run |
| IT-16 | `--dir /tmp` injected into subprocess argv | Directory |
| IT-17 | `--dir /nonexistent` → exit 1 before spawn, stderr indicates path absent | Directory Error |
| IT-18 | `--add-dir /tmp` injected into subprocess argv | Directory |
| IT-19 | `--dir /tmp --add-dir /var` both injected into subprocess argv | Directory |
| IT-20 | `CLR_DIR=/tmp` env var applied when `--dir` absent; dry-run preview confirms | Directory Env |
| IT-21 | `--file <path>` pipes file content to subprocess stdin | File Input |
| IT-22 | `--file /nonexistent` → exit 1 before spawn, stderr indicates file absent | File Error |
| IT-23 | `--file <path> "msg"` file as stdin combined with message as prompt | File Input |
| IT-24 | `--expect "hello"` output matches → exit 0, output unchanged | Expect Match |
| IT-25 | `--expect "hello" --expect-strategy fail` mismatch → exit 3 | Expect Fail |
| IT-26 | `--expect "hello" --expect-strategy default:no` mismatch → exit 0, stdout "no" | Expect Default |
| IT-27 | `--expect-strategy retry` → exit 1 (unsupported for isolated) | Expect Retry |

## Test Coverage Summary

- Happy Path: 1 test (IT-1)
- Error Handling: 2 tests (IT-2, IT-7)
- Default Fallback: 1 test (IT-8)
- Timeout Behavior: 3 tests (IT-3, IT-4, IT-11)
- Mode Selection: 2 tests (IT-5, IT-6)
- Help: 1 test (IT-9)
- Trace: 1 test (IT-10)
- Dry-run: 4 tests (IT-12 through IT-15)
- Directory: 4 tests (IT-16 through IT-19)
- Directory Env: 1 test (IT-20)
- File Input: 2 tests (IT-21, IT-23)
- File Error: 1 test (IT-22)
- Expect Validation: 4 tests (IT-24 through IT-27)

**Total:** 27 test cases

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
- **Expected behavior:** stderr contains `# clr isolated`, `# creds: <path>`, `# timeout: 30s`, env var block (including `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`), and `claude --chrome --model claude-opus-4-6 --effort max --no-session-persistence --dangerously-skip-permissions --print "Fix bug"` before any subprocess attempt; subprocess attempt fails (claude absent in test environment)
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

---

### IT-12: `--dry-run` exits 0, prints command preview without spawning subprocess

- **Setup:** credentials JSON written to a temp file (content `{}`); claude binary absent in test environment
- **Command:** `clr isolated --creds <f> --dry-run`
- **Expected behavior:** exit 0; stdout contains command preview (claude binary + injected flags); no subprocess spawn; stderr empty
- **Exit:** 0
- **Source:** [--dry-run](../../../../docs/cli/param/011_dry_run.md), [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)

---

### IT-13: `--dry-run "msg"` preview contains `--print` and message text

- **Setup:** credentials JSON at temp file (content `{}`)
- **Command:** `clr isolated --creds <f> --dry-run "say hello"`
- **Expected behavior:** exit 0; stdout preview includes `--print` and `say hello`; no subprocess spawn
- **Exit:** 0
- **Source:** [--dry-run](../../../../docs/cli/param/011_dry_run.md)

---

### IT-14: `--dry-run --dir /tmp` preview contains `--dir`

- **Setup:** credentials JSON at temp file; `/tmp` exists
- **Command:** `clr isolated --creds <f> --dry-run --dir /tmp "msg"`
- **Expected behavior:** exit 0; stdout preview contains `--dir`; no subprocess spawn; `--dir` is processed before dry-run exit so dir validation is not performed
- **Exit:** 0
- **Source:** [--dry-run](../../../../docs/cli/param/011_dry_run.md), [--dir](../../../../docs/cli/param/008_dir.md)

---

### IT-15: `--dry-run --add-dir /tmp` preview contains `--add-dir`

- **Setup:** credentials JSON at temp file
- **Command:** `clr isolated --creds <f> --dry-run --add-dir /tmp "msg"`
- **Expected behavior:** exit 0; stdout preview contains `--add-dir`; no subprocess spawn
- **Exit:** 0
- **Source:** [--dry-run](../../../../docs/cli/param/011_dry_run.md), [--add-dir](../../../../docs/cli/param/066_add_dir.md)

---

### IT-16: `--dir /tmp` injected into subprocess argv

- **Setup:** credentials JSON at temp file (content `{}`); fake claude script injected via PATH that echoes all argv; `/tmp` exists
- **Command:** `clr isolated --creds <f> --dir /tmp "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; fake claude stdout contains `--dir`; proving `--dir /tmp` was injected into subprocess argv
- **Exit:** 0
- **Source:** [--dir](../../../../docs/cli/param/008_dir.md), [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)

---

### IT-17: `--dir /nonexistent` → exit 1 before spawn, stderr indicates path absent

- **Setup:** credentials JSON at temp file; `/nonexistent_clr_test_dir_it17` does not exist
- **Command:** `clr isolated --creds <f> --dir /nonexistent_clr_test_dir_it17 "msg"`
- **Expected behavior:** exit 1; stderr contains "does not exist" or "not found"; no subprocess spawn
- **Exit:** 1
- **Source:** [--dir](../../../../docs/cli/param/008_dir.md)

---

### IT-18: `--add-dir /tmp` injected into subprocess argv

- **Setup:** credentials JSON at temp file; fake claude echoes argv; `/tmp` exists
- **Command:** `clr isolated --creds <f> --add-dir /tmp "msg"` (PATH set to fake claude dir)
- **Expected behavior:** stdout contains `--add-dir`; exit 0
- **Exit:** 0
- **Source:** [--add-dir](../../../../docs/cli/param/066_add_dir.md)

---

### IT-19: `--dir /tmp --add-dir /var` both injected into subprocess argv

- **Setup:** credentials JSON at temp file; fake claude echoes argv; `/tmp` and `/var` exist
- **Command:** `clr isolated --creds <f> --dir /tmp --add-dir /var "msg"` (PATH set to fake claude dir)
- **Expected behavior:** stdout contains both `--dir` and `--add-dir`; exit 0
- **Exit:** 0
- **Source:** [--dir](../../../../docs/cli/param/008_dir.md), [--add-dir](../../../../docs/cli/param/066_add_dir.md)

---

### IT-20: `CLR_DIR=/tmp` env var applied when `--dir` absent; dry-run confirms

- **Setup:** credentials JSON at temp file; `CLR_DIR=/tmp` set in env; `CLR_ADD_DIR` unset
- **Command:** `clr isolated --creds <f> --dry-run "msg"` (with `CLR_DIR=/tmp` in environment)
- **Expected behavior:** exit 0; stdout preview contains `--dir`; confirms `CLR_DIR` env var is picked up
- **Exit:** 0
- **Source:** [--dir](../../../../docs/cli/param/008_dir.md)

---

### IT-21: `--file <path>` pipes file content to subprocess stdin

- **Setup:** credentials JSON at temp file; input file with known content (`file_content_it21`); fake claude runs `cat` to echo stdin
- **Command:** `clr isolated --creds <f> --file <input> "process this"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; stdout contains `file_content_it21` (file content piped as stdin to subprocess)
- **Exit:** 0
- **Source:** [--file](../../../../docs/cli/param/025_file.md), [command/02_isolated.md](../../../../docs/cli/command/02_isolated.md)

---

### IT-22: `--file /nonexistent` → exit 1 before spawn, stderr indicates file absent

- **Setup:** creds path provided (non-empty); `/tmp/clr_it22_nonexistent_input.txt` does not exist
- **Command:** `clr isolated --creds /tmp/clr_it22_dummy.json --file /tmp/clr_it22_nonexistent_input.txt "msg"`
- **Expected behavior:** exit 1; stderr contains "does not exist" or "not found"; no subprocess spawn
- **Exit:** 1
- **Source:** [--file](../../../../docs/cli/param/025_file.md)

---

### IT-23: `--file <path> "msg"` file as stdin combined with message as prompt

- **Setup:** credentials JSON at temp file; input file with known content (`combined_input_it23`); fake claude runs `cat`
- **Command:** `clr isolated --creds <f> --file <input> "process this file"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; stdout contains `combined_input_it23` (file content received via stdin)
- **Exit:** 0
- **Source:** [--file](../../../../docs/cli/param/025_file.md)

---

### IT-24: `--expect "hello"` output matches → exit 0, output unchanged

- **Setup:** credentials JSON at temp file; fake claude outputs `hello`
- **Command:** `clr isolated --creds <f> --expect "hello" "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; stdout contains `hello`; no mismatch handling triggered
- **Exit:** 0
- **Source:** [--expect](../../../../docs/cli/param/030_expect.md)

---

### IT-25: `--expect "hello" --expect-strategy fail` mismatch → exit 3

- **Setup:** credentials JSON at temp file; fake claude outputs `world`
- **Command:** `clr isolated --creds <f> --expect "hello" --expect-strategy fail "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 3; stderr contains expect mismatch indication
- **Exit:** 3
- **Source:** [--expect](../../../../docs/cli/param/030_expect.md), [--expect-strategy](../../../../docs/cli/param/031_expect_strategy.md)

---

### IT-26: `--expect "hello" --expect-strategy default:no` mismatch → exit 0, stdout "no"

- **Setup:** credentials JSON at temp file; fake claude outputs `world`
- **Command:** `clr isolated --creds <f> --expect "hello" --expect-strategy "default:no" "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; stdout contains `no` (fallback value replaces subprocess output)
- **Exit:** 0
- **Source:** [--expect-strategy](../../../../docs/cli/param/031_expect_strategy.md)

---

### IT-27: `--expect-strategy retry` → exit 1 (unsupported for isolated)

- **Setup:** credentials JSON at temp file; fake claude outputs `world`; `--expect "hello"` causes mismatch
- **Command:** `clr isolated --creds <f> --expect "hello" --expect-strategy retry "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 1; stderr contains "retry" indicating unsupported strategy
- **Exit:** 1
- **Source:** [--expect-strategy](../../../../docs/cli/param/031_expect_strategy.md)
