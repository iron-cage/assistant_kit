# mcpServers

Registers external (subprocess/stdio) and in-process SDK-defined MCP servers.

### Forms

| | Value |
|-|-------|
| TS Field | `mcpServers?: Record<string, McpServerConfig>` |
| Python Field | `mcp_servers` (confirmed via official MCP example: `mcp_servers={"playwright": {"command": "npx", ...}}`) |
| CLI Equivalent | `--mcp-config` — [`../../../claude_code/docs/param/039_mcp_config.md`](../../../claude_code/docs/param/039_mcp_config.md) |

### Type

map: server name → `McpServerConfig` (either an external subprocess spec like `{ command, args }`, or the `McpSdkServerConfigWithInstance` object returned by `createSdkMcpServer()`)

### Default

`{}`

### Since

SDK GA

### Description

A single field covers two distinct registration shapes: (1) external servers, launched as their own subprocess by the underlying `claude` process (e.g. `{ playwright: { command: "npx", args: ["@playwright/mcp@latest"] } }` from the official MCP example) — behaviorally identical to what `--mcp-config` already does at the CLI level; and (2) in-process SDK servers, where the value is the object `createSdkMcpServer()` returns directly (see [`../api/003_custom_tool_definition.md`](../api/003_custom_tool_definition.md)) — no subprocess, no `command`/`args`, just a live server instance handed straight to the SDK. The key under which either shape is registered becomes the `{server_name}` segment in `mcp__{server_name}__{tool_name}` addressing.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master curated parameter table |
| doc | [../api/003_custom_tool_definition.md](../api/003_custom_tool_definition.md) | `createSdkMcpServer()` — the in-process registration shape |
| behavior | [../behavior/003_s3_custom_tools_in_process.md](../behavior/003_s3_custom_tools_in_process.md) | In-process vs. external execution distinction |
| doc | [../../../claude_code/docs/param/039_mcp_config.md](../../../claude_code/docs/param/039_mcp_config.md) | CLI-level equivalent for the external-server shape only |
