# Tool: WaitForMcpServers

Wait for MCP servers that are still connecting in the background.

### Category

Extensibility

### Permission Required

No

### Description

Waits for MCP servers that are still connecting in the background, so a request
can use their tools without restarting. Only appears when tool search is disabled
(since ToolSearch handles the wait when tool search is enabled). This tool is
model-initiated — the model calls it when it detects that needed tools are not
yet available.

### Parameters

None.

### Since

v2.1+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [038_tool_search.md](038_tool_search.md) | Deferred tool loading (alternative) |
| doc | [../subcommand/006_mcp.md](../subcommand/006_mcp.md) | MCP server management |
