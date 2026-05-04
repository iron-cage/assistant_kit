# Parameter :: `--max-tokens`

Edge case tests for the max tokens parameter. Tests validate numeric value forwarding, boundary rejection, and help documentation.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--9---max-tokens)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--max-tokens 200000` → forwarded to assembled command | Happy Path |
| EC-2 | `--max-tokens` without value → exit 1 | Missing Value |
| EC-3 | Default → `--max-tokens 200000` in assembled command | Default |
| EC-4 | `--max-tokens 0` → accepted (zero is valid) | Boundary Values |
| EC-5 | `--help` lists `--max-tokens` | Documentation |
| EC-6 | `--max-tokens` with non-numeric value → exit 1 | Type Validation |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Missing Value: 1 test (EC-2)
- Default: 1 test (EC-3)
- Boundary Values: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Type Validation: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: `--max-tokens 200000` forwarded:

- **Given:** clean environment
- **When:** `clr --dry-run --max-tokens 200000 "Fix bug"`
- **Then:** Assembled command contains `--max-tokens 200000`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--9---max-tokens)
---

### EC-2: `--max-tokens` without value → exit 1:

- **Given:** clean environment
- **When:** `clr --max-tokens`
- **Then:** Exit 1; error about missing `--max-tokens` value
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--9---max-tokens)
---

### EC-3: Default → `--max-tokens 200000` injected:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--max-tokens 200000`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--9---max-tokens)
---

### EC-4: `--max-tokens 0` accepted:

- **Given:** clean environment
- **When:** `clr --dry-run --max-tokens 0 "Fix bug"`
- **Then:** Assembled command contains `--max-tokens 0`; no rejection
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--9---max-tokens)
---

### EC-5: `--help` lists `--max-tokens`:

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--max-tokens`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md#command--2-help)
---

### EC-6: Non-numeric value → exit 1:

- **Given:** clean environment
- **When:** `clr --dry-run --max-tokens unlimited "Fix bug"`
- **Then:** Exit 1; error indicating `--max-tokens` requires a numeric value
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--9---max-tokens)
