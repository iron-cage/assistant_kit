# Parameter :: `--verbosity`

Edge case tests for the runner verbosity level parameter. Tests validate level range enforcement and output gating.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--12---verbosity)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--verbosity 0` → minimal runner output | Boundary Values |
| EC-2 | `--verbosity 5` → maximum runner output | Boundary Values |
| EC-3 | `--verbosity 6` → exit 1 (out of range) | Boundary Values |
| EC-4 | `--verbosity` without value → exit 1 | Missing Value |
| EC-5 | Default verbosity (level 3) applied when unset | Default |
| EC-6 | `--verbosity` with non-numeric value → exit 1 | Type Validation |

## Test Coverage Summary

- Boundary Values: 3 tests (EC-1, EC-2, EC-3)
- Missing Value: 1 test (EC-4)
- Default: 1 test (EC-5)
- Type Validation: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: `--verbosity 0` → minimal output:

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 0 "Fix bug"`
- **Then:** Runner output minimized; command preview still shown
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--12---verbosity)
---

### EC-2: `--verbosity 5` → maximum output:

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 5 "Fix bug"`
- **Then:** Maximum runner diagnostic output shown; exit 0
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--12---verbosity)
---

### EC-3: `--verbosity 6` → exit 1:

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 6 "Fix bug"`
- **Then:** Exit 1; error indicating verbosity must be 0–5
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--12---verbosity)
---

### EC-4: `--verbosity` without value → exit 1:

- **Given:** clean environment
- **When:** `clr --verbosity`
- **Then:** Exit 1; error about missing `--verbosity` value
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--12---verbosity)
---

### EC-5: Default verbosity level 3 when unset:

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Runner applies default verbosity level 3 behavior; no error
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--12---verbosity)
---

### EC-6: Non-numeric value → exit 1:

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity high "Fix bug"`
- **Then:** Exit 1; error indicating `--verbosity` requires a numeric value 0–5
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--12---verbosity)
