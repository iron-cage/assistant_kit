# CLI Parameter: --validation-delay

Number of seconds to wait between automatic retry attempts when the Validation
error class (exit 3, `--expect` mismatch) triggers a retry. Has no effect when
effective retry count for Validation is 0 or when `--expect-strategy` is not
`retry`.

- **Type:** u32 (seconds)
- **Default:** `auto` (inherits from `--retry-default-delay`, Tier 3 fallback)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr ask "yes or no?" --expect "yes|no" --expect-strategy retry --validation-delay 5
CLR_VALIDATION_DELAY=5 clr ask "confirm?" --expect "yes" --expect-strategy retry
```

**Note:** Default is `auto` — inherits from `--retry-default-delay` (Tier 3
fallback, default 30s). `--retry-override-delay` (Tier 1) beats this value when set.

**Note:** Only meaningful when `--expect-strategy retry` is active. With
`--expect-strategy fail`, the runner exits 3 immediately — no delay is applied.

**Note:** The delay is applied **between** attempts: after a Validation mismatch
and before the next subprocess invocation.

**Note:** A value of `0` causes an immediate retry. Useful for LLM re-prompting
where the cost of waiting is higher than the cost of an extra API call.

**Env var:** `CLR_VALIDATION_DELAY` — accepts a decimal integer string (u32 seconds);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override-delay ?? --validation-delay ?? --retry-default-delay (30)
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

- [`--retry-on-validation`](048_retry_on_validation.md) — enables Validation retry and sets max count
- [`--expect`](030_expect.md) — defines the expected values
- [`--expect-strategy`](031_expect_strategy.md) — must be `retry` for delay to apply
- [`--retry-override-delay`](055_retry_override_delay.md) — Tier 1: overrides all class-specific delays
- [`--retry-default-delay`](057_retry_default_delay.md) — Tier 3: fallback delay for unset classes
