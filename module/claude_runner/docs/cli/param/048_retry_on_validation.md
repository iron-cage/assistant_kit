# CLI Parameter: --retry-on-validation

Maximum number of automatic retries when `--expect` validation fails (Validation
error class, exit code 3). When `--expect-strategy retry` is active and the
subprocess output does not match `--expect`, `clr` re-invokes the subprocess,
decrementing the retry counter. On exhaustion, the runner exits with code 3.

- **Type:** u8 (0–255)
- **Default:** `auto` (inherits from `--retry-default`, Tier 3 fallback)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Replaces:** `--expect-retries` (renamed + renumbered from 032)

```sh
# Try up to 4 times total (1 initial + 3 retries)
clr ask "yes or no?" --expect "yes|no" --expect-strategy retry --retry-on-validation 3

# Disable validation retry (overrides fallback default)
clr ask "yes or no?" --expect "yes|no" --expect-strategy retry --retry-on-validation 0

CLR_RETRY_ON_VALIDATION=3 clr ask "yes or no?" --expect "yes|no" --expect-strategy retry
```

**Note:** Default is `auto` — inherits from `--retry-default` (Tier 3 fallback,
default 2). Set to `0` to explicitly disable retry for Validation regardless of
fallback. `--retry-override` (Tier 1) beats this value when set.

**Note:** Only fires when `--expect-strategy retry` is active. With
`--expect-strategy fail` (or `default:V`), the runner exits 3 immediately on
mismatch — the Validation retry count is not consulted.

**Note:** The value is the number of *re-invocations*, not total attempts.
`--retry-on-validation 3` means up to 4 total runs (1 initial + 3 retries).

**Note:** Validation is a CLR-layer error class — it does not go through
`classify_error()` and has no `ErrorKind` variant. The trigger is the `--expect`
pattern mismatch check in `apply_expect_validation()`.

**Note:** When a retry fires, `clr` emits to stderr:
`"[Validation] output mismatch — retrying (attempt M/N)…"`.
On exhaustion: `"Error: [Validation] output did not match --expect (exit 3)"`.

**Env var:** `CLR_RETRY_ON_VALIDATION` — accepts a decimal integer string (0–255);
invalid values are rejected at parse time with exit 1;
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override ?? --retry-on-validation ?? --retry-default (2)
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | auto | 3-tier resolution in retry wrapper |
| 5 | [`ask`](../command/05_ask.md) | auto | Same behavior; pure alias for run |

### See Also

- [`--validation-delay`](049_validation_delay.md) — seconds to wait between Validation retry attempts
- [`--expect`](030_expect.md) — defines the expected values
- [`--expect-strategy`](031_expect_strategy.md) — must be `retry` for this param to take effect
- [`--retry-override`](054_retry_override.md) — Tier 1: overrides all class-specific counts
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count for unset classes
- [`type/14_error_class.md`](../type/14_error_class.md) § Validation — error class definition
