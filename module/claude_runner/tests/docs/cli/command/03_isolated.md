# Test: `isolated`

Integration test planning for the `isolated` command. See [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md) for specification.

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
| IT-28 | `--file` with >64 KiB stdout → no pipe deadlock | Pipe Buffering |
| IT-29 | `--output-file` tees output to disk | Output File |
| IT-30 | `--strip-fences` removes outer fences | Strip Fences |
| IT-31 | `--output-style summary` renders CLR envelope | Output Style |
| IT-32 | `--summary-fields minimal` limits rendered fields | Summary Fields |
| IT-33 | `CLR_OUTPUT_FILE` env var fallback | Output Env |
| IT-34 | `CLR_STRIP_FENCES=1` env var fallback | Output Env |
| IT-35 | `CLR_OUTPUT_STYLE=summary` env var fallback | Output Env |
| IT-36 | `CLR_SUMMARY_FIELDS=minimal` env var fallback | Output Env |
| IT-37 | `CLR_JOURNAL=bogus` env var → exit 1, error names env var | Error: Invalid Env |
| IT-46 | `--model sonnet` overrides isolated's injected `opus` default | Native Flag: model |
| IT-47 | `--effort medium` overrides isolated's injected `max` default | Native Flag: effort |
| IT-48 | `--no-effort-max` suppresses the injected `--effort` flag entirely | Native Flag: no-effort-max |
| IT-49 | `--system-prompt "You are terse"` forwarded to subprocess | Native Flag: system-prompt |
| IT-50 | `--append-system-prompt "Also: be terse"` forwarded to subprocess | Native Flag: append-system-prompt |
| IT-51 | `--json-schema schema.json` forwarded to subprocess | Native Flag: json-schema |
| IT-52 | `--mcp-config mcp.json` forwarded to subprocess | Native Flag: mcp-config |
| IT-53 | `--allowed-tools "Read,Grep"` forwarded to subprocess | Native Flag: allowed-tools |
| IT-54 | `--disallowed-tools "Bash"` forwarded to subprocess | Native Flag: disallowed-tools |
| IT-55 | `--max-budget-usd 2.50` forwarded to subprocess | Native Flag: max-budget-usd |
| IT-56 | `--max-turns 10` forwarded to subprocess | Native Flag: max-turns |
| IT-57 | `--no-chrome` suppresses `--chrome` injection | Native Flag: no-chrome |
| IT-58 | `CLR_MODEL=sonnet` env var → equivalent to `--model sonnet` | Env Var Fallback |
| IT-59 | `--args-file` JSON `"effort":"medium"` → equivalent to `--effort medium` | JSON Config Fallback |
| IT-60 | `--effort max -- --effort low` passthrough last-wins preserved | Passthrough Regression |
| IT-61 | `clr isolated --help` lists all 12 new native flags | Help Listing |

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
- Pipe Buffering: 1 test (IT-28)
- Output File: 1 test (IT-29)
- Strip Fences: 1 test (IT-30)
- Output Style: 1 test (IT-31)
- Summary Fields: 1 test (IT-32)
- Output Env: 4 tests (IT-33 through IT-36)
- Error: Invalid Env: 1 test (IT-37)
- Native Flag: model: 1 test (IT-46)
- Native Flag: effort: 1 test (IT-47)
- Native Flag: no-effort-max: 1 test (IT-48)
- Native Flag: system-prompt: 1 test (IT-49)
- Native Flag: append-system-prompt: 1 test (IT-50)
- Native Flag: json-schema: 1 test (IT-51)
- Native Flag: mcp-config: 1 test (IT-52)
- Native Flag: allowed-tools: 1 test (IT-53)
- Native Flag: disallowed-tools: 1 test (IT-54)
- Native Flag: max-budget-usd: 1 test (IT-55)
- Native Flag: max-turns: 1 test (IT-56)
- Native Flag: no-chrome: 1 test (IT-57)
- Env Var Fallback: 1 test (IT-58)
- JSON Config Fallback: 1 test (IT-59)
- Passthrough Regression: 1 test (IT-60)
- Help Listing: 1 test (IT-61)

**Total:** 53 test cases

---

### IT-1: `--creds file.json "msg"` → runs in isolated HOME, exit 0

- **Setup:** valid credentials JSON at `/tmp/it1_creds.json`; `claude` binary in PATH
- **Command:** `clr isolated --creds /tmp/it1_creds.json "What is 2+2?"`
- **Expected behavior:** subprocess runs with `HOME=<temp>`; temp HOME contains only `.claude/.credentials.json`; claude produces output; exit 0
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

---

### IT-2: `--creds missing.json` → exit 1, file-not-found error

- **Setup:** `/tmp/it2_missing.json` does not exist
- **Command:** `clr isolated --creds /tmp/it2_missing.json "test"`
- **Expected behavior:** exit 1; stderr contains "not found" or equivalent; no subprocess launched
- **Exit:** 1
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

---

### IT-3: `--timeout 0 "msg"` → exit 2 (timeout, no creds refresh)

- **Setup:** valid credentials JSON at `/tmp/it3_creds.json`; subprocess does not refresh creds before blocking
- **Command:** `clr isolated --creds /tmp/it3_creds.json --timeout 0 "Long running task"`
- **Expected behavior:** subprocess attempted; wait window expires immediately; creds not refreshed → exit 2
- **Exit:** 2
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [--timeout](../../../../docs/cli/param/020_timeout.md)

---

### IT-4: `--timeout 0` with creds refresh → exit 0, creds updated in-place

- **Setup:** expired-token credentials at `/tmp/it4_creds.json`; `claude` performs OAuth refresh at startup before blocking on REPL
- **Command:** `clr isolated --creds /tmp/it4_creds.json --timeout 0`
- **Expected behavior:** subprocess refreshes token before blocking; `clr isolated` detects refresh → exit 0; `/tmp/it4_creds.json` contains updated token
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [--timeout](../../../../docs/cli/param/020_timeout.md)

---

### IT-5: `--creds file.json` (no message) → interactive REPL in isolation

- **Setup:** valid credentials JSON at `/tmp/it5_creds.json`; TTY available
- **Command:** `clr isolated --creds /tmp/it5_creds.json` (no message)
- **Expected behavior:** Claude starts in interactive REPL mode inside isolated HOME; stdin/stdout connected to subprocess; no `--print` injected
- **Exit:** 0 (when REPL exits)
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

---

### IT-6: `--creds file.json -- --version` → `--version` passed through to claude

- **Setup:** valid credentials JSON at `/tmp/it6_creds.json`; `claude --version` exits 0
- **Command:** `clr isolated --creds /tmp/it6_creds.json -- --version`
- **Expected behavior:** subprocess receives `--version` flag; version string printed to stdout; exit 0
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

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
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [--creds](../../../../docs/cli/param/019_creds.md)

---

### IT-9: `clr isolated --help` → exit 0, prints isolated-specific help

- **Command:** `clr isolated --help` (also: `clr isolated -h`)
- **Expected behavior:** exit 0; stdout contains `--creds`, `--timeout`, and `--help`; no subprocess launched; no error in stderr
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

---

### IT-10: `--creds <f> --trace "msg"` → call details on stderr before execution attempt

- **Setup:** credentials JSON written to a temp file `<f>` (file is readable); claude binary absent in test environment
- **Command:** `clr isolated --creds <f> --trace "Fix bug"` (no `--dry-run`; trace fires before subprocess attempt)
- **Expected behavior:** stderr contains `# clr isolated`, `# creds: <path>`, `# timeout: 30s`, env var block (including `CLAUDE_CODE_MAX_OUTPUT_TOKENS=128000`), and `claude --chrome --model claude-opus-4-8 --effort max --no-session-persistence --dangerously-skip-permissions --print "Fix bug"` before any subprocess attempt; subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md), [--trace](../../../../docs/cli/param/013_trace.md)

---

### IT-11: Timeout with partial stdout → exit 2, error includes accumulated output (BUG-243)

- **Setup:** fake-claude script that emits one line of output then blocks indefinitely (e.g. `echo "partial output"; sleep 60`); credentials JSON at `/tmp/it11_creds.json`; script injected via PATH or `CLR_CLAUDE_BIN`
- **Command:** `clr isolated --creds /tmp/it11_creds.json --timeout 1 "test"`
- **Expected behavior:** subprocess is killed after 1 second; exit 2 (timeout without credentials refresh); the partial stdout emitted before the timeout is included in the error output — diagnostic context is not discarded
- **Exit:** 2
- **Source:** [--timeout](../../../../docs/cli/param/020_timeout.md), [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)
- **Note:** Implemented in TSK-196 (BUG-243); test function `timeout_includes_partial_stdout` in `tests/bug_reproducers_239_244_test.rs`; also covered by EC-7 in [tests/docs/cli/param/020_timeout.md](../param/020_timeout.md)

---

### IT-12: `--dry-run` exits 0, prints command preview without spawning subprocess

- **Setup:** credentials JSON written to a temp file (content `{}`); claude binary absent in test environment
- **Command:** `clr isolated --creds <f> --dry-run`
- **Expected behavior:** exit 0; stdout contains command preview (claude binary + injected flags); no subprocess spawn; stderr empty
- **Exit:** 0
- **Source:** [--dry-run](../../../../docs/cli/param/011_dry_run.md), [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

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
- **Source:** [--dir](../../../../docs/cli/param/008_dir.md), [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

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
- **Source:** [--file](../../../../docs/cli/param/025_file.md), [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

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

---

### IT-28: `--file` with >64 KiB stdout → no pipe deadlock

- **Setup:** credentials JSON at temp file; fake claude echoes >64 KiB of output; `--file` used to pipe stdin
- **Command:** `clr isolated --creds <f> --file <input> "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; all output captured without deadlock; background reader threads drain stdout/stderr concurrently
- **Exit:** 0
- **Source:** [--file](../../../../docs/cli/param/025_file.md), [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

---

### IT-29: `--output-file` tees output to disk

- **Setup:** credentials JSON at temp file; fake claude echoes known text; output file path in temp dir
- **Command:** `clr isolated --creds <f> --output-file <path> "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; stdout contains known text; output file at `<path>` contains same text
- **Exit:** 0
- **Source:** [--output-file](../../../../docs/cli/param/029_output_file.md), [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)

---

### IT-30: `--strip-fences` removes outer fences

- **Setup:** credentials JSON at temp file; fake claude emits fenced python code (` ```python\nprint("hi")\n``` `)
- **Command:** `clr isolated --creds <f> --strip-fences "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; stdout contains `print("hi")` without fence lines
- **Exit:** 0
- **Source:** [--strip-fences](../../../../docs/cli/param/026_strip_fences.md)

---

### IT-31: `--output-style summary` renders CLR envelope

- **Setup:** credentials JSON at temp file; fake claude emits JSON with `"type":"result"` and `"result":"hello"`
- **Command:** `clr isolated --creds <f> --output-style summary "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; stdout contains rendered summary (key:val format with `---` separator); raw JSON not in stdout
- **Exit:** 0
- **Source:** [--output-style](../../../../docs/cli/param/070_output_style.md)

---

### IT-32: `--summary-fields minimal` limits rendered fields

- **Setup:** credentials JSON at temp file; fake claude emits CLR envelope JSON with `"type":"result"`, `"result":"hello"`, `"num_turns":5`
- **Command:** `clr isolated --creds <f> --output-style summary --summary-fields minimal "msg"` (PATH set to fake claude dir)
- **Expected behavior:** exit 0; stdout contains `result` field; stdout does NOT contain `num_turns` (excluded by minimal profile)
- **Exit:** 0
- **Source:** [--summary-fields](../../../../docs/cli/param/071_summary_fields.md)

---

### IT-33: `CLR_OUTPUT_FILE` env var fallback

- **Setup:** credentials JSON at temp file; fake claude echoes known text; `CLR_OUTPUT_FILE=<path>` in env; no `--output-file` flag
- **Command:** `clr isolated --creds <f> "msg"` (with `CLR_OUTPUT_FILE` set)
- **Expected behavior:** exit 0; output file at `<path>` contains known text
- **Exit:** 0
- **Source:** [--output-file](../../../../docs/cli/param/029_output_file.md)

---

### IT-34: `CLR_STRIP_FENCES=1` env var fallback

- **Setup:** credentials JSON at temp file; fake claude emits fenced code; `CLR_STRIP_FENCES=1` in env; no `--strip-fences` flag
- **Command:** `clr isolated --creds <f> "msg"` (with `CLR_STRIP_FENCES=1` set)
- **Expected behavior:** exit 0; stdout contains code without fence lines
- **Exit:** 0
- **Source:** [--strip-fences](../../../../docs/cli/param/026_strip_fences.md)

---

### IT-35: `CLR_OUTPUT_STYLE=summary` env var fallback

- **Setup:** credentials JSON at temp file; fake claude emits CLR JSON envelope; `CLR_OUTPUT_STYLE=summary` in env; no `--output-style` flag
- **Command:** `clr isolated --creds <f> "msg"` (with `CLR_OUTPUT_STYLE=summary` set)
- **Expected behavior:** exit 0; stdout contains rendered summary format (not raw JSON)
- **Exit:** 0
- **Source:** [--output-style](../../../../docs/cli/param/070_output_style.md)

---

### IT-36: `CLR_SUMMARY_FIELDS=minimal` env var fallback

- **Setup:** credentials JSON at temp file; fake claude emits CLR JSON envelope; `CLR_OUTPUT_STYLE=summary` + `CLR_SUMMARY_FIELDS=minimal` in env
- **Command:** `clr isolated --creds <f> "msg"` (with both env vars set)
- **Expected behavior:** exit 0; stdout contains `result` field; stdout does NOT contain `num_turns`
- **Exit:** 0
- **Source:** [--summary-fields](../../../../docs/cli/param/071_summary_fields.md)


---

### IT-37: `CLR_JOURNAL=bogus` env var → exit 1 with error naming the env var

- **Setup:** credentials JSON at temp file; `CLR_JOURNAL=bogus` in env; `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run "x"` (with `CLR_JOURNAL=bogus` set)
- **Expected behavior:** exit 1; stderr contains `"CLR_JOURNAL"` and `"invalid"`
- **Exit:** 1
- **Source:** Fix — `apply_isolated_env_vars()` validates `CLR_JOURNAL` consistently with `apply_env_vars()` in `env.rs`

---

### IT-46: `--model sonnet` overrides isolated's injected `opus` default

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --model sonnet "msg"`
- **Expected behavior:** exit 0; `--dry-run` preview stdout contains `--model sonnet`; confirms native flag overrides the default `opus` alias injection
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/003_model.md](../../../../docs/cli/param/003_model.md)

---

### IT-47: `--effort medium` overrides isolated's injected `max` default

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --effort medium "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--effort medium`; confirms native flag overrides the default `--effort max` injection
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/017_effort.md](../../../../docs/cli/param/017_effort.md)

---

### IT-48: `--no-effort-max` suppresses the injected `--effort` flag entirely

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --no-effort-max "msg"`
- **Expected behavior:** exit 0; preview stdout does NOT contain `--effort` anywhere; confirms `--no-effort-max` suppresses the automatic injection completely
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/018_no_effort_max.md](../../../../docs/cli/param/018_no_effort_max.md)

---

### IT-49: `--system-prompt "You are terse"` forwarded to subprocess

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --system-prompt "You are terse" "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--system-prompt` and the value `You are terse`
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/015_system_prompt.md](../../../../docs/cli/param/015_system_prompt.md)

---

### IT-50: `--append-system-prompt "Also: be terse"` forwarded to subprocess

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --append-system-prompt "Also: be terse" "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--append-system-prompt` and the value `Also: be terse`
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/016_append_system_prompt.md](../../../../docs/cli/param/016_append_system_prompt.md)

---

### IT-51: `--json-schema schema.json` forwarded to subprocess

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --json-schema schema.json "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--json-schema schema.json`
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/023_json_schema.md](../../../../docs/cli/param/023_json_schema.md)

---

### IT-52: `--mcp-config mcp.json` forwarded to subprocess

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --mcp-config mcp.json "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--mcp-config mcp.json`
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/024_mcp_config.md](../../../../docs/cli/param/024_mcp_config.md)

---

### IT-53: `--allowed-tools "Read,Grep"` forwarded to subprocess

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --allowed-tools "Read,Grep" "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--allowed-tools` and the value `Read,Grep`
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/063_allowed_tools.md](../../../../docs/cli/param/063_allowed_tools.md)

---

### IT-54: `--disallowed-tools "Bash"` forwarded to subprocess

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --disallowed-tools "Bash" "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--disallowed-tools` and the value `Bash`
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/064_disallowed_tools.md](../../../../docs/cli/param/064_disallowed_tools.md)

---

### IT-55: `--max-budget-usd 2.50` forwarded to subprocess

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --max-budget-usd 2.50 "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--max-budget-usd 2.50`
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/065_max_budget_usd.md](../../../../docs/cli/param/065_max_budget_usd.md)

---

### IT-56: `--max-turns 10` forwarded to subprocess

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --max-turns 10 "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--max-turns 10`
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/062_max_turns.md](../../../../docs/cli/param/062_max_turns.md)

---

### IT-57: `--no-chrome` suppresses `--chrome` injection

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` flag prevents subprocess spawn
- **Command:** `clr isolated --creds <f> --dry-run --no-chrome "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--no-chrome`; `--chrome` must NOT appear in the preview
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/021_no_chrome.md](../../../../docs/cli/param/021_no_chrome.md)

---

### IT-58: `CLR_MODEL=sonnet` env var → equivalent to `--model sonnet`

- **Setup:** credentials JSON at temp file (content `{}`); `CLR_MODEL=sonnet` set in environment; `--dry-run` prevents subprocess spawn; no `--model` flag given
- **Command:** `clr isolated --creds <f> --dry-run "msg"` (with `CLR_MODEL=sonnet` in env)
- **Expected behavior:** exit 0; preview stdout contains `--model sonnet`; confirms `CLR_MODEL` fallback is equivalent to the native `--model` flag (IT-46)
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/003_model.md](../../../../docs/cli/param/003_model.md)

---

### IT-59: `--args-file` JSON `"effort":"medium"` → equivalent to `--effort medium`

- **Setup:** credentials JSON at temp file (content `{}`); JSON config file containing `{"effort": "medium"}`; `--dry-run` prevents subprocess spawn; no `--effort` flag given
- **Command:** `clr isolated --creds <f> --args-file <cfg> --dry-run "msg"`
- **Expected behavior:** exit 0; preview stdout contains `--effort medium`; confirms `--args-file` JSON config is equivalent to the native `--effort` flag (IT-47)
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/075_args_file.md](../../../../docs/cli/param/075_args_file.md)

---

### IT-60: `--effort max -- --effort low` passthrough last-wins preserved

- **Setup:** credentials JSON at temp file (content `{}`); `--dry-run` prevents subprocess spawn; native `--effort max` set before `--`; raw `--effort low` passed after `--`
- **Command:** `clr isolated --creds <f> --dry-run --effort max -- --effort low "msg"`
- **Expected behavior:** exit 0; the last occurrence of `--effort` in preview stdout is `--effort low`; confirms passthrough still wins over the native flag when both are present
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md), [param/017_effort.md](../../../../docs/cli/param/017_effort.md)

---

### IT-61: `clr isolated --help` lists all 12 new native flags

- **Setup:** no credentials needed; invoking built-in help
- **Command:** `clr isolated --help`
- **Expected behavior:** exit 0; stdout contains all 12 new native flags: `--model`, `--effort`, `--no-effort-max`, `--system-prompt`, `--append-system-prompt`, `--json-schema`, `--mcp-config`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--max-turns`, `--no-chrome`
- **Exit:** 0
- **Source:** [command/03_isolated.md](../../../../docs/cli/command/03_isolated.md)
