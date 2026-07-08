# Gate new invocations when concurrent session limit is reached

**Persona:** Developer or CI system that runs multiple automated `clr` invocations in parallel and needs to avoid hitting Claude API rate limits caused by too many concurrent sessions.
**Goal:** Limit the number of concurrent non-interactive Claude Code sessions so that new `clr` invocations wait when the system already has `--max-sessions` active non-interactive sessions, reducing rate limit errors from parallel pipelines. Interactive invocations are never subject to this gate.
**Benefit:** Prevents rate-limit errors from parallel automation by serializing new invocations when the session limit is reached.
**Priority:** Medium

### Acceptance Criteria

- AC-001: When active non-interactive Claude processes < `--max-sessions`, `clr` proceeds immediately with no gate messages to stderr
- AC-002: When active non-interactive Claude processes >= `--max-sessions`, `clr` emits a waiting message to stderr (unless `--quiet`) and polls every 30 seconds
- AC-003: When 1000 attempts are exhausted without a slot opening, `clr` emits an error message to stderr and exits with code 1
- AC-004: `--max-sessions 0` disables the gate; `clr` proceeds immediately with no process scan or messages
- AC-005: `CLR_MAX_SESSIONS=N` is equivalent to `--max-sessions N` when the CLI flag is absent; CLI flag wins when both are present
- AC-006: In `--dry-run` mode, the gate is not triggered; the command preview is produced immediately
- AC-007: Interactive invocations are never gated — they proceed immediately regardless of `--max-sessions` or the number of active sessions
- AC-008: The active session count used for gating counts only non-interactive (print-mode) Claude processes; interactive sessions are excluded from the count
- AC-009: `CLR_GATE_POLL_SECS`/`CLR_GATE_MAX_ATTEMPTS` override the gate's poll interval (default 30s) and attempt limit (default 1000) with no corresponding CLI flag or JSON key; invalid values silently fall back to the default
- AC-010: `clr` sleeps between attempts but not after the final attempt, so an `N`-attempt sequence elapses `(N-1) * poll_secs` seconds before the gate-exhaustion path fires

<!-- BUG-399 (task/claude_runner/bug/unverified/399_timeout_gate_wait_undocumented.md) —
     --timeout does not bound this gate-wait phase, by design; this doc did not
     cross-reference that boundary. See 036_timeout.md and param/033_max_sessions.md. -->

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
5. `clr --interactive "task"` (20 non-interactive sessions active, `--max-sessions 10`) — interactive invocations bypass the gate entirely and proceed immediately, regardless of active count
6. `CLR_GATE_POLL_SECS=5 CLR_GATE_MAX_ATTEMPTS=12 clr --max-sessions 1 --retry-override 0 "task"` — gate exhausts after ~55s (11 sleeps x 5s) instead of the ~29970s production default; `--retry-override 0` disables the runner-retry wrapper so exhaustion surfaces on the first pass

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 22 | [022_session_isolation_subdir.md](022_session_isolation_subdir.md) | `--subdir` isolates sessions by topic; `--max-sessions` limits total concurrent count |
| 18 | [018_env_var_configuration.md](018_env_var_configuration.md) | `CLR_MAX_SESSIONS` is an instance of the CLR_* env var system |
