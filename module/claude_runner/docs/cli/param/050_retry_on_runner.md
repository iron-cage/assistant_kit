# CLI Parameter: --retry-on-runner

Maximum number of automatic retries when a Runner error class condition occurs
(CLR-layer failures: binary not found, spawn failed, gate timeout, output file
write error; exit code 1). These conditions are detected by the CLR runner
layer before or after subprocess execution — they do not go through
`classify_error()` and have no `ErrorKind` variant.

- **Type:** u8 (0–255)
- **Default:** `auto` (inherits from `--retry-default`, Tier 3 fallback)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"retry-on-runner"`

```sh
clr -p "task" --retry-on-runner 1                 # retry once on runner failure
clr -p "task" --retry-on-runner 0                 # disable retry for Runner class
CLR_RETRY_ON_RUNNER=1 clr -p "task"               # env-var equivalent
```

**Note:** Default is `auto` — inherits from `--retry-default` (Tier 3 fallback,
default 2). Set to `0` to explicitly disable retry for Runner regardless of
fallback. `--retry-override` (Tier 1) beats this value when set.

**Note:** Runner errors are usually persistent (missing binary, permission denied).
Retry is useful primarily for gate timeout (concurrent session slots may free up)
and transient spawn failures.

**Note:** The value is the number of *re-invocations*, not total attempts.

**Note:** When a retry fires, `clr` emits to stderr:
`"[Runner] <message> — retrying in Xs (attempt M/N)…"`.
On exhaustion: `"Error: [Runner] <message> — retries exhausted (exit 1)"`.

**Env var:** `CLR_RETRY_ON_RUNNER` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override ?? --retry-on-runner ?? --retry-default (2)
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

- [`--runner-delay`](051_runner_delay.md) — seconds to wait between Runner retry attempts
- [`--retry-override`](054_retry_override.md) — Tier 1: overrides all class-specific counts
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count for unset classes
- [`type/14_error_class.md`](../type/14_error_class.md) § Runner — error class definition
