# JSONL

### Scope

- **Purpose**: Specify the JSONL format used by Claude Code to store conversation entries in session files.
- **Responsibility**: Master file for the `jsonl` collection — lists all 10 format concept instances covering entry types, content blocks, usage tracking, and threading model.
- **In Scope**: User entry fields, assistant entry fields, content block types (text, thinking, tool_use, tool_result), usage object, conversation threading via `parentUuid`, sidechain/agent entry format.
- **Out of Scope**: Storage directory layout and file naming (→ [`../storage/`](../storage/readme.md)); ancillary formats (history.jsonl, todos, shell-snapshots, commands) (→ [`../formats/`](../formats/readme.md)); settings file format (→ [`../settings/`](../settings/readme.md)).

**File location**: `~/.claude/projects/{project-id}/{session-id}.jsonl`

**Format**: One JSON object per line, no trailing commas, valid JSON on each line.

### Overview Table

| ID | Name | Responsibility |
|----|------|----------------|
| [001](001_common_fields.md) | Common Fields | Fields present in all entry types: uuid, parentUuid, timestamp, type, cwd, sessionId, version, gitBranch, userType, isSidechain |
| [002](002_user_entry.md) | User Entry | User message format: message.role, message.content (string), thinkingMetadata |
| [003](003_assistant_entry.md) | Assistant Entry | Assistant message format: message.model, message.id, message.content (array), stop_reason, requestId |
| [004](004_text_block.md) | Text Block | Plain text response content block: `{"type":"text","text":"..."}` |
| [005](005_thinking_block.md) | Thinking Block | Extended thinking/reasoning block: `{"type":"thinking","thinking":"...","signature":"..."}` |
| [006](006_tool_use_block.md) | Tool Use Block | Tool invocation block: `{"type":"tool_use","id":"...","name":"...","input":{}}` |
| [007](007_tool_result_block.md) | Tool Result Block | Tool execution result block: `{"type":"tool_result","tool_use_id":"...","content":"...","is_error":false}` |
| [008](008_usage_object.md) | Usage Object | Token usage tracking: input_tokens, cache_creation_input_tokens, cache_read_input_tokens, output_tokens, service_tier |
| [009](009_threading_model.md) | Threading Model | Conversation threading via parentUuid: singly-linked chain from null root to final turn |
| [010](010_sidechain_sessions.md) | Sidechain Sessions | Agent/sub-agent entry format: isSidechain, agentId, slug; flat and hierarchical storage layouts |

### Entry Type Summary

| Type | `type` field | `message.content` | Key additional fields |
|------|-------------|-------------------|-----------------------|
| User | `"user"` | string (plain text) | `thinkingMetadata` |
| Assistant | `"assistant"` | array of content blocks | `requestId`, `message.model`, `message.usage` |

### Type-Specific Requirements

All `jsonl` doc instances must include:

1. **Title**: `# JSONL: {Concept Name}` — using `JSONL` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Schema** (H3): Field table or JSON example illustrating the format concept
4. **Notes** (H3, optional): Parsing considerations, version history, known exceptions
5. **Cross-References** (H3): Flat table with `Type | File | Responsibility` columns

### Parsing Considerations

JSONL files should be parsed line-by-line:
- **Malformed JSON**: Skip line, log warning, continue
- **Missing required fields**: Error for that entry
- **Unknown fields**: Ignore (forward compatibility)
- **Empty lines**: Skip

For large sessions: stream line-by-line instead of loading entire file.

### Cross-Collection Dependencies

**This entity depends on**:
- `../storage/` — file location and naming conventions for session JSONL files
- `../behavior/` — behaviors B10, B12, B15, B17 reference JSONL field semantics

**This entity consumed by**:
- `../../../../module/claude_storage/src/` — parser implementation
- `../../../../module/claude_storage/docs/` — storage implementation docs
- `../behavior/` — evidence E9 references threading model
