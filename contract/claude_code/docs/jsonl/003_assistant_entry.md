# JSONL: Assistant Entry

### Scope

- **Purpose**: Specify the assistant message entry format including the message envelope, content array, and API response metadata.
- **Responsibility**: Authoritative instance for assistant JSONL entries — fields specific to `type: "assistant"` entries.
- **In Scope**: `message.model`, `message.id`, `message.type`, `message.role`, `message.content` (array), `message.stop_reason`, `message.stop_sequence`, `message.usage`, `requestId`.
- **Out of Scope**: Common fields (→ [001_common_fields.md](001_common_fields.md)); content block types within the array (→ [004–007](004_text_block.md)); usage object (→ [008_usage_object.md](008_usage_object.md)).

### Schema

```json
{
  "parentUuid": "a6f3bd8c-5575-4eab-82b0-b856f7a02833",
  "isSidechain": false,
  "userType": "external",
  "cwd": "/home/alice/pro",
  "sessionId": "8d795a1c-c81d-4010-8d29-b4e678272419",
  "version": "2.0.31",
  "gitBranch": "master",
  "message": {
    "model": "claude-sonnet-4-5-20250929",
    "id": "msg_01AEieWYMdbGML9PEKCmB36v",
    "type": "message",
    "role": "assistant",
    "content": [
      { "type": "thinking", "thinking": "...", "signature": "..." },
      { "type": "text", "text": "Looking at options..." },
      { "type": "tool_use", "id": "toolu_...", "name": "Bash", "input": {} }
    ],
    "stop_reason": "end_turn",
    "stop_sequence": null,
    "usage": { "input_tokens": 9, "output_tokens": 6, "service_tier": "standard" }
  },
  "requestId": "req_011CUwHuh7iPfwQNAXEeEYrP",
  "type": "assistant",
  "uuid": "56a226b5-0ec6-4214-af16-b13cc326f8dc",
  "timestamp": "2025-11-08T23:30:21.913Z"
}
```

### Assistant-Specific Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `message.model` | string | ✅ | Claude model used (e.g., `"claude-sonnet-4-5-20250929"`) |
| `message.id` | string | ✅ | API message ID (e.g., `"msg_01ABC..."`) |
| `message.type` | string | ✅ | Always `"message"` |
| `message.role` | string | ✅ | Always `"assistant"` |
| `message.content` | array | ✅ | Array of content blocks (text, thinking, tool_use, tool_result) |
| `message.stop_reason` | string \| null | ✅ | Why generation stopped: `"end_turn"`, `"stop_sequence"`, `"max_tokens"` |
| `message.stop_sequence` | string \| null | ✅ | The stop sequence that triggered (if any) |
| `message.usage` | object | ✅ | Token usage statistics |
| `requestId` | string | ✅ | API request ID (e.g., `"req_011..."`) |

### Notes

**`message.content` is an array**: Contrast with user entries where `message.content` is a string. May contain multiple blocks in one response (thinking + text + tool_use is common).

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| jsonl | [001_common_fields.md](001_common_fields.md) | Common fields: uuid, parentUuid, timestamp, type, cwd, sessionId, etc. |
| jsonl | [002_user_entry.md](002_user_entry.md) | User entry format (message.content is string) |
| jsonl | [004_text_block.md](004_text_block.md) | Text content block |
| jsonl | [005_thinking_block.md](005_thinking_block.md) | Thinking content block |
| jsonl | [006_tool_use_block.md](006_tool_use_block.md) | Tool use content block |
| jsonl | [007_tool_result_block.md](007_tool_result_block.md) | Tool result content block |
| jsonl | [008_usage_object.md](008_usage_object.md) | Usage object full schema |
