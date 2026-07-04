# CLI Parameter: --timeout (run/ask)

Maximum seconds to wait for the Claude subprocess to complete on the `run`/`ask`
dispatch paths. When the subprocess does not exit within this limit, `clr` sends
SIGKILL, emits an error message to stderr, and exits with code 4. A value of `0`
disables the watchdog entirely (unlimited runtime).

- **Type:** u32 (seconds; 0 = unlimited)
- **Default:** `3600` for print-mode (`run`/`ask`); `0` (unlimited) for interactive mode
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"timeout"`

```sh
clr -p "long task" --timeout 300          # kill after 5 minutes
clr -p "quick check" --timeout 30         # kill if not done in 30s
CLR_TIMEOUT=120 clr -p "task"             # env-var equivalent of --timeout 120
clr -p "task" --timeout 0                 # opt out: no watchdog (unlimited; overrides 3600s default)
clr -p "task" --timeout 60 --dry-run      # parsed; dry-run skips subprocess
```

**Note:** `--timeout 0` means **unlimited** — no watchdog thread is started and
`clr` waits indefinitely for the subprocess. This is an explicit opt-out of the
default 3600-second watchdog for print-mode (`run`/`ask`).

**Default behavior (TSK-227 / BUG-305):** When `--timeout` is absent and `CLR_TIMEOUT` is
unset, print-mode (`run_print_mode()`) uses `DEFAULT_PRINT_TIMEOUT_SECS = 3600` (1 hour)
as the watchdog. This prevents unattended sessions from running indefinitely. Interactive
mode (`run_interactive()`) retains an unlimited default — interactive sessions are
user-attended and must not be killed by an arbitrary deadline.

**Cross-command parity:** The `--timeout` parameter on `isolated`/`refresh`
(see [`020_timeout.md`](020_timeout.md)) uses the same semantics: `0` = unlimited
(no watchdog). All four commands treat `--timeout 0` identically.

**Note:** When the timeout fires, `clr` emits to stderr:
`"Error: timeout after {N}s"` and exits with code 4. Any partial stdout accumulated
before the kill is discarded (unlike isolated/refresh which preserve partial output).

**Note:** In `--dry-run` mode, no subprocess is spawned and the watchdog is never
started. The flag is parsed and accepted; dry-run output is produced immediately.

**Note:** The watchdog applies to both print-mode (`run_print_mode()`) and interactive mode
(`run_interactive()`), but their defaults differ. Print-mode: `DEFAULT_PRINT_TIMEOUT_SECS`
(3600 s) when `--timeout` absent. Interactive: `0` (unlimited) when `--timeout` absent.
When `--timeout N` is given explicitly, both paths use `N`.

**Env var:** `CLR_TIMEOUT` — also applies to `isolated`/`refresh` (see
[003_env_param.md](../003_env_param.md) Section 2 for the isolated/refresh mechanics).
For `run`/`ask`, `CLR_TIMEOUT` is read as a u32; `0` means unlimited. Invalid
values are silently ignored (parse failure → field stays at `None`, resolved to
`DEFAULT_PRINT_TIMEOUT_SECS` for print-mode or `0` for interactive). CLI flag
wins when both are present.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--quiet`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | `DEFAULT_PRINT_TIMEOUT_SECS` (3600) for print-mode; `0` (unlimited) for interactive | Watchdog spawned when resolved timeout > 0 |
| 5 | [`ask`](../command/05_ask.md) | Same as `run` (pure alias) | Same behavior; pure alias for run |

### See Also

- [`020_timeout.md`](020_timeout.md) — `--timeout` for `isolated`/`refresh` (same semantics: 0 = unlimited)
- [`invariant/007_print_mode_timeout.md`](../invariant/007_print_mode_timeout.md) — invariant governing `DEFAULT_PRINT_TIMEOUT_SECS` and the print-mode vs interactive asymmetry
