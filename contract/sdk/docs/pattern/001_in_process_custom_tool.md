# Pattern: In-Process Custom Tool

### Problem

A caller wants Claude to invoke custom logic (query an internal service, read from a proprietary format, enforce a domain-specific guardrail) as a tool call, without paying the cost of an external MCP server subprocess and without being subject to the Bash tool's own timeout/backgrounding rules (documented in `contract/claude_code/docs/behavior/036_b36_background_task_lifecycle.md`).

### Solution

Define the tool's schema and handler with `tool()`, bundle one or more such tools into a server object with `createSdkMcpServer()`, and register that server object directly under a key in `options.mcpServers` — no `command`/`args` subprocess spec, just the live object:

```typescript
import { tool, createSdkMcpServer, query } from "@anthropic-ai/claude-agent-sdk";
import { z } from "zod";

const getWeather = tool(
  "get_weather",
  "Get current weather for a city",
  { city: z.string() },
  async (args) => ({ content: [{ type: "text", text: `Weather in ${args.city}: sunny` }] }),
);

const weatherServer = createSdkMcpServer({ name: "weather", tools: [getWeather] });

for await (const message of query({
  prompt: "What's the weather in Tokyo?",
  options: { mcpServers: { weather: weatherServer }, allowedTools: ["mcp__weather__get_weather"] },
})) {
  console.log(message);
}
```

Claude addresses the tool as `mcp__weather__get_weather` (see [S5](../behavior/005_s5_mcp_tool_naming.md)); `handler` executes as a plain async function in the same process that called `query()` (see [S3](../behavior/003_s3_custom_tools_in_process.md)) — no OS subprocess, no Bash-tool harness, no artificial timeout ceiling beyond whatever the handler's own code imposes.

### Applicability

Any host application embedding the SDK (TypeScript or Python) that needs custom tool logic tighter-integrated than an external MCP server allows — e.g. direct access to the host process's in-memory state, database connections already open in that process, or credentials the host doesn't want to expose to a separate subprocess. Not directly applicable to a pure-Rust caller (no official binding — see [`002_rust_bridge_strategies.md`](002_rust_bridge_strategies.md)) except as a design reference for what a Rust-side equivalent mechanism would need to replicate.

### Consequences

**Benefits**: no subprocess-spawn latency per tool call; no serialization boundary between tool logic and host application state; not bound by Bash-tool-specific timeout/backgrounding env vars. **Costs**: the tool's `handler` code must be written in TypeScript or Python (the SDK's own runtime) — it cannot itself be a thin Rust FFI call without its own bridging problem; a long-running or blocking `handler` with a bug (e.g. no internal timeout) can hang the entire `query()` call indefinitely, since there is no external harness imposing a ceiling on its behalf.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| entity | [readme.md](readme.md) | Master pattern index |
| behavior | [../behavior/003_s3_custom_tools_in_process.md](../behavior/003_s3_custom_tools_in_process.md) | Underlying behavior this pattern relies on |
| behavior | [../behavior/005_s5_mcp_tool_naming.md](../behavior/005_s5_mcp_tool_naming.md) | Naming convention used in the example |
| api | [../api/003_custom_tool_definition.md](../api/003_custom_tool_definition.md) | `tool()`/`createSdkMcpServer()` full signatures |
| pattern | [002_rust_bridge_strategies.md](002_rust_bridge_strategies.md) | Why this pattern doesn't directly transfer to a Rust caller |
