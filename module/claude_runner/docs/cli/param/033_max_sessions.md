# CLI Parameter: --max-sessions

Maximum number of concurrent non-interactive (print-mode) Claude Code sessions allowed
before this invocation blocks. Interactive invocations are never gated — they proceed
immediately regardless of this limit or the number of active sessions. When the active
non-interactive session count meets or exceeds this limit, `clr` polls every 30 seconds
for up to 1000 attempts, then exits with code 1. Setting `0` disables the gate entirely
(unlimited sessions, no process scan).

- **Type:** u32
- **Default:** 6
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`isolated`](../command/03_isolated.md)
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

**Note:** When the gate waits, `clr` emits a message to stderr each polling cycle (unless `--quiet`):
`"Info: {count}/{max} sessions active; waiting {poll_secs}s for a slot... (attempt {n}/{max_attempts})"`.
When a slot opens, `clr` proceeds without a message. After `max_attempts` failed attempts (no slot
opened), gate exhaustion is routed through the Runner-class retry wrapper (`apply_runner_retry()`):
on final exhaustion (no retries remaining, e.g. `--retry-override 0`) `clr` emits
`"Error: [Runner] session gate timed out — {count} active sessions, max-sessions={max} — retries
exhausted (exit 1)"` and exits with code 1; otherwise it emits a `[Runner] ... — retrying...`
message and restarts the full `max_attempts`-poll sequence. For `run`/`ask`, `poll_secs` (default
30), `max_attempts` (default 1000), and the opt-in staleness reclaim threshold (default unset) are
tunable across the full 5-tier chain — `--gate-poll-secs`/`--gate-max-attempts`/`--gate-stale-secs`
CLI flags, `"gate-poll-secs"`/`"gate-max-attempts"`/`"gate-stale-secs"` JSON keys,
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

<!-- BUG-399 (task/claude_runner/bug/closed/399_timeout_gate_wait_undocumented.md) —
     --timeout does not bound this gate-wait phase, by design; this doc did not
     cross-reference that boundary. See 036_timeout.md and user_story/025_concurrency_gate.md. -->

**Note:** This gate-wait ceiling is entirely independent of `--timeout` (see
[036_timeout.md](036_timeout.md)) — `--timeout` only governs the subprocess-execution phase
that begins after the gate admits the invocation. An invocation queued in the gate is not
affected by `--timeout` at all.

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
| 1 | [`run`](../command/01_run.md) | 6 | Gate applied before subprocess launch; non-interactive only |
| 5 | [`ask`](../command/05_ask.md) | 6 | Same behavior; pure alias for run |
| 3 | [`isolated`](../command/03_isolated.md) | 6 | Same gate mechanism; CLI flag + `"max-sessions"` JSON key + `CLR_MAX_SESSIONS` (no config tier — `isolated` has none); `--dry-run` bypasses exactly as for run/ask |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 25 | [025_concurrency_gate.md](../user_story/025_concurrency_gate.md) | Developer |
