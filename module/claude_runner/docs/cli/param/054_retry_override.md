# CLI Parameter: --retry-override

Tier 1 override: forces the retry count for **all** error classes in a single
invocation. When set, this value beats every class-specific `--retry-on-<class>`
parameter. When unset (`auto`), the class-specific value is used; if that is
also `auto`, the Tier 3 fallback (`--retry-default`) applies.

- **Type:** u8 (0–255)
- **Default:** `auto` (unset — class-specific or fallback values apply)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
# Force all error classes to retry 5 times, 60s delay
clr -p "task" --retry-override 5 --retry-override-delay 60

# Disable all retries in one flag
clr -p "task" --retry-override 0

# Override beats class-specific: Transient gets 1 retry, not 5
clr -p "task" --retry-override 1 --retry-on-transient 5

CLR_RETRY_OVERRIDE=3 clr -p "task"                  # env-var equivalent
```

**Note:** This is a convenience parameter for uniform per-invocation control.
Without it, applying the same retry count to all 8 error classes requires 8
separate `--retry-on-<class>` flags.

**Note:** `--retry-override 0` disables all retries regardless of class-specific
and fallback settings — useful for CI jobs that must fail fast.

**Note:** The value is the number of *re-invocations*, not total attempts.
`--retry-override 2` means up to 3 total runs per class.

**Note:** Applies to print-mode execution (`run_print_mode()`) only.

**Env var:** `CLR_RETRY_OVERRIDE` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective_count(class) = --retry-override ?? --retry-on-<class> ?? --retry-default (2)
```

This parameter is Tier 1 — the leftmost (highest priority) in the chain.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | auto | 3-tier resolution in `run_print_mode()` |
| 5 | [`ask`](../command/05_ask.md) | auto | Same behavior; pure alias for run |

### See Also

- [`--retry-override-delay`](055_retry_override_delay.md) — Tier 1: forces delay for all classes
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count when both override and class-specific are unset
- [`type/14_error_class.md`](../type/14_error_class.md) § Strategy Configuration — full 3-tier hierarchy
