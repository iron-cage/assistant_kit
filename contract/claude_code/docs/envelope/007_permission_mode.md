# ENVELOPE: Permission Mode

### Scope

- **Purpose**: Specify the `permission-mode` envelope, which records each transition of the session's tool-permission posture.
- **Responsibility**: Authoritative instance for the `permission-mode` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `permission-mode` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "permission-mode"` · **Class B** (Session-Scoped) · **96,521 lines** (1.91% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `permissionMode` | string | always |

The nine common fields are omitted from the table; Class B membership fixes which of them are present — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md).

Captured example:

```json
{
  "type": "permission-mode",
  "permissionMode": "bypassPermissions",
  "sessionId": "5b1ee6c7-f43c-4f17-a4ea-451a32950e5d"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Security-relevant state, recorded in the clear.** Transitions into and out of a permissive posture are visible in the log, which makes the session file an audit trail for how tool calls were authorized.

**Also appears as a field on `user` lines.** `permissionMode` is present on a small fraction of `user` entries as well; the envelope records the transition, the field records the posture in force for that turn.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) | Class B field contract this kind satisfies |
| envelope | [002_user.md](002_user.md) | `permissionMode` field on user turns |
| tool | [`../tool/readme.md`](../tool/readme.md) | Tool catalog whose invocation this mode gates |
