# CLI Parameter: --retry-on-transient

Maximum number of automatic retries when the Claude subprocess exits with a
transient rate-limit error (`ErrorKind::RateLimit`, Transient error class, exit
code 2). When the subprocess exits 2 and the output does not match a
`QuotaExhausted` pattern, `clr` waits `--transient-delay` seconds and
re-invokes the subprocess, decrementing the retry counter. On exhaustion, `clr`
emits an exhaustion message to stderr and propagates exit code 2.

- **Type:** u8 (0–255)
- **Default:** `auto` (inherits from `--retry-default`, Tier 3 fallback)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Replaces:** `--retry-on-rate-limit` (renamed)
- **JSON Key:** `"retry-on-transient"`

```sh
clr -p "refactor module" --retry-on-transient 3     # retry up to 3 times on rate-limit exit
clr -p "task" --retry-on-transient 2 --transient-delay 60  # retry twice, wait 60s each
CLR_RETRY_ON_TRANSIENT=2 clr -p "task"              # env-var equivalent
clr -p "task" --retry-on-transient 0                 # disable retry for Transient class
```

**Note:** Default is `auto` — inherits from `--retry-default` (Tier 3 fallback,
default 2). Set to `0` to explicitly disable retry for Transient regardless of
fallback. `--retry-override` (Tier 1) beats this value when set.

**Note:** The value is the number of *re-invocations*, not total attempts.
`--retry-on-transient 2` means up to 3 total runs (1 initial + 2 retries).

**Note:** Applies to print-mode execution (`run_print_mode()`) only. Interactive
mode is not retried — session continuity makes retry semantics ambiguous.

**Note:** In `--dry-run` mode, no subprocess is spawned and no retry logic fires.
The flag is parsed and accepted; the dry-run preview is printed immediately.

**Note:** When a retry fires, `clr` emits to stderr:
`"[Transient] <message> — retrying in Xs (attempt M/N)…"`.
On exhaustion: `"Error: [Transient] <message> — retries exhausted (exit 2)"`.

**Env var:** `CLR_RETRY_ON_TRANSIENT` — accepts a decimal integer string (0–255);
invalid values are silently ignored (parse failure → field stays at auto/unset);
CLI flag wins when both are present.

### 3-Tier Resolution

```
effective = --retry-override ?? --retry-on-transient ?? --retry-default (2)
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

- [`--transient-delay`](035_transient_delay.md) — seconds to wait between Transient retry attempts
- [`--retry-override`](054_retry_override.md) — Tier 1: overrides all class-specific counts
- [`--retry-default`](056_retry_default.md) — Tier 3: fallback count for unset classes
- [`type/14_error_class.md`](../type/14_error_class.md) § Transient — error class definition
