# Parameter :: `--no-skip-permissions`

Edge case tests for the permission bypass flag. Tests validate the flag's effect on the assembled command and help documentation.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--5---no-skip-permissions)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Default → `--dangerously-skip-permissions` in assembled command | Default |
| EC-2 | `--no-skip-permissions` → skip flag absent from assembled command | Suppression |
| EC-3 | `--no-skip-permissions` with message → message still forwarded | Interaction |
| EC-4 | `--no-skip-permissions` without message → accepted, no error | Edge Case |
| EC-5 | `--help` lists `--no-skip-permissions` | Documentation |
| EC-6 | `--no-skip-permissions` + `--verbose` → both flags coexist | Interaction |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Suppression: 1 test (EC-2)
- Interaction: 2 tests (EC-3, EC-6)
- Edge Case: 1 test (EC-4)
- Documentation: 1 test (EC-5)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: Default → `--dangerously-skip-permissions` injected:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--dangerously-skip-permissions`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--5---no-skip-permissions)
---

### EC-2: `--no-skip-permissions` suppresses the skip flag:

- **Given:** clean environment
- **When:** `clr --dry-run --no-skip-permissions "Fix bug"`
- **Then:** Assembled command does NOT contain `--dangerously-skip-permissions`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--5---no-skip-permissions)
---

### EC-3: `--no-skip-permissions` + message → message forwarded:

- **Given:** clean environment
- **When:** `clr --dry-run --no-skip-permissions "Explain this"`
- **Then:** Message present in assembled command; skip flag absent
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--5---no-skip-permissions)
---

### EC-4: `--no-skip-permissions` without message → no error:

- **Given:** clean environment
- **When:** `clr --dry-run --no-skip-permissions`
- **Then:** Exit 0; assembled command built without skip flag
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--5---no-skip-permissions)
---

### EC-5: `--help` lists `--no-skip-permissions`:

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--no-skip-permissions`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md#command--2-help)
---

### EC-6: `--no-skip-permissions` + `--verbose` → both coexist:

- **Given:** clean environment
- **When:** `clr --dry-run --no-skip-permissions --verbose "Fix bug"`
- **Then:** Assembled command contains `--verbose`; does NOT contain `--dangerously-skip-permissions`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--5---no-skip-permissions)
