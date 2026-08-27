# ATTACHMENT: Task Reminder

### Scope

- **Purpose**: Specify the `task_reminder` payload, which records the task list as injected into a turn's context.
- **Responsibility**: Authoritative instance for the `task_reminder` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `task_reminder`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "task_reminder"` · **70,130 lines** (17.22% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | array | always |
| `itemCount` | number | always |

Captured example — the `attachment` object only:

```json
{
  "type": "task_reminder",
  "content": [],
  "itemCount": 0
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`itemCount` is a redundant but authoritative count.** It always accompanies `content`, so a consumer can validate its own parse of the task list against it.

**Snapshot, not delta.** Unlike the `*_delta` payload kinds, each reminder carries the full list — the current state is the newest line, with no folding required.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
