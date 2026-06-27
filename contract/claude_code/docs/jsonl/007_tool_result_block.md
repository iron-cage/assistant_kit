# JSONL: Tool Result Block

### Scope

- **Purpose**: Specify the tool execution result content block format within user message content arrays.
- **Responsibility**: Authoritative instance for `type: "tool_result"` content blocks — structure, tool_use_id pairing, is_error flag.
- **In Scope**: `type`, `tool_use_id`, `content`, `is_error` fields; placement in user entries; pairing with tool_use blocks.
- **Out of Scope**: Tool use invocation block (→ [006_tool_use_block.md](006_tool_use_block.md)); other block types (→ [004](004_text_block.md), [005](005_thinking_block.md)).

### Schema

```json
{
  "type": "tool_result",
  "tool_use_id": "toolu_01ABC123",
  "content": "file1.txt\nfile2.txt\nfile3.txt",
  "is_error": false
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"tool_result"` |
| `tool_use_id` | string | ✅ | References the `id` of the corresponding `tool_use` block |
| `content` | string | ✅ | Tool execution output (stdout/stderr or structured result) |
| `is_error` | boolean | ✅ | `true` if the tool invocation failed; `false` on success |

### Notes

**Placement**: Tool result blocks appear in **user** entries (not assistant entries), as part of the API turn structure where the human side submits tool results back to the model.

**`tool_use_id` pairing**: References the `id` field from the corresponding `tool_use` block in the preceding assistant entry. This is the programmatic link between invocation and result.

**`content` type**: A plain string. May contain multi-line output (newline-delimited), JSON, or error messages.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| jsonl | [002_user_entry.md](002_user_entry.md) | User entry: `message.content` field that can contain this block |
| jsonl | [006_tool_use_block.md](006_tool_use_block.md) | Tool use block whose `id` is referenced by `tool_use_id` |
| format | [`../formats/007_json_response.md`](../formats/007_json_response.md) | JSON response format that uses this block in its `content` array |
