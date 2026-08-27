# ATTACHMENT: Total Tokens Reminder

### Scope

- **Purpose**: Specify the `total_tokens_reminder` payload, which records the context budget remaining at a point in the session.
- **Responsibility**: Authoritative instance for the `total_tokens_reminder` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `total_tokens_reminder`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "total_tokens_reminder"` · **118,623 lines** (29.12% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `text` | string | always |

Captured example — the `attachment` object only:

```json
{
  "type": "total_tokens_reminder",
  "text": "<total_tokens>14837330 tokens left</total_tokens>"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**The value is embedded in prose, not a number field.** `text` carries the literal string `<total_tokens>N tokens left</total_tokens>`; extracting the budget means parsing that string.

**The most common payload kind in the store**, at 29.1% of all attachments. Budget reminders are injected routinely, not only near exhaustion.

**This is the only in-log record of context budget.** A consumer reconstructing how close a session came to its limit has no other source short of re-tokenizing the transcript.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
