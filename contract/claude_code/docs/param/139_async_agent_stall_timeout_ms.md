# async_agent_stall_timeout_ms

Stall/no-progress timeout for background subagents.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_ASYNC_AGENT_STALL_TIMEOUT_MS` |
| Config Key | — |

### Type

integer (milliseconds)

### Default

`600000` (10 minutes)

### Since

≤v2.1.197 (documented)

### Description

Bounds how long a backgrounded subagent may go without emitting a streaming
progress event before it is treated as stalled. The timer resets on every
streaming progress event, so it measures gaps between progress, not total
runtime.

This is a distinct axis from
[`CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS`](131_print_bg_wait_ceiling_ms.md):
the print-mode ceiling only starts counting once the main turn has finished
and the process is winding down, and it bounds total wait time regardless of
whether the agent is actively progressing. This stall timeout can fire in
the middle of an otherwise-healthy session, well before exit, specifically
because an agent has gone quiet — not because it has run long in absolute
terms.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [131_print_bg_wait_ceiling_ms.md](131_print_bg_wait_ceiling_ms.md) | Distinct timeout axis — total print-mode exit wait, not stall detection |
| doc | [003_agent.md](003_agent.md) | The Agent tool whose background dispatches this timeout governs |
