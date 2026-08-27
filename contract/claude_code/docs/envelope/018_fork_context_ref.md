# ENVELOPE: Fork Context Ref

### Scope

- **Purpose**: Specify the `fork-context-ref` envelope, which links a forked session to the session and entry it was forked from.
- **Responsibility**: Authoritative instance for the `fork-context-ref` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `fork-context-ref` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "fork-context-ref"` · **Class C** (Detached) · **104 lines** (0.0021% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `agentId` *(ambient)* | string | always |
| `contextLength` | number | always |
| `parentLastUuid` | string | always |
| `parentSessionId` | string | always |

Fields marked *(ambient)* are Class A envelope decorations that appear across several kinds rather than payload specific to this one. The nine common fields are omitted from the table; Class C membership fixes which of them are present — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md).

Captured example:

```json
{
  "type": "fork-context-ref",
  "agentId": "a49d14f0df964da24",
  "parentSessionId": "0ef25586-db1d-4c25-af0e-84d637b98148",
  "parentLastUuid": "7664327c-ef4b-47f1-844b-51bc99972b7c",
  "contextLength": 204
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All four fields are universal** — `agentId`, `parentSessionId`, `parentLastUuid`, and `contextLength` always appear together.

**The only cross-session link in the log.** [`../behavior/018_b18_no_cross_session_links.md`](../behavior/018_b18_no_cross_session_links.md) establishes that entries do not reference other sessions; this envelope is the documented exception, operating at session rather than entry granularity.

**`contextLength` records the inherited context size** at fork time, which is what lets a consumer distinguish a fork from a fresh session with a coincidentally similar prefix.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md) | Class C field contract this kind satisfies |
| behavior | [`../behavior/021_b21_fork_session.md`](../behavior/021_b21_fork_session.md) | Fork behavior producing this record |
| behavior | [`../behavior/018_b18_no_cross_session_links.md`](../behavior/018_b18_no_cross_session_links.md) | No-cross-session-links rule this kind excepts |
