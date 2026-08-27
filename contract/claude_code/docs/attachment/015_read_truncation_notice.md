# ATTACHMENT: Read Truncation Notice

### Scope

- **Purpose**: Specify the `read_truncation_notice` payload, which marks a tool result cut short on its way into context.
- **Responsibility**: Authoritative instance for the `read_truncation_notice` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `read_truncation_notice`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "read_truncation_notice"` · **1,678 lines** (0.412% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `banner` | string | always |
| `toolUseID` | string | always |

Captured example — the `attachment` object only:

```json
{
  "type": "read_truncation_notice",
  "banner": "[Truncated: PARTIAL view — /home/user1/pro/genai/governance/principles_general.rulebook…",
  "toolUseID": "toolu_0196WMdq5jeFuT9y2VBtrBJJ"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`toolUseID` identifies which result was truncated**, so the loss is attributable to a specific call rather than to the turn generally.

**A silent-truncation detector.** Any consumer reasoning about what the model actually saw must account for these 1,678 lines, or it will assume the model read content it never received.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
