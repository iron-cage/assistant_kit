# Parameter :: `--dry-run`

Edge case tests for the dry-run flag. Tests validate command preview behavior, exit code, and that no execution occurs.

**Source:** [11_dry_run.md](../../../../docs/cli/param/11_dry_run.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--dry-run` prints command without executing | Behavioral Divergence |
| EC-2 | Without `--dry-run` → execution attempted (not preview) | Behavioral Divergence |
| EC-3 | `--dry-run` output goes to stdout | Output Stream |
| EC-4 | `--dry-run` exit code is always 0 (preview succeeds) | Exit Code |
| EC-5 | `--help` lists `--dry-run` | Documentation |
| EC-6 | `--dry-run` with all other flags → full command preview shown | Interaction |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Output Stream: 1 test (EC-3)
- Exit Code: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Interaction: 1 test (EC-6)

**Total:** 6 edge cases


## Test Cases
---

### EC-1: `--dry-run` prints command, no execution

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Command line printed to stdout; claude NOT actually invoked
- **Exit:** 0
- **Source:** [11_dry_run.md](../../../../docs/cli/param/11_dry_run.md)
---

### EC-2: Without `--dry-run` → execution attempted, no preview output

- **Given:** clean environment
- **When:** `clr --trace "Fix bug"` (no `--dry-run`)
- **Then:** Stderr contains assembled command (trace confirms execution was attempted, not previewed); stdout does NOT contain the multi-line env+command dry-run preview; process exits non-zero if claude binary is absent from test environment
- **Exit:** 1 (claude absent in test environment)
- **Source:** [11_dry_run.md](../../../../docs/cli/param/11_dry_run.md)
---

### EC-3: `--dry-run` output goes to stdout

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Command preview on stdout; stderr is empty
- **Exit:** 0
- **Source:** [11_dry_run.md](../../../../docs/cli/param/11_dry_run.md)
---

### EC-4: `--dry-run` exit code is 0

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Process exits with code 0
- **Exit:** 0
- **Source:** [11_dry_run.md](../../../../docs/cli/param/11_dry_run.md)
---

### EC-5: `--help` lists `--dry-run`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--dry-run`
- **Exit:** 0
- **Source:** [command.md](../../../../docs/cli/command.md#command--2-help)
---

### EC-6: `--dry-run` with multiple flags → full preview

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink --no-effort-max --verbose "Fix bug"`
- **Then:** All flags visible in command preview; no execution
- **Exit:** 0
- **Source:** [11_dry_run.md](../../../../docs/cli/param/11_dry_run.md)
