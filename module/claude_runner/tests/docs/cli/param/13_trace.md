# Parameter :: `--trace`

Edge case tests for the trace flag. Tests validate command echoing to stderr before execution.

**Source:** [13_trace.md](../../../../docs/cli/param/13_trace.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--trace` → assembled command echoed to stderr; exit varies with claude availability | Behavioral Divergence |
| EC-2 | Without `--trace` → no command echo to stderr | Behavioral Divergence |
| EC-3 | `--trace` + `--dry-run` → preview shown but trace still on stderr | Interaction |
| EC-4 | `--trace` without message → trace output on stderr; no error | Edge Case |
| EC-5 | `--help` lists `--trace` | Documentation |
| EC-6 | `--trace` + env vars → env vars included in trace output | Trace Content |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Interaction: 1 test (EC-3)
- Edge Case: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Trace Content: 1 test (EC-6)

**Total:** 6 edge cases


## Test Cases
---

### EC-1: `--trace` → command echoed to stderr before invocation

- **Given:** clean environment
- **When:** `clr --trace "Fix bug"` (no `--dry-run`; trace fires before invocation)
- **Then:** Stderr contains assembled command (trace output written before claude is invoked); exit varies with claude availability
- **Exit:** 0 (claude present) or 1 (claude absent in test environment)
- **Source:** [13_trace.md](../../../../docs/cli/param/13_trace.md)
---

### EC-2: Without `--trace` → no command echo

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Stderr is empty (no command echo)
- **Exit:** 0
- **Source:** [13_trace.md](../../../../docs/cli/param/13_trace.md)
---

### EC-3: `--trace` + `--dry-run` → preview on stdout; trace NOT on stderr (dry-run wins)

- **Given:** clean environment
- **When:** `clr --trace --dry-run "Fix bug"`
- **Then:** Command preview on stdout; stderr is EMPTY (`handle_dry_run` returns before trace fires)
- **Exit:** 0
- **Source:** [13_trace.md](../../../../docs/cli/param/13_trace.md)
---

### EC-4: `--trace --dry-run` without message → stdout preview; stderr empty (dry-run wins)

- **Given:** clean environment
- **When:** `clr --trace --dry-run`
- **Then:** Exit 0; assembled command on stdout (dry-run output); stderr is EMPTY (trace does not fire when dry-run wins)
- **Exit:** 0
- **Source:** [13_trace.md](../../../../docs/cli/param/13_trace.md)
---

### EC-5: `--help` lists `--trace`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--trace`
- **Exit:** 0
- **Source:** [command.md](../../../../docs/cli/command.md#command--2-help)
---

### EC-6: `--trace` output includes environment context (without `--dry-run`)

- **Given:** clean environment
- **When:** `clr --trace "Fix bug"` (no `--dry-run`; claude absent in test environment)
- **Then:** Stderr contains env vars and assembled command line (trace fires before invocation attempt); stdout empty or shows error from failed subprocess
- **Exit:** 1 (claude absent in test environment)
- **Source:** [13_trace.md](../../../../docs/cli/param/13_trace.md)
