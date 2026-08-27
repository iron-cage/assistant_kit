# ENVELOPE: Mode

### Scope

- **Purpose**: Specify the `mode` envelope, which records each transition of the session's operating mode.
- **Responsibility**: Authoritative instance for the `mode` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `mode` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "mode"` · **Class B** (Session-Scoped) · **245,422 lines** (4.86% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `mode` | string | always |

The nine common fields are omitted from the table; Class B membership fixes which of them are present — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md).

Captured example:

```json
{
  "type": "mode",
  "mode": "normal",
  "sessionId": "2028248c-7841-46d3-bd16-c26c3f5c06bc"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**One line per transition, not per turn.** The field carries the mode entered; the mode in force at any point is the value from the most recent preceding line, not a per-turn annotation.

**Distinct from `permission-mode`.** The two are separate envelopes tracking separate state; neither implies the other.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) | Class B field contract this kind satisfies |
| param | [`../param/readme.md`](../param/readme.md) | CLI parameters that set the initial mode |
