# Parameter :: 14. `--no-ultrathink`

Disable the default ultrathink message suffix. Normally `clr` appends
`"\n\nultrathink"` after every message before sending it to the `claude`
subprocess, which triggers Claude's extended thinking mode. `--no-ultrathink`
suppresses this transformation so the message is sent verbatim.

- **Type:** bool (standalone flag)
- **Default:** false (ultrathink suffix is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

```sh
clr "Fix the auth bug"                # sends: "Fix the auth bug\n\nultrathink"
clr --no-ultrathink "Fix the auth bug" # sends: "Fix the auth bug" (verbatim)
```

**Note:** The ultrathink suffix is not added when the message already ends
with `"ultrathink"` (idempotent guard — prevents double append when the message
already carries `"\n\nultrathink"` from a prior injection or the user
explicitly writes it themselves).

**Note:** Applies only to message-bearing invocations. Bare `clr` (no message,
interactive REPL) is not affected.
