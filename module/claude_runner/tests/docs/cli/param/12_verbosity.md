# Parameter :: `--verbosity`

Edge case tests for the runner verbosity level parameter. Tests validate level range enforcement and output gating.

**Source:** [012_verbosity.md](../../../../docs/cli/param/012_verbosity.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--verbosity 0` → minimal runner output (lower boundary) | Behavioral Divergence |
| EC-2 | `--verbosity 5` → maximum runner output (upper boundary) | Behavioral Divergence |
| EC-3 | `--verbosity 6` → exit 1 (out of range) | Boundary Values |
| EC-4 | `--verbosity` without value → exit 1 | Missing Value |
| EC-5 | Default verbosity (level 3) applied when unset | Default |
| EC-6 | `--verbosity` with non-numeric value → exit 1 | Type Validation |
| EC-7 | `--verbosity -1` → exit 1 (below range) | Boundary Values |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Boundary Values: 2 tests (EC-3, EC-7)
- Missing Value: 1 test (EC-4)
- Default: 1 test (EC-5)
- Type Validation: 1 test (EC-6)

**Total:** 7 edge cases


## Test Cases
---

### EC-1: `--verbosity 0` → minimal output

- **Given:** clean environment with valid credentials
- **When:** `clr --print --verbosity 0 "echo test"`
- **Then:** Stdout contains Claude response only; stderr has no runner diagnostic lines
- **Exit:** 0
- **Source:** [012_verbosity.md](../../../../docs/cli/param/012_verbosity.md)
- **Commands:** run, ask
---

### EC-2: `--verbosity 5` → maximum output

- **Given:** clean environment with valid credentials
- **When:** `clr --print --verbosity 5 "echo test"`
- **Then:** Stdout contains Claude response; stderr has runner diagnostic lines including command assembly details
- **Exit:** 0
- **Source:** [012_verbosity.md](../../../../docs/cli/param/012_verbosity.md)
- **Commands:** run, ask
---

### EC-3: `--verbosity 6` → exit 1

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity 6 "Fix bug"`
- **Then:** Exit 1; error indicating verbosity must be 0–5
- **Exit:** 1
- **Source:** [012_verbosity.md](../../../../docs/cli/param/012_verbosity.md)
- **Commands:** run, ask
---

### EC-4: `--verbosity` without value → exit 1

- **Given:** clean environment
- **When:** `clr --verbosity`
- **Then:** Exit 1; error about missing `--verbosity` value
- **Exit:** 1
- **Source:** [012_verbosity.md](../../../../docs/cli/param/012_verbosity.md)
- **Commands:** run, ask
---

### EC-5: Default verbosity level 3 when unset

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Runner applies default verbosity level 3 behavior; no error
- **Exit:** 0
- **Source:** [012_verbosity.md](../../../../docs/cli/param/012_verbosity.md)
- **Commands:** run, ask
---

### EC-6: Non-numeric value → exit 1

- **Given:** clean environment
- **When:** `clr --dry-run --verbosity high "Fix bug"`
- **Then:** Exit 1; error indicating `--verbosity` requires a numeric value 0–5
- **Exit:** 1
- **Source:** [012_verbosity.md](../../../../docs/cli/param/012_verbosity.md)
- **Commands:** run, ask
---

### EC-7: `--verbosity -1` → exit 1 (below range)

- **Given:** clean environment
- **When:** `clr --verbosity -1 "Fix bug"`
- **Then:** Exit 1; error indicating verbosity value is out of range (minimum is 0)
- **Exit:** 1
- **Source:** [012_verbosity.md](../../../../docs/cli/param/012_verbosity.md)
- **Commands:** run, ask
