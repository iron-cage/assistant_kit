# CLI Parameter: --no-ultrathink

Disable the default ultrathink message suffix. Normally `clr` appends
`"\n\nultrathink"` after every message before sending it to the `claude`
subprocess, which triggers Claude's extended thinking mode. `--no-ultrathink`
suppresses this transformation so the message is sent verbatim.

- **Type:** bool (standalone flag)
- **Default:** false (ultrathink suffix is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

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
| 5 | [`ask`](../command/05_ask.md) | true | Default ON for ask |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
