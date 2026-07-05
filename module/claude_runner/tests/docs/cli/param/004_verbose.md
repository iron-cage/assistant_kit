# Parameter :: `--verbose`

Edge case tests for the verbose flag. Tests validate passthrough to claude subprocess and help documentation.

**Source:** [004_verbose.md](../../../../docs/cli/param/004_verbose.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--verbose` → flag forwarded to assembled command | Behavioral Divergence |
| EC-2 | Without `--verbose` → flag absent from assembled command | Behavioral Divergence |
| EC-3 | `--verbose` with message → both present in assembled command | Interaction |
| EC-4 | `--verbose` standalone (no message) → flag forwarded | Edge Case |
| EC-5 | `--help` lists `--verbose` | Documentation |
| EC-6 | `--verbose` is idempotent (specified twice → no duplication) | Idempotent |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Interaction: 1 test (EC-3)
- Edge Case: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Idempotent: 1 test (EC-6)

**Total:** 6 edge cases


## Test Cases
---

### EC-1: `--verbose` forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --verbose "Fix bug"`
- **Then:** Assembled command contains `--verbose`
- **Exit:** 0
- **Source:** [004_verbose.md](../../../../docs/cli/param/004_verbose.md)
- **Commands:** run, ask
---

### EC-2: Without `--verbose` → absent from assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--verbose`
- **Exit:** 0
- **Source:** [004_verbose.md](../../../../docs/cli/param/004_verbose.md)
- **Commands:** run, ask
---

### EC-3: `--verbose` with message → both present

- **Given:** clean environment
- **When:** `clr --dry-run --verbose "Explain this"`
- **Then:** Assembled command contains `--verbose` and the message
- **Exit:** 0
- **Source:** [004_verbose.md](../../../../docs/cli/param/004_verbose.md)
- **Commands:** run, ask
---

### EC-4: `--verbose` without message → forwarded, no error

- **Given:** clean environment
- **When:** `clr --dry-run --verbose`
- **Then:** Exit 0; assembled command contains `--verbose`; no rejection
- **Exit:** 0
- **Source:** [004_verbose.md](../../../../docs/cli/param/004_verbose.md)
- **Commands:** run, ask
---

### EC-5: `--help` lists `--verbose`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--verbose`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask
---

### EC-6: `--verbose` specified twice → not duplicated

- **Given:** clean environment
- **When:** `clr --dry-run --verbose --verbose "Fix bug"`
- **Then:** Assembled command contains `--verbose` at most once (no duplication)
- **Exit:** 0
- **Source:** [004_verbose.md](../../../../docs/cli/param/004_verbose.md)
- **Commands:** run, ask
