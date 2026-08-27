# SYSTEM EVENT: Informational

### Scope

- **Purpose**: Specify the `informational` subtype, a general-purpose notice channel.
- **Responsibility**: Authoritative instance for the `informational` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `informational`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "informational"` · **31 lines** (0.069% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | string | always |
| `isMeta` | boolean | always |
| `level` | string | always |

Captured example — full line:

```json
{
  "parentUuid": "3b8327d7-3d47-4d5d-95aa-d5d11d9f5ae1",
  "isSidechain": false,
  "type": "system",
  "subtype": "informational",
  "content": "Session model kimi-k3 could not be restored (not a model this version of Claude Code re…",
  "isMeta": false,
  "timestamp": "2026-08-20T09:12:02.901Z",
  "uuid": "c4d34482-7213-4a44-9344-03eddf1bcab6",
  "level": "warning"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All three fields are universal** — `content`, `isMeta`, and `level`.

**Carries `level` despite being informational by name**, so the field is present but its value need not be the lowest severity.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
