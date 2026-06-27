# JSONL: User Entry

### Scope

- **Purpose**: Specify the user message entry format including the message envelope and thinkingMetadata.
- **Responsibility**: Authoritative instance for user JSONL entries — fields specific to `type: "user"` entries.
- **In Scope**: `message.role`, `message.content` (string), `thinkingMetadata` object and its subfields.
- **Out of Scope**: Common fields shared with assistant entries (→ [001_common_fields.md](001_common_fields.md)); assistant-specific fields (→ [003_assistant_entry.md](003_assistant_entry.md)).

### Schema

```json
{
  "parentUuid": null,
  "isSidechain": false,
  "userType": "external",
  "cwd": "/home/alice/pro",
  "sessionId": "8d795a1c-c81d-4010-8d29-b4e678272419",
  "version": "2.0.31",
  "gitBranch": "master",
  "type": "user",
  "message": {
    "role": "user",
    "content": "command to repeat something every hour?"
  },
  "uuid": "a6f3bd8c-5575-4eab-82b0-b856f7a02833",
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

### User-Specific Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `message.role` | string | ✅ | Always `"user"` |
| `message.content` | string | ✅ | The user's message text (plain string, not array) |
| `thinkingMetadata` | object | ❌ | Configuration for extended thinking (present when thinking is configured) |
| `thinkingMetadata.level` | string | — | Thinking level: `"low"`, `"medium"`, `"high"` |
| `thinkingMetadata.disabled` | boolean | — | Whether thinking is disabled for this message |
| `thinkingMetadata.triggers` | array | — | Trigger words that enabled thinking |
| `thinkingMetadata.triggers[].start` | number | — | Character offset where trigger starts |
| `thinkingMetadata.triggers[].end` | number | — | Character offset where trigger ends |
| `thinkingMetadata.triggers[].text` | string | — | The trigger word (e.g., `"ultrathink"`) |

### Notes

**`message.content` is a string**: In user entries, `message.content` is a plain string. In assistant entries it is an array of content blocks. This asymmetry is important for parsing — check `type` first to determine content shape.

**`thinkingMetadata`**: Optional; present when the user has configured extended thinking (e.g., by typing "ultrathink"). The `triggers` array identifies which words in the message enabled thinking.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| jsonl | [001_common_fields.md](001_common_fields.md) | Common fields: uuid, parentUuid, timestamp, type, cwd, sessionId, etc. |
| jsonl | [003_assistant_entry.md](003_assistant_entry.md) | Assistant entry format (message.content is array) |
| jsonl | [005_thinking_block.md](005_thinking_block.md) | Thinking blocks triggered by `thinkingMetadata` in this entry |
| jsonl | [007_tool_result_block.md](007_tool_result_block.md) | Tool result blocks that appear in user entries |
