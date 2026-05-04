# Parameter :: `--model`

Edge case tests for the model selection parameter. Tests validate value forwarding, missing-value rejection, and help documentation.

**Source:** [params.md](../../../../docs/cli/params.md#parameter--3---model)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--model sonnet` → flag forwarded to claude | Happy Path |
| EC-2 | `--model` without value → exit 1 | Missing Value |
| EC-3 | `--model` at end of argv → exit 1 | Boundary Values |
| EC-4 | Any model string accepted (no validation) | Permissive |
| EC-5 | `--help` lists `--model` | Documentation |
| EC-6 | `--model` with message → both forwarded correctly | Interaction |

## Test Coverage Summary

- Happy Path: 1 test (EC-1)
- Missing Value: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Permissive: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Interaction: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases
---

### EC-1: `--model sonnet` → forwarded to assembled command:

- **Given:** clean environment
- **When:** `clr --dry-run --model sonnet "Fix bug"`
- **Then:** Assembled command contains `--model sonnet`
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--3---model)
---

### EC-2: `--model` without value → exit 1:

- **Given:** clean environment
- **When:** `clr --model`
- **Then:** Exit 1; error about missing `--model` value
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--3---model)
---

### EC-3: `--model` at end of argv → exit 1:

- **Given:** clean environment
- **When:** `clr "Fix bug" --model`
- **Then:** Exit 1; error about missing `--model` value
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--3---model)
---

### EC-4: Any model string accepted without validation:

- **Given:** clean environment
- **When:** `clr --dry-run --model custom-model-xyz "Fix bug"`
- **Then:** Assembled command contains `--model custom-model-xyz`; no rejection
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--3---model)
---

### EC-5: `--help` lists `--model`:

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--model`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md#command--2-help)
---

### EC-6: `--model` + message → both forwarded:

- **Given:** clean environment
- **When:** `clr --dry-run --model opus "Explain this"`
- **Then:** Assembled command contains both `--model opus` and the message argument
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md#parameter--3---model)
