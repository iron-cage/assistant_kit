# CLI Parameter: --gate-stale-secs

Staleness threshold (seconds) for reclaiming a live-but-stalled slot in the `--max-sessions`
concurrency gate. When a slot's recorded owner process is alive in `/proc` but has not
updated its gate state file within `stale_secs` seconds, `clr` treats it as stalled and
reclaims the slot — allowing the waiting invocation to proceed. When unset (the default),
live slot owners are never reclaimed regardless of how long they have been running. Does
not apply to `isolated` (env-var resolution only there).

- **Type:** u64 (seconds), optional
- **Default:** unset (`None` — feature disabled; live owners deny unconditionally)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`topic`](../command/11_topic.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"gate-stale-secs"`

```sh
clr --max-sessions 2 --gate-stale-secs 300 "task"     # reclaim if owner stalled >5min
clr --max-sessions 1 --gate-stale-secs 3600 "task"    # reclaim if owner stalled >1h
CLR_GATE_STALE_SECS=600 clr --max-sessions 2 "task"   # env-var equivalent
```

**Note:** An unset or invalid value resolves to `None` (feature off) — never a numeric
fallback. Setting `--gate-stale-secs 0` is equivalent to "stale immediately" and would
reclaim any live slot on first poll (use with care). This "no fallback" contract for
`None` was established by BUG-400 to prevent silent staleness-reclaim in configurations
that never opted into it.

**Note:** The staleness check compares the gate state file's last-modified time against
`wall_clock - stale_secs`. A live process that is genuinely still running but hasn't
updated its state file within `stale_secs` will be reclaimed — set conservatively if
Claude Code sessions may legitimately be silent for long periods.

**5-tier resolution (`run`/`ask` only):**
1. `--gate-stale-secs` CLI flag (highest priority)
2. `"gate-stale-secs"` JSON key via `--args-file`
3. `CLR_GATE_STALE_SECS` env var
4. `gate_stale_secs` config-file key
5. Default `None` / unset (lowest priority — feature off)

`isolated` resolves this value env-var-only (`CLR_GATE_STALE_SECS` + default), consistent
with its narrower parameter surface; see
[003_env_param.md](../003_env_param.md#env-param-5-gate-runtime-configuration).

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, `--max-sessions`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | unset (`None`) | Full 5-tier resolution; feature off by default |
| 5 | [`ask`](../command/05_ask.md) | unset (`None`) | Alias for run; same behavior |
| 11 | [`topic`](../command/11_topic.md) | unset (`None`) | Identical to `ask`; delegates to `run`'s handler |

### See Also

- [`033_max_sessions.md`](033_max_sessions.md) — concurrency gate overview and cross-reference to all 3 tuning knobs
- [`082_gate_poll_secs.md`](082_gate_poll_secs.md) — poll interval between gate attempts
- [`083_gate_max_attempts.md`](083_gate_max_attempts.md) — attempt limit before exhaustion
- [`003_env_param.md`](../003_env_param.md#env-param-5-gate-runtime-configuration) — env var details (Section 5)
