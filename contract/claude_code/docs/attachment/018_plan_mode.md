# ATTACHMENT: Plan Mode

### Scope

- **Purpose**: Specify the `plan_mode` payload, which records a plan-mode reminder injected into a turn.
- **Responsibility**: Authoritative instance for the `plan_mode` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `plan_mode`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "plan_mode"` · **53 lines** (0.013% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `isSubAgent` | boolean | always |
| `planExists` | boolean | always |
| `planFilePath` | string | always |
| `reminderType` | string | always |

Captured example — the `attachment` object only:

```json
{
  "type": "plan_mode",
  "reminderType": "full",
  "isSubAgent": false,
  "planFilePath": "/home/user1/.claude/plans/calm-hatching-brooks.md",
  "planExists": false
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All four fields are universal**, including `isSubAgent` — plan-mode reminders are emitted inside subagents as well as the main thread.

**`planExists` is a boolean, `planFilePath` a path.** A reminder can name a path for a plan that does not yet exist.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
