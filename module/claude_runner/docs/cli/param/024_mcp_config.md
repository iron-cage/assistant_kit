# CLI Parameter: --mcp-config

Path to an MCP (Model Context Protocol) configuration JSON file. Forwarded
directly to the `claude` subprocess as `--mcp-config <path>`. May be specified
multiple times to load multiple MCP server configs.

- **Type:** [`McpConfigPath`](../type/11_mcp_config_path.md)
- **Default:** — (unset; no MCP servers loaded)
- **Command:** [`run`](../command/01_run.md)
- **Group:** [Claude-Native Flags](../param_group/01_claude_native_flags.md)
- **JSON Key:** `"mcp-config"`

```sh
clr --mcp-config ~/.claude/mcp.json "Fix bug"
clr --mcp-config server1.json --mcp-config server2.json "Use all MCP tools"
```

**Note:** Each config file must be a JSON object conforming to the MCP server
configuration format. Multiple `--mcp-config` flags are forwarded individually,
each as a separate `--mcp-config` argument to `claude`.

**Note:** Paths are resolved relative to the caller's working directory (after
any `--dir` change is applied).

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`McpConfigPath`](../type/11_mcp_config_path.md) | Semantic | String | valid filesystem path, valid JSON |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 1 | [Claude-Native Flags](../param_group/01_claude_native_flags.md) | Full | `--print`, `--model`, `--verbose`, `--effort`, `--json-schema` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | — | Repeatable (0+) |
| 5 | [`ask`](../command/05_ask.md) | — | Repeatable (0+) |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 19 | [019_mcp_config_injection.md](../user_story/019_mcp_config_injection.md) | Developer |
