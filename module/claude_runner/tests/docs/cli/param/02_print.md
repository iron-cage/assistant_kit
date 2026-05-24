# Parameter :: `--print` / `-p`

Edge case tests for the print mode flag. Tests validate auto-enable with message, explicit alias, and mutual exclusivity.

**Source:** [002_print.md](../../../../docs/cli/param/002_print.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Message without `--print` → print mode auto-enabled | Behavioral Divergence |
| EC-2 | `-p` alias behaves identically to `--print` | Alias |
| EC-3 | `--print` without message → exit 1 | Boundary Values |
| EC-4 | Message + `--interactive` disables auto-print | Behavioral Divergence |
| EC-5 | `--print` explicit + message → same as default | Redundant Flag |
| EC-6 | `--help` output contains `--print` / `-p` | Documentation |

## Test Coverage Summary

- Alias: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Behavioral Divergence: 2 tests (EC-1, EC-4)
- Redundant Flag: 1 test (EC-5)
- Documentation: 1 test (EC-6)

**Total:** 6 edge cases


## Test Cases
---

### EC-1: Message auto-enables print mode

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--print`
- **Exit:** 0
- **Source:** [002_print.md](../../../../docs/cli/param/002_print.md)
- **Commands:** run, ask
---

### EC-2: `-p` alias behaves identically to `--print`

- **Given:** clean environment
- **When:** `clr --dry-run -p "Fix bug"` and `clr --dry-run --print "Fix bug"`
- **Then:** Both assembled commands are byte-identical
- **Exit:** 0
- **Source:** [002_print.md](../../../../docs/cli/param/002_print.md)
- **Commands:** run, ask
---

### EC-3: `--print` without message → exit 1

- **Given:** clean environment
- **When:** `clr --print`
- **Then:** Exit code 1; error message indicates print mode requires a message
- **Exit:** 1
- **Source:** [002_print.md](../../../../docs/cli/param/002_print.md)
- **Commands:** run, ask
---

### EC-4: Message + `--interactive` disables auto-print

- **Given:** clean environment
- **When:** `clr --dry-run --interactive "Fix bug"`
- **Then:** Assembled command does NOT contain `--print`
- **Exit:** 0
- **Source:** [002_print.md](../../../../docs/cli/param/002_print.md)
- **Commands:** run, ask
---

### EC-5: `--print` explicit + message → same as default

- **Given:** clean environment
- **When:** `clr --dry-run --print "Fix bug"`
- **Then:** Assembled command contains `--print`; identical behavior to omitting the flag
- **Exit:** 0
- **Source:** [002_print.md](../../../../docs/cli/param/002_print.md)
- **Commands:** run, ask
---

### EC-6: `--help` lists `--print` / `-p`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--print` and `-p`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, ask
