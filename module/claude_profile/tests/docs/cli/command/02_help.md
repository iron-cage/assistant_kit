# Test: `.help`

Integration test planning for the `.help` command. See [commands.md](../../../../docs/cli/commands.md#command--2-help) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `.help` lists all registered visible commands | Content |
| IT-2 | `.help` excludes hidden commands (only bare `.`) | Visibility |
| IT-3 | `.help` shows usage line with binary name | Content |
| IT-4 | `.help` exits 0 | Exit Code |
| IT-5 | `.help` output includes command purposes | Content |
| IT-6 | `.help` output includes parameter hints per command | Content |
| IT-7 | `.help` does not appear in its own listing | Visibility |
| IT-8 | `.help` output is stable across repeated invocations | Stability |

### Test Coverage Summary

- Content: 4 tests
- Visibility: 2 tests
- Exit Code: 1 test
- Stability: 1 test

**Total:** 8 integration tests

---

### IT-1: `.help` lists all registered visible commands

- **Given:** clean environment
- **When:** `clp .help`
- **Then:** Output contains all 9 registered command names.; all 9 registered command names present; removed commands absent
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-2: `.help` excludes hidden commands

- **Given:** clean environment
- **When:** `clp .help`
- **Then:** The "Commands:" section lists exactly 9 entries; bare `.` and `.help` are both absent.; bare `.` absent; `.help` absent; exactly 9 command entries
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-3: `.help` shows usage line with binary name

- **Given:** clean environment
- **When:** `clp .help`
- **Then:** Output contains a line matching `Usage: clp <command> [params]`.; usage line with `clp` binary name is present
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-4: `.help` exits 0

- **Given:** clean environment
- **When:** `clp .help`
- **Then:** Help text printed to stdout; exit code 0.
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-5: `.help` output includes command purposes

- **Given:** clean environment
- **When:** `clp .help`
- **Then:** Each command entry in the listing is followed by its purpose text as defined in the spec.; all 9 command purpose descriptions present in output
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-6: `.help` output includes parameter hints per command

- **Given:** clean environment
- **When:** `clp .help`
- **Then:** Commands that accept parameters show parameter names or placeholders (e.g., `name::`, `v::`, `format::`, `dry::`, `threshold::`).; parameter hints visible for commands that accept parameters
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-7: `.help` does not appear in its own listing

- **Given:** clean environment
- **When:** `clp .help`
- **Then:** Exactly 9 command entries in the "Commands:" section; `.help` is absent.; `.help` does not appear as a listed command; exactly 9 entries
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)

---

### IT-8: `.help` output is stable across repeated invocations

- **Given:** clean environment
- **When:** `clp .help` (run 3 times)
- **Then:** All 3 invocations produce identical stdout.; on all runs; all 3 outputs are byte-identical
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--2-help)
