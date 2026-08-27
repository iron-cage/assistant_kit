# SYSTEM EVENT: Local Command

### Scope

- **Purpose**: Specify the `local_command` subtype, which records execution of a command handled locally rather than by the model.
- **Responsibility**: Authoritative instance for the `local_command` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `local_command`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "local_command"` · **15,599 lines** (34.51% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | string | always |
| `isMeta` | boolean | always |
| `level` | string | always |

Captured example — full line:

```json
{
  "parentUuid": "b01cf4d4-0c2d-4ba5-b2cf-e987c9599bdb",
  "isSidechain": false,
  "type": "system",
  "subtype": "local_command",
  "content": "<local-command-stderr>Error during compaction: You've hit your session limit · resets 4…",
  "level": "info",
  "timestamp": "2026-08-26T10:21:02.827Z",
  "uuid": "a242663e-8dbd-43c4-8b50-05b92216de03",
  "isMeta": false
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All three fields are universal** — `content`, `level`, and `isMeta`, making this the most regular subtype in the collection.

**Records commands like `/compact` and `/model`** that never reach the API. A consumer counting user actions from `user` lines alone will miss all 15,599 of them.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
