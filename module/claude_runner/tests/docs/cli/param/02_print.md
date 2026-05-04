# Parameter :: `--print` / `-p`

Edge case tests for the print mode flag. Tests validate auto-enable with message, explicit alias, and mutual exclusivity.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--2---print)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Message without `--print` → print mode auto-enabled | Default |
| EC-2 | `-p` alias behaves identically to `--print` | Alias |
| EC-3 | `--print` without message → exit 1 | Boundary Values |
| EC-4 | Message + `--interactive` disables auto-print | Interaction |
| EC-5 | `--print` explicit + message → same as default | Redundant Flag |
| EC-6 | `--help` output contains `--print` / `-p` | Documentation |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Alias: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Interaction: 1 test (EC-4)
- Redundant Flag: 1 test (EC-5)
- Documentation: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: Message auto-enables print mode:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--print`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--2---print)
---

### EC-2: `-p` alias behaves identically to `--print`:

- **Given:** clean environment
- **When:** `clr --dry-run -p "Fix bug"` and `clr --dry-run --print "Fix bug"`
- **Then:** Both assembled commands are byte-identical
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--2---print)
---

### EC-3: `--print` without message → exit 1:

- **Given:** clean environment
- **When:** `clr --print`
- **Then:** Exit code 1; error message indicates print mode requires a message
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--2---print)
---

### EC-4: Message + `--interactive` disables auto-print:

- **Given:** clean environment
- **When:** `clr --dry-run --interactive "Fix bug"`
- **Then:** Assembled command does NOT contain `--print`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--2---print)
---

### EC-5: `--print` explicit + message → same as default:

- **Given:** clean environment
- **When:** `clr --dry-run --print "Fix bug"`
- **Then:** Assembled command contains `--print`; identical behavior to omitting the flag
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--2---print)
---

### EC-6: `--help` lists `--print` / `-p`:

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--print` and `-p`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md#command--2-help)
