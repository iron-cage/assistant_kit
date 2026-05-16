# Parameter :: 6. `--interactive`

Opt into interactive TTY passthrough when a message is given. Without
this flag, providing a message defaults to print mode (captured output).
Use `--interactive` when you want live Claude streaming output while
also providing an initial prompt.

- **Type:** bool (standalone flag)
- **Default:** false (print mode when message given)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

```sh
clr --interactive "Fix bug"               # TTY passthrough with initial prompt
clr --interactive "Continue" --dir /proj  # interactive, specific directory
```

**Note:** No effect when no message is given — bare `clr` is always interactive.
