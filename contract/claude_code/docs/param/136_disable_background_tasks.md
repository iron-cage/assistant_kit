# disable_background_tasks

Global kill-switch for all background-task functionality.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` |
| Config Key | — |

### Type

bool

### Default

`false` (unset)

### Since

≤v2.1.197 (documented)

### Description

Set to `1` to disable all background-task functionality at once: the
`run_in_background` parameter on the Bash tool, the same parameter on
subagent/Agent dispatch, the auto-backgrounding heuristic (see
[140_auto_background_tasks.md](140_auto_background_tasks.md)), and the
interactive Ctrl+B shortcut. Everything that would otherwise be eligible to
run in the background instead runs synchronously to completion.

Source-confirmed tied directly into the Agent tool's own lifecycle handling,
at the exact point a turn would normally release an agent to keep running in
the background:

```js
case "agent-stopped": {
  let c = Boolean(Fe.CLAUDE_CODE_DISABLE_BACKGROUND_TASKS);
  // ...
  awaitCompletion: c
  // ...
}
```

Setting this forces `awaitCompletion = true` at that point, overriding
whatever background-or-not request was originally made and blocking the
orchestrator until the agent is actually done. It is the blunt-instrument
alternative to [131_print_bg_wait_ceiling_ms.md](131_print_bg_wait_ceiling_ms.md):
instead of "let things background, but never let `-p` kill them before they
finish," this says "nothing is ever backgrounded in the first place" —
trading concurrency for the certainty that nothing can be orphaned mid-flight
by an exit-wait ceiling or exit-handoff event, since there is nothing left
for either to prematurely terminate.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [131_print_bg_wait_ceiling_ms.md](131_print_bg_wait_ceiling_ms.md) | Narrower alternative — bounds the wait instead of disabling backgrounding entirely |
| doc | [140_auto_background_tasks.md](140_auto_background_tasks.md) | The auto-backgrounding heuristic this also disables |
| doc | [003_agent.md](003_agent.md) | The Agent tool whose dispatch/stop logic this flag gates |
