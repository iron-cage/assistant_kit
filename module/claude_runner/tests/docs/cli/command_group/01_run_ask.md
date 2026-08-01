# Command Group :: run / ask

Structural-equivalence tests for the run/ask command group: verifying `ask` truly
shares `run`'s handler function and parameter set, with zero default divergence.

**Source:** [command_group/01_run_ask.md](../../../../docs/cli/command_group/01_run_ask.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CG-1 | `clr ask "q"` dry-run output identical to `clr "q"` dry-run output | Equivalence |
| CG-2 | `clr ask --dry-run` (no message) identical to `clr --dry-run` | Equivalence |

## Test Coverage Summary

- Equivalence: 2 tests (CG-1, CG-2)

**Total:** 2 test cases. Both are already fully specified as `command/05_ask.md` IT-1/IT-2 —
this file indexes them under the command_group entity rather than re-specifying identical
Given/When/Then content. See [command/05_ask.md](../command/05_ask.md) for full case detail.

## Test Cases

---

### CG-1: `clr ask "q"` dry-run identical to `clr "q"` dry-run

Full specification: [command/05_ask.md](../command/05_ask.md) IT-1.

- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md), [command_group/01_run_ask.md](../../../../docs/cli/command_group/01_run_ask.md)
- **Commands:** run, ask

---

### CG-2: `clr ask --dry-run` (no message) identical to `clr --dry-run`

Full specification: [command/05_ask.md](../command/05_ask.md) IT-2.

- **Source:** [command/05_ask.md](../../../../docs/cli/command/05_ask.md), [command_group/01_run_ask.md](../../../../docs/cli/command_group/01_run_ask.md)
- **Commands:** run, ask
