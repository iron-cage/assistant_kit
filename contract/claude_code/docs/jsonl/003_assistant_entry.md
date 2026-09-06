# JSONL: Assistant Entry

### Scope

- **Purpose**: Specify the assistant message entry format including the message envelope, content array, and API response metadata.
- **Responsibility**: Authoritative instance for assistant JSONL entries — fields specific to `type: "assistant"` entries.
- **In Scope**: `message.model`, `message.id`, `message.type`, `message.role`, `message.content` (array), `message.stop_reason`, `message.stop_sequence`, `message.usage`, `message.stop_details`, `message.diagnostics`, `requestId`, `effort`, the `attribution*` field family, the rare API-error field family.
- **Out of Scope**: Common fields (→ [001_common_fields.md](001_common_fields.md)); content block types within the array (→ [004–007](004_text_block.md)); usage object (→ [008_usage_object.md](008_usage_object.md)).

### Schema

```json
{
  "parentUuid": "feed0003-0000-4000-8000-000000000003",
  "isSidechain": false,
  "userType": "external",
  "cwd": "/home/alice/pro",
  "sessionId": "feed0001-0000-4000-8000-000000000001",
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
    "usage": { "input_tokens": 9, "output_tokens": 6, "service_tier": "standard" },
    "stop_details": null,
    "diagnostics": {}
  },
  "requestId": "req_011CUwHuh7iPfwQNAXEeEYrP",
  "type": "assistant",
  "effort": "high",
  "attributionSkill": "commit",
  "uuid": "feed0004-0000-4000-8000-000000000004",
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
| `message.stop_details` | object \| null | ❌ | Present on ~99.75% of entries since v2.1.74; internal shape not yet catalogued |
| `message.diagnostics` | object | ❌ | Present on ~95.1% of entries since v2.1.197; internal shape not yet catalogued |
| `message.context_management` | object | ❌ | Rare (0.55%); spans the full observed version range. Related to long-context handling — internal shape not yet catalogued |
| `message.container` | object | ❌ | Rare (0.46%); spans the full observed version range. Present alongside `isApiErrorMessage`/sandboxed-tool-use entries in the sample — internal shape not yet catalogued |
| `requestId` | string | ✅ | API request ID (e.g., `"req_011..."`) |
| `effort` | string | ❌ | Reasoning-effort level (e.g., `"high"`). Observed **only** at v2.1.220 in the local store — the newest and most recently introduced field in this collection |
| `attributionSkill` | string | ❌ | Name of the skill this response is attributed to, when invoked via one. First observed v2.1.197 |
| `attributionAgent` | string | ❌ | Name of the subagent this response is attributed to, when invoked via one. First observed v2.1.197 |
| `attributionPlugin` | string | ❌ | Name of the plugin this response is attributed to. Rare (0.13%); first observed v2.1.197 |
| `attributionMcpServer` | string | ❌ | Name of the MCP server this response is attributed to. Extremely rare (0.001%); observed only at v2.1.220 |
| `attributionMcpTool` | string | ❌ | Name of the MCP tool this response is attributed to. Extremely rare (0.001%); observed only at v2.1.220 |
| `isApiErrorMessage` | boolean | ❌ | Marks this entry as representing an API error rather than a normal turn. Rare (0.46%) |
| `error` | string \| object | ❌ | Error payload, present when `isApiErrorMessage` is set. Rare (0.22%) |
| `apiErrorStatus` | number | ❌ | HTTP-style status code for the API error. Rare (0.16%) |
| `apiErrorIsTransient` | boolean | ❌ | Whether the API error is expected to be transient/retryable. Very rare (0.008%) |
| `errorDetails` | object | ❌ | Additional structured error detail. Very rare (0.005%) |

### Notes

**`message.content` is an array**: Contrast with user entries, where `message.content` is usually (94.3%) also an array but can be a plain string for genuine human-typed text — see [002_user_entry.md](002_user_entry.md) Notes for the corrected, evidence-based breakdown. Assistant `message.content` may contain multiple blocks in one response (thinking + text + tool_use is common).

**Attribution field family**: `attributionSkill`/`attributionAgent`/`attributionPlugin`/`attributionMcpServer`/`attributionMcpTool` are mutually-exclusive-in-practice provenance tags — at most one was observed set per entry in the sampled data, though no doc instance confirms this is an enforced invariant rather than a sampling coincidence.

**API-error field family**: `isApiErrorMessage`, `error`, `apiErrorStatus`, `apiErrorIsTransient`, `errorDetails` are all rare (<0.5%) and were not previously documented anywhere in this collection. Whether these overlap with the `fault/` collection's CLI-level error taxonomy is unconfirmed — `fault/` documents `clr`-observable failures (terminal errors, silent failure modes, behavioral quirks), while these fields are JSONL-entry-level API response metadata; the relationship has not been investigated.

### Since

pre-v1.0 (unverified) for the original 9 documented fields. The remaining fields were newly catalogued in a 2026-09-06 full local-store scan (17,210 session files, 5,446,921 lines, versions v2.0.56–v2.1.220): `effort`, `attributionMcpServer`, `attributionMcpTool`, and `session_id` (see [001_common_fields.md](001_common_fields.md)) were observed **only** at v2.1.220, the newest version sampled — likely very recent additions, though the true introduction version could be anywhere at or before v2.1.220 since this store does not contain every intermediate release. `attributionSkill`/`attributionAgent`/`attributionPlugin`/`message.diagnostics` first appear at v2.1.197. `message.stop_details` first appears at v2.1.74. `message.context_management` and `message.container` span the full sampled range and predate it.

**Verify yourself**: `grep -c '"effort"' <session>.jsonl` on any session file — nonzero only for sessions recorded on a `version` at or after whatever release actually introduced it; cross-check with `grep -o '"version":"[^"]*"' <session>.jsonl | sort -u` to read that session's own version stamp(s).

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
