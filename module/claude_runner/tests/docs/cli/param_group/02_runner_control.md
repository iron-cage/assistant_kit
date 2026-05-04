# Parameter Group :: Runner Control

Interaction tests for Group 2 (Runner Control): `--no-skip-permissions`, `--interactive`, `--new-session`, `--dir`, `--max-tokens`, `--session-dir`, `--dry-run`, `--verbosity`, `--trace`, `--no-ultrathink`, `--no-effort-max`. Tests validate these flags coexist without conflict.

**Source:** [parameter_groups.md#group--2-runner-control](../../../../docs/cli/parameter_groups.md#group--2-runner-control)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--dry-run` + `--no-ultrathink` → both applied in preview | Interaction |
| EC-2 | `--new-session` + `--session-dir` → both accepted | Interaction |
| EC-3 | `--no-skip-permissions` + `--no-effort-max` → both suppressed | Interaction |
| EC-4 | All runner control flags together → no conflict | Combined |

## Test Coverage Summary

- Interaction: 3 tests (EC-1, EC-2, EC-3)
- Combined: 1 test (EC-4)

**Total:** 4 edge cases

## Test Cases
---

### EC-1: `--dry-run` + `--no-ultrathink` → preview shows suppression:

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink "Fix bug"`
- **Then:** Preview shows no ultrathink suffix; `--dry-run` prevents execution
- **Exit:** 0
- **Source:** [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--2-runner-control)
---

### EC-2: `--new-session` + `--session-dir` → both accepted:

- **Given:** clean environment
- **When:** `clr --dry-run --new-session --session-dir /tmp/sessions "Fix bug"`
- **Then:** Assembled command contains `--session-dir /tmp/sessions`; no `-c` flag
- **Exit:** 0
- **Source:** [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--2-runner-control)
---

### EC-3: `--no-skip-permissions` + `--no-effort-max` → both suppressed:

- **Given:** clean environment
- **When:** `clr --dry-run --no-skip-permissions --no-effort-max "Fix bug"`
- **Then:** Assembled command has no `--dangerously-skip-permissions` and no `--effort`
- **Exit:** 0
- **Source:** [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--2-runner-control)
---

### EC-4: All runner control flags together → no conflict:

- **Given:** clean environment
- **When:** `clr --dry-run --no-skip-permissions --interactive --new-session --verbosity 2 --no-ultrathink --no-effort-max "Fix bug"`
- **Then:** Exit 0; all flags applied without conflict; command assembled correctly
- **Exit:** 0
- **Source:** [parameter_groups.md](../../../../docs/cli/parameter_groups.md#group--2-runner-control)
