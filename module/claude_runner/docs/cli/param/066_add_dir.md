# CLI Parameter: --add-dir

Add an additional directory for Claude Code to access beyond the working directory.

- **Type:** path
- **Default:** — (Claude Code default; no additional directories when not specified)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **Validation:** requires a value; `--add-dir` at end of argv → error
- **JSON Key:** `"add-dir"`

```sh
clr "Fix bug" --add-dir /path/to/shared/lib
clr --add-dir ../common "Review dependencies"
```

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema`, `--mcp-config`, `--output-format`, `--max-turns`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--fallback-model` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | — |
| 2 | [`isolated`](../command/02_isolated.md) | — | Repeatable; `CLR_ADD_DIR` env fallback; injected into subprocess command (TSK-329) |
| 5 | [`ask`](../command/05_ask.md) | — | — |
