# CLI Parameter: --retry-delay

Number of seconds to wait between automatic retry attempts when
`--retry-on-rate-limit` is active and a transient rate-limit exit is detected.
Has no effect when `--retry-on-rate-limit` is `0` (the default).

- **Type:** u32 (seconds)
- **Default:** `60`
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr -p "task" --retry-on-rate-limit 2 --retry-delay 30  # wait 30s between retries
clr -p "task" --retry-on-rate-limit 1 --retry-delay 0   # retry immediately (no sleep)
CLR_RETRY_DELAY=30 clr -p "task" --retry-on-rate-limit 1  # env-var equivalent
clr -p "task" --retry-on-rate-limit 0 --retry-delay 10  # silently ignored (no retry configured)
```

**Note:** The delay is applied **between** attempts: after a rate-limit exit is
detected and before the next subprocess invocation. There is no delay before the
first attempt.

**Note:** A value of `0` causes an immediate retry with no sleep between attempts.
Useful for integration tests that use fake scripts with controlled exit sequences.

**Note:** `--retry-delay` is silently ignored when `--retry-on-rate-limit` is `0`
(no retry configured) — the parameter is parsed and accepted without error, but
the delay value is never used.

**Env var:** `CLR_RETRY_DELAY` — accepts a decimal integer string (u32 seconds);
invalid values are silently ignored (parse failure → field stays at default 60);
CLI flag wins when both are present.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 60 | Delay applied inside retry wrapper |
| 5 | [`ask`](../command/05_ask.md) | 60 | Same behavior; pure alias for run |

### See Also

- [`--retry-on-rate-limit`](034_retry_on_rate_limit.md) — enables retry and sets max count
