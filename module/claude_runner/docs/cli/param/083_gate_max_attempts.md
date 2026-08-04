# CLI Parameter: --gate-max-attempts

Maximum number of admission attempts before the `--max-sessions` concurrency gate declares
exhaustion. When all `max_attempts` have failed without a slot opening, the gate routes
through the Runner-class retry wrapper (`apply_runner_retry()`): with retries remaining,
a `[Runner] ... — retrying...` message is emitted and the full attempt sequence restarts;
on final exhaustion (e.g. `--retry-override 0`) `clr` emits an error and exits 1. Does
not apply to `isolated` (env-var resolution only there).

- **Type:** u32
- **Default:** `1000`
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"gate-max-attempts"`

```sh
clr --max-sessions 2 --gate-max-attempts 10 "task"     # give up after 10 attempts
clr --max-sessions 1 --gate-poll-secs 5 --gate-max-attempts 12 --retry-override 0 "task"
# total gate-wait ceiling = (12-1) × 5s = 55s before exit 1
CLR_GATE_MAX_ATTEMPTS=5 clr --max-sessions 2 "task"    # env-var equivalent
```

**Note:** `clr` sleeps `poll_secs` between attempts but NOT after the final attempt, so an
`N`-attempt sequence elapses `(N-1) × poll_secs` seconds before gate exhaustion fires. The
default `1000` × `30s` poll interval = ~500 minutes total gate-wait before exhaustion in
the default configuration. See [`--gate-poll-secs`](082_gate_poll_secs.md) for the interval.

**5-tier resolution (`run`/`ask` only):**
1. `--gate-max-attempts` CLI flag (highest priority)
2. `"gate-max-attempts"` JSON key via `--args-file`
3. `CLR_GATE_MAX_ATTEMPTS` env var
4. `gate_max_attempts` config-file key
5. Default `1000` (lowest priority)

`isolated` resolves this value env-var-only (`CLR_GATE_MAX_ATTEMPTS` + default), consistent
with its narrower parameter surface; see
[003_env_param.md](../003_env_param.md#env-param-5-gate-runtime-configuration).

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, `--max-sessions`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | `1000` | Full 5-tier resolution |
| 5 | [`ask`](../command/05_ask.md) | `1000` | Alias for run; same behavior |

### See Also

- [`033_max_sessions.md`](033_max_sessions.md) — concurrency gate overview and cross-reference to all 3 tuning knobs
- [`082_gate_poll_secs.md`](082_gate_poll_secs.md) — poll interval (total gate-wait = `(max_attempts-1) × poll_secs`)
- [`084_gate_stale_secs.md`](084_gate_stale_secs.md) — staleness reclaim threshold
- [`003_env_param.md`](../003_env_param.md#env-param-5-gate-runtime-configuration) — env var details (Section 5)
