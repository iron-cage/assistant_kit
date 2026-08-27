# ENVELOPE: Result

### Scope

- **Purpose**: Specify the `result` envelope, which marks the completion of a cache-keyed subagent invocation and carries its output.
- **Responsibility**: Authoritative instance for the `result` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `result` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "result"` · **Class C** (Detached) · **285 lines** (0.0056% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `agentId` *(ambient)* | string | always |
| `key` | string | always |
| `result` | object | always |

Fields marked *(ambient)* are Class A envelope decorations that appear across several kinds rather than payload specific to this one. The nine common fields are omitted from the table; Class C membership fixes which of them are present — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md).

Captured example:

```json
{
  "type": "result",
  "key": "v2:e4fb869a9e45109d7dd274050147aad5dd7a12374304aef92ddb85bb2fdaa9f3",
  "agentId": "a7c955b166ea04774",
  "result": {
    "candidates": [
      "…"
    ]
  }
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**44 fewer than `started`.** The gap is invocations that began and did not produce a recorded result — a directly measurable subagent failure signal.

**`result` carries the output payload**, which is what distinguishes this kind from `started` structurally; the other two fields are identical.

**Carries no `sessionId`.** Attribution to a session comes only from the containing file.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md) | Class C field contract this kind satisfies |
| envelope | [015_started.md](015_started.md) | The paired initiation record |
| behavior | [`../behavior/027_b27_agent_no_os_process.md`](../behavior/027_b27_agent_no_os_process.md) | Subagents run in-process, not as OS processes |
