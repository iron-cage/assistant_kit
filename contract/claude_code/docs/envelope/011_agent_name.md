# ENVELOPE: Agent Name

### Scope

- **Purpose**: Specify the `agent-name` envelope, which records the display name assigned to a subagent.
- **Responsibility**: Authoritative instance for the `agent-name` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `agent-name` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "agent-name"` · **Class B** (Session-Scoped) · **22,415 lines** (0.444% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `agentName` | string | always |

The nine common fields are omitted from the table; Class B membership fixes which of them are present — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md).

Captured example:

```json
{
  "type": "agent-name",
  "agentName": "add-weekly-reset-table",
  "sessionId": "7c056fde-e40e-49c3-b644-23bc334b96bf"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Session-scoped but not agent-scoped.** The line carries `sessionId` and a name, with no `agentId` — correlating a name to a specific subagent requires matching against `agentId`-bearing kinds by position in the file.

**Distinct from the agent slug.** The slug is a filesystem-safe identifier documented in [`../behavior/015_b15_agent_slug.md`](../behavior/015_b15_agent_slug.md); this is the human-facing label.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) | Class B field contract this kind satisfies |
| behavior | [`../behavior/015_b15_agent_slug.md`](../behavior/015_b15_agent_slug.md) | Agent slug — the filesystem identifier, distinct from this name |
| behavior | [`../behavior/012_b12_agent_session_id.md`](../behavior/012_b12_agent_session_id.md) | Agent session identity |
