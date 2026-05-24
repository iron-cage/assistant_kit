# CLI Parameter: --no-chrome

Suppress the automatic `--chrome` injection. When set, no `--chrome` flag
is forwarded to the `claude` subprocess.

- **Type:** bool (standalone flag)
- **Default:** false (chrome injection is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr "Fix bug"              # sends: --chrome (default)
clr --no-chrome "Fix bug"  # sends: no --chrome flag
```

**Note:** Use `--no-chrome` when running in headless or CI environments
where no Chrome instance is available, or when you want to prevent the
Claude-in-Chrome browser integration from activating.

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

*None — no user story directly exercises `--no-chrome`.*
