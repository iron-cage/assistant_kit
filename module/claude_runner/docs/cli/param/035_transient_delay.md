# CLI Parameter: --transient-delay

Number of seconds to wait between automatic retry attempts when the Transient
error class (`ErrorKind::RateLimit`) triggers a retry. Has no effect when
effective retry count for Transient is 0.

- **Type:** u32 (seconds)
- **Default:** `auto` (inherits from `--retry-default-delay`, Tier 3 fallback; effective default = 30)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Replaces:** `--retry-delay` (renamed; no longer shared with Unknown class)

```sh
clr -p "task" --retry-on-transient 2 --transient-delay 60  # wait 60s between retries
clr -p "task" --retry-on-transient 1 --transient-delay 0   # retry immediately (no sleep)
CLR_TRANSIENT_DELAY=60 clr -p "task"                       # env-var equivalent
```

**Note:** Default is `auto` — inherits from `--retry-default-delay` (Tier 3
fallback, default 30s). `--retry-override-delay` (Tier 1) beats this value when set.

**Note:** The delay is applied **between** attempts: after a Transient error is
detected and before the next subprocess invocation. There is no delay before the
first attempt.

**Note:** A value of `0` causes an immediate retry with no sleep between attempts.
Useful for integration tests that use fake scripts with controlled exit sequences.

**Env var:** `CLR_TRANSIENT_DELAY` — accepts a decimal integer string (u32 seconds);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override-delay ?? --transient-delay ?? --retry-default-delay (30)
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

- [`--retry-on-transient`](034_retry_on_transient.md) — enables Transient retry and sets max count
- [`--retry-override-delay`](055_retry_override_delay.md) — Tier 1: overrides all class-specific delays
- [`--retry-default-delay`](057_retry_default_delay.md) — Tier 3: fallback delay for unset classes
