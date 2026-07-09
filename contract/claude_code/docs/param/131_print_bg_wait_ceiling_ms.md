# print_bg_wait_ceiling_ms

Bounds how long print/headless mode waits for outstanding background tasks before exiting.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` |
| Config Key | — |

### Type

integer (milliseconds)

### Default

`600000` (10 minutes) — confirmed via the fallback constant in the installed
binary (`... ?? 600000`)

### Since

≤ v2.1.197 (undocumented — not present in any changelog entry 001-098; confirmed
only via string/reference inspection of the installed binary)

### Description

In `-p`/`--print` mode (see [051_print.md](051_print.md)) — where the process is
expected to exit once its main turn completes rather than stay interactive —
this value caps how long the process will keep waiting for outstanding
background shells, agents, or workflows before it finalizes exit anyway.
Decompiled logic, confirmed verbatim against the installed v2.1.197 binary
(minified names `E3c`/`v3c`/`w3c` preserved as shipped; `Fe` is the
process-env accessor object used throughout this binary):

```js
function E3c() {
  return Fe.CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS ?? KFf; // KFf = 600000
}
function v3c({ runningBackgroundTasks: e, inputClosed: t, hasMainThreadQueued: n,
               hasActiveTeammates: r, hasPendingNotification: o, ceilingExceeded: s,
               deadline: i, swept: a, now: l }) {
  if (!(t && !n && !r && e.length > 0 && (s || (!o && !e.some(XX)))))
    return { deadline: null, swept: false, shouldSweep: false };
  if (i === null)
    return { deadline: s ? l : l + tZo, swept: s, shouldSweep: s }; // tZo = 5000
  if (l < i)
    return { deadline: i, swept: a, shouldSweep: false };
  return { deadline: i, swept: true, shouldSweep: !a };
}
function w3c(e, t) {
  for (let n of e)
    if (Dw(n)) w(`print wind-down: killing background shell ${n.id} ("...`); // truncated
}
```

`E3c()` reads this env var with `KFf` (`600000`, i.e. 10 minutes) as the
fallback. `v3c()` is the tick function: the deadline clock only starts once
stdin is closed, no main-thread work is queued, no active teammates/subagents
remain, at least one background task is still running, and either the ceiling
was already exceeded or there is no pending notification and no task matches
predicate `XX` (exact semantics not decompiled — plausibly "is blocking/
foreground-relevant"). Once started, the deadline is `now + tZo` (`5000` ms —
a 5-second grace period) unless the ceiling was already exceeded, in which case
the deadline is immediate. `w3c()` is the actual sweep executor invoked once
`shouldSweep` is true: it iterates running background tasks and kills each one
matching predicate `Dw(n)`, logging `print wind-down: killing background shell
{id} (...)`.

Both `claude_storage/.claude/settings.local.json` (a sibling project in this
workspace) and `claude_runner` itself set this to `0` — confirmed for
`claude_runner` via direct read of `claude_runner_core/src/command/mod.rs`:
`ClaudeCommand::new()` sets `print_bg_wait_ceiling_ms: Some(0)`
unconditionally, injected as `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=0` by
`env_pairs()` on every invocation (see
[`module/claude_runner/docs/cli/003_env_param.md`](../../../../module/claude_runner/docs/cli/003_env_param.md)
for clr's own default-injection documentation of this and 4 sibling
`CLAUDE_CODE_*` variables).

**Corrected interpretation of `0`** (an earlier version of this document
asserted the opposite — retracted below): the `ra > 0` guard is the critical
detail. The caller that feeds `v3c()` computes `ceilingExceeded` as
`ra > 0 && So !== null && Zr - So >= ra`, where `ra = E3c()` is this
variable's resolved value and `So` is the timestamp the process entered the
wind-down state. Setting the ceiling to `0` makes `ra > 0` permanently false,
so `ceilingExceeded` can never become true: **the ceiling-based forced-sweep
path is disabled, not instant-triggered.** This document previously read `0`
as "do not wait at all for background work in print mode, exit immediately"
— that reading does not survive tracing the actual conjunction and is
retracted.

This does not mean unconditional infinite waiting, though: `v3c()`'s entry
condition is `(s || (!o && !e.some(XX)))` — with `s` (`ceilingExceeded`)
pinned false by the guard above, entry can still occur via the second
disjunct (no pending notification AND no running background task matches
undecompiled predicate `XX`), which still drives a sweep after a fixed `tZo`
(`5000`ms) grace period, independent of this variable's value entirely.
Whether typical background Bash/Agent work matches `XX` is not resolved from
static analysis alone.

Independent corroboration: [`docs/claude_code_background_task_env_vars.md`](../../../../docs/claude_code_background_task_env_vars.md)
(repo root) documents this same variable from the official docs, an
independent community mirror, and **direct causal testing against a live
process** — concluding "`0` = wait indefinitely" for Agent/Workflow dispatches
specifically, and separately confirming a fixed ~5-second grace period for
plain background Bash tasks that this ceiling variable does not affect. That
empirical result and this session's static decompilation agree on the same
two-path structure: a ceiling-gated path for agent/workflow work, and a
separate fixed-grace path for plain background shells.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [051_print.md](051_print.md) | `-p`/`--print` mode this ceiling applies to |
| doc | [128_bg_tasks_report_running.md](128_bg_tasks_report_running.md) | Related "is a background task still outstanding" signal consumed by the same wait/sweep logic |
| doc | [../../../../module/claude_runner/docs/cli/003_env_param.md](../../../../module/claude_runner/docs/cli/003_env_param.md) | `claude_runner`'s own injected-default documentation for this and 4 sibling `CLAUDE_CODE_*` variables |
| doc | [../../../../docs/claude_code_background_task_env_vars.md](../../../../docs/claude_code_background_task_env_vars.md) | Independent, empirically-tested (live-process) confirmation of the `0` = wait-indefinitely reading |
