# CLI Parameter: --new-session

Disable the default session continuation. Normally `clr` passes
`-c` to claude on every invocation, resuming the most recent conversation.
`--new-session` suppresses that flag, starting a genuinely fresh session
with no prior context.

- **Type:** bool (standalone flag)
- **Default:** false (continuation is automatic)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr --new-session "Analyse this new codebase from scratch"
clr --new-session "Review this PR fresh" --model opus
```

**Note:** Use when switching to a genuinely unrelated task where prior
conversation context would be misleading or harmful.

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
| 1 | [001_interactive_repl.md](../user_story/001_interactive_repl.md) | Developer |
| 3 | [003_interactive_with_message.md](../user_story/003_interactive_with_message.md) | Developer |
| 5 | [005_project_specific_execution.md](../user_story/005_project_specific_execution.md) | Developer |
| 7 | [007_fresh_session.md](../user_story/007_fresh_session.md) | Developer |
| 15 | [015_ask_mode.md](../user_story/015_ask_mode.md) | Developer |
