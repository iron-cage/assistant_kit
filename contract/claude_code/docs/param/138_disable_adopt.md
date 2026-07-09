# disable_adopt

Stop in-flight work from carrying over when a session is backgrounded.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_DISABLE_ADOPT` |
| Config Key | — |

### Type

bool

### Default

`false` (unset)

### Since

v2.1.195+

### Description

Governs a different lifecycle point than
[`CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF`](129_disable_bg_exit_handoff.md): that
variable covers a background *session* (agent-view/supervisor architecture)
losing its shells/workflows across a supervisor stop/restart/update. This
variable covers the user explicitly backgrounding the current session (`←`
or `/background`) — normally any in-flight background shells, workflows, and
(as of v2.1.198+) subagents carry over to continue running after the session
is backgrounded. Setting this to `1` stops that carry-over instead, and
presents a confirmation prompt before backgrounding proceeds.

`CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF` alone does not affect this path —
backgrounding via `←`/`/background` still hands work off normally unless
this separate variable is also set. Setting `CLAUDE_DISABLE_ADOPT` turns off
both handoff paths at once.

Naming note: this variable has **no** `_CODE_` infix (`CLAUDE_DISABLE_ADOPT`,
not `CLAUDE_CODE_DISABLE_ADOPT`) — easily confused with its closest semantic
sibling [`CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF`](129_disable_bg_exit_handoff.md),
which does carry the infix, since the two are documented and used together.
See [140_auto_background_tasks.md](140_auto_background_tasks.md) for the
full list of siblings sharing this trap.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [129_disable_bg_exit_handoff.md](129_disable_bg_exit_handoff.md) | Sibling variable — process-exit handoff instead of user-initiated backgrounding; naming pattern differs, see note above |
| doc | [137_job_dir.md](137_job_dir.md) | Precondition marker for the exit-handoff path this variable also disables |
| doc | [140_auto_background_tasks.md](140_auto_background_tasks.md) | Sibling no-`_CODE_`-infix variable |
