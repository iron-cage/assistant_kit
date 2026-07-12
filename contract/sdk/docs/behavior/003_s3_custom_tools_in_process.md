# Behavior S3: Custom Tools Execute In-Process, Not Through the Bash Harness

### Scope

- **Purpose**: Document that SDK custom tools (`tool()` + `createSdkMcpServer()`) run as in-process callbacks in the SDK caller's own Node.js/Python process, not as OS subprocesses dispatched through Claude Code's built-in Bash-tool machinery.
- **Responsibility**: Authoritative instance for behavior S3.
- **In Scope**: The in-process execution model and its practical consequence (no Bash-tool-imposed timeout ceiling on hand-written tool logic); the `mcp__{server}__{tool}` naming this produces (→ [S5](005_s5_mcp_tool_naming.md)).
- **Out of Scope**: `tool()`/`createSdkMcpServer()` exact signatures (→ [`../api/003_custom_tool_definition.md`](../api/003_custom_tool_definition.md)); the reusable pattern this enables (→ [`../pattern/001_in_process_custom_tool.md`](../pattern/001_in_process_custom_tool.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Since**: SDK GA | **Evidence**: E2, E6, E8

A custom tool is defined with `tool(name, description, inputSchema, handler)` where `handler: (args, extra) => Promise<CallToolResult>` is a plain async function living in the SDK caller's own process — not a description of a command to `exec()`. One or more such tools are bundled via `createSdkMcpServer({ name, tools })` into an in-process MCP server, then wired into a `query()` call through `options.mcpServers`. Claude then invokes it exactly like any other MCP tool, over the `mcp__{server}__{tool}` naming convention (see S5) — but the "server" on the other end of that MCP connection is not a separate process at all; it is a function call inside the same runtime that called `query()`.

Anthropic's own comparison table states this explicitly when contrasting the SDK against Managed Agents: Custom tools are "In-process Python or TypeScript functions" for the Agent SDK, versus "Claude triggers the tool; you execute and return results" for Managed Agents (a real out-of-process round trip). This is the single most consequential behavior for this crate's purpose: this workspace's own `claude_runner` Bash-tool-mediated commands are subject to the Bash tool's own timeout/backgrounding rules (documented at length in `contract/claude_code/docs/behavior/036_b36_background_task_lifecycle.md` and `assistant_kit/claude_runner/docs/claude_code_background_task_env_vars.md`) — a hand-written SDK custom tool that itself spawns a long-running subprocess is bound by none of that; it is ordinary in-process async code, constrained only by whatever timeout the tool author writes (or doesn't write) into its own `handler`.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E2 | S3 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | `tool()` signature | `handler: (args, extra) => Promise<CallToolResult>` |
| E6 | S3 | Doc | `https://code.claude.com/docs/en/agent-sdk/typescript` | `tool()` / `createSdkMcpServer()` code block | Handler is a plain in-process async function, not a subprocess dispatch |
| E8 | S3 | Doc | `https://code.claude.com/docs/en/agent-sdk/overview` | "Agent SDK vs Managed Agents" table | "Custom tools: In-process Python or TypeScript functions" (Agent SDK) vs. "Claude triggers the tool; you execute and return results" (Managed Agents) |

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master index |
| behavior | [005_s5_mcp_tool_naming.md](005_s5_mcp_tool_naming.md) | Naming convention produced by this registration |
| api | [../api/003_custom_tool_definition.md](../api/003_custom_tool_definition.md) | `tool()` / `createSdkMcpServer()` exact signatures |
| pattern | [../pattern/001_in_process_custom_tool.md](../pattern/001_in_process_custom_tool.md) | Reusable pattern built on this behavior |
| doc | [`../../../claude_code/docs/behavior/036_b36_background_task_lifecycle.md`](../../../claude_code/docs/behavior/036_b36_background_task_lifecycle.md) | The Bash-tool-side timeout/backgrounding machinery this behavior sidesteps |
