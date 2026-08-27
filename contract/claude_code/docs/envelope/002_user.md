# ENVELOPE: User

### Scope

- **Purpose**: Specify the `user` envelope and the fact that a majority of its lines are tool results rather than human input.
- **Responsibility**: Authoritative instance for the `user` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `user` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "user"` · **Class A** (Full Envelope) · **1,371,543 lines** (27.16% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `message` | object | always |
| `promptId` | string | 1,357,584 of 1,371,543 (99.0%) |
| `slug` *(ambient)* | string | 1,350,926 of 1,371,543 (98.5%) |
| `entrypoint` *(ambient)* | string | 1,289,969 of 1,371,543 (94.1%) |
| `sourceToolAssistantUUID` | string | 1,272,540 of 1,371,543 (92.8%) |
| `toolUseResult` | object | 827,978 of 1,371,543 (60.4%) |
| `agentId` *(ambient)* | string | 545,251 of 1,371,543 (39.8%) |
| `session_id` *(ambient)* | string | 138,360 of 1,371,543 (10.1%) |
| `permissionMode` | string | 26,280 of 1,371,543 (1.9%) |
| `promptSource` | string | 22,145 of 1,371,543 (1.6%) |
| `isMeta` | boolean | 19,419 of 1,371,543 (1.4%) |
| `isCompactSummary` | boolean | 18,392 of 1,371,543 (1.3%) |
| `isVisibleInTranscriptOnly` | boolean | 18,392 of 1,371,543 (1.3%) |
| `origin` | object | 14,410 of 1,371,543 (1.1%) |
| `sourceToolUseID` | string | 835 of 1,371,543 (0.06%) |
| `toolEndsTurn` | boolean | 277 of 1,371,543 (0.02%) |
| `interruptedByShutdown` | boolean | 172 of 1,371,543 (0.01%) |
| `thinkingMetadata` | object | 147 of 1,371,543 (0.01%) |
| `interruptedMessageId` | string | 108 of 1,371,543 (0.008%) |
| `toolDenialKind` | string | 80 of 1,371,543 (0.006%) |
| `queuePriority` | string | 33 of 1,371,543 (0.002%) |
| `todos` | array | 4 of 1,371,543 (0.000%) |
| `userFeedback` | string | 1 of 1,371,543 (0.000%) |

Fields marked *(ambient)* are Class A envelope decorations that appear across several kinds rather than payload specific to this one. The nine common fields are omitted from the table; Class A membership fixes which of them are present — see [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md).

Captured example:

```json
{
  "parentUuid": null,
  "isSidechain": false,
  "promptId": "c472d49b-289c-40fe-ac9d-d4db7b64b375",
  "type": "user",
  "message": {
    "role": "user",
    "content": "Generate a conventional commit message for the staged changes.\n\nSTAGED CHANGES SUMMARY:…"
  },
  "uuid": "e2ede909-f85c-4589-872c-b48399c55c61",
  "timestamp": "2026-08-04T20:01:56.757Z",
  "permissionMode": "bypassPermissions",
  "promptSource": "sdk"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Most `user` lines are not from the user.** `toolUseResult` is present on 60.4% of them — those lines carry a tool's output, not a typed prompt. A consumer that treats every `user` line as human input overstates user turns by roughly 2.5x.

**`isMeta` marks harness-authored lines**, and `isCompactSummary` marks a compaction summary injected in the user role. Both are user-role lines that no human wrote.

**`isVisibleInTranscriptOnly` marks display-only content** — present on the same population as `isCompactSummary`. These lines exist for transcript rendering and should not be replayed as prompts.

**`promptId` groups a prompt with its retries and continuations**; `promptSource` records where it came from when the source was not an interactive keystroke.

**`toolDenialKind` and `toolEndsTurn` are rare but load-bearing** — the first records a permission denial, the second that a tool result terminated the turn. Both are absent on the overwhelming majority of lines.

### Since

Observed 2.0.56 – 2.1.220 (20 distinct versions) — the full range present in the sampled store, so the floor is a sampling artifact and not a claim about introduction.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this kind satisfies |
| jsonl | [`../jsonl/002_user_entry.md`](../jsonl/002_user_entry.md) | Internal field detail of the user entry |
| jsonl | [`../jsonl/007_tool_result_block.md`](../jsonl/007_tool_result_block.md) | Tool-result block carried in `toolUseResult` |
| behavior | [`../behavior/025_b25_auto_compact_window.md`](../behavior/025_b25_auto_compact_window.md) | Compaction window producing `isCompactSummary` lines |
