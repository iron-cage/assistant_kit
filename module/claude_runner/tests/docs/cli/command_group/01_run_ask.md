# Command Group :: run / ask / topic

Structural-equivalence tests for the run/ask/topic command group: verifying `ask` truly
shares `run`'s handler function and parameter set with zero default divergence, and that
`topic` shares the same handler with exactly one documented default divergence (`--topic`).

**Source:** [command_group/01_run_ask.md](../../../../docs/cli/command_group/01_run_ask.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CG-1 | `clr ask "q"` dry-run output identical to `clr "q"` dry-run output | Equivalence |
| CG-2 | `clr ask --dry-run` (no message) identical to `clr --dry-run` | Equivalence |
| CG-3 | `clr topic --topic NAME "q"` dry-run identical to `clr ask --topic NAME "q"` dry-run | Equivalence |

## Test Coverage Summary

- Equivalence: 3 tests (CG-1, CG-2, CG-3)

**Total:** 3 test cases. CG-1/CG-2 are already fully specified as `command/05_ask.md` IT-1/IT-2;
CG-3 is already fully specified as `command/11_topic.md` IT-3 — this file indexes them under the
command_group entity rather than re-specifying identical Given/When/Then content. CG-3 neutralizes
the group's one documented default divergence (explicit `--topic` overrides `topic`'s auto-slug),
confirming the shared handler is otherwise identical. See
[command/05_ask.md](../command/05_ask.md) and [command/11_topic.md](../command/11_topic.md) for
full case detail.

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

---

### CG-3: `clr topic --topic NAME "q"` dry-run identical to `clr ask --topic NAME "q"` dry-run

Full specification: [command/11_topic.md](../command/11_topic.md) IT-3.

- **Source:** [command/11_topic.md](../../../../docs/cli/command/11_topic.md), [command_group/01_run_ask.md](../../../../docs/cli/command_group/01_run_ask.md)
- **Commands:** ask, topic
