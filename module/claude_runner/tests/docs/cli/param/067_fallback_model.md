# Parameter :: `--fallback-model`

Edge case tests for the fallback model parameter. Tests validate value forwarding, missing-value rejection, and help documentation.

**Source:** [067_fallback_model.md](../../../../docs/cli/param/067_fallback_model.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--fallback-model sonnet` → flag forwarded to claude | Behavioral Divergence |
| EC-2 | `--fallback-model` without value → exit 1 | Missing Value |
| EC-3 | `--fallback-model` at end of argv → exit 1 | Boundary Values |
| EC-4 | Any model string accepted (no validation) | Permissive |
| EC-5 | `--help` lists `--fallback-model` | Documentation |
| EC-6 | Without `--fallback-model` → no `--fallback-model` flag in assembled command | Behavioral Divergence |
| EC-7 | `CLR_FALLBACK_MODEL=haiku` env var → forwarded | Env Var |

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

### EC-1: `--fallback-model sonnet` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --fallback-model sonnet "Fix bug"`
- **Then:** Assembled command contains `--fallback-model sonnet`
- **Exit:** 0
- **Source:** [067_fallback_model.md](../../../../docs/cli/param/067_fallback_model.md)
- **Commands:** run, ask
---

### EC-2: `--fallback-model` without value → exit 1

- **Given:** clean environment
- **When:** `clr --fallback-model`
- **Then:** Exit 1; error about missing `--fallback-model` value
- **Exit:** 1
- **Source:** [067_fallback_model.md](../../../../docs/cli/param/067_fallback_model.md)
- **Commands:** run, ask
---

### EC-3: `--fallback-model` at end of argv → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --fallback-model`
- **Then:** Exit 1; error about missing `--fallback-model` value
- **Exit:** 1
- **Source:** [067_fallback_model.md](../../../../docs/cli/param/067_fallback_model.md)
- **Commands:** run, ask
---

### EC-4: Any model string accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --fallback-model custom-fallback "Fix bug"`
- **Then:** Assembled command contains `--fallback-model custom-fallback`; no rejection
- **Exit:** 0
- **Source:** [067_fallback_model.md](../../../../docs/cli/param/067_fallback_model.md)
- **Commands:** run, ask
---

### EC-5: `--help` lists `--fallback-model`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--fallback-model`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, ask
---

### EC-6: Without `--fallback-model` → no `--fallback-model` flag in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--fallback-model`
- **Exit:** 0
- **Source:** [067_fallback_model.md](../../../../docs/cli/param/067_fallback_model.md)
- **Commands:** run, ask
---

### EC-7: `CLR_FALLBACK_MODEL=haiku` env var → forwarded

- **Given:** `CLR_FALLBACK_MODEL=haiku`
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--fallback-model haiku`
- **Exit:** 0
- **Source:** [067_fallback_model.md](../../../../docs/cli/param/067_fallback_model.md)
- **Commands:** run, ask
