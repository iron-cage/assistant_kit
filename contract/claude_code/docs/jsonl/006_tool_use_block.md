# JSONL: Tool Use Block

### Scope

- **Purpose**: Specify the tool invocation content block format within assistant message content arrays.
- **Responsibility**: Authoritative instance for `type: "tool_use"` content blocks — structure, id/name/input fields.
- **In Scope**: `type`, `id`, `name`, `input` fields; relationship to tool result blocks.
- **Out of Scope**: Tool result response block (→ [007_tool_result_block.md](007_tool_result_block.md)); other block types (→ [004](004_text_block.md), [005](005_thinking_block.md)); tool behavior flags (→ [`../behavior/016_b16_tools_flag.md`](../behavior/016_b16_tools_flag.md)).

### Schema

```json
{
  "type": "tool_use",
  "id": "toolu_01ABC123",
  "name": "Bash",
  "input": {
    "command": "ls -la",
    "description": "List files"
  }
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | ✅ | Always `"tool_use"` |
| `id` | string | ✅ | Tool use ID (e.g., `"toolu_01ABC123"`); referenced by the corresponding `tool_result` block |
| `name` | string | ✅ | Tool name (e.g., `"Bash"`, `"Edit"`, `"Read"`, `"Write"`, `"Agent"`) |
| `input` | object | ✅ | Tool-specific input parameters as a JSON object |

### Notes

**`id` pairing**: Each `tool_use` block has a unique `id` that is referenced by the corresponding `tool_result` block in the following user entry (`tool_use_id` field). This id is the programmatic link between invocation and result.

**`input` schema**: Tool-specific. `Bash` has `command` and optionally `description`; `Edit` has `file_path`, `old_string`, `new_string`; `Read` has `file_path`; etc.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| jsonl | [003_assistant_entry.md](003_assistant_entry.md) | Assistant entry: `message.content` array that contains this block |
| jsonl | [007_tool_result_block.md](007_tool_result_block.md) | Tool result block that references this block's `id` |
| behavior | [`../behavior/016_b16_tools_flag.md`](../behavior/016_b16_tools_flag.md) | `--tools` flag that controls which tools can be invoked |
