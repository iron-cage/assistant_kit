# CLI Parameter: --retry-on-api-error

Maximum number of automatic retries when the Claude subprocess exits with an
`ErrorKind::ApiError` classification (output contains `"API Error: "`). When
`classify_error()` returns `ApiError`, `clr` waits `--api-error-delay` seconds
and re-invokes the subprocess, decrementing the retry counter. On exhaustion,
`clr` emits an exhaustion message to stderr and propagates the subprocess exit code.

- **Type:** u8 (0–255)
- **Default:** `0` (no automatic retry on API error — abort immediately)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr -p "task" --retry-on-api-error 2                   # retry up to 2 times on API error
clr -p "task" --retry-on-api-error 1 --api-error-delay 0  # retry once, no delay
CLR_RETRY_ON_API_ERROR=2 clr -p "task"                 # env-var equivalent
clr -p "task" --retry-on-api-error 0                   # explicit no-retry (same as default)
clr -p "task" --retry-on-api-error 1 --dry-run         # parsed; dry-run skips retry logic
```

**Note:** Only `ErrorKind::ApiError` is retried by this parameter. The following
error kinds are **never** retried regardless of the configured value:
- `QuotaExhausted` — period-boundary quota exhaustion (non-transient); higher
  priority than `ApiError` in `classify_error()` priority order
- `AuthError` — authentication failure; higher priority than `ApiError`
- `RateLimit` — handled separately by `--retry-on-rate-limit`
- `Signal` — subprocess killed by signal
- `Unknown` — handled separately by `--retry-on-unknown-error`

**Note:** The value is the number of *re-invocations*, not total attempts.
`--retry-on-api-error 2` means up to 3 total runs (1 initial + 2 retries).

**Note:** Applies to print-mode execution (`run_print_mode()`) only. Interactive
mode is not retried — session continuity makes retry semantics ambiguous.

**Note:** In `--dry-run` mode, no subprocess is spawned and no retry logic fires.
The flag is parsed and accepted; the dry-run preview is printed immediately.

**Note:** When a retry fires, `clr` emits a message to stderr (at verbosity >= 2,
the default): `"Info: API error; retrying ({N} retries remaining)..."`.
On exhaustion: `"Error: API error retries exhausted after {N+1} attempt(s)."`.

**Env var:** `CLR_RETRY_ON_API_ERROR` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure -> field stays at default 0);
CLI flag wins when both are present.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 0 | Retry logic wraps `run_print_mode()` call |
| 5 | [`ask`](../command/05_ask.md) | 0 | Same behavior; pure alias for run |

### See Also

- [`--api-error-delay`](038_api_error_delay.md) — seconds to wait between API error retry attempts
- [`--retry-on-rate-limit`](034_retry_on_rate_limit.md) — retry on transient rate-limit (separate mechanism)
- [`--retry-on-unknown-error`](039_retry_on_unknown_error.md) — retry on unclassified failures
- [`type/14_error_class.md`](../type/14_error_class.md) § Service — error class for API errors
