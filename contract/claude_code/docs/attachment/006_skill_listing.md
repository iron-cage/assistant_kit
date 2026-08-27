# ATTACHMENT: Skill Listing

### Scope

- **Purpose**: Specify the `skill_listing` payload, which records the catalog of skills available to a turn.
- **Responsibility**: Authoritative instance for the `skill_listing` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `skill_listing`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "skill_listing"` · **24,098 lines** (5.92% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | string | always |
| `isInitial` | boolean | always |
| `names` | array | always |
| `skillCount` | number | always |

Captured example — the `attachment` object only:

```json
{
  "type": "skill_listing",
  "content": "- firecrawl\n- firecrawl-agent\n- firecrawl-browser\n- firecrawl-crawl\n- firecrawl-downloa…",
  "skillCount": 123,
  "isInitial": true,
  "names": [
    "firecrawl",
    "firecrawl-agent",
    "firecrawl-browser",
    "…"
  ]
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`isInitial` is universal on this kind**, which makes reconstruction tractable: fold forward from the most recent `isInitial: true` rather than from the start of the session.

**`names` and `content` are both present**, so a consumer can read the roster without parsing the rendered catalog text.

**`skillCount` validates the parse** — it always accompanies `names`.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
