# ATTACHMENT: Compact File Reference

### Scope

- **Purpose**: Specify the `compact_file_reference` payload, which preserves a file citation through a `/compact` operation.
- **Responsibility**: Authoritative instance for the `compact_file_reference` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `compact_file_reference`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "compact_file_reference"` · **39,596 lines** (9.72% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `displayPath` | string | always |
| `filename` | string | always |

Captured example — the `attachment` object only:

```json
{
  "type": "compact_file_reference",
  "filename": "/data/repos/yrd_review/pr_review/3007_obox-systems__2025_anthony_leasefi_backend__pr22_…",
  "displayPath": "../../pr_review/3007_obox-systems__2025_anthony_leasefi_backend__pr22__r2/028_pre_revie…"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Reference, not content.** It carries `filename` and `displayPath` but no file body — the content is not re-injected, only the citation survives.

**Both fields are universal.** Contrast the `file` payload, which carries content and an optional `truncated` marker.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
