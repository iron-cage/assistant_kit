# CLI Parameter: --retry-on-auth

Maximum number of automatic retries when the Claude subprocess exits with an
`ErrorKind::AuthError` classification (Auth error class; output contains
`"authentication_error"` (Fix BUG-314) or `"Your organization does not have access to Claude"`).
When `classify_error()` returns `AuthError`, `clr` waits `--auth-delay` seconds and
re-invokes the subprocess up to `--retry-on-auth` times, using the same 3-tier retry
resolution as all other error classes (Fix BUG-325). On exhaustion, `clr` emits an
exhaustion message to stderr and propagates the subprocess exit code.

- **Type:** u8 (0–255)
- **Default:** `auto` (inherits from `--retry-default`, Tier 3 fallback)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"retry-on-auth"`

```sh
clr -p "task" --retry-on-auth 1                 # retry once on auth failure
clr -p "task" --retry-on-auth 0                 # disable retry for Auth class
CLR_RETRY_ON_AUTH=1 clr -p "task"               # env-var equivalent
```

**Note:** Default is `auto` — inherits from `--retry-default` (Tier 3 fallback,
default 2). Set to `0` to explicitly disable retry for Auth regardless of
fallback. `--retry-override` (Tier 1) beats this value when set.

**Note:** Auth errors are typically persistent (wrong credentials, revoked
access). Retry is rarely useful unless the error is caused by a transient
provisioning delay. For immediate account switching, see the deferred
`--on-auth-error` parameter in `type/14_error_class.md`.

**Note:** The value is the number of *re-invocations*, not total attempts.

**Note:** When a retry fires, `clr` emits to stderr:
`"[Auth] <message> — retrying in Xs (attempt M/N)…"`.
On exhaustion: `"Error: [Auth] <message> — retries exhausted (exit N)"`.

**Env var:** `CLR_RETRY_ON_AUTH` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override ?? --retry-on-auth ?? --retry-default (2)
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

- [`--auth-delay`](043_auth_delay.md) — seconds to wait between Auth retry attempts
- [`--retry-override`](054_retry_override.md) — Tier 1: overrides all class-specific counts
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count for unset classes
- [`type/14_error_class.md`](../type/14_error_class.md) § Auth — error class definition
