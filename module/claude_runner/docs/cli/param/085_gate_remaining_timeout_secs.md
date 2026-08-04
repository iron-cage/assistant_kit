# Env Var: CLR_REMAINING_TIMEOUT_SECS

Remaining external timeout budget (in seconds) available to the `--max-sessions` concurrency
gate. When set, `wait_for_session_slot()` clamps its effective attempt count to
`floor(remaining_secs / poll_secs).max(1)` so gate-wait polling does not outlive a wrapping
job-runner deadline (e.g. a `wplan_executor` job with a finite wall-clock budget).

This is an env-var-only parameter: it is set by the **caller** (a job runner, orchestration
script, or operator wrapper) before spawning `clr`, never by the `clr` operator directly.
There is no CLI flag, JSON key, or config-file tier for this value.

- **Type:** u64 (seconds) or absent
- **Default:** absent (unset) — no budget clamp; gate polls to `CLR_GATE_MAX_ATTEMPTS`
- **Applies to:** `run`, `ask`, `isolated` (any invocation that triggers the concurrency gate)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
# job runner sets remaining budget before spawning clr:
CLR_REMAINING_TIMEOUT_SECS=3600 clr --max-sessions 3 "task"
# gate polls at most floor(3600/30)=120 attempts instead of default 1000

# tight budget — remaining is less than one poll interval:
CLR_REMAINING_TIMEOUT_SECS=10 CLR_GATE_POLL_SECS=30 clr --max-sessions 1 "task"
# floor(10/30)=0 → .max(1)=1 attempt (gate checks once, then emits budget-exhaustion error)
```

**Note:** When the budget clamp fires (effective attempts exhausted before
`CLR_GATE_MAX_ATTEMPTS`), `clr` emits a **distinct** diagnostic:
`"gate-wait budget exhausted — {count} print sessions, max-sessions={max}, budget={N} attempt(s)"`.
This is intentionally different from the ordinary gate-timeout message
(`"session gate timed out"`) so operators can identify budget-exhaustion events in job stderr
output without counting attempt lines manually.

**Note:** The `.max(1)` floor guarantees at least one admission attempt is always made before
the budget-exhaustion error fires — a remaining budget smaller than one poll interval does not
silently skip the gate check.

**Note:** When `CLR_REMAINING_TIMEOUT_SECS` is absent or non-numeric, the feature is off and
the gate behaves exactly as before (polls up to `CLR_GATE_MAX_ATTEMPTS`). Non-numeric values
resolve to `None` silently — no error, no crash.

**Note:** The effective attempt count shown in gate progress lines reflects the clamped value
when a budget is imposed: `attempt={n}/{effective_max}` rather than the unclamped
`CLR_GATE_MAX_ATTEMPTS`.

**1-tier resolution (env-var only — no CLI flag, JSON key, or config-file tier):**
1. `CLR_REMAINING_TIMEOUT_SECS` env var (the only tier — absent = feature off)

This differs from `CLR_GATE_POLL_SECS` / `CLR_GATE_MAX_ATTEMPTS` / `CLR_GATE_STALE_SECS`,
which support the full 5-tier chain for `run`/`ask`. `CLR_REMAINING_TIMEOUT_SECS` is
deliberately env-var-only because it is a runtime property injected by a calling process, not
a configuration choice made by the `clr` operator.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Env-var only | `--dry-run`, `--quiet`, `--trace`, `--max-sessions`, ... |

### See Also

- [`033_max_sessions.md`](033_max_sessions.md) — concurrency gate overview
- [`082_gate_poll_secs.md`](082_gate_poll_secs.md) — poll interval (determines how many budget-clamp attempts fit in the remaining window)
- [`083_gate_max_attempts.md`](083_gate_max_attempts.md) — the attempt ceiling that `CLR_REMAINING_TIMEOUT_SECS` clamps below
- [`003_env_param.md`](../003_env_param.md#env-param-5-gate-runtime-configuration) — gate env var details (Section 5)
