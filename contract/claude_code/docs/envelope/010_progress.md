# ENVELOPE: Progress

### Scope

- **Purpose**: Specify the `progress` envelope, a retired streaming-progress record.
- **Responsibility**: Authoritative instance for the `progress` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `progress` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "progress"` · **Class A** (Full Envelope) · **41,517 lines** (0.822% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `data` | object | always |
| `parentToolUseID` | string | always |
| `slug` *(ambient)* | string | always |
| `toolUseID` | string | always |
| `agentId` *(ambient)* | string | 19,214 of 41,517 (46.3%) |
| `entrypoint` *(ambient)* | string | 138 of 41,517 (0.33%) |

Fields marked *(ambient)* are Class A envelope decorations that appear across several kinds rather than payload specific to this one. The nine common fields are omitted from the table; Class A membership fixes which of them are present — see [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md).

Captured example:

```json
{
  "parentUuid": "b1d8a682-c849-4b50-b193-e7e9a147ec13",
  "isSidechain": false,
  "type": "progress",
  "data": {
    "type": "hook_progress",
    "hookEvent": "PreToolUse",
    "hookName": "PreToolUse:Bash",
    "command": "/home/user1/.claude/hooks/rtk-rewrite.sh"
  },
  "toolUseID": "toolu_01W4fQX422C4ZRKSrNvdDeth",
  "parentToolUseID": "toolu_01W4fQX422C4ZRKSrNvdDeth",
  "uuid": "7ea7d3f0-7d04-47e1-8b0e-3c444e540092",
  "timestamp": "2026-06-04T17:48:59.162Z",
  "userType": "external"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Retired, not merely absent.** Its observed range both starts and ends strictly inside the store's version span, which makes this a genuine lifecycle signal in both directions: introduced after the oldest data in the store and gone before the newest.

**`toolUseID` and `parentToolUseID` are both universal**, so a progress line always identifies both the tool call it reports on and that call's parent — enough to reconstruct a nesting tree without consulting any other kind.

**A consumer reading only recent sessions will never encounter it.** Historical logs still contain 41,517 of these lines, so an archive-processing consumer must still handle the kind.

### Since

Observed 2.1.50 – 2.1.81 (7 distinct versions). The range starts strictly inside the store's 2.0.56 – 2.1.220 span, which is a genuine introduction signal. The range ends strictly inside that span, which is a genuine retirement signal.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this kind satisfies |
| jsonl | [`../jsonl/006_tool_use_block.md`](../jsonl/006_tool_use_block.md) | Tool-use block referenced by `toolUseID` |
| behavior | [`../behavior/036_b36_background_task_lifecycle.md`](../behavior/036_b36_background_task_lifecycle.md) | Background task lifecycle this kind once reported on |
