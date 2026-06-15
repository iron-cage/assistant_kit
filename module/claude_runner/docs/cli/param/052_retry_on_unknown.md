# CLI Parameter: --retry-on-unknown

Maximum number of automatic retries when the Claude subprocess exits with an
`ErrorKind::Unknown` classification (Unknown error class; nonzero exit, no
recognized error pattern, exit code not 2, exit code <= 128). When
`classify_error()` returns `Unknown`, `clr` waits `--unknown-delay` seconds
and re-invokes the subprocess, decrementing the retry counter. On exhaustion,
`clr` emits an exhaustion message to stderr and propagates the subprocess
exit code.

- **Type:** u8 (0–255)
- **Default:** `auto` (inherits from `--retry-default`, Tier 3 fallback; effective default = 2)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Replaces:** `--retry-on-unknown-error` (renamed + renumbered from 039)

```sh
clr -p "task" --retry-on-unknown 1                # retry once on unknown failure
clr -p "task" --retry-on-unknown 0                # disable retry for Unknown class
CLR_RETRY_ON_UNKNOWN=1 clr -p "task"              # env-var equivalent
```

**Note:** Default is `auto` — inherits from `--retry-default` (Tier 3 fallback,
default 2). Set to `0` to explicitly disable retry for Unknown regardless of
fallback. `--retry-override` (Tier 1) beats this value when set.

**Note:** The value is the number of *re-invocations*, not total attempts.
`--retry-on-unknown 1` means up to 2 total runs (1 initial + 1 retry).

**Note:** Applies to print-mode execution (`run_print_mode()`) only. Interactive
mode is not retried — session continuity makes retry semantics ambiguous.

**Note:** When a retry fires, `clr` emits to stderr:
`"[Unknown] <message> — retrying in Xs (attempt M/N)…"`.
On exhaustion: `"Error: [Unknown] <message> — retries exhausted (exit N)"`.

**Env var:** `CLR_RETRY_ON_UNKNOWN` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override ?? --retry-on-unknown ?? --retry-default (2)
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

- [`--unknown-delay`](053_unknown_delay.md) — seconds to wait between Unknown retry attempts
- [`--retry-override`](054_retry_override.md) — Tier 1: overrides all class-specific counts
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count for unset classes
- [`type/14_error_class.md`](../type/14_error_class.md) § Unknown — error class definition
