# CLI Parameter: --retry-on-rate-limit

Maximum number of automatic retries when the Claude subprocess exits with a
transient rate-limit error (`ErrorKind::RateLimit`, exit code 2). When the
subprocess exits 2 and the output does not match a `QuotaExhausted` pattern,
`clr` waits `--retry-delay` seconds and re-invokes the subprocess, decrementing
the retry counter. On exhaustion, `clr` emits an exhaustion message to stderr
and propagates exit code 2.

- **Type:** u8 (0–255)
- **Default:** `0` (no automatic retry; current behavior preserved)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr -p "refactor module" --retry-on-rate-limit 3    # retry up to 3 times on rate-limit exit
clr -p "task" --retry-on-rate-limit 2 --retry-delay 30  # retry twice, wait 30s each
CLR_RETRY_ON_RATE_LIMIT=2 clr -p "task"             # env-var equivalent
clr -p "task" --retry-on-rate-limit 0               # default; no retry (explicit)
clr -p "task" --retry-on-rate-limit 1 --dry-run     # parsed; dry-run skips retry logic
```

**Note:** Only `ErrorKind::RateLimit` (transient HTTP 429) is retried. The following
error kinds are **never** retried regardless of the configured value:
- `QuotaExhausted` — period-boundary quota exhaustion (non-transient); detected by
  output pattern matching (e.g. `"You've hit your limit"`)
- `AuthError` — authentication failure
- `ApiError` — non-rate-limit API error
- `Signal` — subprocess killed by signal
- `Unknown` — unclassified failure

**Note:** The value is the number of *re-invocations*, not total attempts.
`--retry-on-rate-limit 2` means up to 3 total runs (1 initial + 2 retries).

**Note:** Applies to print-mode execution (`run_print_mode()`) only. Interactive
mode is not retried — session continuity makes retry semantics ambiguous.

**Note:** In `--dry-run` mode, no subprocess is spawned and no retry logic fires.
The flag is parsed and accepted; the dry-run preview is printed immediately.

**Note:** When a retry fires, `clr` emits a message to stderr (at verbosity ≥ 2,
the default): `"Info: rate-limit exit; retrying ({N} retries remaining)..."`.
On exhaustion: `"Error: rate-limit retries exhausted after {N+1} attempt(s)."`.

**Env var:** `CLR_RETRY_ON_RATE_LIMIT` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at default 0);
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

- [`--retry-delay`](035_retry_delay.md) — seconds to wait between retry attempts
