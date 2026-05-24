# CLI Type: McpConfigPath

Filesystem path to an MCP (Model Context Protocol) configuration JSON file.
Each value becomes one `--mcp-config` argument forwarded to the `claude`
subprocess.

- **Purpose:** Filesystem path to an MCP configuration JSON file
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** must be a valid filesystem path; file should exist and be valid JSON
- **Parsing:** consumed as the next token after `--mcp-config`; repeatable
- **Methods:** —

```sh
clr --mcp-config /path/to/mcp.json "task"
clr --mcp-config server1.json --mcp-config server2.json "task"
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `--mcp-config` |
| 5 | [`ask`](../command/05_ask.md) | `--mcp-config` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 24 | [`--mcp-config`](../param/024_mcp_config.md) | 2 |
