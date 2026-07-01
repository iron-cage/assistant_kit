# CLI Parameter: --retry-override-delay

Tier 1 override: forces the delay (in seconds) between retry attempts for
**all** error classes in a single invocation. When set, this value beats every
class-specific `--<class>-delay` parameter. When unset (`auto`), the
class-specific delay is used; if that is also `auto`, the Tier 3 fallback
(`--retry-default-delay`) applies.

- **Type:** u32 (seconds)
- **Default:** `auto` (unset — class-specific or fallback values apply)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"retry-override-delay"`

```sh
# Force all error classes to use 60s delay
clr -p "task" --retry-override 3 --retry-override-delay 60

# Override-delay beats class-specific: Transient gets 60s, not 10s
clr -p "task" --retry-override-delay 60 --transient-delay 10

CLR_RETRY_OVERRIDE_DELAY=60 clr -p "task"            # env-var equivalent
```

**Note:** `--retry-override-delay 0` forces immediate retries for all classes
(no sleep between attempts).

**Note:** The delay is applied **between** attempts: after an error is detected
and before the next subprocess invocation. There is no delay before the first
attempt.

**Env var:** `CLR_RETRY_OVERRIDE_DELAY` — accepts a decimal integer string (u32
seconds); invalid values are silently ignored (parse failure → field stays at
auto/unset); CLI flag wins when both are present.

### 3-Tier Resolution

```
effective_delay(class) = --retry-override-delay ?? --<class>-delay ?? --retry-default-delay (30)
```

This parameter is Tier 1 — the leftmost (highest priority) in the chain.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | auto | Delay applied inside retry wrapper |
| 5 | [`ask`](../command/05_ask.md) | auto | Same behavior; pure alias for run |

### See Also

- [`--retry-override`](054_retry_override.md) — Tier 1: forces count for all classes
- [`--retry-default-delay`](057_retry_default_delay.md) — Tier 3: fallback delay when both override and class-specific are unset
- [`type/14_error_class.md`](../type/14_error_class.md) § Strategy Configuration — full 3-tier hierarchy
