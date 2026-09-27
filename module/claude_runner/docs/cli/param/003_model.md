# CLI Parameter: --model

Select the Claude model for this invocation.

- **Type:** [`ModelName`](../type/04_model_name.md)
- **Default:** `claude-opus-5-5` (`DEFAULT_MODEL` in `claude_runner_core`) — an explicit ID, not the `opus` alias, which the Claude Code binary resolves per release and per settings
- **Fallback:** when `--model` is absent and `CLR_MODEL` is unset: `model` from `.clr.toml` (project) / `~/.clr/config.toml` (user, project overrides user) if set; falls through to the built-in `claude-opus-5-5` when neither config-file tier sets a value. Exception: while the seat's env block pins `ANTHROPIC_MODEL` (`~/.claude/settings.json`), both the config-tier `model` and the built-in default are withheld for `run`/`ask`/`topic` — no `--model` is emitted and the seat's own binding applies — see [Provider Gate](../config_param.md#provider-gate); `isolated`'s separate lookup is unaffected.
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
| 1 | [`run`](../command/01_run.md) | `claude-opus-5-5` (via config tiers then fallback) | Withheld on a non-anthropic seat ([Provider Gate](../config_param.md#provider-gate)) |
| 5 | [`ask`](../command/05_ask.md) | `claude-opus-5-5` (via config tiers then fallback) | Same as `run` |
| 3 | [`isolated`](../command/03_isolated.md) | `claude-opus-5-5` (via config tiers then fallback) | Native flag; when absent falls back to project `.clr.toml` → user `~/.clr/config.toml` → `claude-opus-5-5`; never provider-gated; env: `CLR_MODEL` |
| 11 | [`topic`](../command/11_topic.md) | `claude-opus-5-5` (via config tiers then fallback) | Identical to `ask`; delegates to `run`'s handler |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 2 | [002_print_mode_capture.md](../user_story/002_print_mode_capture.md) | Developer |
| 7 | [007_fresh_session.md](../user_story/007_fresh_session.md) | Developer |
| 17 | [017_model_selection.md](../user_story/017_model_selection.md) | Developer |
