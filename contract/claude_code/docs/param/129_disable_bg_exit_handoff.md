# disable_bg_exit_handoff

Disables handing off in-flight background shells and workflows to the next Claude Code process on exit.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF` |
| Config Key | — |

### Type

bool

### Default

false (exit handoff enabled — outstanding background shells/workflows are
handed off across a process exit/restart by default)

### Since

≤ v2.1.197 (undocumented — not present in any changelog entry 001-098; confirmed
only via string/reference inspection of the installed binary)

### Description

Gates the handoff function that runs when the Claude Code process exits (e.g.
between conversation turns, or on restart). Decompiled logic (minified, function
name not preserved):

```js
function computeHandoff(jobs) {
  if (!isBackgroundCapable() || !CLAUDE_JOB_DIR || CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF)
    return { shells: [], workflows: [] };
  // otherwise: partition `jobs` into surviving `shells` (plain Bash background
  // commands with no agentId) and `workflows`, to be resumed by the next process
  ...
}
```

When this var is set, the function returns immediately with empty lists — no
in-flight background work is handed off, so it is simply abandoned when the
process exits. When unset (default), the handoff computation runs normally.

Note the `shells` partition is filtered to entries with `agentId === undefined`
— i.e. plain `Bash` background commands. Background `Agent`-tool subagents and
`Workflow`-tool runs are tracked through separate paths (`workflows`, and
whatever governs bare Agent-tool dispatches). This session independently
observed plain background `Bash` commands surviving a process exit correctly
(the output file was intact and readable after an ambiguous "stopped"
notification), while background `Agent`-tool subagents were reported as having
"lost" their in-process state entirely across the same kind of boundary —
consistent with those two job kinds having different handoff robustness, though
this was not confirmed against source, only inferred from observed behavior
plus this static analysis.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [128_bg_tasks_report_running.md](128_bg_tasks_report_running.md) | Related idle/exit-readiness gate for outstanding background tasks |
| doc | [130_disable_bg_shell_pressure_reap.md](130_disable_bg_shell_pressure_reap.md) | Separate background-shell lifecycle control (memory-pressure reaping, not exit handoff) |
| doc | [../version/093_v2_1_196.md](../version/093_v2_1_196.md) | "long-running commands and workflows now survive the session's process being stopped, restarted, or updated" — the feature this var disables |
| doc | [../version/091_v2_1_193.md](../version/091_v2_1_193.md) | "Fixed backgrounding (←←) spuriously cancelling with 'N background tasks would be abandoned' when all running tasks carry over to the new session" |
