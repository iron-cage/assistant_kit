# CLI Parameter: --interactive

Opt into interactive TTY passthrough when a message is given. Without
this flag, providing a message — or invoking `clr` with stdin that is not a terminal, or
with `--file`/piped stdin content and no message — defaults to print mode (captured
output). Use `--interactive` when you want live Claude streaming output while
also providing an initial prompt.

- **Type:** bool (standalone flag)
- **Default:** false (print mode when message given, stdin is not a terminal, or `--file`/piped stdin content is present)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"interactive"`

```sh
clr --interactive "Fix bug"               # TTY passthrough with initial prompt
clr --interactive "Continue" --dir /proj  # interactive, specific directory
```

**Note:** Still has an effect when no message is given: without `--interactive`, a bare
`clr` invoked from a genuine terminal opens the interactive REPL, but the same bare
invocation under non-TTY stdin (piped, redirected, non-interactive shell) routes to print
mode instead (Fix(BUG-425)). `--interactive` forces the interactive/REPL route regardless
of TTY state — it is the escape hatch for cases like resuming a prior session with no new
message under non-TTY stdin, where print-mode routing would otherwise apply.

This override reaches only the three *inferred* print-mode triggers (message presence,
non-TTY stdin, `--file`/stdin content) — it does not reach an explicitly requested print
mode. `-p`/`--print`, `CLR_PRINT`, or JSON config `"print"` each settle the mode-selection
question outright, so combining `--interactive` with any of them still routes to print
mode (see [006_cli_design.md](../../feature/006_cli_design.md) § Design : Mode selection).

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | false | — |
| 5 | [`ask`](../command/05_ask.md) | false | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 3 | [003_interactive_with_message.md](../user_story/003_interactive_with_message.md) | Developer |
| 5 | [005_project_specific_execution.md](../user_story/005_project_specific_execution.md) | Developer |
