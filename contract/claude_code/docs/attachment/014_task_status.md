# ATTACHMENT: Task Status

### Scope

- **Purpose**: Specify the `task_status` payload, which records a background task changing state.
- **Responsibility**: Authoritative instance for the `task_status` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `task_status`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "task_status"` · **3,613 lines** (0.887% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `deltaSummary` | null | always |
| `description` | string | always |
| `outputFilePath` | string | always |
| `status` | string | always |
| `taskId` | string | always |
| `taskType` | string | always |

Captured example — the `attachment` object only:

```json
{
  "type": "task_status",
  "taskId": "a56f2730c703c6671",
  "taskType": "local_agent",
  "description": "PR 6 review — client fidelity lens",
  "status": "completed",
  "deltaSummary": null,
  "outputFilePath": "/tmp/claude-1001/-data-repos-yrd-review-2026-sybe-dev-pr-6/29bda0bd-a138-4c67-a741-034b…"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All six fields are universal** — `taskId`, `taskType`, `status`, `description`, `deltaSummary`, and `outputFilePath` always appear together, so a transition record is never partial.

**`outputFilePath` is the durable handle.** It is the path a consumer reads to retrieve the task's actual output, which outlives the in-memory task registry.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
