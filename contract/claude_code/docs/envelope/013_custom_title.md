# ENVELOPE: Custom Title

### Scope

- **Purpose**: Specify the `custom-title` envelope, which records a title the user set explicitly.
- **Responsibility**: Authoritative instance for the `custom-title` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `custom-title` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "custom-title"` · **Class B** (Session-Scoped) · **4,276 lines** (0.085% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `customTitle` | string | always |

The nine common fields are omitted from the table; Class B membership fixes which of them are present — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md).

Captured example:

```json
{
  "type": "custom-title",
  "customTitle": "ultrathink",
  "sessionId": "6a62a9ae-99a4-430b-b12c-600fa8b0298c"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Takes precedence over `ai-title`.** Both kinds can appear in one session; the user's title is the one displayed.

**Roughly one per 36 auto-generated titles.** Explicit titling is rare relative to automatic titling, so a consumer must not assume this kind is present.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) | Class B field contract this kind satisfies |
| envelope | [006_ai_title.md](006_ai_title.md) | Auto-generated title this one overrides |
