# Parameter :: `--interactive`

Edge case tests for the interactive TTY passthrough flag. Tests validate print-mode suppression and help documentation.

**Source:** [06_interactive.md](../../../../docs/cli/param/06_interactive.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Message without `--interactive` → print mode (--print present) | Behavioral Divergence |
| EC-2 | Message + `--interactive` → no `--print` in assembled command | Behavioral Divergence |
| EC-3 | `--interactive` without message → REPL with TTY (no --print) | Edge Case |
| EC-4 | `--interactive` + `--dry-run` → command preview, no --print | Interaction |
| EC-5 | `--help` lists `--interactive` | Documentation |
| EC-6 | `--interactive` with `--verbose` → both flags forwarded | Interaction |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 1 test (EC-3)
- Interaction: 2 tests (EC-4, EC-6)
- Documentation: 1 test (EC-5)

**Total:** 6 edge cases


## Test Cases
---

### EC-1: Without `--interactive` → `--print` present

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--print`
- **Exit:** 0
- **Source:** [06_interactive.md](../../../../docs/cli/param/06_interactive.md)
---

### EC-2: `--interactive` suppresses `--print`

- **Given:** clean environment
- **When:** `clr --dry-run --interactive "Fix bug"`
- **Then:** Assembled command does NOT contain `--print`
- **Exit:** 0
- **Source:** [06_interactive.md](../../../../docs/cli/param/06_interactive.md)
---

### EC-3: `--interactive` without message → REPL, no --print

- **Given:** clean environment
- **When:** `clr --dry-run --interactive`
- **Then:** Assembled command does NOT contain `--print`; no error
- **Exit:** 0
- **Source:** [06_interactive.md](../../../../docs/cli/param/06_interactive.md)
---

### EC-4: `--interactive` + `--dry-run` → preview, no --print

- **Given:** clean environment
- **When:** `clr --dry-run --interactive "Fix bug"`
- **Then:** Command preview shown; `--print` absent from preview
- **Exit:** 0
- **Source:** [06_interactive.md](../../../../docs/cli/param/06_interactive.md)
---

### EC-5: `--help` lists `--interactive`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--interactive`
- **Exit:** 0
- **Source:** [command.md](../../../../docs/cli/command.md#command--2-help)
---

### EC-6: `--interactive` + `--verbose` → both forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --interactive --verbose "Fix bug"`
- **Then:** Assembled command contains `--verbose`; does NOT contain `--print`
- **Exit:** 0
- **Source:** [06_interactive.md](../../../../docs/cli/param/06_interactive.md)
