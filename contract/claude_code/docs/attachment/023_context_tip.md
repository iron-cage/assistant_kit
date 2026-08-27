# ATTACHMENT: Context Tip

### Scope

- **Purpose**: Specify the `context_tip` payload, the rarest attachment kind in the store.
- **Responsibility**: Authoritative instance for the `context_tip` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `context_tip`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "context_tip"` · **1 lines** (0.0002% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `tip` | object | always |

Captured example — the `attachment` object only:

```json
{
  "type": "context_tip",
  "tip": {
    "tip": "You're running a large multi-agent verification cycle (11 agents across 5 rounds) and t…",
    "featureId": "background-agents-list",
    "action": "claude agents"
  }
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**A single occurrence across 5,049,738 lines.** Its field set — `tip` alone — is documented from that one line and is provisional.

**The extreme case for defensive dispatch.** A parser validated against a large sample can still be the first to meet this kind in production.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
