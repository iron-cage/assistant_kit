# CLI Parameter: --timeout (run/ask/topic)

Maximum seconds to wait for the Claude subprocess to complete on the `run`/`ask`
dispatch paths. When the subprocess does not exit within this limit, `clr` sends
SIGKILL, emits an error message to stderr, and exits with code 4. A value of `0`
disables the watchdog entirely (unlimited runtime).

- **Type:** u32 (seconds; 0 = unlimited)
- **Default:** `0` (unlimited) for print-mode and interactive alike — a watchdog runs only when expressed (TSK-503)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`topic`](../command/11_topic.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"timeout"`

```sh
clr -p "long task" --timeout 300          # kill after 5 minutes
clr -p "quick check" --timeout 30         # kill if not done in 30s
CLR_TIMEOUT=120 clr -p "task"             # env-var equivalent of --timeout 120
clr -p "task" --timeout 0                 # express unlimited explicitly (same watchdog behavior as the default)
clr -p "task" --timeout 60 --dry-run      # parsed; dry-run skips subprocess
```

**Note:** `--timeout 0` means **unlimited** — no watchdog thread is started and
`clr` waits indefinitely for the subprocess. Since TSK-503 this matches the default
behavior; expressing `0` remains meaningful as an explicit, self-documenting opt-out
(and, like the default, contributes no gate-wait budget — see the BUG-445 note below).

**Default behavior (TSK-503):** When `--timeout` is absent and `CLR_TIMEOUT` is unset,
no watchdog is armed on any path — `DEFAULT_PRINT_TIMEOUT_SECS = 0` (unlimited). A
session is killed by clr's watchdog only when the caller expressed a nonzero timeout.
Historical: TSK-227/BUG-305 introduced a 3600 s (1 h) print-mode default; TSK-503
retired it because it killed long agentic sessions mid-work while clr simultaneously
neutralizes claude's inner wind-down ceiling (`CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=0`)
precisely to let background work run.

**Cross-command parity:** The `--timeout` parameter on `isolated`/`refresh`
(see [`020_timeout.md`](020_timeout.md)) uses the same semantics: `0` = unlimited
(no watchdog). All five commands treat `--timeout 0` identically (`topic` via its
delegation to `run`'s handler).

**Note:** When the timeout fires, `clr` emits to stderr:
`"Error: timeout after {N}s"` and exits with code 4. Any partial stdout accumulated
before the kill is discarded (unlike isolated/refresh which preserve partial output).

**Note:** In `--dry-run` mode, no subprocess is spawned and the watchdog is never
started. The flag is parsed and accepted; dry-run output is produced immediately.

**Note:** The watchdog machinery serves both print-mode (`run_print_mode()`) and interactive
mode (`run_interactive()`); since TSK-503 both default to `0` (unlimited) when `--timeout`
is absent. When `--timeout N` is given explicitly, both paths use `N`. (Interactive's
TTY/non-TTY resolution split from BUG-425 survives so the `_CLR_DEFAULT_TIMEOUT` test hook
can arm the non-TTY branch; a genuine TTY is always exempt.)

**Env var:** `CLR_TIMEOUT` — also applies to `isolated`/`refresh` (see
[003_env_param.md](../003_env_param.md) Section 2 for the isolated/refresh mechanics).
For `run`/`ask`, `CLR_TIMEOUT` is read as a u32; `0` means unlimited. Invalid
values are silently ignored (parse failure → field stays at `None`, resolved to
`DEFAULT_PRINT_TIMEOUT_SECS = 0` — unlimited on every path). CLI flag
wins when both are present.

<!-- BUG-399 (task/claude_runner/bug/completed/399_timeout_gate_wait_undocumented.md) —
     originally: --timeout did not bound the --max-sessions gate-wait phase, by design, and
     this doc did not cross-reference that boundary. Superseded for expressed timeouts by
     BUG-445 Fix Location #2 (the note below). -->

**Note — gate-wait defaulting (BUG-445):** An *expressed* `--timeout N` (the flag, or an
applied `CLR_TIMEOUT`) also defaults the `--max-sessions` gate-wait budget to `N` seconds
when `CLR_REMAINING_TIMEOUT_SECS` is absent or unparseable — the gate clamps its attempts
to `N / poll_secs` and its deadline announcement names the source (`engaged (Ns from
--timeout ...)`). A parseable `CLR_REMAINING_TIMEOUT_SECS` always wins over the flag
(it is the per-dispatch coupling signal — see
[085_gate_remaining_timeout_secs.md](085_gate_remaining_timeout_secs.md)). An explicit
`--timeout 0` is an unlimited opt-out: no execution bound, no gate budget. When NO timeout
is expressed (flag and env both absent), no built-in default reaches the gate (moot in
production since TSK-503 zeroed the print default, but the `_CLR_DEFAULT_TIMEOUT` test hook
can still arm one and the shield guards that path) — gate-wait falls back to its own
ceiling, `CLR_GATE_POLL_SECS` x
`CLR_GATE_MAX_ATTEMPTS` (default 30s x 1000 = ~8.3h), and only then is total wall-clock
exposure gate-wait time PLUS the execution timeout. Verify:
`clr -p --max-sessions 1 --timeout 5 x` under a held slot reports
`gate-deadline  engaged (5s from --timeout ...)` on stderr. See
[025_concurrency_gate.md](../user_story/025_concurrency_gate.md) and
[033_max_sessions.md](033_max_sessions.md) for gate-wait mechanics.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | `DEFAULT_PRINT_TIMEOUT_SECS` (0 = unlimited) — print-mode and interactive alike | Watchdog spawned only when resolved timeout > 0 (i.e. expressed) |
| 5 | [`ask`](../command/05_ask.md) | Same as `run` (pure alias) | Same behavior; pure alias for run |
| 11 | [`topic`](../command/11_topic.md) | Same as `run` | Identical to `ask`; delegates to `run`'s handler |

### See Also

- [`020_timeout.md`](020_timeout.md) — `--timeout` for `isolated`/`refresh` (same semantics: 0 = unlimited)
- [`invariant/007_print_mode_timeout.md`](../../invariant/007_print_mode_timeout.md) — invariant governing the expressed-only watchdog contract (`DEFAULT_PRINT_TIMEOUT_SECS = 0`)
