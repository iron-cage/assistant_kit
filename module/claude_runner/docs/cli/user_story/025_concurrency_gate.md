# Gate new invocations when concurrent session limit is reached

**Persona:** Developer or CI system that runs multiple automated `clr` invocations in parallel and needs to avoid hitting Claude API rate limits caused by too many concurrent sessions.
**Goal:** Limit the number of concurrent non-interactive Claude Code sessions across `run`, `ask`, and `isolated` so that new `clr` invocations wait when the system already has `--max-sessions` active non-interactive sessions, reducing rate limit errors from parallel pipelines. Interactive invocations are never subject to this gate.
**Benefit:** Prevents rate-limit errors from parallel automation by serializing new invocations when the session limit is reached.
**Priority:** Medium

### Acceptance Criteria

<!-- BUG-480 — fixed: AC-001 now states the full conjunction (census AND slot claim); census-only "proceeds immediately" had been false since BUG-387's slot-CAS second condition -->
- AC-001: Admission is a conjunction: when active non-interactive Claude processes < `--max-sessions` AND `clr` atomically claims a gate slot file, it proceeds immediately with no gate messages to stderr. A census below the limit alone is not sufficient — if every slot file is held by a live owner, `clr` waits with a `slot held by another session` diagnostic naming the measured occupancy (`slots=H/M`; see [invariant/013](../../invariant/013_slot_wait_message_differentiation.md))
- AC-002: When active non-interactive Claude processes >= `--max-sessions`, `clr` emits a waiting message to stderr (unless `--quiet`) and polls every 30 seconds
- AC-003: When 1000 attempts are exhausted without a slot opening, `clr` emits an error message to stderr and exits with code 1
- AC-004: `--max-sessions 0` disables the gate; `clr` proceeds immediately with no process scan or messages
- AC-005: `CLR_MAX_SESSIONS=N` is equivalent to `--max-sessions N` when the CLI flag is absent; CLI flag wins when both are present
- AC-006: In `--dry-run` mode, the gate is not triggered; the command preview is produced immediately
- AC-007: Interactive invocations are never gated — they proceed immediately regardless of `--max-sessions` or the number of active sessions
- AC-008: The active session count used for gating counts only non-interactive (print-mode) Claude processes; interactive sessions are excluded from the count
- AC-009: For `run`/`ask`, `--gate-poll-secs`/`--gate-max-attempts`/`--gate-stale-secs` (equivalently `CLR_GATE_POLL_SECS`/`CLR_GATE_MAX_ATTEMPTS`/`CLR_GATE_STALE_SECS`, a `"gate-poll-secs"`/`"gate-max-attempts"`/`"gate-stale-secs"` JSON key, or a `gate_poll_secs`/`gate_max_attempts`/`gate_stale_secs` config-file key) override the gate's poll interval (default 30s), attempt limit (default 1000), and staleness reclaim threshold (default unset); invalid values silently fall back to the default. For `isolated`, these 3 knobs remain env-var-only — no CLI flag, JSON key, or config-file tier.
- AC-010: `clr` sleeps between attempts but not after the final attempt, so an `N`-attempt sequence elapses `(N-1) * poll_secs` seconds before the gate-exhaustion path fires
- AC-011: An *expressed* finite `--timeout N` (flag or applied `CLR_TIMEOUT`) defaults the
  gate-wait budget to `N` seconds when `CLR_REMAINING_TIMEOUT_SECS` is absent or unparseable
  (BUG-445); a parseable `CLR_REMAINING_TIMEOUT_SECS` wins. With no expressed timeout (the
  built-in defaults never qualify) or an explicit `--timeout 0` opt-out, the gate's
  poll/attempt ceiling (`CLR_GATE_POLL_SECS`/`CLR_GATE_MAX_ATTEMPTS`, default ~8.3h) applies
  independently of the execution timeout; see [036_timeout.md](../param/036_timeout.md)
- AC-012: `clr isolated` is gated by `--max-sessions` through the same 3-tier chain as `run`/`ask` (CLI flag + `"max-sessions"` JSON key via `--args-file` + `CLR_MAX_SESSIONS` env var — no config-file tier); `--dry-run` bypasses the gate for `isolated` exactly as it does for `run`/`ask`
- AC-013: When the process scanner cannot read the process list (e.g. `/proc` unavailable), `clr` fails loudly with a `GateUnavailable` Runner-class error instead of silently proceeding as if the gate were disabled; `--max-sessions 0` bypasses this check entirely

<!-- BUG-399 (task/claude_runner/bug/completed/399_timeout_gate_wait_undocumented.md) —
     originally: --timeout did not bound this gate-wait phase, by design, and this doc did
     not cross-reference that boundary. Superseded for expressed timeouts by BUG-445 Fix
     Location #2 (AC-011 above). See 036_timeout.md and param/033_max_sessions.md. -->

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`run`](../command/01_run.md) | Primary command; gate applies before subprocess launch |
| 5 | [`ask`](../command/05_ask.md) | Same behavior; pure alias for run |
| 3 | [`isolated`](../command/03_isolated.md) | Same gate mechanism; 3-tier (CLI flag + `"max-sessions"` JSON key via `--args-file` + `CLR_MAX_SESSIONS` env var; no config-file tier) |
| 11 | [`topic`](../command/11_topic.md) | Same gate mechanism as `run`/`ask` via delegation |

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
7. `clr --gate-poll-secs 5 --gate-max-attempts 12 --max-sessions 1 --retry-override 0 "task"` — CLI-flag equivalent of step 6, for `run`/`ask` only (`isolated` has no `--gate-poll-secs`/`--gate-max-attempts` flags)
8. `clr isolated --max-sessions 1 "task"` (1 non-interactive session already active) — `isolated` now waits for a slot exactly like `run`/`ask`, instead of bypassing the gate entirely
9. `clr --max-sessions 5 "task"` with `/proc` unavailable — `clr` fails loudly with a `GateUnavailable` Runner-class error instead of silently proceeding as if the gate were disabled; `clr --max-sessions 0 "task"` still bypasses this check

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 22 | [022_session_isolation_topic.md](022_session_isolation_topic.md) | `--topic` isolates sessions by topic; `--max-sessions` limits total concurrent count |
| 18 | [018_env_var_configuration.md](018_env_var_configuration.md) | `CLR_MAX_SESSIONS` is an instance of the CLR_* env var system |
