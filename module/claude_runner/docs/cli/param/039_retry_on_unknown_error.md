# CLI Parameter: --retry-on-unknown-error

Maximum number of automatic retries when the Claude subprocess exits with an
`ErrorKind::Unknown` classification (nonzero exit, no recognized error pattern,
exit code not 2, exit code <= 128). When `classify_error()` returns `Unknown`,
`clr` waits `--retry-delay` seconds and re-invokes the subprocess, decrementing
the retry counter. On exhaustion, `clr` emits an exhaustion message to stderr
and propagates the subprocess exit code.

- **Type:** u8 (0–255)
- **Default:** `0` (no automatic retry on unknown error — abort immediately)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr -p "task" --retry-on-unknown-error 1               # retry once on unknown failure
CLR_RETRY_ON_UNKNOWN_ERROR=1 clr -p "task"             # env-var equivalent
clr -p "task" --retry-on-unknown-error 0               # explicit no-retry (same as default)
clr -p "task" --retry-on-unknown-error 1 --dry-run     # parsed; dry-run skips retry logic
```

**Note:** Only `ErrorKind::Unknown` is retried by this parameter. The following
error kinds are **never** retried regardless of the configured value:
- `QuotaExhausted` — period-boundary quota exhaustion (non-transient)
- `AuthError` — authentication failure
- `ApiError` — handled separately by `--retry-on-api-error`
- `RateLimit` — handled separately by `--retry-on-rate-limit`
- `Signal` — subprocess killed by signal

**Note:** Unknown retries use `--retry-delay` for the delay between attempts
(shared with rate-limit retries). There is no dedicated delay parameter for
unknown error retries.

**Note:** The value is the number of *re-invocations*, not total attempts.
`--retry-on-unknown-error 1` means up to 2 total runs (1 initial + 1 retry).

**Note:** Applies to print-mode execution (`run_print_mode()`) only. Interactive
mode is not retried — session continuity makes retry semantics ambiguous.

**Note:** In `--dry-run` mode, no subprocess is spawned and no retry logic fires.
The flag is parsed and accepted; the dry-run preview is printed immediately.

**Note:** When a retry fires, `clr` emits a message to stderr (at verbosity >= 2,
the default): `"Info: unknown error; retrying ({N} retries remaining)..."`.
On exhaustion: `"Error: unknown error retries exhausted after {N+1} attempt(s)."`.

**Env var:** `CLR_RETRY_ON_UNKNOWN_ERROR` — accepts a decimal integer string (0–255);
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

- [`--retry-delay`](035_retry_delay.md) — seconds to wait between retry attempts (shared with rate-limit retries)
- [`--retry-on-rate-limit`](034_retry_on_rate_limit.md) — retry on transient rate-limit (separate mechanism)
- [`--retry-on-api-error`](037_retry_on_api_error.md) — retry on API errors (separate mechanism, own delay)
- [`type/14_error_class.md`](../type/14_error_class.md) § Unknown — error class for unclassified failures
