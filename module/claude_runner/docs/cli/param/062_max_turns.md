# CLI Parameter: --max-turns

Limit the number of agentic turns Claude Code may take before stopping.

- **Type:** u32
- **Default:** — (Claude Code default; unlimited when not specified)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; `--max-turns` at end of argv → error
- **JSON Key:** `"max-turns"`

```sh
clr "Fix all tests" --max-turns 5
clr --max-turns 10 "Refactor module"
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--output-format`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--add-dir`, `--fallback-model` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |
