# ATTACHMENT: File

### Scope

- **Purpose**: Specify the `file` payload, which records a file's content as injected into a turn.
- **Responsibility**: Authoritative instance for the `file` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `file`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "file"` · **32,801 lines** (8.05% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | object | always |
| `displayPath` | string | always |
| `filename` | string | always |
| `truncated` | boolean | 2 of 32,801 (0.006%) |

Captured example — the `attachment` object only:

```json
{
  "type": "file",
  "filename": "/data/repos/yrd_review/2025_anthony_leasefi_backend/pr_22/readme.md",
  "content": {
    "type": "text",
    "file": {
      "…": "…"
    }
  },
  "displayPath": "readme.md"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`truncated` is vanishingly rare but load-bearing.** It appears on 2 lines out of 32,801. When present, `content` is not the whole file, and any consumer treating the payload as authoritative file state will be wrong for that line.

**Distinct from `edited_text_file` and `nested_memory`**, which carry a post-edit snippet and a discovered `CLAUDE.md` respectively. All three carry file content; only this one represents a plain read injection.

**`displayPath` and `filename` differ.** The former is the path as shown to the user, the latter as resolved on disk.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
