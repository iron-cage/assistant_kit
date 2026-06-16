# Type :: `ErrorClass`

Validation tests for the `ErrorClass` taxonomy. `ErrorClass` is a documentation-only type — tests validate that observable CLI exit behaviors align with the class table, not that a runtime type exists.

**Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)
**Invariant:** [invariant/006_exit_codes.md](../../../../docs/invariant/006_exit_codes.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Success → exit 0 | Success |
| TC-2 | Runner — binary not found → exit 1 | Runner |
| TC-3 | Runner — gate timeout → exit 1 | Runner |
| TC-4 | Runner — output file write error → exit 1 | Runner |
| TC-5 | Transient — exit 2, no text | Transient |
| TC-6 | Account — exit 2 + quota text | Account |
| TC-7 | Process — timeout (CLR watchdog) → exit 4 | Process |
| TC-8 | Validation — expect mismatch → exit 3 | Validation |
| TC-9 | Process — signal → exit > 128 | Process |
| TC-10 | Timeout uses exit 4, not exit 2 | Disambiguation |
| TC-11 | Exit-2 disambiguation: QuotaExhausted vs RateLimit | Disambiguation |
| TC-12 | Runner — spawn failed → stderr contains `[Runner]` prefix | Runner (BUG-298) |

## Test Coverage Summary

- Success: 1 test (TC-1)
- Runner: 4 tests (TC-2, TC-3, TC-4, TC-12)
- Transient: 1 test (TC-5)
- Account: 1 test (TC-6)
- Process: 2 tests (TC-7, TC-9)
- Validation: 1 test (TC-8)
- Disambiguation: 2 tests (TC-10, TC-11)

**Total:** 12 test cases

## Test Cases

---

### TC-1: Success → exit 0

- **Given:** fake `claude` process that exits 0
- **When:** `clr --print "msg"` with `--max-sessions 0`
- **Then:** `clr` exits 0 (Success class)
- **Exit:** 0
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-2: Runner — binary not found → exit 1

- **Given:** PATH does not contain `claude` binary
- **When:** `clr --print "msg"`
- **Then:** exit 1; stderr contains `"claude binary not found in PATH"` (Runner class)
- **Exit:** 1
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-3: Runner — gate timeout → exit 1

- **Given:** session count ≥ `--max-sessions` and gate exhausts all wait attempts
- **When:** `clr --max-sessions 1 --print "msg"` with 1 claude session already running
- **Then:** exit 1; stderr contains gate timeout message (Runner class)
- **Exit:** 1
- **Note:** Gate-blocked in environments with ≥1 live claude session; reliable only in containers
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-4: Runner — output file write error → exit 1

- **Given:** `--output-file` path points to an unwritable location
- **When:** `clr --print --output-file /root/no_permission.txt "msg"` (non-root)
- **Then:** exit 1; stderr contains `"failed to write output file"` (Runner class)
- **Exit:** 1
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-5: Transient — exit 2, no text

- **Given:** fake `claude` that exits 2 with no stdout/stderr
- **When:** `clr --print --max-sessions 0 "msg"`
- **Then:** `clr` exits 2 (Transient class — RateLimit)
- **Exit:** 2
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-6: Account — exit 2 + quota text

- **Given:** fake `claude` that exits 2 and prints `"You've hit your limit"` to stdout
- **When:** `clr --print --max-sessions 0 "msg"`
- **Then:** `clr` exits 2 (Account class — QuotaExhausted)
- **Exit:** 2
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-7: Process — CLR timeout watchdog → exit 4 + stderr label

- **Given:** fake `claude` that sleeps indefinitely; `--timeout 1`
- **When:** `clr --print --timeout 1 --max-sessions 0 "msg"`
- **Then:** exit 4; stderr contains `"Error: timeout after 1s"` (Process class — Timeout)
- **Exit:** 4
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-8: Validation — expect mismatch → exit 3

- **Given:** fake `claude` that prints `"foo"`; `--expect "bar"`
- **When:** `clr --print --max-sessions 0 --expect "bar" "msg"`
- **Then:** exit 3; stderr contains expect-mismatch text (Validation class)
- **Exit:** 3
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-9: Process — signal → exit > 128

- **Given:** fake `claude` killed by SIGTERM (signal 15)
- **When:** subprocess receives SIGTERM during execution
- **Then:** `clr` exits 143 (128 + 15) (Process class — Signal)
- **Exit:** 143
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-10: Timeout uses exit 4, not exit 2

- **Given:** CLR watchdog kills subprocess (Timeout); subprocess rate-limit exits 2 (RateLimit)
- **When:** both conditions produce their respective exit codes
- **Then:** Timeout → exit 4 with stderr `"Error: timeout after "` prefix; RateLimit → exit 2 with no such prefix on stderr; no ambiguity
- **Exit:** 4 (Timeout), 2 (RateLimit)
- **Note:** Documentation-level test: verified by TC-7 (Timeout exits 4) and TC-5 (RateLimit exits 2)
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-11: Exit-2 disambiguation — QuotaExhausted has text content

- **Given:** quota-exhausted exit (exit 2 + `"You've hit your limit"`); rate-limit exit (exit 2, no text)
- **When:** both produce exit 2
- **Then:** QuotaExhausted → stdout/stderr contains `"You've hit your limit"`; RateLimit → no such text
- **Exit:** 2 in both cases
- **Note:** Documentation-level test: verified by TC-6 (Account has text) and TC-5 (Transient has none)
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)

---

### TC-12: Runner — spawn failed → stderr contains `[Runner]` prefix

- **Given:** `claude` binary exists at the PATH location but has no execute permission (`chmod 000`)
- **When:** `clr --print "msg"` with `--max-sessions 0`
- **Then:** exit 1; stderr contains `"[Runner]"` prefix before the error message (e.g. `Error: [Runner] failed to execute Claude Code: permission denied (os error 13) (exit 1)`)
- **Exit:** 1
- **Note:** test_kind: bug_reproducer(BUG-298). Validates that `spawn_error_msg()` and all spawn-error call sites prepend `[Runner]` as specified in § Console Output Format. Currently failing — fix tracked in BUG-298.
- **Source:** [type/14_error_class.md](../../../../docs/cli/type/14_error_class.md)
