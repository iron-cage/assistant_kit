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
| IT-6 | `.` output includes usage line with `<command>` syntax | Content |
| IT-7 | `.` with trailing unknown param still shows help | Robustness |
| IT-8 | `.` output is stable across repeated invocations | Stability |
| IT-9 | `.` output shows grouped section headers, not a flat list | Format |
| IT-10 | `.` output contains no per-command parameter listings | Format |
| IT-11 | `.` output includes Options section with format/dry/name hints | Content |
| IT-12 | Piped `.` output contains no ANSI escape sequences | Output |

### Test Coverage Summary

- Delegation: 1 test
- Exit Code: 1 test
- Visibility: 1 test
- Content: 4 tests
- Robustness: 1 test
- Stability: 1 test
- Format: 2 tests
- Output: 1 test

**Total:** 12 integration tests

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
- **Then:** stdout contains all 9 visible command names (`.accounts`, `.account.save`, `.account.use`, `.account.delete`, `.token.status`, `.paths`, `.usage`, `.credentials.status`, `.account.limits`); does NOT contain `.account.list` or `.account.status`
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

### IT-6: `.` output includes usage line with `<command>` syntax

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .`
- **Then:** stdout contains `Usage: clp <command>`
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

---

### IT-9: `.` output shows grouped section headers, not a flat list

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .`
- **Then:** stdout contains both "Account management" and "Status & info" as group headers; commands appear indented under each group
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-10: `.` output contains no per-command parameter listings

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .`
- **Then:** stdout does NOT contain `[name::EMAIL]`, does NOT contain `format::text|json` within a command line (Options section mentions it, but command rows do not)
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-11: `.` output includes Options section with cross-command hints

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .`
- **Then:** stdout contains "Options:" followed by "format::text|json", "dry::bool", and "name::EMAIL" on separate lines
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)

---

### IT-12: Piped `.` output contains no ANSI escape sequences

- **Given:** clean environment, `clp` piped to a command (non-TTY stdout)
- **When:** `clp .` (stdout captured as bytes, not a terminal)
- **Then:** stdout contains no ESC (`\x1b`) characters; all text is plain ASCII
- **Exit:** 0
- **Source:** [commands.md — .](../../../../docs/cli/commands.md#command--1-)
