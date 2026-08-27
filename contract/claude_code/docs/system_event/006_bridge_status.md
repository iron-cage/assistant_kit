# SYSTEM EVENT: Bridge Status

### Scope

- **Purpose**: Specify the `bridge_status` subtype, which records the state of a remote-control bridge connection.
- **Responsibility**: Authoritative instance for the `bridge_status` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `bridge_status`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "bridge_status"` · **86 lines** (0.190% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | string | always |
| `isMeta` | boolean | always |
| `url` | string | always |

Captured example — full line:

```json
{
  "parentUuid": "46292ed4-67d9-4657-a549-a353d81012fa",
  "isSidechain": false,
  "type": "system",
  "subtype": "bridge_status",
  "content": "/remote-control is active. Code in CLI or at https://claude.ai/code/session_0112Kn9ox2s…",
  "url": "https://claude.ai/code/session_0112Kn9ox2sSVUVVxAaPhCZZ",
  "isMeta": false,
  "timestamp": "2026-03-17T21:18:24.890Z",
  "uuid": "33767bd3-5c95-4659-b019-72e6df03f0e1"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All three fields are universal** — `content`, `url`, and `isMeta`.

**Absent entirely from stores that never use remote control**, so its absence carries no information about a Claude Code version's capabilities.

**Carries no `level`** despite reporting connection state, so severity must be inferred from `content`.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
