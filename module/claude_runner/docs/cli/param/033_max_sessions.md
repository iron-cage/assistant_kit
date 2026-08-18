# CLI Parameter: --max-sessions

Maximum number of concurrent non-interactive (print-mode) Claude Code sessions allowed
before this invocation blocks. Interactive invocations are never gated — they proceed
immediately regardless of this limit or the number of active sessions. When the active
non-interactive session count meets or exceeds this limit, `clr` polls every 30 seconds
for up to 1000 attempts, then exits with code 1. Setting `0` disables the gate entirely
(unlimited sessions, no process scan).

- **Type:** u32
- **Default:** 8
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`topic`](../command/11_topic.md), [`isolated`](../command/03_isolated.md)
- **JSON Key:** `"max-sessions"` (run/ask/isolated all three — `isolated` sets it via `apply_json_config_isolated()` same as run/ask; no config-file tier for `isolated`, which has no config-file tier for any parameter)

```sh
clr --max-sessions 5 "refactor module"      # block if >=5 Claude sessions active
clr --max-sessions 1 "single task"          # strict: only 1 session at a time
clr --max-sessions 0 "unrestricted"         # gate disabled; proceeds immediately
CLR_MAX_SESSIONS=3 clr "fix bug"            # env-var equivalent of --max-sessions 3
clr --max-sessions 3 --dry-run "preview"    # dry-run: gate skipped; shows assembled command
```

**Note:** Session count is determined by scanning `/proc/{pid}/cmdline` for entries
whose basename is exactly `"claude"`, excluding the calling process, **counting only
non-interactive (print-mode) processes**. The count reflects all running non-interactive
Claude Code processes system-wide, not per-project.

<!-- BUG-480 task/claude_runner/bug/480_gate_diagnostic_hides_slot_occupancy.md — fixed: verbatim gate-wait format below carries the conditional slots={held}/{max} field (slot-side causes only); exhaustion messages carry slots={held}/{max} held -->
**Note:** When the gate waits, `clr` emits a structured timestamp-prefixed message to stderr each
polling cycle (unless `--quiet`) in the form:
`"{ts}gate-wait  active={count}/{max}[ slots={held}/{max}] attempt={n}/{effective_max} wait={poll_secs}s (reason: {cause})"`,
where `{ts}` is a `claude_core::trace_ts()` prefix (`"YYYY-MM-DD · HH:MM:SS UTC · "`), `{cause}` is
one of `"at capacity"`, `"slot held by another session"`, or `"lost reservation race"` (INV-013),
`[ slots={held}/{max}]` is the measured slot occupancy, present only when `{cause}` is one of the
two slot-side causes and omitted on `"at capacity"` lines (the acquisition sweep never ran there,
so occupancy is unmeasured — INV-013 § Measured occupancy, BUG-480),
and `{effective_max}` reflects the `CLR_REMAINING_TIMEOUT_SECS` budget clamp if active (see
[`085_gate_remaining_timeout_secs.md`](085_gate_remaining_timeout_secs.md)). The first denied
attempt of a gate entry is additionally preceded by a one-time `gate-deadline` line announcing
the resolved state of the deadline clamp and the staleness-reclaim protection (BUG-481; format
in [`085_gate_remaining_timeout_secs.md`](085_gate_remaining_timeout_secs.md)). When a slot opens,
`clr` proceeds without a message. After `effective_max` failed attempts, gate exhaustion is routed
through the Runner-class retry wrapper (`apply_runner_retry()`): on final exhaustion (e.g.
`--retry-override 0`) `clr` emits `"Error: [Runner] session gate timed out — {count} print
sessions, max-sessions={max}[, slots={held}/{max} held] — retries exhausted (exit 1)"` and exits with
code 1 (budget exhaustion emits `"gate-wait budget exhausted"` instead, with the same conditional
`[, slots={held}/{max} held]` suffix — see
[`085_gate_remaining_timeout_secs.md`](085_gate_remaining_timeout_secs.md)); otherwise it emits a
`[Runner] ... — retrying...` message and restarts the full attempt sequence. For `run`/`ask`, `poll_secs` (default
30), `max_attempts` (default 1000), and the opt-in staleness reclaim threshold (default unset) are
tunable across the full 5-tier chain — `--gate-poll-secs`/`--gate-max-attempts`/`--gate-stale-secs`
CLI flags (see [`082_gate_poll_secs.md`](082_gate_poll_secs.md),
[`083_gate_max_attempts.md`](083_gate_max_attempts.md),
[`084_gate_stale_secs.md`](084_gate_stale_secs.md)), `"gate-poll-secs"`/`"gate-max-attempts"`/`"gate-stale-secs"` JSON keys,
`CLR_GATE_POLL_SECS`/`CLR_GATE_MAX_ATTEMPTS`/`CLR_GATE_STALE_SECS` env vars, and
`gate_poll_secs`/`gate_max_attempts`/`gate_stale_secs` config-file keys — see
[003_env_param.md](../003_env_param.md#env-param-5-gate-runtime-configuration). `isolated` resolves
these same 3 knobs env-var-only (no CLI flag, JSON key, or config-file tier), consistent with its
narrower parameter surface elsewhere. `clr` sleeps `poll_secs` between attempts but not after the
final attempt, so an `N`-attempt sequence elapses `(N-1) * poll_secs` seconds before exhaustion fires.

**Note:** Before polling begins, `clr` verifies the process scanner can actually read the process
list. If it cannot (e.g. `/proc` is unavailable, or `CLR_PROC_DIR` is misconfigured), `clr` fails
loudly instead of silently proceeding as if the gate were disabled: `"Error: [Runner] session gate
unavailable — process scanner cannot read the process list (--max-sessions requires working /proc;
pass --max-sessions 0 to disable the gate) (exit 1)"`. This check is skipped entirely when
`--max-sessions 0` — the disable escape hatch survives even a broken process scanner.

<!-- BUG-399 (task/claude_runner/bug/completed/399_timeout_gate_wait_undocumented.md) —
     originally: --timeout did not bound this gate-wait phase, by design. Superseded for
     expressed timeouts by BUG-445 Fix Location #2 (the note below). See 036_timeout.md
     and user_story/025_concurrency_gate.md. -->

**Note:** This gate-wait ceiling is independent of `--timeout` only when no timeout is
expressed. An *expressed* `--timeout N` (flag or applied `CLR_TIMEOUT`) defaults the
gate-wait budget to `N` seconds when `CLR_REMAINING_TIMEOUT_SECS` is absent or unparseable
(BUG-445; a parseable env var wins, and the built-in defaults — 30 s `isolated`, and
print-mode's, 3600 s then and 0 since TSK-503 — never reach the gate). See
[036_timeout.md](036_timeout.md) and
[085_gate_remaining_timeout_secs.md](085_gate_remaining_timeout_secs.md).

**Note:** In `--dry-run` mode, the session gate is not triggered — the command preview
is printed immediately without checking or waiting for active sessions.

**Note:** `0` = unlimited: the gate is completely disabled and `clr` proceeds immediately
without scanning for active sessions.

**Note:** Interactive invocations (no `-p`/`--print` and no non-interactive `--message`
dispatch) skip this gate entirely — they proceed immediately without a process scan,
regardless of `--max-sessions` or the number of active sessions.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 8 | Gate applied before subprocess launch; non-interactive only |
| 5 | [`ask`](../command/05_ask.md) | 8 | Same behavior; pure alias for run |
| 3 | [`isolated`](../command/03_isolated.md) | 8 | Same gate mechanism; CLI flag + `"max-sessions"` JSON key + `CLR_MAX_SESSIONS` (no config tier — `isolated` has none); `--dry-run` bypasses exactly as for run/ask |
| 11 | [`topic`](../command/11_topic.md) | 8 | Identical to `ask`; delegates to `run`'s handler |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 25 | [025_concurrency_gate.md](../user_story/025_concurrency_gate.md) | Developer |
