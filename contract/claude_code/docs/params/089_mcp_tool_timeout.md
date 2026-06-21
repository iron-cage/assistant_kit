# mcp_tool_timeout

Timeout in milliseconds for MCP tool invocations.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `MCP_TOOL_TIMEOUT` |
| Config Key | — |

### Type

integer (milliseconds)

### Default

Binary default (unspecified)

### Since

v2.1.142

### Description

Sets the maximum time in milliseconds that Claude Code will wait for an MCP
tool to respond before timing out. Applies to all MCP-provided tool calls.
Useful for controlling latency when MCP servers are slow or unresponsive.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [039_mcp_config.md](039_mcp_config.md) | MCP server configuration |
| doc | [041_mcp_servers.md](041_mcp_servers.md) | Inline MCP server definitions |
