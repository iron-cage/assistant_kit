# Parameter :: `--session-dir`

Edge case tests for the session directory parameter. Tests validate path forwarding, missing-value rejection, and help documentation.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--10---session-dir)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--session-dir /path` → forwarded to assembled command | Happy Path |
| EC-2 | `--session-dir` without value → exit 1 | Missing Value |
| EC-3 | Default (no `--session-dir`) → flag absent from assembled command | Default |
| EC-4 | `--session-dir` + `--new-session` → both accepted | Interaction |
| EC-5 | `--help` lists `--session-dir` | Documentation |
| EC-6 | Non-existent path accepted without validation at runner layer | Permissive |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Missing Value: 1 test (EC-2)
- Default: 1 test (EC-3)
- Interaction: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Permissive: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: `--session-dir /path` forwarded:

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /tmp/sessions "Fix bug"`
- **Then:** Assembled command contains `--session-dir /tmp/sessions`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10---session-dir)
---

### EC-2: `--session-dir` without value → exit 1:

- **Given:** clean environment
- **When:** `clr --session-dir`
- **Then:** Exit 1; error about missing `--session-dir` value
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10---session-dir)
---

### EC-3: Default → no `--session-dir` in assembled command:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--session-dir`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10---session-dir)
---

### EC-4: `--session-dir` + `--new-session` → no conflict:

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /tmp/sessions --new-session "Fix bug"`
- **Then:** Both flags present in assembled command; no error
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10---session-dir)
---

### EC-5: `--help` lists `--session-dir`:

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--session-dir`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md#command--2-help)
---

### EC-6: Non-existent path accepted without validation:

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /no/such/dir "Fix bug"`
- **Then:** Exit 0; assembled command contains `--session-dir /no/such/dir` (no path validation)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--10---session-dir)
