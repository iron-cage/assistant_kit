# API: `tool()` and `createSdkMcpServer()`

### Scope

- **Purpose**: Document the two functions used together to define and register an in-process custom tool.
- **Responsibility**: Authoritative signature reference for custom-tool registration.
- **In Scope**: Both function signatures, the `ToolAnnotations` type, and their combination into `options.mcpServers`.
- **Out of Scope**: Why this executes in-process (→ [`../behavior/003_s3_custom_tools_in_process.md`](../behavior/003_s3_custom_tools_in_process.md)); the naming this produces (→ [`../behavior/005_s5_mcp_tool_naming.md`](../behavior/005_s5_mcp_tool_naming.md)); the reusable pattern (→ [`../pattern/001_in_process_custom_tool.md`](../pattern/001_in_process_custom_tool.md)).

```typescript
function tool<Schema extends AnyZodRawShape>(
  name: string,
  description: string,
  inputSchema: Schema,
  handler: (args: InferShape<Schema>, extra: unknown) => Promise<CallToolResult>,
  extras?: { annotations?: ToolAnnotations }
): SdkMcpToolDefinition<Schema>;

function createSdkMcpServer(options: {
  name: string;
  version?: string;
  tools?: Array<SdkMcpToolDefinition<any>>;
}): McpSdkServerConfigWithInstance;
```

`inputSchema` is a Zod raw shape — the tool's argument schema is defined with the `zod` validation library, and `handler`'s `args` parameter is inferred from it at compile time (`InferShape<Schema>`). The returned `SdkMcpToolDefinition` is not itself passed to `query()`; it is collected into a `createSdkMcpServer({ name, tools: [...] })` call, and *that* server object goes into `options.mcpServers` under a chosen key — the key becomes the `{server_name}` segment of the resulting `mcp__{server_name}__{tool_name}` address (see [S5](../behavior/005_s5_mcp_tool_naming.md)).

`ToolAnnotations` (optional per-tool metadata, all fields default to a "normal/unsafe" posture unless overridden):

```typescript
type ToolAnnotations = {
  title?: string; // default: undefined
  readOnlyHint?: boolean; // default: false
  destructiveHint?: boolean; // default: true
  idempotentHint?: boolean; // default: false
  openWorldHint?: boolean; // default: true
};
```

`destructiveHint` and `openWorldHint` both default to `true` — the SDK's own default posture treats a hand-written custom tool as potentially destructive and side-effecting unless the tool author explicitly marks it otherwise, which is directly relevant to how cautiously a Rust-side bridge tool (e.g., one that shells out to arbitrary commands) should be annotated.

### Behaviors

| File | Relationship |
|------|--------------|
| [../behavior/003_s3_custom_tools_in_process.md](../behavior/003_s3_custom_tools_in_process.md) | Why `handler` runs in-process rather than as a subprocess |
| [../behavior/005_s5_mcp_tool_naming.md](../behavior/005_s5_mcp_tool_naming.md) | How `createSdkMcpServer`'s `name` becomes part of the tool's address |

### Params

| File | Relationship |
|------|--------------|
| [../param/006_mcp_servers.md](../param/006_mcp_servers.md) | `options.mcpServers` — where the returned server config is registered |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/001_in_process_custom_tool.md](../pattern/001_in_process_custom_tool.md) | End-to-end reusable pattern combining these two functions |
