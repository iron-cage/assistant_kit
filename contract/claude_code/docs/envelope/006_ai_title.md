# ENVELOPE: AI Title

### Scope

- **Purpose**: Specify the `ai-title` envelope, which records a title Claude Code generated for the conversation.
- **Responsibility**: Authoritative instance for the `ai-title` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `ai-title` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "ai-title"` · **Class B** (Session-Scoped) · **152,720 lines** (3.02% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `aiTitle` | string | always |

The nine common fields are omitted from the table; Class B membership fixes which of them are present — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md).

Captured example:

```json
{
  "type": "ai-title",
  "aiTitle": "Generate conventional commit message",
  "sessionId": "3b47c98c-a978-4a11-9ea7-03c867fbe52f"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Regenerated, not written once.** With over 150,000 lines across the store, titles are re-emitted as conversations evolve. The current title is the last such line in the file, not the first.

**Distinct from `custom-title`.** A user-set title is a separate envelope and does not overwrite these lines; a consumer wanting the displayed title must consider both and prefer the user's.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) | Class B field contract this kind satisfies |
| envelope | [013_custom_title.md](013_custom_title.md) | User-set title, which takes precedence over this one |
