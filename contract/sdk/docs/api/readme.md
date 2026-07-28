# API Doc Entity

Library interface documentation for the Agent SDK's TypeScript surface (`@anthropic-ai/claude-agent-sdk`). This is a canonical `api/` doc entity per `doc_des.rulebook.md` (library interfaces), unlike this crate's `behavior/`/`param/`/`pattern/` collections, which are local extensions inherited from the `contract/claude_code` precedent.

### Scope

- **Purpose**: Provide an authoritative, precise reference for the SDK's exported functions, types, and objects, as fetched from official documentation.
- **Responsibility**: Master table and per-symbol detail files for `query()`, `Options`, `tool()`/`createSdkMcpServer()`, the `Query` control object, the `SDKMessage` stream, and the `CanUseTool` permission callback.
- **In Scope**: TypeScript signatures exactly as documented at `https://code.claude.com/docs/en/agent-sdk/typescript`; Python equivalents noted where the official overview page's code samples make the mapping unambiguous (snake_case field names, `ClaudeAgentOptions` instead of `Options`).
- **Out of Scope**: Session-management free functions (`listSessions`, `getSessionMessages`, etc. — narrow, session-store-management surface not relevant to the invocation-model question this crate exists to inform); full field-by-field `Options` reference (→ [`../param/`](../param/readme.md), a curated subset only).

### Responsibility Table

| File | Responsibility |
|------|-----------------|
| readme.md | Master API table (this file) |
| 001_query_function.md | `query()` — the SDK's core entry point |
| 002_options_type.md | `Options`/`ClaudeAgentOptions` — full field list at a glance |
| 003_custom_tool_definition.md | `tool()` + `createSdkMcpServer()` — custom tool registration |
| 004_query_control_object.md | `Query` — the live control object `query()` returns |
| 005_sdk_message_stream.md | `SDKMessage` — the streamed message type union |
| 006_permission_callback.md | `CanUseTool` — per-call permission override callback |
