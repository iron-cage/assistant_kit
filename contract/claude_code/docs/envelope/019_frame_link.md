# ENVELOPE: Frame Link

### Scope

- **Purpose**: Specify the `frame-link` envelope, the rarest kind in the store.
- **Responsibility**: Authoritative instance for the `frame-link` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `frame-link` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "frame-link"` · **Class B** (Session-Scoped) · **6 lines** (0.0001% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `frameUrl` | string | always |
| `path` | string | always |

The nine common fields are omitted from the table; Class B membership fixes which of them are present — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md).

Captured example:

```json
{
  "type": "frame-link",
  "sessionId": "77278f1b-e1b3-4c77-8288-e5f631d652fa",
  "path": "/tmp/claude-1001/-home-user1-pro-genai-dev/77278f1b-e1b3-4c77-8288-e5f631d652fa/scratch…",
  "frameUrl": "https://claude.ai/code/artifact/20a1f49c-5e43-4e3d-b24f-811aba3ecfba",
  "timestamp": "2026-07-19T01:08:39.065Z"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Six occurrences across the entire store.** Its field set is documented from those six lines and must be treated as provisional — a seventh observation could reveal optional fields absent from all six.

**One of three Class B kinds carrying `timestamp`.**

**A consumer will almost never see this kind**, which makes it exactly the kind that breaks a parser written against observed data. Handle unknown and near-unknown kinds by skipping, never by erroring.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) | Class B field contract this kind satisfies |
