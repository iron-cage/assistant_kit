# CLI Parameter: --no-skip-permissions

Disable the automatic `--dangerously-skip-permissions` flag that `clr` injects into every
invocation by default. Without this flag, every `clr` call silently passes
`--dangerously-skip-permissions` to the `claude` subprocess, bypassing all tool permission
prompts.

- **Type:** bool (standalone flag)
- **Default:** false (bypass is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"no-skip-permissions"`

```sh
clr --no-skip-permissions "Fix bug"   # bypass disabled — claude will prompt for tool approvals
```

**Note:** `--dangerously-skip-permissions` is no longer a user-facing flag. It is injected
automatically unless `--no-skip-permissions` is given. See the
[Default Flags Invariant](../../invariant/001_default_flags.md#invariant-statement) in the invariant.

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

| # | User Story | Persona |
|---|------------|---------|
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
