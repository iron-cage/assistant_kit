# auto_background_tasks

Force-enable automatic backgrounding of long-running agent tasks.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_AUTO_BACKGROUND_TASKS` |
| Config Key | — |

### Type

bool

### Default

`false` (unset)

### Since

≤v2.1.197 (documented)

### Description

Set to `1` to force-enable automatic backgrounding: subagents that run past
roughly two minutes are automatically moved to the background rather than
continuing to block the main thread. Decompiled usage, confirmed verbatim
against the installed v2.1.197 binary:

```js
function Rbm() {
  if (ct(process.env.CLAUDE_AUTO_BACKGROUND_TASKS)) return 120000;
  return 0;
}
```

`Rbm()` resolves the auto-backgrounding threshold in milliseconds: `120000`
(2 minutes, confirming the "~2 minutes" figure above) when the var is truthy
(`ct()` — an undecompiled truthy-string-coercion helper used throughout this
binary), else `0`. The `0` return for the unset case reads as a sentinel
consistent with this collection's general "threshold `0` = feature disabled"
pattern (see [131_print_bg_wait_ceiling_ms.md](131_print_bg_wait_ceiling_ms.md)
for the same `> 0` guard shape) — i.e. auto-backgrounding is off by default,
not merely "off after a 0ms threshold."

**Is there an opposite/force-disable variable?** No. A search of the
installed binary for any `CLAUDE_(CODE_)?(DISABLE|NO|FORCE)_?AUTO_?BACKGROUND`-shaped
name returns no match. The unset/default state (`Rbm()` returns `0`) already
is the "off" state — no separate variable is needed to force it off. The
nearest broader lever, [`CLAUDE_CODE_DISABLE_BACKGROUND_TASKS`](136_disable_background_tasks.md),
is **not** a precise negation of this variable: a second, independent usage
site —

```js
nIo = Ce(() => {
  let e = kbm().omit({ cwd: !0 });
  return $Kt || lX() ? e.omit({ run_in_background: !0 }) : e;
});
// where $Kt = Fe.CLAUDE_CODE_DISABLE_BACKGROUND_TASKS
```

— confirms it also strips the `run_in_background` parameter from the Agent
tool's schema entirely, i.e. it disables explicit/manual backgrounding too,
not just this auto-triggering heuristic. Leaving `CLAUDE_AUTO_BACKGROUND_TASKS`
unset (or falsy) is the only way to suppress just the automatic heuristic
while keeping manual backgrounding available.

Naming note: this variable has **no** `_CODE_` infix, unlike most other
variables in this collection — the name is `CLAUDE_AUTO_BACKGROUND_TASKS`,
not `CLAUDE_CODE_AUTO_BACKGROUND_TASKS`. A search for the `_CODE_` form
returns zero matches in the installed binary; only the shorter name exists.
This is easy to get wrong when working from memory or an unverified summary.
The same trap recurs elsewhere in this collection —
[075_autocompact_pct_override.md](075_autocompact_pct_override.md),
[137_job_dir.md](137_job_dir.md), [138_disable_adopt.md](138_disable_adopt.md),
and [139_async_agent_stall_timeout_ms.md](139_async_agent_stall_timeout_ms.md)
share the same no-`_CODE_` shape.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [127_bg_classifier_model.md](127_bg_classifier_model.md) | Adjacent `_CODE_`-prefixed variable — naming contrast |
| doc | [136_disable_background_tasks.md](136_disable_background_tasks.md) | Global kill-switch this heuristic is subject to — broader, not a precise negation of this variable |
| doc | [075_autocompact_pct_override.md](075_autocompact_pct_override.md) | Sibling no-`_CODE_`-infix variable |
| doc | [137_job_dir.md](137_job_dir.md) | Sibling no-`_CODE_`-infix variable |
| doc | [138_disable_adopt.md](138_disable_adopt.md) | Sibling no-`_CODE_`-infix variable |
| doc | [139_async_agent_stall_timeout_ms.md](139_async_agent_stall_timeout_ms.md) | Sibling no-`_CODE_`-infix variable |
