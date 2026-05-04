# Parameter :: `--verbose`

Edge case tests for the verbose flag. Tests validate passthrough to claude subprocess and help documentation.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--4---verbose)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--verbose` → flag forwarded to assembled command | Happy Path |
| EC-2 | Without `--verbose` → flag absent from assembled command | Default |
| EC-3 | `--verbose` with message → both present in assembled command | Interaction |
| EC-4 | `--verbose` standalone (no message) → flag forwarded | Edge Case |
| EC-5 | `--help` lists `--verbose` | Documentation |
| EC-6 | `--verbose` is idempotent (specified twice → no duplication) | Idempotent |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Default: 1 test (EC-2)
- Interaction: 1 test (EC-3)
- Edge Case: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Idempotent: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: `--verbose` forwarded to assembled command:

- **Given:** clean environment
- **When:** `clr --dry-run --verbose "Fix bug"`
- **Then:** Assembled command contains `--verbose`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--4---verbose)
---

### EC-2: Without `--verbose` → absent from assembled command:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--verbose`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--4---verbose)
---

### EC-3: `--verbose` with message → both present:

- **Given:** clean environment
- **When:** `clr --dry-run --verbose "Explain this"`
- **Then:** Assembled command contains `--verbose` and the message
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--4---verbose)
---

### EC-4: `--verbose` without message → forwarded, no error:

- **Given:** clean environment
- **When:** `clr --dry-run --verbose`
- **Then:** Exit 0; assembled command contains `--verbose`; no rejection
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--4---verbose)
---

### EC-5: `--help` lists `--verbose`:

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--verbose`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md#command--2-help)
---

### EC-6: `--verbose` specified twice → not duplicated:

- **Given:** clean environment
- **When:** `clr --dry-run --verbose --verbose "Fix bug"`
- **Then:** Assembled command contains `--verbose` at most once (no duplication)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--4---verbose)
