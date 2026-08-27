# ENVELOPE: Started

### Scope

- **Purpose**: Specify the `started` envelope, which marks the beginning of a cache-keyed subagent invocation.
- **Responsibility**: Authoritative instance for the `started` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `started` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "started"` · **Class C** (Detached) · **329 lines** (0.0065% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `agentId` *(ambient)* | string | always |
| `key` | string | always |

Fields marked *(ambient)* are Class A envelope decorations that appear across several kinds rather than payload specific to this one. The nine common fields are omitted from the table; Class C membership fixes which of them are present — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md).

Captured example:

```json
{
  "type": "started",
  "key": "v2:bee5577d315d935177121a61370e1a5578848ed50535d3e6e21af752052a4d33",
  "agentId": "a8b09f3ccbc7f1905"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Pairs with `result`.** The two share `key` and `agentId`; a `started` with no matching `result` is an invocation that did not complete.

**`key` is a cache key, not an identifier.** Two invocations with identical inputs share a key, so `key` alone does not uniquely identify an invocation — the pair `(key, agentId)` does.

**Carries no `sessionId`.** Attribution to a session comes only from the containing file.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md) | Class C field contract this kind satisfies |
| envelope | [016_result.md](016_result.md) | The paired completion record |
| behavior | [`../behavior/037_b37_subagent_cache_ttl.md`](../behavior/037_b37_subagent_cache_ttl.md) | Subagent cache isolation and TTL underlying `key` |
