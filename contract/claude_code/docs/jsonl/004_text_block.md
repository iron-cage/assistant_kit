# JSONL: Text Block

### Scope

- **Purpose**: Specify the text content block format within assistant message content arrays.
- **Responsibility**: Authoritative instance for `type: "text"` content blocks.
- **In Scope**: `type`, `text` fields; usage context within assistant message content arrays.
- **Out of Scope**: Other block types (→ [005](005_thinking_block.md), [006](006_tool_use_block.md), [007](007_tool_result_block.md)); assistant entry envelope (→ [003_assistant_entry.md](003_assistant_entry.md)).

### Schema

```json
{
  "type": "text",
  "text": "Here's the answer to your question..."
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"text"` |
| `text` | string | ✅ | Plain text response from Claude |

### Notes

The text block is the most common content block type. It contains the primary prose response. A single assistant message may contain multiple text blocks interspersed with tool use blocks.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| jsonl | [003_assistant_entry.md](003_assistant_entry.md) | Assistant entry: `message.content` array that contains this block |
| format | [`../format/007_json_response.md`](../format/007_json_response.md) | JSON response format that uses this block in its `content` array |
