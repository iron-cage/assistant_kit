# CLI Parameter: --disallowed-tools

Prevent Claude Code from using the specified tools.

- **Type:** string
- **Default:** — (Claude Code default; no tools disallowed when not specified)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; `--disallowed-tools` at end of argv → error
- **JSON Key:** `"disallowed-tools"`

```sh
clr "Refactor code" --disallowed-tools "Bash"
clr --disallowed-tools "Write,Edit" "Analyze this"
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--output-format`, `--max-turns`, `--allowed-tools`, `--max-budget-usd`, `--add-dir`, `--fallback-model` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 5 | [`ask`](../command/05_ask.md) | — | — |
