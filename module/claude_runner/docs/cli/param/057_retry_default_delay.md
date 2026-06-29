# CLI Parameter: --retry-default-delay

Tier 3 fallback: delay (in seconds) between retry attempts for any error class
whose class-specific `--<class>-delay` is unset (`auto`) and whose Tier 1
override delay is also unset. This is the only retry delay parameter with a
concrete built-in default.

- **Type:** u32 (seconds)
- **Default:** `30` (thirty seconds between retry attempts)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
# Increase fallback delay for all classes
clr -p "task" --retry-default-delay 60

# Immediate retry for all classes without explicit delay
clr -p "task" --retry-default-delay 0

# Fallback=60s, but Service specifically set to 5s
clr -p "task" --retry-default-delay 60 --service-delay 5

CLR_RETRY_DEFAULT_DELAY=60 clr -p "task"              # env-var equivalent
```

**Note:** `--retry-default-delay 0` causes immediate retries for all classes
that don't have an explicit class-specific delay. Classes with explicit
`--<class>-delay` values are unaffected.

**Note:** The delay is applied **between** attempts: after an error is detected
and before the next subprocess invocation. There is no delay before the first
attempt.

**Env var:** `CLR_RETRY_DEFAULT_DELAY` — accepts a decimal integer string (u32
seconds); invalid values are silently ignored (parse failure → field stays at
default 30); CLI flag wins when both are present.

### 3-Tier Resolution

```
effective_delay(class) = --retry-override-delay ?? --<class>-delay ?? --retry-default-delay (30)
```

This parameter is Tier 3 — the rightmost (lowest priority, concrete default) in the chain.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 30 | Fallback delay in `run_print_mode()` |
| 5 | [`ask`](../command/05_ask.md) | 30 | Same behavior; pure alias for run |

### See Also

- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count
- [`--retry-override-delay`](055_retry_override_delay.md) — Tier 1: beats this value when set
- [`type/14_error_class.md`](../type/14_error_class.md) § Strategy Configuration — full 3-tier hierarchy
