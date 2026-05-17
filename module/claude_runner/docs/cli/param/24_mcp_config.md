# Parameter :: 24. `--mcp-config`

Path to an MCP (Model Context Protocol) configuration JSON file. Forwarded
directly to the `claude` subprocess as `--mcp-config <path>`. May be specified
multiple times to load multiple MCP server configs.

- **Type:** [`McpConfigPath`](../type.md#type--11-mcpconfigpath)
- **Default:** — (unset; no MCP servers loaded)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Claude-Native Flags](../param_group.md#group--1-claude-native-flags)

```sh
clr --mcp-config ~/.claude/mcp.json "Fix bug"
clr --mcp-config server1.json --mcp-config server2.json "Use all MCP tools"
```

**Note:** Each config file must be a JSON object conforming to the MCP server
configuration format. Multiple `--mcp-config` flags are forwarded individually,
each as a separate `--mcp-config` argument to `claude`.

**Note:** Paths are resolved relative to the caller's working directory (after
any `--dir` change is applied).
