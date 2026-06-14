# CLI Parameter: --api-error-delay

Number of seconds to wait between automatic retry attempts when
`--retry-on-api-error` is active and an `ErrorKind::ApiError` is detected.
Has no effect when `--retry-on-api-error` is `0`.

- **Type:** u32 (seconds)
- **Default:** `30`
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr -p "task" --retry-on-api-error 2 --api-error-delay 30  # wait 30s between retries
clr -p "task" --retry-on-api-error 1 --api-error-delay 0   # retry immediately (no sleep)
CLR_API_ERROR_DELAY=30 clr -p "task" --retry-on-api-error 1  # env-var equivalent
clr -p "task" --retry-on-api-error 0 --api-error-delay 10  # silently ignored (no retry configured)
```

**Note:** The delay is applied **between** attempts: after an API error is
detected and before the next subprocess invocation. There is no delay before the
first attempt.

**Note:** A value of `0` causes an immediate retry with no sleep between attempts.
Useful for integration tests that use fake scripts with controlled exit sequences.

**Note:** `--api-error-delay` is silently ignored when `--retry-on-api-error` is `0`
(no retry configured) — the parameter is parsed and accepted without error, but
the delay value is never used.

**Note:** This delay is independent of `--retry-delay` (which controls rate-limit
retries). Each retry mechanism has its own delay parameter.

**Env var:** `CLR_API_ERROR_DELAY` — accepts a decimal integer string (u32 seconds);
invalid values are silently ignored (parse failure -> field stays at default 30);
CLI flag wins when both are present.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 30 | Delay applied inside API error retry wrapper |
| 5 | [`ask`](../command/05_ask.md) | 30 | Same behavior; pure alias for run |

### See Also

- [`--retry-on-api-error`](037_retry_on_api_error.md) — enables API error retry and sets max count
- [`--retry-delay`](035_retry_delay.md) — delay for rate-limit retries (separate mechanism)
