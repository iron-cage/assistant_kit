# CLI Parameter: --dry-run

Print the assembled command that would be executed without actually
invoking the Claude Code subprocess. Useful for debugging flag
combinations.

- **Type:** bool (standalone flag)
- **Default:** false
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"dry-run"`

```sh
clr --dry-run "test" --model sonnet --max-tokens 50000
# Output includes: claude --dangerously-skip-permissions -c --print --model sonnet "test\n\nultrathink"
# Note: --chrome absent in print mode (BUG-304 suppression); present only in interactive mode
```

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
| 2 | [`isolated`](../command/02_isolated.md) | false | Prints temp-HOME env+command to stderr (same `emit_credential_trace` path as `--trace`); no temp HOME created, no subprocess spawned |
| 3 | [`refresh`](../command/03_refresh.md) | false | Same as isolated: stderr env+command output via `emit_credential_trace`; no subprocess spawned |
| 5 | [`ask`](../command/05_ask.md) | false | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 4 | [004_dry_run_preview.md](../user_story/004_dry_run_preview.md) | Developer |
| 6 | [006_verbose_debugging.md](../user_story/006_verbose_debugging.md) | Developer |
| 8 | [008_trace_execution.md](../user_story/008_trace_execution.md) | Developer |
| 18 | [018_env_var_configuration.md](../user_story/018_env_var_configuration.md) | Developer |
