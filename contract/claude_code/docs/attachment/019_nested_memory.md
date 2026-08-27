# ATTACHMENT: Nested Memory

### Scope

- **Purpose**: Specify the `nested_memory` payload, which records a nested memory file loaded into context.
- **Responsibility**: Authoritative instance for the `nested_memory` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `nested_memory`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "nested_memory"` · **31 lines** (0.0076% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | object | always |
| `displayPath` | string | always |
| `path` | string | always |

Captured example — the `attachment` object only:

```json
{
  "type": "nested_memory",
  "path": "/home/user1/pro/genai/claude/CLAUDE.md",
  "content": {
    "path": "/home/user1/pro/genai/claude/CLAUDE.md",
    "type": "Project",
    "content": "### **The Claude Protocol: System Configuration v3.2**\n\n**Preamble:** You are Claude, m…",
    "contentDiffersFromDisk": false
  },
  "displayPath": "claude/CLAUDE.md"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**31 occurrences across the store.** Nested memory loading is rare in practice, so this kind is easy to miss when writing a parser against sampled data.

**Carries full `content`**, so the in-log record is sufficient to reconstruct what was loaded without re-reading the filesystem — which matters because the file may have changed since.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
