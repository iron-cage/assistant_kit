# CLI Parameter: --retry-on-account

Maximum number of automatic retries when the Claude subprocess exits with an
`ErrorKind::QuotaExhausted` classification (Account error class; output
contains `"You've hit your limit"`). When `classify_error()` returns
`QuotaExhausted`, `clr` waits `--account-delay` seconds and re-invokes the
subprocess, decrementing the retry counter. On exhaustion, `clr` emits an
exhaustion message to stderr and propagates the subprocess exit code.

- **Type:** u8 (0–255)
- **Default:** `0` (no retry; opt-in only — quota resets are hours-long, making short-delay retry counterproductive)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr -p "task" --retry-on-account 3                 # opt-in: retry up to 3 times on quota exhaustion
clr -p "task" --retry-on-account 0                 # explicit zero (same as default)
CLR_RETRY_ON_ACCOUNT=2 clr -p "task"               # env-var equivalent
```

**Note:** Default is `0` — Account class does not retry unless explicitly opted in.
Quota resets are measured in hours; a 30-second retry delay wastes wall-clock time
without meaningful chance of success. Use `--retry-on-account N` with a long
`--account-delay` only when the task duration may span a billing-period boundary.
`--retry-override` (Tier 1) beats this value when set.

**Note:** `QuotaExhausted` is checked first in `classify_error()` priority order.
A response that matches both `"You've hit your limit"` and `"API Error: "` is
always classified as Account, never Service.

**Note:** Retrying a quota-exhausted error is only useful for very long-running
batch workflows where the billing period may reset between retry attempts (set
a high `--account-delay` in this case). For immediate account switching, see
the deferred `--on-quota-exhausted` parameter in `type/14_error_class.md`.

**Note:** The value is the number of *re-invocations*, not total attempts.

**Note:** When a retry fires, `clr` emits to stderr:
`"[Account] <message> — retrying in Xs (attempt M/N)…"`.
On exhaustion: `"Error: [Account] <message> — retries exhausted (exit N)"`.

**Env var:** `CLR_RETRY_ON_ACCOUNT` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override ?? --retry-on-account ?? class_default(Account=0) ?? --retry-default (2)
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 0 | class default = 0; 4-tier resolution in `run_print_mode()` |
| 5 | [`ask`](../command/05_ask.md) | 0 | Same behavior; pure alias for run |

### See Also

- [`--account-delay`](041_account_delay.md) — seconds to wait between Account retry attempts
- [`--retry-override`](054_retry_override.md) — Tier 1: overrides all class-specific counts
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count for unset classes
- [`type/14_error_class.md`](../type/14_error_class.md) § Account — error class definition
