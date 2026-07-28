# Behavior S5: SDK Custom Tools Are Named `mcp__{server_name}__{tool_name}`

### Scope

- **Purpose**: Document the exact naming convention Claude uses to address SDK-registered custom tools, and its collision/wildcard semantics for allow/deny lists.
- **Responsibility**: Authoritative instance for behavior S5.
- **In Scope**: The `mcp__{server}__{tool}` format; its interaction with `allowedTools`/`disallowedTools` wildcard patterns (`mcp__server`, `mcp__server__*`, `mcp__*`).
- **Out of Scope**: How the server/tools are registered in the first place (→ [S3](003_s3_custom_tools_in_process.md)); the `allowedTools`/`disallowedTools` fields themselves (→ [`../param/004_allowed_tools.md`](../param/004_allowed_tools.md), [`../param/005_disallowed_tools.md`](../param/005_disallowed_tools.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Since**: SDK GA | **Evidence**: E2

A tool named `"bash"` registered inside a server created as `createSdkMcpServer({ name: "workspace", tools: [...] })` is addressed by Claude — and by the caller's own `toolAliases`/`allowedTools`/`disallowedTools` configuration — as `mcp__workspace__bash`. This is identical in shape to how any external (non-SDK, subprocess-based) MCP server's tools are named in Claude Code generally, meaning Claude does not distinguish "in-process SDK tool" from "out-of-process stdio MCP server tool" at the naming layer — the distinction is purely about where the `handler` code actually runs (S3).

Three wildcard forms are documented for `disallowedTools`: `mcp__server` removes all tools from that one server; `mcp__server__*` removes all tools matching that pattern (functionally identical to the plain server-name form in the documented example); `mcp__*` removes every MCP tool from every server, SDK-registered or external, in one entry.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E2 | S5 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | "MCP Tool Naming Convention" section | `mcp__{server}__{tool}` format; `toolAliases` example (`"Bash": "mcp__workspace__bash"`); three documented disallow wildcard patterns |

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master index |
| behavior | [003_s3_custom_tools_in_process.md](003_s3_custom_tools_in_process.md) | How the server/tool pair this naming addresses gets registered |
| api | [../api/003_custom_tool_definition.md](../api/003_custom_tool_definition.md) | `createSdkMcpServer()` `name` field that becomes the naming prefix |
| param | [../param/004_allowed_tools.md](../param/004_allowed_tools.md) | Allowlist field consuming this naming scheme |
| param | [../param/005_disallowed_tools.md](../param/005_disallowed_tools.md) | Denylist field consuming this naming scheme, incl. wildcard forms |
