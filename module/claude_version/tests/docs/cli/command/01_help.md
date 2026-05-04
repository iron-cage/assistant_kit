# Test: `.help` / `.` / empty argv

Integration test planning for help output triggers. See [commands.md](../../../../docs/cli/commands.md#command--1-help) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `cm .` → help output, exit 0 | Alias |
| IT-2 | `cm` (empty argv) → help output, exit 0 | Empty State |
| IT-3 | `cm .help` → help output, exit 0 | Happy Path |
| IT-4 | `cm .status .help` → `.help` anywhere in argv triggers help, exit 0 | FR-02 |
| IT-5 | Help output goes to stdout; stderr is empty | Output Stream |
| IT-6 | Help output contains all visible command names | Content |
| IT-7 | `.help` does not appear in its own command listing | Visibility |
| IT-8 | Help output is stable across repeated invocations | Stability |

## Test Coverage Summary

- Happy Path: 1 test
- Alias: 1 test
- Empty State: 1 test
- FR-02 (anywhere-in-argv): 1 test
- Output Stream: 1 test
- Content: 1 test
- Visibility: 1 test
- Stability: 1 test

**Total:** 8 tests

---

### IT-1: `cm .` → help output, exit 0

- **Given:** clean environment
- **When:** `cm .`
- **Then:** Contains command listing; mentions `.help`.; help output shown
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--1-help), adapter.rs bare-dot handling

---

### IT-2: `cm` (empty argv) → help output, exit 0

- **Given:** clean environment
- **When:** `cm` (no arguments)
- **Then:** Same help output as `.help`.; help output shown
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--1-help), [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-3: `cm .help` → help output, exit 0

- **Given:** clean environment
- **When:** `cm .help`
- **Then:** All 12 commands listed; usage line present.; help listing complete
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--1-help), [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-4: `cm .status .help` → `.help` anywhere in argv triggers help, exit 0

- **Given:** clean environment
- **When:** `cm .status .help`
- **Then:** Help output (not `.status` output); help wins over `.status` when `.help` present anywhere
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md), adapter.rs `.help`-anywhere detection

---

### IT-5: Help output goes to stdout; stderr is empty

- **Given:** clean environment
- **When:** `cm .help`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--1-help)

---

### IT-6: Help output contains all visible command names

- **Given:** clean environment
- **When:** `cm .help`
- **Then:** All visible command names present in stdout output
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--1-help)

---

### IT-7: `.help` does not appear in its own command listing

- **Given:** clean environment
- **When:** `cm .help`
- **Then:** The command listing section does not contain a `.help` entry
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--1-help)

---

### IT-8: Help output is stable across repeated invocations

- **Given:** clean environment
- **When:** `cm .help` (run 3 times)
- **Then:** all 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [commands.md — .help](../../../../docs/cli/commands.md#command--1-help)
