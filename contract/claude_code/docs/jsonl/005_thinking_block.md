# JSONL: Thinking Block

### Scope

- **Purpose**: Specify the extended thinking/reasoning content block format within assistant message content arrays.
- **Responsibility**: Authoritative instance for `type: "thinking"` content blocks — structure, signature field, and version history.
- **In Scope**: `type`, `thinking`, `signature` fields; cryptographic signature; when thinking blocks appear.
- **Out of Scope**: `thinkingMetadata` in user entries (→ [002_user_entry.md](002_user_entry.md)); other block types (→ [004](004_text_block.md), [006](006_tool_use_block.md), [007](007_tool_result_block.md)).

### Schema

```json
{
  "type": "thinking",
  "thinking": "Let me analyze this step by step...",
  "signature": "ErUBCkYIARgCIkDlgq..."
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"thinking"` |
| `thinking` | string | ✅ | Extended reasoning/chain-of-thought content |
| `signature` | string | ✅ | Base64-encoded cryptographic signature verifying authenticity |

### Notes

**Signature**: Added in Claude Code v2.0.30+. Cryptographically signed thinking content to verify authenticity — prevents tampering with reasoning content when it is passed back in subsequent messages.

**When present**: Thinking blocks appear when extended thinking is enabled (user typed "ultrathink" or similar trigger in `thinkingMetadata`). They precede text and tool use blocks in the content array.

**Ordering**: Typically appears first in the content array: `[thinking, text, ...]` or `[thinking, tool_use, ...]`.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| jsonl | [002_user_entry.md](002_user_entry.md) | `thinkingMetadata` in user entries (triggers thinking) |
| jsonl | [003_assistant_entry.md](003_assistant_entry.md) | Assistant entry: `message.content` array that contains this block |
| format | [`../formats/007_json_response.md`](../formats/007_json_response.md) | JSON response format that uses this block in its `content` array |
