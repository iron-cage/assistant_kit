# ATTACHMENT: Ultrathink Effort

### Scope

- **Purpose**: Specify the `ultrathink_effort` payload, whose presence is its entire signal.
- **Responsibility**: Authoritative instance for the `ultrathink_effort` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `ultrathink_effort`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "ultrathink_effort"` · **14,322 lines** (3.52% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

_None. The payload object carries only its `type` discriminator._

Captured example — the `attachment` object only:

```json
{
  "type": "ultrathink_effort"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**The payload object carries only `type`.** There are no other fields on any of its 14,322 occurrences.

**A parser requiring at least one field beyond the discriminator will reject it.** This is the concrete case that makes 'empty payload' a required capability rather than a defensive nicety.

**Correlates with the `effort` field on `assistant` turns**, which records the effort level itself; this payload records only that elevation occurred.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
