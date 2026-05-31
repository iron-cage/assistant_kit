# CLI Parameter: --no-persist

Disable session persistence. Forwards `--no-session-persistence` to the
`claude` subprocess, preventing the session from being saved to disk.

- **Type:** bool (standalone flag)
- **Default:** false (session persistence is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr "Fix bug"              # session saved to disk (default)
clr --no-persist "Fix bug" # session not saved; cannot be resumed
```

**Note:** Use `--no-persist` for ephemeral, stateless queries that must not
pollute session history — disposable scripted invocations, one-shot queries,
or test runs where resumability is undesired.

**Note:** Unlike `--new-session` (which starts fresh but still saves the new
session), `--no-persist` creates an entirely unsaved session.

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
| 5 | [`ask`](../command/05_ask.md) | true | Default ON for ask |

### Referenced User Stories

*None — no user story directly exercises `--no-persist`.*
