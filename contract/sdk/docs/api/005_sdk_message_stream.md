# API: `SDKMessage` Stream

### Scope

- **Purpose**: Document the union of message types `query()` yields as an async iterator.
- **Responsibility**: Authoritative reference for the message stream shape.
- **In Scope**: The `SDKMessage` union member list and common cross-cutting properties.
- **Out of Scope**: The `Query` object's non-iterator control methods (→ [`004_query_control_object.md`](004_query_control_object.md)).

```typescript
type SDKMessage =
  | SDKAssistantMessage
  | SDKUserMessage
  | SDKToolUseMessage
  | SDKToolResultMessage
  | SDKResultMessage
  | SDKThinkingMessage
  | SDKTextMessage
  | SDKControlRequestMessage
  | SDKControlResponseMessage
  | SDKSystemMessage
  | SDKTaskProgressMessage
  | SDKHookStartedMessage
  | SDKHookProgressMessage
  | SDKHookResponseMessage
  | SDKPartialMessage;
```

Common properties observed across variants (not every field appears on every variant): `type: string` (discriminant), `content?: unknown`, `uuid?: string`, `thinking?: string`, `text?: string`, `tool_use_id?: string`, `parent_tool_use_id?: string` (present specifically on subagent-originated messages — the official Subagents example calls this out explicitly as how a caller tells which messages came from which spawned subagent). `SDKResultMessage` carries the final `result` field the official quickstart samples check for (`if (\"result\" in message) console.log(message.result)` / `if hasattr(message, "result")`). `SDKSystemMessage` with `subtype === "init"` is what the official Sessions example reads `session_id`/`data["session_id"]` off of to capture a session for later `resume`.

The 15-member union, plus the further `SDKControlRequestMessage`/`SDKControlResponseMessage` pair specifically, is the concrete message-level evidence that a real bidirectional protocol runs under `query()` — a single-shot `--print --output-format json` invocation (`clr`'s current mode) produces exactly one JSON object on stdout, not a discriminated 15-variant stream; see [S2](../behavior/002_s2_stream_json_control_protocol.md).

### Behaviors

| File | Relationship |
|------|--------------|
| [../behavior/002_s2_stream_json_control_protocol.md](../behavior/002_s2_stream_json_control_protocol.md) | This union's breadth is direct evidence for that behavior's protocol claim |
| [../behavior/008_s8_session_identity_options_vs_flags.md](../behavior/008_s8_session_identity_options_vs_flags.md) | `SDKSystemMessage` `"init"` subtype's `session_id` capture, used with `resume` |
