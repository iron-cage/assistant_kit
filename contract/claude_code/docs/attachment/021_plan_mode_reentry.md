# ATTACHMENT: Plan Mode Reentry

### Scope

- **Purpose**: Specify the `plan_mode_reentry` payload, which marks a session returning to plan mode.
- **Responsibility**: Authoritative instance for the `plan_mode_reentry` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `plan_mode_reentry`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "plan_mode_reentry"` · **15 lines** (0.0037% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `planFilePath` | string | always |

Captured example — the `attachment` object only:

```json
{
  "type": "plan_mode_reentry",
  "planFilePath": "/home/user1/.claude/plans/calm-hatching-brooks.md"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`planFilePath` is the sole field** — unlike `plan_mode_exit`, there is no `planExists` companion.

**The rarest of the three plan-mode transition kinds.**

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
