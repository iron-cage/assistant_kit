# Parameter :: `--max-turns`

Edge case tests for the max turns parameter. Tests validate value forwarding, missing-value rejection, and help documentation.

**Source:** [062_max_turns.md](../../../../docs/cli/param/062_max_turns.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--max-turns 5` → flag forwarded to claude | Behavioral Divergence |
| EC-2 | `--max-turns` without value → exit 1 | Missing Value |
| EC-3 | `--max-turns` at end of argv → exit 1 | Boundary Values |
| EC-4 | Any numeric string accepted (no validation) | Permissive |
| EC-5 | `--help` lists `--max-turns` | Documentation |
| EC-6 | Without `--max-turns` → no `--max-turns` flag in assembled command | Behavioral Divergence |
| EC-7 | `CLR_MAX_TURNS=10` env var → forwarded | Env Var |

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

### EC-1: `--max-turns 5` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --max-turns 5 "Fix bug"`
- **Then:** Assembled command contains `--max-turns 5`
- **Exit:** 0
- **Source:** [062_max_turns.md](../../../../docs/cli/param/062_max_turns.md)
- **Commands:** run, ask
---

### EC-2: `--max-turns` without value → exit 1

- **Given:** clean environment
- **When:** `clr --max-turns`
- **Then:** Exit 1; error about missing `--max-turns` value
- **Exit:** 1
- **Source:** [062_max_turns.md](../../../../docs/cli/param/062_max_turns.md)
- **Commands:** run, ask
---

### EC-3: `--max-turns` at end of argv → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --max-turns`
- **Then:** Exit 1; error about missing `--max-turns` value
- **Exit:** 1
- **Source:** [062_max_turns.md](../../../../docs/cli/param/062_max_turns.md)
- **Commands:** run, ask
---

### EC-4: Any numeric string accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --max-turns 999 "Fix bug"`
- **Then:** Assembled command contains `--max-turns 999`; no rejection
- **Exit:** 0
- **Source:** [062_max_turns.md](../../../../docs/cli/param/062_max_turns.md)
- **Commands:** run, ask
---

### EC-5: `--help` lists `--max-turns`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--max-turns`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask
---

### EC-6: Without `--max-turns` → no `--max-turns` flag in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--max-turns`
- **Exit:** 0
- **Source:** [062_max_turns.md](../../../../docs/cli/param/062_max_turns.md)
- **Commands:** run, ask
---

### EC-7: `CLR_MAX_TURNS=10` env var → forwarded

- **Given:** `CLR_MAX_TURNS=10`
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--max-turns 10`
- **Exit:** 0
- **Source:** [062_max_turns.md](../../../../docs/cli/param/062_max_turns.md)
- **Commands:** run, ask
