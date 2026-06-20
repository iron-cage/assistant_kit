# CLI Parameter: --max-budget-usd

Set a maximum dollar budget for the Claude Code session.

- **Type:** f64
- **Default:** — (Claude Code default; no budget limit when not specified)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; `--max-budget-usd` at end of argv → error

```sh
clr "Fix all bugs" --max-budget-usd 5.00
clr --max-budget-usd 0.50 "Quick question"
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--output-format`, `--max-turns`, `--allowed-tools`, `--disallowed-tools`, `--add-dir`, `--fallback-model` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |
