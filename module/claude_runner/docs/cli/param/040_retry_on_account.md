# CLI Parameter: --retry-on-account

Maximum number of automatic retries when the Claude subprocess exits with an
`ErrorKind::QuotaExhausted` classification (Account error class; output
contains `"You've hit your limit"`). When `classify_error()` returns
`QuotaExhausted`, `clr` waits `--account-delay` seconds and re-invokes the
subprocess, decrementing the retry counter. On exhaustion, `clr` emits an
exhaustion message to stderr and propagates the subprocess exit code.

- **Type:** u8 (0–255)
- **Default:** `auto` (inherits from `--retry-default`, Tier 3 fallback; effective default = 2)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"retry-on-account"`

```sh
clr -p "task" --retry-on-account 3                 # retry up to 3 times on quota exhaustion
clr -p "task" --retry-on-account 0                 # explicit zero: disable Account retry
CLR_RETRY_ON_ACCOUNT=2 clr -p "task"               # env-var equivalent
```

**Note:** `--retry-override` (Tier 1) beats this value when set.

**Note:** `QuotaExhausted` is checked first in `classify_error()` priority order.
A response that matches both `"You've hit your limit"` and `"API Error: "` is
always classified as Account, never Service.

**Note:** For immediate account switching instead of retrying, see the deferred
`--on-quota-exhausted` parameter in `type/14_error_class.md`.

**Note:** The value is the number of *re-invocations*, not total attempts.

**Note:** When a retry fires, `clr` emits to stderr:
`"[Account] <message> — retrying in Xs (attempt M/N)…"`.
On exhaustion: `"Error: [Account] <message> — retries exhausted (exit N)"`, followed by
the captured stdout rendered through `render_summary()` (summary mode) or raw (raw mode).
In summary mode, `<message>` is the `"result"` field extracted from the JSON envelope
(e.g., `"You've hit your limit · resets 2:40pm (Europe/Kiev)"`), not the full JSON blob.

**Env var:** `CLR_RETRY_ON_ACCOUNT` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override ?? --retry-on-account ?? --retry-default (2)
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | auto | 3-tier resolution in `run_print_mode()` |
| 5 | [`ask`](../command/05_ask.md) | auto | Same behavior; pure alias for run |

### See Also

- [`--account-delay`](041_account_delay.md) — seconds to wait between Account retry attempts
- [`--retry-override`](054_retry_override.md) — Tier 1: overrides all class-specific counts
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count for unset classes
- [`type/14_error_class.md`](../type/14_error_class.md) § Account — error class definition
