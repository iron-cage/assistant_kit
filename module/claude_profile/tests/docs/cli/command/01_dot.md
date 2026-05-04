# Test: `.`

Integration test planning for the `.` command. See [commands.md](../../../../docs/cli/commands.md#command--1-) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `.` produces identical output to `.help` | Delegation |
| IT-2 | `.` exits 0 | Exit Code |
| IT-3 | `.` is hidden from help listing | Visibility |
| IT-4 | `.` output lists all 9 visible commands; removed commands absent | Content |
| IT-5 | `.` output excludes bare `.` from listing | Content |
| IT-6 | `.` output includes usage line | Content |
| IT-7 | `.` with trailing unknown param still shows help | Robustness |
| IT-8 | `.` output is stable across repeated invocations | Stability |

### Test Coverage Summary

- Delegation: 1 test
- Exit Code: 1 test
- Visibility: 1 test
- Content: 3 tests
- Robustness: 1 test
- Stability: 1 test

**Total:** 8 integration tests

---

### IT-1: `.` produces identical output to `.help`

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .` and `clp .help`
- **Then:** stdout of both invocations is byte-identical
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-2: `.` exits 0

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .`
- **Then:** process exits with code 0
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-3: `.` is hidden from help listing

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .`
- **Then:** stdout does not contain a bare `.` command entry in the Commands section
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-4: `.` output lists all visible commands

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .`
- **Then:** stdout contains all 9 visible command names (`.accounts`, `.account.save`, `.account.switch`, `.account.delete`, `.token.status`, `.paths`, `.usage`, `.credentials.status`, `.account.limits`); does NOT contain `.account.list` or `.account.status`
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-5: `.` output excludes bare `.` from listing

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .`
- **Then:** the Commands section contains exactly 9 entries; no entry matches a bare `.` standalone command
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-6: `.` output includes usage line

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .`
- **Then:** stdout contains `Usage: clp`
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-7: `.` with trailing unknown param still shows help

- **Given:** clean environment, `clp` on PATH
- **When:** `clp . foo::bar`
- **Then:** stdout is identical to bare `clp .` output; unknown param silently ignored
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-8: `.` output is stable across repeated invocations

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .` (run 3 times)
- **Then:** all 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)
