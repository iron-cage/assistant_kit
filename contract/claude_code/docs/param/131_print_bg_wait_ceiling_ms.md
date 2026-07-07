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

A sibling project in this same workspace (`claude_storage/.claude/settings.local.json`)
already sets this to `"0"` — i.e. do not wait at all for background work in
print mode, exit immediately. That is a real precedent for headless/CI-style
invocations that would rather abandon outstanding background work than block
the parent process; `claude_runner` itself does not currently set this (it
invokes `claude` as a subprocess and manages its own timeout via
`CLR_TIMEOUT`/`gate_poll_secs`/`gate_max_attempts` instead — see
`module/claude_runner/docs/cli/003_env_param.md`).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [051_print.md](051_print.md) | `-p`/`--print` mode this ceiling applies to |
| doc | [128_bg_tasks_report_running.md](128_bg_tasks_report_running.md) | Related "is a background task still outstanding" signal consumed by the same wait/sweep logic |
| doc | [../../../../module/claude_runner/docs/cli/003_env_param.md](../../../../module/claude_runner/docs/cli/003_env_param.md) | `claude_runner`'s own `CLR_*` timeout/gate vars — the runner-native analogue of this upstream ceiling |
