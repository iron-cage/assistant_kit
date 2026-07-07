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

Decompiled logic, confirmed verbatim against the installed v2.1.197 binary
(minified name `u0l` preserved as shipped; `Fe` is the process-env accessor
object used throughout this binary):

```js
function u0l(e, t, n, r, o, s) {
  Pve(s, `bash:${e}`, n);
  let i;
  if (s === void 0 && !xr() && !Fe.CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP) {
    let a = () => {
      let l = n.get(e);
      if (l?.status !== "running" || l.notified || Date.now() - NA() < Exm || gEr() || yKe(n.all()))
        return;
      Ie("task_local_shell_pressure_reap");
      tYt(e, t, "killed", void 0, n, r, o, s);
      nve(e, n);
    };
    process.on("memoryPressure", a);
    i = () => process.off("memoryPressure", a); // inferred symmetric teardown; exact call truncated in strings extraction
  }
}
```

The guard before reaping fires is stricter than "idle": the shell must be
`status === "running"`, not already `notified`, have had no activity for at
least `Exm` (elapsed-time threshold, exact value not decompiled) per `NA()`
(last-activity timestamp accessor), and two further predicates `gEr()`/`yKe(n.all())`
must both be false (exact semantics not decompiled — plausibly "any foreground
dependents" / "any other shell already mid-reap"). Reaping is wired through
Node's own `"memoryPressure"` process event, not a poll loop. The telemetry
event fired on an actual reap is `task_local_shell_pressure_reap`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [129_disable_bg_exit_handoff.md](129_disable_bg_exit_handoff.md) | Related but distinct: exit-handoff survival vs. live-process pressure reaping |
| doc | [../version/091_v2_1_193.md](../version/091_v2_1_193.md) | Changelog entry introducing this var |
| doc | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool whose background commands this reaping targets |
