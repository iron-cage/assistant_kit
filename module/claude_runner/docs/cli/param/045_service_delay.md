# CLI Parameter: --service-delay

Number of seconds to wait between automatic retry attempts when the Service
error class (`ErrorKind::ApiError`) triggers a retry. Has no effect when
effective retry count for Service is 0.

- **Type:** u32 (seconds)
- **Default:** `auto` (inherits from `--retry-default-delay`, Tier 3 fallback)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Replaces:** `--api-error-delay` (renamed + renumbered from 038)

```sh
clr -p "task" --retry-on-service 2 --service-delay 60  # wait 60s between retries
clr -p "task" --retry-on-service 1 --service-delay 0   # retry immediately (no sleep)
CLR_SERVICE_DELAY=60 clr -p "task"                     # env-var equivalent
```

**Note:** Default is `auto` — inherits from `--retry-default-delay` (Tier 3
fallback, default 30s). `--retry-override-delay` (Tier 1) beats this value when set.

**Note:** The delay is applied **between** attempts: after a Service error is
detected and before the next subprocess invocation. There is no delay before the
first attempt.

**Note:** A value of `0` causes an immediate retry with no sleep between attempts.

**Env var:** `CLR_SERVICE_DELAY` — accepts a decimal integer string (u32 seconds);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override-delay ?? --service-delay ?? --retry-default-delay (30)
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | auto | Delay applied inside retry wrapper |
| 5 | [`ask`](../command/05_ask.md) | auto | Same behavior; pure alias for run |

### See Also

- [`--retry-on-service`](044_retry_on_service.md) — enables Service retry and sets max count
- [`--retry-override-delay`](055_retry_override_delay.md) — Tier 1: overrides all class-specific delays
- [`--retry-default-delay`](057_retry_default_delay.md) — Tier 3: fallback delay for unset classes
