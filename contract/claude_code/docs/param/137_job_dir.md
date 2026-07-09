# job_dir

Internal marker present when running under the supervisor/agent-view architecture.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_JOB_DIR` |
| Config Key | — |

### Type

string (directory path)

### Default

unset

### Since

≤v2.1.197 (undocumented — confirmed only via binary reference inspection; no
usage beyond the precondition check below was found)

### Description

Source-confirmed as a required precondition inside the background-task
exit-handoff function, alongside
[`CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF`](129_disable_bg_exit_handoff.md):

```js
if (!yi() || !Fe.CLAUDE_JOB_DIR || Fe.CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF)
  return { shells: [], workflows: [] };
```

Reads as an internal marker set by the surrounding process supervisor when a
session is running under the agent-view/background-session architecture —
not something intended to be set by hand. Its presence (alongside `yi()`,
an undecompiled internal predicate) gates whether the exit-handoff path has
anything to hand off at all; absent it, handoff always returns empty
regardless of what background work is outstanding.

Naming note: like several siblings in this collection, this variable has no
`_CODE_` infix (`CLAUDE_JOB_DIR`, not `CLAUDE_CODE_JOB_DIR`) — see
[140_auto_background_tasks.md](140_auto_background_tasks.md) for the full
list of siblings sharing this trap.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [129_disable_bg_exit_handoff.md](129_disable_bg_exit_handoff.md) | The handoff behavior this variable is a precondition for |
| doc | [138_disable_adopt.md](138_disable_adopt.md) | Adjacent lifecycle point — backgrounding via `←`/`/background` instead of process exit |
| doc | [140_auto_background_tasks.md](140_auto_background_tasks.md) | Sibling no-`_CODE_`-infix variable |
