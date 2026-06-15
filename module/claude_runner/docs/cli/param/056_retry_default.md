# CLI Parameter: --retry-default

Tier 3 fallback: retry count applied to any error class whose class-specific
`--retry-on-<class>` is unset (`auto`) and whose Tier 1 override is also
unset. This is the only retry parameter with a concrete built-in default.

- **Type:** u8 (0–255)
- **Default:** `2` (two automatic retries per error class)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
# Increase fallback retry count for all classes
clr -p "task" --retry-default 5

# Disable retry for all classes that don't have explicit class-specific settings
clr -p "task" --retry-default 0

# Fallback=3, but Transient specifically set to 0 (disabled)
clr -p "task" --retry-default 3 --retry-on-transient 0

CLR_RETRY_DEFAULT=5 clr -p "task"                    # env-var equivalent
```

**Note:** The value is the number of *re-invocations*, not total attempts.
`--retry-default 2` (the built-in default) means up to 3 total runs per class.

**Note:** `--retry-default 0` effectively disables retry for all error classes
that don't have an explicit class-specific count. Classes with explicit
`--retry-on-<class>` values are unaffected.

**Note:** Applies to print-mode execution (`run_print_mode()`) only.

**Env var:** `CLR_RETRY_DEFAULT` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at default 2);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective_count(class) = --retry-override ?? --retry-on-<class> ?? --retry-default (2)
```

This parameter is Tier 3 — the rightmost (lowest priority, concrete default) in the chain.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 2 | Fallback count in `run_print_mode()` |
| 5 | [`ask`](../command/05_ask.md) | 2 | Same behavior; pure alias for run |

### See Also

- [`--retry-default-delay`](057_retry_default_delay.md) — Tier 3: fallback delay
- [`--retry-override`](054_retry_override.md) — Tier 1: beats this value when set
- [`type/14_error_class.md`](../type/14_error_class.md) § Strategy Configuration — full 3-tier hierarchy
