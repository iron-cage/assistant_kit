# Parameter :: `--model`

Edge case tests for the model selection parameter. Tests validate value forwarding, missing-value rejection, and help documentation.

**Source:** [03_model.md](../../../../docs/cli/param/03_model.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--model sonnet` → flag forwarded to claude | Behavioral Divergence |
| EC-2 | `--model` without value → exit 1 | Missing Value |
| EC-3 | `--model` at end of argv → exit 1 | Boundary Values |
| EC-4 | Any model string accepted (no validation) | Permissive |
| EC-5 | `--help` lists `--model` | Documentation |
| EC-6 | `--model` with message → both forwarded correctly | Interaction |
| EC-7 | Without `--model` → no `--model` flag in assembled command | Behavioral Divergence |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-7)
- Missing Value: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Permissive: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Interaction: 1 test (EC-6)

**Total:** 7 edge cases


## Test Cases
---

### EC-1: `--model sonnet` → forwarded to assembled command

- **Given:** clean environment
- **When:** `clr --dry-run --model sonnet "Fix bug"`
- **Then:** Assembled command contains `--model sonnet`
- **Exit:** 0
- **Source:** [03_model.md](../../../../docs/cli/param/03_model.md)
---

### EC-2: `--model` without value → exit 1

- **Given:** clean environment
- **When:** `clr --model`
- **Then:** Exit 1; error about missing `--model` value
- **Exit:** 1
- **Source:** [03_model.md](../../../../docs/cli/param/03_model.md)
---

### EC-3: `--model` at end of argv → exit 1

- **Given:** clean environment
- **When:** `clr "Fix bug" --model`
- **Then:** Exit 1; error about missing `--model` value
- **Exit:** 1
- **Source:** [03_model.md](../../../../docs/cli/param/03_model.md)
---

### EC-4: Any model string accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --model custom-model-xyz "Fix bug"`
- **Then:** Assembled command contains `--model custom-model-xyz`; no rejection
- **Exit:** 0
- **Source:** [03_model.md](../../../../docs/cli/param/03_model.md)
---

### EC-5: `--help` lists `--model`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--model`
- **Exit:** 0
- **Source:** [command.md](../../../../docs/cli/command.md#command--2-help)
---

### EC-6: `--model` + message → both forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --model opus "Explain this"`
- **Then:** Assembled command contains both `--model opus` and the message argument
- **Exit:** 0
- **Source:** [03_model.md](../../../../docs/cli/param/03_model.md)
---

### EC-7: Without `--model` → no `--model` flag in assembled command

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--model`; claude uses its own default model
- **Exit:** 0
- **Source:** [03_model.md](../../../../docs/cli/param/03_model.md)
