# CLI Parameter: --gate-poll-secs

Seconds to sleep between polling attempts when the `--max-sessions` concurrency gate is
waiting for a slot. After each failed admission attempt, `clr` sleeps `poll_secs` seconds
before trying again. Does not apply to `isolated` (env-var resolution only there).

- **Type:** u64 (seconds)
- **Default:** `30`
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"gate-poll-secs"`

```sh
clr --max-sessions 2 --gate-poll-secs 5 "task"     # poll every 5s instead of 30s
clr --max-sessions 1 --gate-poll-secs 1 "task"     # aggressive polling (1s interval)
CLR_GATE_POLL_SECS=10 clr --max-sessions 2 "task"  # env-var equivalent
clr --max-sessions 1 --gate-poll-secs 5 --gate-max-attempts 12 --retry-override 0 "task"
# total gate-wait ceiling = (12-1) × 5s = 55s before exhaustion
```

**Note:** `clr` sleeps `poll_secs` between attempts but NOT after the final attempt, so an
`N`-attempt sequence elapses `(N-1) × poll_secs` seconds before gate exhaustion fires. See
[`--gate-max-attempts`](083_gate_max_attempts.md) for controlling attempt count.

**Note:** The default 30-second poll interval is intentionally conservative to avoid thrashing
the `/proc` filesystem under load. In automated pipelines that want fast failure detection,
set both `--gate-poll-secs` and `--gate-max-attempts` to smaller values.

**5-tier resolution (`run`/`ask` only):**
1. `--gate-poll-secs` CLI flag (highest priority)
2. `"gate-poll-secs"` JSON key via `--args-file`
3. `CLR_GATE_POLL_SECS` env var
4. `gate_poll_secs` config-file key
5. Default `30` (lowest priority)

`isolated` resolves this value env-var-only (`CLR_GATE_POLL_SECS` + default), consistent
with its narrower parameter surface; see
[003_env_param.md](../003_env_param.md#env-param-5-gate-runtime-configuration).

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, `--max-sessions`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | `30` | Full 5-tier resolution |
| 5 | [`ask`](../command/05_ask.md) | `30` | Alias for run; same behavior |

### See Also

- [`033_max_sessions.md`](033_max_sessions.md) — concurrency gate overview and cross-reference to all 3 tuning knobs
- [`083_gate_max_attempts.md`](083_gate_max_attempts.md) — attempt count (total gate-wait = `(max_attempts-1) × poll_secs`)
- [`084_gate_stale_secs.md`](084_gate_stale_secs.md) — staleness reclaim threshold
- [`003_env_param.md`](../003_env_param.md#env-param-5-gate-runtime-configuration) — env var details (Section 5)
