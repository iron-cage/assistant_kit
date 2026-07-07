# Behavior B36: Five Env Vars Gate Background-Task Survival, Idle-Reap, and Print-Mode Wait

### Scope

- **Purpose**: Document the five `CLAUDE_CODE_*` env vars that together govern the background-task (Bash/Agent/Workflow) lifecycle — which model classifies them, whether they keep the session reported as "running", whether they survive a process exit, whether idle shells get killed under memory pressure, and how long print/headless mode waits for them before exiting anyway.
- **Responsibility**: Authoritative instance for behavior B36 — defines the behavior statement, certainty level, and supporting evidence per var. Tier is UNVERIFIED (no automated test yet, consistent with B27–B35).
- **In Scope**: `CLAUDE_CODE_BG_CLASSIFIER_MODEL`, `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING`, `CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF`, `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP`, `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS`; the decompiled gate/state-machine functions each is read by (`S3c`/`b3c`, `_Ha`/`bHa`, `u0l`, `E3c`/`v3c`/`w3c`); the `agentId === void 0` exclusion of Agent-tool subagent jobs from exit-handoff `shells` survival; numeric constants `KFf = 600000`, `tZo = 5000`.
- **Out of Scope**: Full per-parameter reference (→ [`../param/127_bg_classifier_model.md`](../param/127_bg_classifier_model.md) through [`131_print_bg_wait_ceiling_ms.md`](../param/131_print_bg_wait_ceiling_ms.md)); official changelog entries (→ [`../version/`](../version/readme.md)); rtk/bash subprocess identity (→ [B28](028_b28_bash_rtk_subprocess.md)); Agent-tool OS-process model (→ [B27](027_b27_agent_no_os_process.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 85% | **Tier**: UNVERIFIED | **Since**: ≤ v2.1.197 (installed binary; `DISABLE_BG_SHELL_PRESSURE_REAP` officially dated v2.1.193 per changelog, the other four undated) | **Evidence**: E62, E63, E64, E65, E66

All five vars were located by resolving the installed `claude` binary
(`~/.local/share/claude/versions/2.1.197`, confirmed via `file` as the ELF
executable itself, not a symlink or directory) and extracting `strings -a`
output, then bounding each variable-name match with `grep -aoP` to recover its
surrounding minified source. Four of the five decompile to a complete,
readable function body; `CLAUDE_CODE_BG_CLASSIFIER_MODEL` only decompiles to a
schema/export-map declaration (weaker evidence — hence the overall Certainty
is pulled down from the ~95% each of the other four would individually merit).

| Env Var | Responsible For | Default | Key Function(s) |
|---|---|---|---|
| `CLAUDE_CODE_BG_CLASSIFIER_MODEL` | Overrides the model used for lightweight background-task classifier calls | binary default (unconfirmed) | `KDu` (schema entry, `Ue.str()`) |
| `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` | Forces session state to stay "running" while background tasks are outstanding | `false`/unset | `S3c()`, `b3c()` |
| `CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF` | Disables handoff of in-flight background shells/workflows to the next process on exit | `false` (handoff enabled) | `_Ha()`, `bHa()` |
| `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP` | Disables killing idle background shells under host memory pressure | `false` (reaping enabled) | `u0l()` |
| `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` | Caps how long `-p`/`--print` mode waits for outstanding background work before exiting | `600000` (`KFf`, 10 min) | `E3c()`, `v3c()`, `w3c()` |

**`CLAUDE_CODE_BG_TASKS_REPORT_RUNNING`** — gate confirmed verbatim (`Fe` = process-env accessor):

```js
function b3c({ inputClosed: e, runningTasks: t }) {
  return e && t.some((n) => XX(n) && wv(n));
}
function S3c({ inputClosed: e, currentState: t, hasRunningBgTasks: n }) {
  if (n && Fe.CLAUDE_CODE_BG_TASKS_REPORT_RUNNING) return !1;
  return !e && t === "running";
}
```

**`CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF`** — the handoff computation and its consumer, confirmed verbatim:

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

The `shells` survivor set requires both `Tbo(r, t)` (predicate, exact
semantics not decompiled) **and** `r.agentId === void 0` — i.e. plain `Bash`
background commands only. A background `Agent`-tool subagent dispatch carries
a non-undefined `agentId` and so can never land in `shells`; it is routed
(if at all) through `workflows`/`Ebo`, gated separately. This is the most
plausible static-analysis explanation for an asymmetry directly observed this
session: a plain backgrounded `Bash` test run survived a process-exit boundary
intact (output file read back successfully after an ambiguous "stopped"
notification), while four separate backgrounded `Agent`-tool MAAV gate
dispatches were all reported as having "lost" their in-process state across
the same kind of boundary. Not fully proven — whether Agent-tool jobs are
excluded from survival entirely, or merely routed through the separately-gated
`workflows` path and failing there for an unrelated reason, was not resolved
from static analysis alone.

**`CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP`** — confirmed verbatim:

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
    i = () => process.off("memoryPressure", a); // inferred symmetric teardown; call truncated in strings extraction
  }
}
```

Reaping is wired through Node's own `"memoryPressure"` process event (not a
poll loop), gated by an elapsed-time-since-activity threshold `Exm` (exact
value not decompiled) via a `NA()` last-activity accessor, plus two further
guard predicates `gEr()`/`yKe(n.all())` whose exact semantics were not
decompiled. Telemetry event on an actual reap: `task_local_shell_pressure_reap`.
This is the only one of the five vars independently confirmed in an official
changelog entry (v2.1.193 — see [E65](#evidence) and
[`../version/091_v2_1_193.md`](../version/091_v2_1_193.md)).

**`CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS`** — confirmed verbatim, including both numeric constants:

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

The deadline clock in `v3c()` only starts once stdin is closed, no main-thread
work is queued, no active teammates/subagents remain, at least one background
task is running, and either the ceiling was already exceeded or there is no
pending notification and no task matches predicate `XX` (shared with `b3c()`
above — exact semantics not decompiled, plausibly "is blocking/foreground-relevant").
Once started, the deadline is `now + tZo` (5-second grace period) unless the
ceiling is already exceeded, in which case it fires immediately. `w3c()` is the
edge-triggered sweep executor, killing tasks matching predicate `Dw(n)`.

**`CLAUDE_CODE_BG_CLASSIFIER_MODEL`** — weaker evidence: decompiles only to a
module export-map entry (`CLAUDE_CODE_BG_CLASSIFIER_MODEL:()=>KDu`) grouped
alongside `ANTHROPIC_SMALL_FAST_MODEL`, `CLAUDE_CODE_AUTO_MODE_MODEL`, and
`CLAUDE_CODE_ALWAYS_ENABLE_EFFORT`, plus a schema declaration (`KDu=Ue.str()`,
i.e. string-typed). No direct consumption call site was found in the strings
output — confirms the var is real, registered, and string-typed, but not its
runtime default or exact consuming call site.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E62 | B36 | Code | Binary analysis — `strings -a -n 8` + `grep -aoP` on `~/.local/share/claude/versions/2.1.197` — this session (2026-07-07) | `strings`-output line 122948 (bare name) and line 272025 (export map) | `CLAUDE_CODE_BG_CLASSIFIER_MODEL:()=>KDu` in module export map, adjacent to `ANTHROPIC_SMALL_FAST_MODEL`, `CLAUDE_CODE_AUTO_MODE_MODEL`, `CLAUDE_CODE_ALWAYS_ENABLE_EFFORT`; schema declaration `KDu=Ue.str()` found in the same settings-schema object as `JDu=Ue.bool()`, `QDu=Ue.bool()`, etc. No direct `KDu(...)` call site found. |
| E63 | B36 | Code | Binary analysis — same binary/method — this session (2026-07-07) | `strings`-output line 296515/296542, functions `S3c`/`b3c` | `function b3c({inputClosed:e,runningTasks:t}){return e&&t.some((n)=>XX(n)&&wv(n))}function S3c({inputClosed:e,currentState:t,hasRunningBgTasks:n}){if(n&&Fe.CLAUDE_CODE_BG_TASKS_REPORT_RUNNING)return!1;return!e&&t==="running"}` — full function bodies recovered verbatim. |
| E64 | B36 | Code | Binary analysis — same binary/method — this session (2026-07-07) | `strings`-output line 274061, functions `_Ha`/`bHa` | `function _Ha(e){if(!yi()\|\|!Fe.CLAUDE_JOB_DIR\|\|Fe.CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF)return{shells:[],workflows:[]};let t=nEe(e),n=Object.values(e);return{shells:n.filter((r)=>Tbo(r,t)&&r.agentId===void 0),workflows:n.filter((r)=>Ebo(r,t))}}function bHa({shells:e,workflows:t}){let n=Fe.CLAUDE_JOB_DIR;for(let o of t)o.abortController...` (truncated by strings extraction after `abortController`). Confirms `agentId===void 0` as an explicit filter condition on the `shells` survivor set. |
| E65 | B36 | Code + Doc | Binary analysis (same method, this session, 2026-07-07) + official changelog `../version/091_v2_1_193.md` | `strings`-output line 277007, function `u0l`; changelog entry v2.1.193 | `function u0l(e,t,n,r,o,s){Pve(s,\`bash:${e}\`,n);let i;if(s===void 0&&!xr()&&!Fe.CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP){let a=()=>{let l=n.get(e);if(l?.status!=="running"\|\|l.notified\|\|Date.now()-NA()<Exm\|\|gEr()\|\|yKe(n.all()))return;Ie("task_local_shell_pressure_reap"),tYt(e,t,"killed",void 0,n,r,o,s),nve(e,n)};process.on("memoryPressure",a),i=()=>process.off("memory...` (truncated). Changelog: "Added automatic memory-pressure reaping for idle background shell commands (disable with `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP=1`)" — the only one of the five vars officially documented. |
| E66 | B36 | Code | Binary analysis — same binary/method — this session (2026-07-07) | `strings`-output line 296515/296542, functions `E3c`/`v3c`/`w3c`; constants `KFf`, `tZo` | `function E3c(){return Fe.CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS??KFf}` plus full `v3c()`/`w3c()` bodies (see Behavior section above). Constants independently confirmed via `grep -aoP "(?<![a-zA-Z0-9_])KFf\s*=\s*[0-9]+"` → `KFf=600000`; same technique → `tZo=5000`. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [027_b27_agent_no_os_process.md](027_b27_agent_no_os_process.md) | Agent subagents are API inference threads, not OS processes — context for why `agentId` (not PID) is the discriminator in `_Ha`'s `shells` filter |
| behavior | [029_b29_bash_claude_env.md](029_b29_bash_claude_env.md) | Same binary-analysis methodology (Code-type evidence, decompiled minified functions) applied to a different subsystem |
| param | [../param/127_bg_classifier_model.md](../param/127_bg_classifier_model.md) | `CLAUDE_CODE_BG_CLASSIFIER_MODEL` full parameter entry |
| param | [../param/128_bg_tasks_report_running.md](../param/128_bg_tasks_report_running.md) | `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` full parameter entry |
| param | [../param/129_disable_bg_exit_handoff.md](../param/129_disable_bg_exit_handoff.md) | `CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF` full parameter entry |
| param | [../param/130_disable_bg_shell_pressure_reap.md](../param/130_disable_bg_shell_pressure_reap.md) | `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP` full parameter entry |
| param | [../param/131_print_bg_wait_ceiling_ms.md](../param/131_print_bg_wait_ceiling_ms.md) | `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` full parameter entry |
| doc | [../version/091_v2_1_193.md](../version/091_v2_1_193.md) | Changelog introducing `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP` |
| doc | [../version/093_v2_1_196.md](../version/093_v2_1_196.md) | Changelog: "long-running commands and workflows now survive the session's process being stopped, restarted, or updated" |
