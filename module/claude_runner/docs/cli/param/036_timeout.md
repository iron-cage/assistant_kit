# CLI Parameter: --timeout (run/ask)

Maximum seconds to wait for the Claude subprocess to complete on the `run`/`ask`
dispatch paths. When the subprocess does not exit within this limit, `clr` sends
SIGKILL, emits an error message to stderr, and exits with code 2. A value of `0`
disables the watchdog entirely (unlimited runtime; current default behavior).

- **Type:** u32 (seconds; 0 = unlimited)
- **Default:** `0` (no timeout; unlimited runtime)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr -p "long task" --timeout 300          # kill after 5 minutes
clr -p "quick check" --timeout 30         # kill if not done in 30s
CLR_TIMEOUT=120 clr -p "task"             # env-var equivalent of --timeout 120
clr -p "task" --timeout 0                 # default: no watchdog (unlimited)
clr -p "task" --timeout 60 --dry-run      # parsed; dry-run skips subprocess
```

**Note:** `--timeout 0` means **unlimited** — no watchdog thread is started and
`clr` waits indefinitely for the subprocess. This is the default behavior and
preserves backward compatibility with all existing `clr run`/`clr ask` invocations.

**IMPORTANT — semantic difference from isolated/refresh:** The `--timeout` parameter
on `isolated`/`refresh` (see [`020_timeout.md`](020_timeout.md)) uses `0` to mean
*immediate expiry* (the subprocess deadline is set to `now`). On `run`/`ask`, `0`
means *unlimited* (watchdog not started). These are opposite semantics for the same
flag value on different commands.

**Note:** When the timeout fires, `clr` emits to stderr:
`"Error: timeout after {N}s"` and exits with code 2. Any partial stdout accumulated
before the kill is discarded (unlike isolated/refresh which preserve partial output).

**Note:** In `--dry-run` mode, no subprocess is spawned and the watchdog is never
started. The flag is parsed and accepted; dry-run output is produced immediately.

**Note:** Applies to both print-mode (`run_print_mode()`) and interactive mode
(`run_interactive()`). Both execution paths receive the same watchdog treatment.

**Env var:** `CLR_TIMEOUT` — also applies to `isolated`/`refresh` (see
[env_param.md](../env_param.md) Section 2 for the isolated/refresh mechanics).
For `run`/`ask`, `CLR_TIMEOUT` is read as a u32; `0` means unlimited. Invalid
values are silently ignored (parse failure → field stays at default 0). CLI flag
wins when both are present.

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | `--dry-run`, `--verbosity`, `--trace`, ... |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | 0 (unlimited) | Watchdog spawned when timeout > 0 |
| 5 | [`ask`](../command/05_ask.md) | 0 (unlimited) | Same behavior; pure alias for run |

### See Also

- [`020_timeout.md`](020_timeout.md) — `--timeout` for `isolated`/`refresh` (different semantics: 0 = immediate expiry)
