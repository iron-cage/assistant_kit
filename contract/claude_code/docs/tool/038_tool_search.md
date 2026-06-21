# Tool: ToolSearch

Search for and load deferred MCP tools by name or keyword.

### Category

Extensibility

### Permission Required

No

### Description

Fetches full schema definitions for deferred tools so they can be called. When
MCP tool search is enabled, tool schemas are not loaded eagerly — instead, only
tool names are listed in `<available-deferred-tools>`. This tool takes a query,
matches it against the deferred tool list, and returns the matched tools'
complete JSONSchema definitions. Once a tool's schema is returned, it becomes
callable.

Query forms:
- `"select:Read,Edit,Grep"` — fetch exact tools by name
- `"notebook jupyter"` — keyword search with max_results
- `"+slack send"` — require "slack" in name, rank by remaining terms

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | yes | Query to find deferred tools |
| `max_results` | number | no | Maximum results to return (default: 5) |

### Since

v2.1+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [039_wait_for_mcp_servers.md](039_wait_for_mcp_servers.md) | Alternative when tool search disabled |
| doc | [../subcommand/006_mcp.md](../subcommand/006_mcp.md) | MCP server management |
