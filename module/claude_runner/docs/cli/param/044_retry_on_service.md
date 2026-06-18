# CLI Parameter: --retry-on-service

Maximum number of automatic retries when the Claude subprocess exits with an
`ErrorKind::ApiError` classification (Service error class; output contains
`"API Error: "`). When `classify_error()` returns `ApiError`, `clr` waits
`--service-delay` seconds and re-invokes the subprocess, decrementing the retry
counter. On exhaustion, `clr` emits an exhaustion message to stderr and
propagates the subprocess exit code.

- **Type:** u8 (0–255)
- **Default:** `auto` (inherits from `--retry-default`, Tier 3 fallback)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Replaces:** `--retry-on-api-error` (renamed + renumbered from 037)

```sh
clr -p "task" --retry-on-service 3                    # retry up to 3 times on Service error
clr -p "task" --retry-on-service 1 --service-delay 0  # retry once, no delay
CLR_RETRY_ON_SERVICE=2 clr -p "task"                  # env-var equivalent
clr -p "task" --retry-on-service 0                    # disable retry for Service class
```

**Note:** Default is `auto` — inherits from `--retry-default` (Tier 3 fallback,
default 2). Set to `0` to explicitly disable retry for Service regardless of
fallback. `--retry-override` (Tier 1) beats this value when set.

**Note:** `QuotaExhausted` has higher priority than `ApiError` in `classify_error()`
priority order. A response containing both quota text and `"API Error: "` is
classified as Account (QuotaExhausted), not Service.

**Note:** The value is the number of *re-invocations*, not total attempts.
`--retry-on-service 2` means up to 3 total runs (1 initial + 2 retries).

**Note:** Applies to print-mode execution (`run_print_mode()`) only. Interactive
mode is not retried — session continuity makes retry semantics ambiguous.

**Note:** When a retry fires, `clr` emits to stderr:
`"[Service] <message> — retrying in Xs (attempt M/N)…"`.
On exhaustion: `"Error: [Service] <message> — retries exhausted (exit N)"`.

**Env var:** `CLR_RETRY_ON_SERVICE` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override ?? --retry-on-service ?? --retry-default (2)
```

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

- [`--service-delay`](045_service_delay.md) — seconds to wait between Service retry attempts
- [`--retry-override`](054_retry_override.md) — Tier 1: overrides all class-specific counts
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count for unset classes
- [`type/14_error_class.md`](../type/14_error_class.md) § Service — error class definition
