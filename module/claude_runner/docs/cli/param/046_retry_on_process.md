# CLI Parameter: --retry-on-process

Maximum number of automatic retries when the Claude subprocess is killed by an
external signal (`ErrorKind::Signal`, Process error class; exit code > 128).
When `classify_error()` returns `Signal`, `clr` waits `--process-delay` seconds
and re-invokes the subprocess, decrementing the retry counter. On exhaustion,
`clr` emits an exhaustion message to stderr and propagates the subprocess exit
code.

- **Type:** u8 (0–255)
- **Default:** `auto` (inherits from `--retry-default`, Tier 3 fallback)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"retry-on-process"`

```sh
clr -p "task" --retry-on-process 2                # retry twice on signal kill
clr -p "task" --retry-on-process 0                # disable retry for Process class
CLR_RETRY_ON_PROCESS=2 clr -p "task"              # env-var equivalent
```

**Note:** Default is `auto` — inherits from `--retry-default` (Tier 3 fallback,
default 2). Set to `0` to explicitly disable retry for Process regardless of
fallback. `--retry-override` (Tier 1) beats this value when set.

**Note:** Signal kills (SIGTERM, SIGKILL, SIGSEGV, etc.) are reported as exit
code 128 + signal_number per POSIX convention. Common causes: OOM killer
(SIGKILL/137), external termination, infrastructure restarts.

**Note:** The `--timeout` watchdog also produces a signal kill (SIGTERM then
SIGKILL), but the Timeout CLR-layer condition (exit 4) is checked BEFORE
`classify_error()` runs — so timeout never reaches the Process retry loop.

**Note:** The value is the number of *re-invocations*, not total attempts.

**Note:** When a retry fires, `clr` emits to stderr:
`"[Process] signal N — retrying in Xs (attempt M/N)…"`.
On exhaustion: `"Error: [Process] signal N — retries exhausted (exit 128+N)"`.

**Env var:** `CLR_RETRY_ON_PROCESS` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override ?? --retry-on-process ?? --retry-default (2)
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

- [`--process-delay`](047_process_delay.md) — seconds to wait between Process retry attempts
- [`--timeout`](036_timeout.md) — watchdog timeout (exit 4, not retried via Process class)
- [`--retry-override`](054_retry_override.md) — Tier 1: overrides all class-specific counts
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count for unset classes
- [`type/14_error_class.md`](../type/14_error_class.md) § Process — error class definition
