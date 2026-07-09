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
between conversation turns, or on restart). Decompiled logic, confirmed
verbatim against the installed v2.1.197 binary (minified names `_Ha`/`bHa`
preserved as shipped; `Fe` is the process-env accessor object used throughout
this binary):

```js
function _Ha(e) {
  if (!yi() || !Fe.CLAUDE_JOB_DIR || Fe.CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF)
    return { shells: [], workflows: [] };
  let t = nEe(e), n = Object.values(e);
  return {
    shells: n.filter((r) => Tbo(r, t) && r.agentId === void 0),
    workflows: n.filter((r) => Ebo(r, t)),
  };
}
function bHa({ shells: e, workflows: t }) {
  let n = Fe.CLAUDE_JOB_DIR;
  for (let o of t) o.abortController /* ...truncated in strings extraction */;
}
```

When this var is set, `_Ha` returns immediately with empty lists — no in-flight
background work is handed off, so it is simply abandoned when the process
exits. When unset (default), `_Ha` computes the survivor sets normally and
`bHa` consumes them (against `CLAUDE_JOB_DIR`) to resume on the next process.

Confirmed: the `shells` partition requires both `Tbo(r, t)` (predicate,
exact semantics not decompiled) AND `r.agentId === void 0` — i.e. plain `Bash`
background commands only. Background `Agent`-tool subagents carry a non-undefined
`agentId` and so can never qualify as a surviving `shell`; they fall instead
under `workflows` (`Ebo(r, t)`) or under no survival path at all, depending on
how the job was created. This session independently observed plain background
`Bash` commands surviving a process exit correctly (the output file was intact
and readable after an ambiguous "stopped" notification), while background
`Agent`-tool subagents were reported as having "lost" their in-process state
entirely across the same kind of boundary — consistent with, though not fully
proven by, this `agentId` exclusion (whether Agent-tool jobs are excluded
entirely or merely routed through the separately-gated `workflows`/`Ebo` path
was not resolved from static analysis alone).

Naming note: this variable's closest semantic sibling,
[`CLAUDE_DISABLE_ADOPT`](138_disable_adopt.md) (a different lifecycle point
— user-initiated backgrounding rather than process-exit handoff), has **no**
`_CODE_` infix despite this variable carrying one. Easy to mix up when
writing either name from memory, since the two are documented and used
together.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [128_bg_tasks_report_running.md](128_bg_tasks_report_running.md) | Related idle/exit-readiness gate for outstanding background tasks |
| doc | [130_disable_bg_shell_pressure_reap.md](130_disable_bg_shell_pressure_reap.md) | Separate background-shell lifecycle control (memory-pressure reaping, not exit handoff) |
| doc | [../version/093_v2_1_196.md](../version/093_v2_1_196.md) | "long-running commands and workflows now survive the session's process being stopped, restarted, or updated" — the feature this var disables |
| doc | [../version/091_v2_1_193.md](../version/091_v2_1_193.md) | "Fixed backgrounding (←←) spuriously cancelling with 'N background tasks would be abandoned' when all running tasks carry over to the new session" |
