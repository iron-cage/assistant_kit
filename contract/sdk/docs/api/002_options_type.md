# API: `Options` / `ClaudeAgentOptions`

### Scope

- **Purpose**: Provide the full field list of the SDK's options object at a glance, as fetched from the official TypeScript reference.
- **Responsibility**: Authoritative, exhaustive field enumeration — the source this crate's curated `param/` instances are drawn from.
- **In Scope**: Every field name, type, and documented default from the official TypeScript `Options` interface (61 fields, fetched verbatim).
- **Out of Scope**: Prose explanation of any individual field beyond its type/default (→ [`../param/`](../param/readme.md) for the 13 fields curated as dedicated instances).

Full interface, TypeScript, fetched from `https://code.claude.com/docs/en/agent-sdk/typescript`:

```typescript
interface Options {
  abortController?: AbortController; // default: new AbortController()
  additionalDirectories?: string[]; // default: []
  agent?: string; // default: undefined
  agents?: Record<string, AgentDefinition>; // default: undefined
  agentProgressSummaries?: boolean; // default: false
  allowDangerouslySkipPermissions?: boolean; // default: false
  allowedTools?: string[]; // default: []
  betas?: SdkBeta[]; // default: []
  canUseTool?: CanUseTool; // default: undefined
  continue?: boolean; // default: false
  cwd?: string; // default: process.cwd()
  debug?: boolean; // default: false
  debugFile?: string; // default: undefined
  disallowedTools?: string[]; // default: []
  effort?: 'low' | 'medium' | 'high' | 'xhigh' | 'max'; // default: Model default
  enableFileCheckpointing?: boolean; // default: false
  env?: Record<string, string | undefined>; // default: process.env
  executable?: 'bun' | 'deno' | 'node'; // default: Auto-detected
  executableArgs?: string[]; // default: []
  extraArgs?: Record<string, string | null>; // default: {}
  fallbackModel?: string; // default: undefined
  forkSession?: boolean; // default: false
  forwardSubagentText?: boolean; // default: false
  hooks?: Partial<Record<HookEvent, HookCallbackMatcher[]>>; // default: {}
  includeHookEvents?: boolean; // default: false
  includePartialMessages?: boolean; // default: false
  loadTimeoutMs?: number; // default: 60000
  managedSettings?: Settings; // default: undefined
  maxBudgetUsd?: number; // default: undefined
  maxThinkingTokens?: number; // default: undefined (deprecated, use thinking)
  maxTurns?: number; // default: undefined
  mcpServers?: Record<string, McpServerConfig>; // default: {}
  model?: string; // default: Default from CLI
  onElicitation?: (request: ElicitationRequest, options: { signal: AbortSignal }) => Promise<ElicitationResult>; // default: undefined
  outputFormat?: { type: 'json_schema'; schema: JSONSchema }; // default: undefined
  pathToClaudeCodeExecutable?: string; // default: Auto-resolved
  permissionMode?: PermissionMode; // default: 'default'
  permissionPromptToolName?: string; // default: undefined
  persistSession?: boolean; // default: true
  planModeInstructions?: string; // default: undefined
  plugins?: SdkPluginConfig[]; // default: []
  promptSuggestions?: boolean; // default: false
  resume?: string; // default: undefined
  resumeSessionAt?: string; // default: undefined
  sandbox?: SandboxSettings; // default: undefined
  sessionId?: string; // default: Auto-generated
  sessionStore?: SessionStore; // default: undefined
  sessionStoreFlush?: 'batched' | 'eager'; // default: 'batched'
  settings?: string | Settings; // default: undefined
  settingSources?: SettingSource[]; // default: All sources
  skills?: string[] | 'all'; // default: undefined
  spawnClaudeCodeProcess?: (options: SpawnOptions) => SpawnedProcess; // default: undefined
  stderr?: (data: string) => void; // default: undefined
  strictMcpConfig?: boolean; // default: false
  systemPrompt?: string | { type: 'preset'; preset: 'claude_code'; append?: string; excludeDynamicSections?: boolean }; // default: undefined (minimal prompt)
  taskBudget?: { total: number }; // default: undefined
  thinking?: ThinkingConfig; // default: { type: 'adaptive' }
  title?: string; // default: undefined
  toolAliases?: Record<string, string>; // default: undefined
  toolConfig?: ToolConfig; // default: undefined
  tools?: string[] | { type: 'preset'; preset: 'claude_code' }; // default: undefined
}
```

Python's `ClaudeAgentOptions` mirrors this field-for-field in snake_case (`allowed_tools`, `permission_mode`, `mcp_servers`, etc.), confirmed by the official overview page's Python code samples (`ClaudeAgentOptions(allowed_tools=[...], permission_mode="acceptEdits", hooks={...})`); a field-by-field Python name table was not independently fetched and is inferred from the snake_case convention observed across every Python sample on the overview page, not enumerated exhaustively.

13 of these 61 fields are judged most relevant to porting `claude_runner`'s invocation model to SDK-mode and have dedicated instances in [`../param/`](../param/readme.md); the remaining fields (hooks, sandboxing, session-store persistence, budget/thinking controls, plugin/skill loading, etc.) are out of scope for that specific decision and documented only here, in the full verbatim listing above.

### Params

| File | Relationship |
|------|--------------|
| [../param/readme.md](../param/readme.md) | The curated 13-field subset with dedicated per-field instances |

### Behaviors

| File | Relationship |
|------|--------------|
| [../behavior/006_s6_permission_modes_richer_than_cli.md](../behavior/006_s6_permission_modes_richer_than_cli.md) | `permissionMode` field's enum values |
| [../behavior/008_s8_session_identity_options_vs_flags.md](../behavior/008_s8_session_identity_options_vs_flags.md) | `resume`/`sessionId`/`continue`/`forkSession` fields |
