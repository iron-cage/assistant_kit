# Test: `tools`

Integration test planning for the `tools` command. See [command/08_tools.md](../../../../docs/cli/command/08_tools.md) for specification.

`tools` lists all 26 Claude Code built-in tools with name, category, and description in a
plain-style table. Tests verify that the table is printed with correct tool names, categories,
and a caption; that help flags work; and that `tools` is listed in main help output.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `clr tools` exits 0 | Happy path |
| IT-2 | Stdout lists core tool names | Content |
| IT-3 | Stdout lists tool categories | Content |
| IT-4 | Stdout contains table caption with count | Content |
| IT-5 | `clr tools --help` exits 0 | Help flag |
| IT-6 | Stdout lists scheduling and mode tools | Content |
| IT-7 | `clr --help` mentions `tools` command | Help listing |
| IT-8 | `clr tools -h` exits 0 | Help flag |
| IT-9 | `clr tools <unexpected-arg>` exits 1 | Error rejection |

## Test Coverage Summary

- Happy path: 1 test (IT-1)
- Content: 4 tests (IT-2, IT-3, IT-4, IT-6)
- Help flag: 2 tests (IT-5, IT-8)
- Help listing: 1 test (IT-7)
- Error rejection: 1 test (IT-9)

**Total:** 9 tests

---

### IT-1: `clr tools` exits 0

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout is non-empty (table is printed)
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-2: Stdout lists core tool names

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout contains "Read", "Write", "Edit", "Bash", and "Agent"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-3: Stdout lists tool categories

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout contains "File Operations", "Shell", "Search", and "Web"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-4: Stdout contains table caption with count

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout contains "Claude Code Tools" and "26"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-5: `--help` flag

- **Command:** `clr tools --help`
- **Expected behavior:** exit 0; stdout contains "tools"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-6: Stdout lists scheduling and mode tools

- **Command:** `clr tools`
- **Expected behavior:** exit 0; stdout contains "CronCreate", "CronDelete", "CronList", "EnterPlanMode", and "ExitPlanMode"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-7: `clr --help` mentions `tools` command

- **Command:** `clr --help`
- **Expected behavior:** exit 0; stdout contains "tools"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-8: `-h` short flag

- **Command:** `clr tools -h`
- **Expected behavior:** exit 0; stdout contains "tools"
- **Exit:** 0
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)

---

### IT-9: Unexpected argument rejected

- **Command:** `clr tools some-unknown-arg`
- **Expected behavior:** exit 1; stderr contains "does not accept arguments"
- **Exit:** 1
- **Source:** [command/08_tools.md](../../../../docs/cli/command/08_tools.md)
