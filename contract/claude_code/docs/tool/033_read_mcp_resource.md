# Tool: ReadMcpResourceTool

Read a specific MCP resource by URI.

### Category

MCP Resources

### Permission Required

No

### Description

Reads the content of a specific MCP resource identified by its URI. The resource
must be exposed by a currently connected MCP server. Returns the resource content
in the format provided by the server.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `uri` | string | yes | The URI of the MCP resource to read |

### Since

v2.0+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [032_list_mcp_resources.md](032_list_mcp_resources.md) | List available MCP resources |
| doc | [../subcommand/006_mcp.md](../subcommand/006_mcp.md) | MCP server management |
