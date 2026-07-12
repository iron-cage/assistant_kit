# API: `Query` Control Object

### Scope

- **Purpose**: Document the live control surface returned by `query()`, distinct from its role as a plain message stream.
- **Responsibility**: Authoritative reference for every `Query` method.
- **In Scope**: All methods on the `Query` interface, and the sibling `WarmQuery` interface returned by `startup()`.
- **Out of Scope**: The message types these methods produce/consume (→ [`005_sdk_message_stream.md`](005_sdk_message_stream.md)); `query()` itself (→ [`001_query_function.md`](001_query_function.md)).

```typescript
interface Query extends AsyncGenerator<SDKMessage, void> {
  interrupt(): Promise<SDKControlInterruptResponse | undefined>;
  rewindFiles(userMessageId: string, options?: { dryRun?: boolean }): Promise<RewindFilesResult>;
  setPermissionMode(mode: PermissionMode): Promise<void>;
  setModel(model?: string): Promise<void>;
  setMaxThinkingTokens(maxThinkingTokens: number | null): Promise<void>;
  applyFlagSettings(settings: { [K in keyof Settings]?: Settings[K] | null }): Promise<void>;
  initializationResult(): Promise<SDKControlInitializeResponse>;
  reinitialize(): Promise<SDKControlInitializeResponse>;
  supportedCommands(): Promise<SlashCommand[]>;
  supportedModels(): Promise<ModelInfo[]>;
  supportedAgents(): Promise<AgentInfo[]>;
  mcpServerStatus(): Promise<McpServerStatus[]>;
  accountInfo(): Promise<AccountInfo>;
  reconnectMcpServer(serverName: string): Promise<void>;
  toggleMcpServer(serverName: string, enabled: boolean): Promise<void>;
  setMcpServers(servers: Record<string, McpServerConfig>): Promise<McpSetServersResult>;
  streamInput(stream: AsyncIterable<SDKUserMessage>): Promise<void>;
  stopTask(taskId: string): Promise<void>;
  close(): void;
}

interface WarmQuery extends AsyncDisposable {
  query(prompt: string | AsyncIterable<SDKUserMessage>): Query;
  close(): void;
}
```

`Query` being `AsyncGenerator<SDKMessage, void>` *plus* 18 additional methods is the concrete evidence behind [S2](../behavior/002_s2_stream_json_control_protocol.md)'s claim that the SDK drives a long-lived, bidirectionally-controllable subprocess rather than a fire-and-forget request. Notably: `rewindFiles()` (undo file edits back to a given user message), `interrupt()` (mid-generation cancellation), and `streamInput()` (inject further user turns into an already-running query) have no single-flag CLI equivalent documented anywhere in `contract/claude_code` — these are capabilities the control protocol adds on top of the CLI surface, not a re-exposure of existing flags.

### Behaviors

| File | Relationship |
|------|--------------|
| [../behavior/002_s2_stream_json_control_protocol.md](../behavior/002_s2_stream_json_control_protocol.md) | This method surface is the primary evidence for that behavior claim |
| [../behavior/006_s6_permission_modes_richer_than_cli.md](../behavior/006_s6_permission_modes_richer_than_cli.md) | `setPermissionMode()` — live mode changes mid-session |

### Params

| File | Relationship |
|------|--------------|
| [../param/013_can_use_tool.md](../param/013_can_use_tool.md) | Related per-call override; distinct from this object's session-wide `setPermissionMode()` |
