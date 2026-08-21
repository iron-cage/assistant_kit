# CLI Parameter: --fallback-model

Specify a fallback model to use when the primary model is unavailable.

- **Type:** string
- **Default:** — (Claude Code default; no fallback when not specified)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; `--fallback-model` at end of argv → error
- **JSON Key:** `"fallback-model"`
- **Provider Gate:** the config-tier `fallback_model` key is ignored while the seat's env block pins `ANTHROPIC_MODEL` (`~/.claude/settings.json`) — see [Provider Gate](../config_param.md#provider-gate); the CLI flag and `CLR_FALLBACK_MODEL` are unaffected.

```sh
clr "Fix bug" --model opus --fallback-model sonnet
clr --fallback-model haiku "Quick task"
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--output-format`, `--input-format`, `--max-turns`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--add-dir` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |
| 11 | [`topic`](../command/11_topic.md) | — | Identical to `ask`; delegates to `run`'s handler |
