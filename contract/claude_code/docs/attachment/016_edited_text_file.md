# ATTACHMENT: Edited Text File

### Scope

- **Purpose**: Specify the `edited_text_file` payload, which records a snippet of a file after it was edited.
- **Responsibility**: Authoritative instance for the `edited_text_file` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `edited_text_file`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "edited_text_file"` · **1,591 lines** (0.391% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `filename` | string | always |
| `snippet` | string | always |
| `displayPath` | string | 9 of 1,591 (0.57%) |

Captured example — the `attachment` object only:

```json
{
  "type": "edited_text_file",
  "filename": "/data/repos/yrd_review/pr_review/3605_obox-systems__2025_jewel_demo__pr34__r32/009_revi…",
  "snippet": "27\t| Entity Path | Change Type | Level |\n28\t|---|---|---|\n29\t| docs/feature/ | Instance…"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`displayPath` is nearly absent.** It appears on 9 lines out of 1,591 — treat it as optional, not as a companion to `filename`.

**Snippet, not full content.** Contrast the `file` payload, which injects whole files.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
