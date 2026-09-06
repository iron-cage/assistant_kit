# JSONL: User Entry

### Scope

- **Purpose**: Specify the user message entry format including the message envelope, its two distinct `message.content` shapes, and thinkingMetadata.
- **Responsibility**: Authoritative instance for user JSONL entries — fields specific to `type: "user"` entries.
- **In Scope**: `message.role`, `message.content` (string OR array — see Notes), `thinkingMetadata` object and its subfields, `toolUseResult`/`sourceToolAssistantUUID` (tool-result-shaped entries), `promptId`/`promptSource`.
- **Out of Scope**: Common fields shared with assistant entries (→ [001_common_fields.md](001_common_fields.md)); assistant-specific fields (→ [003_assistant_entry.md](003_assistant_entry.md)); the `tool_result` content block's own internal shape (→ [007_tool_result_block.md](007_tool_result_block.md)).

### Schema

```json
{
  "parentUuid": null,
  "isSidechain": false,
  "userType": "external",
  "cwd": "/home/alice/pro",
  "sessionId": "feed0001-0000-4000-8000-000000000001",
  "version": "2.0.31",
  "gitBranch": "master",
  "type": "user",
  "message": {
    "role": "user",
    "content": "command to repeat something every hour?"
  },
  "uuid": "feed0003-0000-4000-8000-000000000003",
  "timestamp": "2025-11-08T23:30:10.039Z",
  "thinkingMetadata": {
    "level": "high",
    "disabled": false,
    "triggers": [
      { "start": 58, "end": 68, "text": "ultrathink" }
    ]
  }
}
```

A second, far more common shape carries a tool result rather than typed text — `message.content` is an array and `sourceToolAssistantUUID` (the `uuid` of the assistant entry whose `tool_use` this answers) is always present:

```json
{
  "parentUuid": "feed0004-0000-4000-8000-000000000004",
  "isSidechain": false,
  "userType": "external",
  "cwd": "/home/alice/pro",
  "sessionId": "feed0001-0000-4000-8000-000000000001",
  "version": "2.1.220",
  "gitBranch": "master",
  "type": "user",
  "sourceToolAssistantUUID": "feed0004-0000-4000-8000-000000000004",
  "message": {
    "role": "user",
    "content": [
      { "type": "tool_result", "tool_use_id": "toolu_...", "content": "...", "is_error": false }
    ]
  },
  "toolUseResult": { "...": "structured summary, tool-specific shape" },
  "uuid": "feed0005-0000-4000-8000-000000000005",
  "timestamp": "2025-11-08T23:30:25.001Z"
}
```

### User-Specific Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `message.role` | string | ✅ | Always `"user"` |
| `message.content` | string \| array | ✅ | Typed human text (plain string) OR one or more content blocks, most commonly `tool_result` (see Notes) |
| `sourceToolAssistantUUID` | string | ❌ | `uuid` of the assistant entry whose `tool_use` this entry answers; present whenever `message.content` carries a `tool_result` block |
| `toolUseResult` | object | ❌ | Tool-specific structured summary of the result; shape varies per tool. Present on a majority but not all tool-result-shaped entries — `sourceToolAssistantUUID` is the reliable discriminator, not this field |
| `promptId` | string | ❌ | Identifier grouping entries produced by one prompt submission |
| `promptSource` | string | ❌ | How the prompt originated. Observed values: `"sdk"`, `"typed"`, `"system"`, `"queued"` |
| `thinkingMetadata` | object | ❌ | Configuration for extended thinking (present when thinking is configured) |
| `thinkingMetadata.level` | string | — | Thinking level: `"low"`, `"medium"`, `"high"` |
| `thinkingMetadata.disabled` | boolean | — | Whether thinking is disabled for this message |
| `thinkingMetadata.triggers` | array | — | Trigger words that enabled thinking |
| `thinkingMetadata.triggers[].start` | number | — | Character offset where trigger starts |
| `thinkingMetadata.triggers[].end` | number | — | Character offset where trigger ends |
| `thinkingMetadata.triggers[].text` | string | — | The trigger word (e.g., `"ultrathink"`) |

### Notes

**`message.content` has two shapes (corrected — previously documented as always a plain string)**: a full local-store scan (2026-09-06 snapshot: 5,446,921 lines across 17,210 session files) found `message.content` is an **array in 94.3%** of `user` entries and a **plain string in only 5.7%**. The array shape carries a `tool_result` block and always co-occurs with `sourceToolAssistantUUID`; the plain-string shape is genuine human-typed text. Contrast with assistant entries, where `message.content` is always an array (never a plain string) — check `sourceToolAssistantUUID`, not just `type`, to tell a human-typed user entry from a tool-result-shaped one.

**Verify yourself**: `grep -c '"sourceToolAssistantUUID"' <session>.jsonl` compared against `grep -c '"type":"user"' <session>.jsonl` (real session files serialize compact JSON, no space after `:`) — the ratio should land near 90%+ for any session with substantial tool use.

**`thinkingMetadata`**: Optional; present when the user has configured extended thinking (e.g., by typing "ultrathink"). The `triggers` array identifies which words in the message enabled thinking. Rare in the current store (147 of 1,456,798 user entries, 0.010%) and observed only up to v2.1.74 — likely superseded by the newer `effort` field on assistant entries (see [003_assistant_entry.md](003_assistant_entry.md)), though no doc instance confirms a formal deprecation.

### Since

pre-v1.0 (unverified) for `message.role`/`message.content`. `sourceToolAssistantUUID`, `promptId`, and `permissionMode` first observed at v2.1.50/v2.1.74 in a 2026-09-06 full-store scan (v2.0.56–v2.1.220 range); `promptSource` first observed at v2.1.197. `thinkingMetadata` observed only up to v2.1.74 despite the scanned range extending to v2.1.220 — a real retirement signal, not a sampling artifact.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| jsonl | [001_common_fields.md](001_common_fields.md) | Common fields: uuid, parentUuid, timestamp, type, cwd, sessionId, etc. |
| jsonl | [003_assistant_entry.md](003_assistant_entry.md) | Assistant entry format (message.content is array) |
| jsonl | [005_thinking_block.md](005_thinking_block.md) | Thinking blocks triggered by `thinkingMetadata` in this entry |
| jsonl | [007_tool_result_block.md](007_tool_result_block.md) | Tool result blocks that appear in user entries |
