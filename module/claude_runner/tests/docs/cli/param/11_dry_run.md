# Parameter :: `--dry-run`

Edge case tests for the dry-run flag. Tests validate command preview behavior, exit code, and that no execution occurs.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--11---dry-run)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--dry-run` prints command without executing | Preview Mode |
| EC-2 | Without `--dry-run` → execution attempted (not preview) | Default |
| EC-3 | `--dry-run` output goes to stdout | Output Stream |
| EC-4 | `--dry-run` exit code is always 0 (preview succeeds) | Exit Code |
| EC-5 | `--help` lists `--dry-run` | Documentation |
| EC-6 | `--dry-run` with all other flags → full command preview shown | Interaction |

## Test Coverage Summary

- Preview Mode: 1 test (EC-1)
- Default: 1 test (EC-2)
- Output Stream: 1 test (EC-3)
- Exit Code: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Interaction: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: `--dry-run` prints command, no execution:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Command line printed to stdout; claude NOT actually invoked
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--11---dry-run)
---

### EC-2: Without `--dry-run` → real execution (not preview):

- **Given:** clean environment (note: actual claude invocation will occur without --dry-run)
- **When:** omit `--dry-run` flag in any clr invocation
- **Then:** Claude subprocess is invoked (not just previewed); test verifies execution path differs
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--11---dry-run)
---

### EC-3: `--dry-run` output goes to stdout:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Command preview on stdout; stderr is empty
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--11---dry-run)
---

### EC-4: `--dry-run` exit code is 0:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Process exits with code 0
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--11---dry-run)
---

### EC-5: `--help` lists `--dry-run`:

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--dry-run`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md#command--2-help)
---

### EC-6: `--dry-run` with multiple flags → full preview:

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink --no-effort-max --verbose "Fix bug"`
- **Then:** All flags visible in command preview; no execution
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--11---dry-run)
