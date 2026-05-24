# CLI Parameter: --interactive

Opt into interactive TTY passthrough when a message is given. Without
this flag, providing a message defaults to print mode (captured output).
Use `--interactive` when you want live Claude streaming output while
also providing an initial prompt.

- **Type:** bool (standalone flag)
- **Default:** false (print mode when message given)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr --interactive "Fix bug"               # TTY passthrough with initial prompt
clr --interactive "Continue" --dir /proj  # interactive, specific directory
```

**Note:** No effect when no message is given — bare `clr` is always interactive.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| bool | Primitive | bool | present/absent |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 15 other params |

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
