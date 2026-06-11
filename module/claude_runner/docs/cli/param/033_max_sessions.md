# CLI Parameter: --max-sessions

Maximum number of concurrent Claude Code sessions allowed before this invocation blocks.
When the active session count meets or exceeds this limit, `clr` polls every 30 seconds
for up to 50 attempts, then exits with code 1. Setting `0` disables the gate entirely
(unlimited sessions, no process scan).

- **Type:** u32
- **Default:** 20
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)

```sh
clr --max-sessions 5 "refactor module"      # block if >=5 Claude sessions active
clr --max-sessions 1 "single task"          # strict: only 1 session at a time
clr --max-sessions 0 "unrestricted"         # gate disabled; proceeds immediately
CLR_MAX_SESSIONS=3 clr "fix bug"            # env-var equivalent of --max-sessions 3
clr --max-sessions 3 --dry-run "preview"    # dry-run: gate skipped; shows assembled command
```

**Note:** Session count is determined by scanning `/proc/{pid}/cmdline` for entries
whose basename is exactly `"claude"`, excluding the calling process. The count reflects
all running Claude Code processes system-wide, not per-project.

**Note:** When the gate waits, `clr` emits a message to stderr each polling cycle (only at
verbosity ≥ 2, which is the default):
`"Info: {count}/{max} sessions active; waiting 30s for a slot... (attempt {n}/{max_attempts})"`.
When a slot opens, `clr` proceeds without a message. After 50 failed attempts (no slot
opened), `clr` emits:
`"Error: --max-sessions {count}/{max} active; gave up after {max_attempts} attempts."`
and exits with code 1.

**Note:** In `--dry-run` mode, the session gate is not triggered — the command preview
is printed immediately without checking or waiting for active sessions.

**Note:** `0` = unlimited: the gate is completely disabled and `clr` proceeds immediately
without scanning for active sessions.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 20 | Gate applied before subprocess launch |
| 5 | [`ask`](../command/05_ask.md) | 20 | Same behavior; pure alias for run |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 25 | [025_concurrency_gate.md](../user_story/025_concurrency_gate.md) | Developer |
