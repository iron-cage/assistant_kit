# Parameter :: `--max-budget-usd`

Edge case tests for the max budget parameter. Tests validate value forwarding, missing-value rejection, and help documentation.

**Source:** [065_max_budget_usd.md](../../../../docs/cli/param/065_max_budget_usd.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--max-budget-usd 5.00` → flag forwarded to claude | Behavioral Divergence |
| EC-2 | `--max-budget-usd` without value → exit 1 | Missing Value |
| EC-3 | `--max-budget-usd` at end of argv → exit 1 | Boundary Values |
| EC-4 | Any numeric string accepted (no validation) | Permissive |
| EC-5 | `--help` lists `--max-budget-usd` | Documentation |
| EC-6 | Without `--max-budget-usd` → no `--max-budget-usd` flag in assembled command | Behavioral Divergence |
| EC-7 | `CLR_MAX_BUDGET_USD=1.50` env var → forwarded | Env Var |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-6)
- Missing Value: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Permissive: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Env Var: 1 test (EC-7)

**Total:** 7 edge cases

## Test Cases
---

### EC-1: `--max-budget-usd 5.00` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --max-budget-usd 5.00 "Fix bug"`
- **Then:** Assembled command contains `--max-budget-usd 5.00`
- **Exit:** 0
- **Source:** [065_max_budget_usd.md](../../../../docs/cli/param/065_max_budget_usd.md)
- **Commands:** run, ask
---

### EC-2: `--max-budget-usd` without value → exit 1

- **Given:** clean environment
- **When:** `clr --max-budget-usd`
- **Then:** Exit 1; error about missing `--max-budget-usd` value
- **Exit:** 1
- **Source:** [065_max_budget_usd.md](../../../../docs/cli/param/065_max_budget_usd.md)
- **Commands:** run, ask
---

### EC-3: `--max-budget-usd` at end of argv → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --max-budget-usd`
- **Then:** Exit 1; error about missing `--max-budget-usd` value
- **Exit:** 1
- **Source:** [065_max_budget_usd.md](../../../../docs/cli/param/065_max_budget_usd.md)
- **Commands:** run, ask
---

### EC-4: Any numeric string accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --max-budget-usd 0.01 "Fix bug"`
- **Then:** Assembled command contains `--max-budget-usd 0.01`; no rejection
- **Exit:** 0
- **Source:** [065_max_budget_usd.md](../../../../docs/cli/param/065_max_budget_usd.md)
- **Commands:** run, ask
---

### EC-5: `--help` lists `--max-budget-usd`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--max-budget-usd`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, ask
---

### EC-6: Without `--max-budget-usd` → no `--max-budget-usd` flag in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--max-budget-usd`
- **Exit:** 0
- **Source:** [065_max_budget_usd.md](../../../../docs/cli/param/065_max_budget_usd.md)
- **Commands:** run, ask
---

### EC-7: `CLR_MAX_BUDGET_USD=1.50` env var → forwarded

- **Given:** `CLR_MAX_BUDGET_USD=1.50`
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--max-budget-usd 1.50`
- **Exit:** 0
- **Source:** [065_max_budget_usd.md](../../../../docs/cli/param/065_max_budget_usd.md)
- **Commands:** run, ask
