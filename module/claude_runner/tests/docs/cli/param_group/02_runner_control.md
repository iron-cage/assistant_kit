# Parameter Group :: Runner Control

Interaction tests for Group 2 (Runner Control): `--no-skip-permissions`, `--interactive`,
`--new-session`, `--dir`, `--max-tokens`, `--session-dir`, `--dry-run`, `--verbosity`,
`--trace`, `--no-ultrathink`, `--no-effort-max`, `--no-chrome`. Tests validate these flags
coexist without conflict and are consumed by the runner, not forwarded to claude.

**Source:** [param_group.md#group--2-runner-control](../../../../docs/cli/param_group.md#group--2-runner-control)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `--dry-run` + `--no-ultrathink` → both applied in preview | Interaction |
| CC-2 | `--new-session` + `--session-dir` → both accepted | Interaction |
| CC-3 | `--no-skip-permissions` + `--no-effort-max` → both suppressed | Interaction |
| CC-4 | All runner control flags together → no conflict | Combined |

## Test Coverage Summary

- Interaction: 3 tests (CC-1, CC-2, CC-3)
- Combined: 1 test (CC-4)

**Total:** 4 edge cases

## Test Cases
---

### CC-1: `--dry-run` + `--no-ultrathink` → preview shows suppression

- **Given:** clean environment
- **When:** `clr --dry-run --no-ultrathink "Fix bug"`
- **Then:** Preview shows no ultrathink suffix; `--dry-run` prevents execution
- **Exit:** 0
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--2-runner-control)
---

### CC-2: `--new-session` + `--session-dir` → both accepted

- **Given:** clean environment
- **When:** `clr --dry-run --new-session --session-dir /tmp/sessions "Fix bug"`
- **Then:** Env block contains `CLAUDE_CODE_SESSION_DIR=/tmp/sessions`; no `-c` flag in assembled command
- **Exit:** 0
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--2-runner-control)
---

### CC-3: `--no-skip-permissions` + `--no-effort-max` → both suppressed

- **Given:** clean environment
- **When:** `clr --dry-run --no-skip-permissions --no-effort-max "Fix bug"`
- **Then:** Assembled command has no `--dangerously-skip-permissions` and no `--effort`
- **Exit:** 0
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--2-runner-control)
---

### CC-4: All runner control flags together → no conflict

- **Given:** clean environment
- **When:** `clr --dry-run --no-skip-permissions --interactive --new-session --dir /tmp/test --max-tokens 100000 --session-dir /tmp/sessions --verbosity 2 --trace --no-ultrathink --no-effort-max --no-chrome "Fix bug"`
- **Then:** Exit 0; all twelve flags accepted without conflict; command assembled correctly; `--chrome` is absent from assembled command; no unknown-flag error for any runner-control flag
- **Exit:** 0
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--2-runner-control)
