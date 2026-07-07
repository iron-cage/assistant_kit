# bg_tasks_report_running

Keeps the session's state reported as "running" while background tasks are still outstanding.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` |
| Config Key | — |

### Type

bool (presence/truthy check, same convention as other undocumented toggle vars —
set to any truthy value to enable)

### Default

false / unset (background tasks do not by themselves keep the session state
reported as "running")

### Since

≤ v2.1.197 (undocumented — not present in any changelog entry 001-098; confirmed
only via string/reference inspection of the installed binary)

### Description

Decompiled logic (minified, function name not preserved) resolves to:

```js
function reportRunningGate({ inputClosed, currentState, hasRunningBgTasks }) {
  if (hasRunningBgTasks && CLAUDE_CODE_BG_TASKS_REPORT_RUNNING) return false;
  return !inputClosed && currentState === "running";
}
```

When background tasks are outstanding (`hasRunningBgTasks`) and this var is set,
the gate short-circuits to `false` — read together with the wait-ceiling logic in
[131_print_bg_wait_ceiling_ms.md](131_print_bg_wait_ceiling_ms.md), this
short-circuit appears to feed into idle/exit-readiness detection so the process
does not treat itself as fully idle while background shells, agents, or
workflows are still in flight. The exact end-user-visible effect (what changes
in the transcript or exit timing) is inferred from static analysis, not
confirmed against official documentation — verify against real behavior before
depending on it operationally.

Directly relevant to background-task reliability: this session independently
observed background `Agent`-tool subagents losing all in-process state with the
message "was running when the previous Claude Code process exited and did not
complete" — a failure mode the v2.1.195/v2.1.196 changelogs describe multiple
fixes for (see [Cross-References](#cross-references)). This var, together with
[129_disable_bg_exit_handoff.md](129_disable_bg_exit_handoff.md), is the most
plausible first place to look when diagnosing that class of failure.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [129_disable_bg_exit_handoff.md](129_disable_bg_exit_handoff.md) | Controls whether outstanding background work is handed off across a process exit at all |
| doc | [131_print_bg_wait_ceiling_ms.md](131_print_bg_wait_ceiling_ms.md) | Ceiling on how long print/headless mode waits for the same outstanding background tasks |
| doc | [../version/093_v2_1_196.md](../version/093_v2_1_196.md) | "Improved background session reliability: long-running commands and workflows now survive the session's process being stopped, restarted, or updated" |
| doc | [../version/092_v2_1_195.md](../version/092_v2_1_195.md) | "Fixed background jobs disappearing from `claude agents` or losing data when written by a newer Claude Code version" |
