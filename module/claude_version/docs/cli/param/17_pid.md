# Parameter :: 17. `pid::`

-- **Summary:** Target a single process by PID for `.ps.kill`.
-- **Type:** u64
-- **Default:** — (absent; absent means bulk kill)
-- **Commands:** `.ps.kill`
-- **Group:** none

When provided, `.ps.kill` kills only the specified PID after validating it belongs to a running Claude Code process. When absent, `.ps.kill` kills all detected Claude Code processes (bulk mode). Exits 1 if the PID is not found or is not a Claude Code process.

- **Type:** u64 (positive integer, valid PID range)
- **Default:** absent (bulk mode — kills all claude processes)
- **Validation:** must be a positive non-zero integer; exits 1 if PID is not a running claude process

```sh
clv.ps.kill pid::287807          # kill one specific claude process
clv.ps.kill pid::287807 dry::1   # preview targeted kill
```

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`.ps.kill`](../command/ps.md#command-8-pskill) | absent | Selects targeted-kill mode; absent = bulk kill |

### Referenced Type

| # | Type |
|---|------|
| 1 | `u64` |

### Referenced User Stories

| # | User Story | Persona |
|---|-----------|---------|
| 1 | [003 Process Lifecycle](../user_story/003_process_lifecycle.md) | Developer (unresponsive session) |
