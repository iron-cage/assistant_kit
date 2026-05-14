# Test: `.help`

Integration test planning for the `.help` command. See [commands.md](../../../../docs/cli/commands.md#command--2-help) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `.help` lists all 9 registered visible commands; removed commands absent | Content |
| IT-2 | `.help` excludes hidden commands (bare `.` and `.help` itself absent from listing) | Visibility |
| IT-3 | `.help` shows usage line with `<command>` syntax | Content |
| IT-4 | `.help` exits 0 | Exit Code |
| IT-5 | `.help` output shows grouped section headers, not a flat list | Format |
| IT-6 | `.help` output contains no per-command parameter listings | Format |
| IT-7 | `.help` output includes Options section with format/dry/name hints | Content |
| IT-8 | `.help` output is stable across repeated invocations | Stability |

### Test Coverage Summary

- Content: 3 tests
- Visibility: 1 test
- Exit Code: 1 test
- Format: 2 tests
- Stability: 1 test

**Total:** 8 integration tests

---

### IT-1: `.help` lists all registered visible commands

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .help`
- **Then:** stdout contains all 9 visible command names (`.accounts`, `.account.save`, `.account.use`, `.account.delete`, `.account.limits`, `.credentials.status`, `.token.status`, `.paths`, `.usage`); does NOT contain `.account.list` or `.account.status`
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-2: `.help` excludes hidden commands

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .help`
- **Then:** bare `.` is absent from the Commands section; `.help` itself does not appear as a listed command; exactly 9 entries present
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-3: `.help` shows usage line with `<command>` syntax

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .help`
- **Then:** stdout contains `Usage: clp <command>`
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-4: `.help` exits 0

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .help`
- **Then:** process exits with code 0; stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-5: `.help` output shows grouped section headers, not a flat list

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .help`
- **Then:** stdout contains both "Account management" and "Status & info" as group headers; commands appear indented under each group
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-6: `.help` output contains no per-command parameter listings

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .help`
- **Then:** stdout does NOT contain `[name::EMAIL]`, does NOT contain `[format::text|json]`, does NOT contain bracket-enclosed parameter syntax in command rows; the Commands section shows names and one-line descriptions only
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-7: `.help` output includes Options section with cross-command hints

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .help`
- **Then:** stdout contains "Options:" followed by "format::text|json", "dry::bool", and "name::EMAIL" on separate lines
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-8: `.help` output is stable across repeated invocations

- **Given:** clean environment, `clp` on PATH
- **When:** `clp .help` (run 3 times)
- **Then:** all 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)
