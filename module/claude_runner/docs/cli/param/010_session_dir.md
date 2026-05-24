# CLI Parameter: --session-dir

Override the session storage directory. Passed via the
`CLAUDE_SESSION_DIR` environment variable.

- **Type:** [`DirectoryPath`](../type/02_directory_path.md)
- **Default:** — (Claude Code default)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **Validation:** requires a value

```sh
clr "Fix bug" --session-dir /tmp/my-sessions
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`DirectoryPath`](../type/02_directory_path.md) | Semantic | String | valid filesystem path |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 15 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [001_interactive_repl.md](../user_story/001_interactive_repl.md) | Developer |
| 5 | [005_project_specific_execution.md](../user_story/005_project_specific_execution.md) | Developer |
