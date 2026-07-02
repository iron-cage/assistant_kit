# Test: `.help` / `.` / empty argv

### Scope

- **Purpose**: Integration test cases for the `.help` command.
- **Responsibility**: Test case index and expected behavior for help triggers.
- **In Scope**: `.help`, `.`, empty argv, `.help` anywhere in argv.
- **Out of Scope**: Parameter edge cases (→ `../param/`), group interactions (→ `../param_group/`).

Integration test planning for help output triggers. See [command/readme.md](../../../../docs/cli/command/readme.md#command--1-help) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clv .` → help output, exit 0 | Alias |
| IT-2 | `clv` (empty argv) → help output, exit 0 | Empty State |
| IT-3 | `clv .help` → help output, exit 0 | Happy Path |
| IT-4 | `clv .status .help` → `.help` anywhere in argv triggers help, exit 0 | FR-02 |
| IT-5 | Help output goes to stdout; stderr is empty | Output Stream |
| IT-6 | Help output contains all visible command names | Content |
| IT-7 | `.help` does not appear in its own command listing | Visibility |
| IT-8 | Help output is stable across repeated invocations | Stability |
| IT-9 | Help output contains grouped section headers | Structure |

## Test Coverage Summary

- Happy Path: 1 test
- Alias: 1 test
- Empty State: 1 test
- FR-02 (anywhere-in-argv): 1 test
- Output Stream: 1 test
- Content: 1 test
- Visibility: 1 test
- Stability: 1 test
- Structure: 1 test

**Total:** 9 tests

---

### IT-1: `clv .` → help output, exit 0

- **Given:** clean environment
- **When:** `clv .`
- **Then:** Contains command listing; mentions `.help`.; help output shown
- **Exit:** 0
- **Source:** [command/readme.md — .help](../../../../docs/cli/command/readme.md#command--1-help), adapter.rs bare-dot handling

---

### IT-2: `clv` (empty argv) → help output, exit 0

- **Given:** clean environment
- **When:** `clv` (no arguments)
- **Then:** Same help output as `.help`.; help output shown
- **Exit:** 0
- **Source:** [command/readme.md — .help](../../../../docs/cli/command/readme.md#command--1-help), [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-3: `clv .help` → help output, exit 0

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** All visible commands listed; usage line present; help listing complete
- **Exit:** 0
- **Source:** [command/readme.md — .help](../../../../docs/cli/command/readme.md#command--1-help), [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### IT-4: `clv .status .help` → `.help` anywhere in argv triggers help, exit 0

- **Given:** clean environment
- **When:** `clv .status .help`
- **Then:** Help output (not `.status` output); help wins over `.status` when `.help` present anywhere
- **Exit:** 0
- **Source:** [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md), adapter.rs `.help`-anywhere detection

---

### IT-5: Help output goes to stdout; stderr is empty

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** stdout is non-empty; stderr is empty
- **Exit:** 0
- **Source:** [command/readme.md — .help](../../../../docs/cli/command/readme.md#command--1-help)

---

### IT-6: Help output contains all visible command names

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** All visible command names present in stdout output
- **Exit:** 0
- **Source:** [command/readme.md — .help](../../../../docs/cli/command/readme.md#command--1-help)

---

### IT-7: `.help` does not appear in its own command listing

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** The command listing section does not contain a `.help` entry
- **Exit:** 0
- **Source:** [command/readme.md — .help](../../../../docs/cli/command/readme.md#command--1-help)

---

### IT-8: Help output is stable across repeated invocations

- **Given:** clean environment
- **When:** `clv .help` (run 3 times)
- **Then:** all 3 stdout captures are byte-identical
- **Exit:** 0
- **Source:** [command/readme.md — .help](../../../../docs/cli/command/readme.md#command--1-help)

---

### IT-9: Help output contains grouped section headers

- **Given:** clean environment
- **When:** `clv .help`
- **Then:** stdout contains all 4 section header strings: "Version Management", "Settings & Config", "Process Lifecycle", "Status"
- **Exit:** 0
- **Source:** [command/root.md — .help](../../../../docs/cli/command/root.md#command--1-help), [feature/005_cli_design.md](../../../../docs/feature/005_cli_design.md)

---

### Source Functions

| Function | File |
|----------|------|
| `tc079_help_command_exits_0` | `integration/framework_test.rs` |
| `tc080_help_lists_12_commands` | `integration/framework_test.rs` |
| `tc082_help_includes_available_commands_section` | `integration/framework_test.rs` |
| `tc091_unknown_command_exits_1` | `integration/framework_test.rs` |
| `tc093_empty_args_exits_0` | `integration/framework_test.rs` |
| `tc094_help_exits_0_and_shows_commands` | `integration/framework_test.rs` |
| `tc095_all_visible_commands_in_help` | `integration/framework_test.rs` |
| `tc01_dot_alias_shows_help` | `integration/read_commands_test.rs` |
| `tc02_empty_argv_shows_help` | `integration/read_commands_test.rs` |
