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
  setMcpPermissionModeOverride(serverName: string, mode: 'default' | 'auto' | null): Promise<{ warning?: string }>;
  setModel(model?: string): Promise<void>;
  setMaxThinkingTokens(maxThinkingTokens: number | null, thinkingDisplay?: 'summarized' | 'omitted' | null): Promise<void>; // @deprecated — prefer query()'s `thinking` option
  applyFlagSettings(settings: { [K in keyof Settings]?: Settings[K] | null }): Promise<void>;
  initializationResult(): Promise<SDKControlInitializeResponse>;
  reinitialize(): Promise<SDKControlInitializeResponse>;
  supportedCommands(): Promise<SlashCommand[]>;
  supportedModels(): Promise<ModelInfo[]>;
  supportedAgents(): Promise<AgentInfo[]>;
  mcpServerStatus(): Promise<McpServerStatus[]>;
  getContextUsage(): Promise<SDKControlGetContextUsageResponse>;
  usage_EXPERIMENTAL_MAY_CHANGE_DO_NOT_RELY_ON_THIS_API_YET(): Promise<SDKControlGetUsageResponse>;
  readFile(path: string, options?: { maxBytes?: number; encoding?: 'utf-8' | 'base64' }): Promise<SDKControlReadFileResponse | null>;
  reloadPlugins(): Promise<SDKControlReloadPluginsResponse>;
  reloadSkills(): Promise<SDKControlReloadSkillsResponse>;
  accountInfo(): Promise<AccountInfo>;
  seedReadState(path: string, mtime: number): Promise<void>;
  reconnectMcpServer(serverName: string): Promise<void>;
  toggleMcpServer(serverName: string, enabled: boolean): Promise<void>;
  setMcpServers(servers: Record<string, McpServerConfig>): Promise<McpSetServersResult>;
  streamInput(stream: AsyncIterable<SDKUserMessage>): Promise<void>;
  stopTask(taskId: string): Promise<void>;
  backgroundTasks(toolUseId?: string): Promise<boolean>;
  close(): void;
}

interface WarmQuery extends AsyncDisposable {
  query(prompt: string | AsyncIterable<SDKUserMessage>): Query;
  close(): void;
}
```

**Verified against:** `@anthropic-ai/claude-agent-sdk` v0.3.207 `sdk.d.ts` (installed package, read directly — not vendor doc pages). This superseded an earlier 18-method snapshot of this doc that predated a local installation being available to check against (see `contract/sdk/docs/behavior/readme.md`'s note that live-package confirmation was "out of scope until... 414... lands"): the real interface carries 25 async control methods plus the synchronous `close()`. `usage_EXPERIMENTAL_MAY_CHANGE_DO_NOT_RELY_ON_THIS_API_YET()` is listed for completeness but is the vendor's own explicitly-unstable API ("EXPERIMENTAL... may change or be removed in any release without notice — do not rely on it yet") — consumers of this doc should treat it as excluded from any "full parity" scope until the vendor stabilizes it. `setMaxThinkingTokens()` is `@deprecated` (vendor now steers callers to `query()`'s `thinking` option) but remains live and is not excluded — deprecation is not removal.

`Query` being `AsyncGenerator<SDKMessage, void>` *plus* 25 additional methods is the concrete evidence behind [S2](../behavior/002_s2_stream_json_control_protocol.md)'s claim that the SDK drives a long-lived, bidirectionally-controllable subprocess rather than a fire-and-forget request. Notably: `rewindFiles()` (undo file edits back to a given user message), `interrupt()` (mid-generation cancellation), and `streamInput()` (inject further user turns into an already-running query) have no single-flag CLI equivalent documented anywhere in `contract/claude_code` — these are capabilities the control protocol adds on top of the CLI surface, not a re-exposure of existing flags.

**Wire protocol note:** the underlying `SDKControlRequestInner` wire union carries additional subtypes with no corresponding public `Query` method (e.g. `get_plan`, `get_session_cost`, `get_settings`, `register_repo_root`, `rename_session`) — these are out of scope for this doc by definition (§ Scope: "All methods on the `Query` interface"), not an omission. A consumer needing full wire-subtype parity rather than public-method parity needs a separate contract doc scoped to the raw protocol.

### Behaviors

| File | Relationship |
|------|--------------|
| [../behavior/002_s2_stream_json_control_protocol.md](../behavior/002_s2_stream_json_control_protocol.md) | This method surface is the primary evidence for that behavior claim |
| [../behavior/006_s6_permission_modes_richer_than_cli.md](../behavior/006_s6_permission_modes_richer_than_cli.md) | `setPermissionMode()` — live mode changes mid-session |

### Params

| File | Relationship |
|------|--------------|
| [../param/013_can_use_tool.md](../param/013_can_use_tool.md) | Related per-call override; distinct from this object's session-wide `setPermissionMode()` |
