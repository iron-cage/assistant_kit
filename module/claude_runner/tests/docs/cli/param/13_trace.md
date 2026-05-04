# Parameter :: `--trace`

Edge case tests for the trace flag. Tests validate command echoing to stderr before execution.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--13---trace)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--trace` → assembled command echoed to stderr before execution | Happy Path |
| EC-2 | Without `--trace` → no command echo to stderr | Default |
| EC-3 | `--trace` + `--dry-run` → preview shown but trace still on stderr | Interaction |
| EC-4 | `--trace` without message → trace output on stderr; no error | Edge Case |
| EC-5 | `--help` lists `--trace` | Documentation |
| EC-6 | `--trace` + env vars → env vars included in trace output | Trace Content |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Default: 1 test (EC-2)
- Interaction: 1 test (EC-3)
- Edge Case: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Trace Content: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: `--trace` → command echoed to stderr:

- **Given:** clean environment
- **When:** `clr --trace "Fix bug"`
- **Then:** Assembled command printed to stderr before claude invocation
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--13---trace)
---

### EC-2: Without `--trace` → no command echo:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Stderr is empty (no command echo)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--13---trace)
---

### EC-3: `--trace` + `--dry-run` → trace on stderr, preview on stdout:

- **Given:** clean environment
- **When:** `clr --trace --dry-run "Fix bug"`
- **Then:** Command preview on stdout; command also echoed on stderr
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--13---trace)
---

### EC-4: `--trace` without message → trace output, no error:

- **Given:** clean environment
- **When:** `clr --trace --dry-run`
- **Then:** Exit 0; assembled command echoed to stderr; no rejection
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--13---trace)
---

### EC-5: `--help` lists `--trace`:

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--trace`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md#command--2-help)
---

### EC-6: `--trace` output includes environment context:

- **Given:** clean environment
- **When:** `clr --trace --dry-run "Fix bug"`
- **Then:** Stderr trace output includes env vars and assembled command line
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--13---trace)
