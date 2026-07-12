# CLI Parameter: --allowed-tools

Restrict Claude Code to only use the specified tools.

- **Type:** string
- **Default:** — (Claude Code default; all tools allowed when not specified)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; `--allowed-tools` at end of argv → error
- **JSON Key:** `"allowed-tools"`

```sh
clr "Fix bug" --allowed-tools "Read,Grep,Edit"
clr --allowed-tools "Bash(git:*)" "Check status"
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--output-format`, `--input-format`, `--max-turns`, `--disallowed-tools`, `--max-budget-usd`, `--add-dir`, `--fallback-model` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |
