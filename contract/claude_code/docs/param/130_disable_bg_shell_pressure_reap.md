# disable_bg_shell_pressure_reap

Disables automatic memory-pressure reaping of idle background shell commands.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP` |
| Config Key | — |

### Type

bool

### Default

false (reaping enabled — idle background shells may be killed under memory
pressure)

### Since

v2.1.193 (date unverified)

### Description

Officially documented in the v2.1.193 changelog: "Added automatic memory-pressure
reaping for idle background shell commands (disable with
`CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP=1`)". When the host is under memory
pressure, Claude Code automatically kills ("reaps") background shell commands
that are idle (not actively producing output) to free resources. Setting this
var to `1` disables that automatic reaping, letting idle background shells run
to completion regardless of memory pressure.

Distinct from [129_disable_bg_exit_handoff.md](129_disable_bg_exit_handoff.md):
that var controls whether outstanding work survives a *process exit*; this var
controls whether a *live* process proactively kills idle background shells
under resource pressure, independent of any exit/restart. Relevant when a
long-running background command (e.g. a multi-minute containerized test suite)
appears to have been killed mid-run with no corresponding process-exit event —
this is the mechanism to rule out first in that scenario, before suspecting
exit handoff.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [129_disable_bg_exit_handoff.md](129_disable_bg_exit_handoff.md) | Related but distinct: exit-handoff survival vs. live-process pressure reaping |
| doc | [../version/091_v2_1_193.md](../version/091_v2_1_193.md) | Changelog entry introducing this var |
| doc | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool whose background commands this reaping targets |
