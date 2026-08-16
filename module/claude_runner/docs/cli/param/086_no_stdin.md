# CLI Parameter: --no-stdin

Never read piped stdin. Disables both stdin JSON config auto-detection and
stdin content forwarding for `run`/`ask`, so a held-open pipe (`tail -f |`, a
FIFO with a live writer, a supervisor-inherited fd) can never block clr.

- **Type:** bool (standalone flag)
- **Default:** false (non-TTY stdin is read and auto-detected by default)
- **Env var:** `CLR_NO_STDIN`
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** *not supported* — deliberate: stdin JSON config is itself delivered via stdin, so by the time any config could be parsed the blocking read would already have happened. The opt-out must arrive on the command line or in the environment.

```sh
tail -f app.log | clr --no-stdin "Summarize"      # held-open pipe ignored; no hang
CLR_NO_STDIN=1 clr "task"                         # env form, same effect
printf '{"model":"x"}' | clr --no-stdin "hi"      # piped JSON config declined too
```

**Note:** The check runs pre-parse as a raw token/env scan (Gate 0 in
`detect_stdin_json()`) — before any stdin read, because argument parsing itself
receives stdin content as input. Without the opt-out, a non-TTY pipe that never
closes would block clr forever, before it could parse or reject anything
(BUG-492).

**Note:** `--file` reads a named file, not stdin, and is unaffected. `isolated`
and `refresh` have their own stdin handling, out of this flag's scope.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 52 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | — |
| 5 | [`ask`](../command/05_ask.md) | false | — |

### Referenced User Stories

*None — no user story directly exercises `--no-stdin`.*
