# Parameter :: `--allowed-tools`

Edge case tests for the allowed tools parameter. Tests validate value forwarding, missing-value rejection, and help documentation.

**Source:** [063_allowed_tools.md](../../../../docs/cli/param/063_allowed_tools.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--allowed-tools "Read,Edit"` → flag forwarded to claude | Behavioral Divergence |
| EC-2 | `--allowed-tools` without value → exit 1 | Missing Value |
| EC-3 | `--allowed-tools` at end of argv → exit 1 | Boundary Values |
| EC-4 | Any string accepted (no validation) | Permissive |
| EC-5 | `--help` lists `--allowed-tools` | Documentation |
| EC-6 | Without `--allowed-tools` → no `--allowed-tools` flag in assembled command | Behavioral Divergence |
| EC-7 | `CLR_ALLOWED_TOOLS="Read,Edit"` env var → forwarded | Env Var |

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

### EC-1: `--allowed-tools "Read,Edit"` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --allowed-tools "Read,Edit" "Fix bug"`
- **Then:** Assembled command contains `--allowed-tools Read,Edit`
- **Exit:** 0
- **Source:** [063_allowed_tools.md](../../../../docs/cli/param/063_allowed_tools.md)
- **Commands:** run, ask
---

### EC-2: `--allowed-tools` without value → exit 1

- **Given:** clean environment
- **When:** `clr --allowed-tools`
- **Then:** Exit 1; error about missing `--allowed-tools` value
- **Exit:** 1
- **Source:** [063_allowed_tools.md](../../../../docs/cli/param/063_allowed_tools.md)
- **Commands:** run, ask
---

### EC-3: `--allowed-tools` at end of argv → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --allowed-tools`
- **Then:** Exit 1; error about missing `--allowed-tools` value
- **Exit:** 1
- **Source:** [063_allowed_tools.md](../../../../docs/cli/param/063_allowed_tools.md)
- **Commands:** run, ask
---

### EC-4: Any tool string accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --allowed-tools "Bash(git:*),Read" "Fix bug"`
- **Then:** Assembled command contains `--allowed-tools`; no rejection
- **Exit:** 0
- **Source:** [063_allowed_tools.md](../../../../docs/cli/param/063_allowed_tools.md)
- **Commands:** run, ask
---

### EC-5: `--help` lists `--allowed-tools`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--allowed-tools`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, ask
---

### EC-6: Without `--allowed-tools` → no `--allowed-tools` flag in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--allowed-tools`
- **Exit:** 0
- **Source:** [063_allowed_tools.md](../../../../docs/cli/param/063_allowed_tools.md)
- **Commands:** run, ask
---

### EC-7: `CLR_ALLOWED_TOOLS="Read,Edit"` env var → forwarded

- **Given:** `CLR_ALLOWED_TOOLS=Read,Edit`
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--allowed-tools Read,Edit`
- **Exit:** 0
- **Source:** [063_allowed_tools.md](../../../../docs/cli/param/063_allowed_tools.md)
- **Commands:** run, ask
