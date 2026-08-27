# ATTACHMENT: Plan File Reference

### Scope

- **Purpose**: Specify the `plan_file_reference` payload, which injects a plan file's content.
- **Responsibility**: Authoritative instance for the `plan_file_reference` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `plan_file_reference`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "plan_file_reference"` · **1,352 lines** (0.332% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `planContent` | string | always |
| `planFilePath` | string | always |

Captured example — the `attachment` object only:

```json
{
  "type": "plan_file_reference",
  "planFilePath": "/home/user1/.claude/plans/calm-hatching-brooks.md",
  "planContent": "# Add a \"Weekly Reset\" current-state table to reset_schedule.md\n\n## Context\n\n`data/rese…"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Both fields are universal** — unlike `compact_file_reference`, this kind carries content (`planContent`) alongside the path.

**The most common of the four plan-related payload kinds** by two orders of magnitude.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
