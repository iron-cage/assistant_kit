# CLI Parameter: --model

Select the Claude model for this invocation.

- **Type:** [`ModelName`](../type/04_model_name.md)
- **Default:** — (Claude Code default)
- **Fallback:** when `--model` is absent and `CLR_MODEL` is unset: `model` from `.clr.toml` (project) / `~/.clr/config.toml` (user, project overrides user) if set; falls through to the Claude Code binary default when neither config-file tier sets a value.
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; `--model` at end of argv → error
- **JSON Key:** `"model"`

```sh
clr "Explain" --model sonnet
clr --model opus "Fix bug"
```

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`ModelName`](../type/04_model_name.md) | Semantic | String | non-empty string accepted by `claude --model` |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [002_print_mode_capture.md](../user_story/002_print_mode_capture.md) | Developer |
| 7 | [007_fresh_session.md](../user_story/007_fresh_session.md) | Developer |
| 17 | [017_model_selection.md](../user_story/017_model_selection.md) | Developer |
