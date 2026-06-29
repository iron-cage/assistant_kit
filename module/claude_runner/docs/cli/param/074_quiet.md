# CLI Parameter: --quiet

Suppress non-fatal runner diagnostics — gate-wait messages, retry warnings, and
error-exhaustion messages. Does not affect Claude Code subprocess output.
Fatal errors (spawn failures, binary-not-found) are always emitted regardless of `--quiet`.
`--dry-run` output is similarly unaffected.

- **Type:** bool
- **Default:** false
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Env:** `CLR_QUIET` (bool: `"1"` or `"true"`; applied only when `--quiet` absent from CLI)

```sh
clr --quiet "Silent run"    # suppress retry/gate/warning output
clr "Normal run"            # runner diagnostics shown (default)
CLR_QUIET=1 clr "Piped"    # env var fallback
```

**Replaces:** `--verbosity 0` from the former `--verbosity` (0–5) parameter.

**Gated output:** When `--quiet` is set, the following CLR-internal messages are suppressed:
- Gate-wait message: `"Waiting for session slot…"` (gate.rs)
- Retry progress: `"[Class] msg — retrying (attempt N/M)…"` (execution.rs)
- Retry-exhaustion: `"Error: [Class] msg — retries exhausted"` (execution.rs)
- Keep-claudecode nested-agent warning (mod.rs)

**Not gated:** Fatal errors from spawn failures, subprocess stderr output, `--dry-run` preview, `--trace` output.

### Since

Introduced to replace the deprecated `--verbosity` (0–5) parameter (TSK-337).

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 45 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | — |
| 5 | [`ask`](../command/05_ask.md) | false | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 6 | [006_verbose_debugging.md](../user_story/006_verbose_debugging.md) | Developer |
