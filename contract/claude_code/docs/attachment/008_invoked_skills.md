# ATTACHMENT: Invoked Skills

### Scope

- **Purpose**: Specify the `invoked_skills` payload, which records which skills ran and embeds their complete text.
- **Responsibility**: Authoritative instance for the `invoked_skills` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `invoked_skills`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "invoked_skills"` · **15,719 lines** (3.86% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `skills` | array | always |

Captured example — the `attachment` object only:

```json
{
  "type": "invoked_skills",
  "skills": [
    {
      "…": "…"
    }
  ]
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Bodies are embedded, not referenced.** Each element of `skills` carries `name`, `path`, and the skill's full `content`. These are among the largest lines in the log.

**This is the dominant cost of reading skill-heavy sessions.** A consumer that needs only the invocation record should read `name` and `path` and discard `content` rather than materializing every body.

**Distinct from `skill_listing`**, which records availability; this records use.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
