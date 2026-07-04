# Gate new invocations when concurrent session limit is reached

**Persona:** Developer or CI system that runs multiple automated `clr` invocations in parallel and needs to avoid hitting Claude API rate limits caused by too many concurrent sessions.
**Goal:** Limit the number of concurrent Claude Code sessions so that new `clr` invocations wait when the system already has `--max-sessions` active sessions, reducing rate limit errors from parallel pipelines.
**Benefit:** Prevents rate-limit errors from parallel automation by serializing new invocations when the session limit is reached.
**Priority:** Medium

### Acceptance Criteria

- AC-001: When active Claude processes < `--max-sessions`, `clr` proceeds immediately with no gate messages to stderr
- AC-002: When active Claude processes >= `--max-sessions`, `clr` emits a waiting message to stderr (unless `--quiet`) and polls every 30 seconds
- AC-003: When 100 attempts are exhausted without a slot opening, `clr` emits an error message to stderr and exits with code 1
- AC-004: `--max-sessions 0` disables the gate; `clr` proceeds immediately with no process scan or messages
- AC-005: `CLR_MAX_SESSIONS=N` is equivalent to `--max-sessions N` when the CLI flag is absent; CLI flag wins when both are present
- AC-006: In `--dry-run` mode, the gate is not triggered; the command preview is produced immediately

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Primary command; gate applies before subprocess launch |
| 5 | [`ask`](../command/05_ask.md) | Same behavior; pure alias for run |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | `--max-sessions` is a Runner Control flag |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 33 | [`--max-sessions`](../param/033_max_sessions.md) | Session count limit; `0` = unlimited |

### Workflow Steps

1. `clr --max-sessions 5 "task"` — gate new invocations when 5 or more Claude sessions are active
2. `CLR_MAX_SESSIONS=3 clr "task"` — apply session limit via environment variable
3. `clr --max-sessions 0 "task"` — disable the gate; proceed immediately regardless of active sessions
4. `clr --max-sessions 5 --dry-run "task"` — bypass the gate in dry-run mode

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 22 | [022_session_isolation_subdir.md](022_session_isolation_subdir.md) | `--subdir` isolates sessions by topic; `--max-sessions` limits total concurrent count |
| 18 | [018_env_var_configuration.md](018_env_var_configuration.md) | `CLR_MAX_SESSIONS` is an instance of the CLR_* env var system |
