# Parameter :: `--disallowed-tools`

Edge case tests for the disallowed tools parameter. Tests validate value forwarding, missing-value rejection, and help documentation.

**Source:** [064_disallowed_tools.md](../../../../docs/cli/param/064_disallowed_tools.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--disallowed-tools "Bash"` → flag forwarded to claude | Behavioral Divergence |
| EC-2 | `--disallowed-tools` without value → exit 1 | Missing Value |
| EC-3 | `--disallowed-tools` at end of argv → exit 1 | Boundary Values |
| EC-4 | Any string accepted (no validation) | Permissive |
| EC-5 | `--help` lists `--disallowed-tools` | Documentation |
| EC-6 | Without `--disallowed-tools` → no `--disallowed-tools` flag in assembled command | Behavioral Divergence |
| EC-7 | `CLR_DISALLOWED_TOOLS="Bash"` env var → forwarded | Env Var |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-6)
- Missing Value: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Permissive: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Env Var: 1 test (EC-7)

**Total:** 7 edge cases

## Test Cases
---

### EC-1: `--disallowed-tools "Bash"` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --disallowed-tools "Bash" "Fix bug"`
- **Then:** Assembled command contains `--disallowed-tools Bash`
- **Exit:** 0
- **Source:** [064_disallowed_tools.md](../../../../docs/cli/param/064_disallowed_tools.md)
- **Commands:** run, ask
---

### EC-2: `--disallowed-tools` without value → exit 1

- **Given:** clean environment
- **When:** `clr --disallowed-tools`
- **Then:** Exit 1; error about missing `--disallowed-tools` value
- **Exit:** 1
- **Source:** [064_disallowed_tools.md](../../../../docs/cli/param/064_disallowed_tools.md)
- **Commands:** run, ask
---

### EC-3: `--disallowed-tools` at end of argv → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --disallowed-tools`
- **Then:** Exit 1; error about missing `--disallowed-tools` value
- **Exit:** 1
- **Source:** [064_disallowed_tools.md](../../../../docs/cli/param/064_disallowed_tools.md)
- **Commands:** run, ask
---

### EC-4: Any tool string accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --disallowed-tools "Write,Edit" "Fix bug"`
- **Then:** Assembled command contains `--disallowed-tools`; no rejection
- **Exit:** 0
- **Source:** [064_disallowed_tools.md](../../../../docs/cli/param/064_disallowed_tools.md)
- **Commands:** run, ask
---

### EC-5: `--help` lists `--disallowed-tools`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--disallowed-tools`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, ask
---

### EC-6: Without `--disallowed-tools` → no `--disallowed-tools` flag in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--disallowed-tools`
- **Exit:** 0
- **Source:** [064_disallowed_tools.md](../../../../docs/cli/param/064_disallowed_tools.md)
- **Commands:** run, ask
---

### EC-7: `CLR_DISALLOWED_TOOLS="Bash"` env var → forwarded

- **Given:** `CLR_DISALLOWED_TOOLS=Bash`
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--disallowed-tools Bash`
- **Exit:** 0
- **Source:** [064_disallowed_tools.md](../../../../docs/cli/param/064_disallowed_tools.md)
- **Commands:** run, ask
