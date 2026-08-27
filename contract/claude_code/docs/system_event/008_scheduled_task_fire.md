# SYSTEM EVENT: Scheduled Task Fire

### Scope

- **Purpose**: Specify the `scheduled_task_fire` subtype, which marks a cron-style scheduled task firing.
- **Responsibility**: Authoritative instance for the `scheduled_task_fire` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `scheduled_task_fire`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "scheduled_task_fire"` · **32 lines** (0.071% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | string | always |
| `isMeta` | boolean | always |

Captured example — full line:

```json
{
  "parentUuid": "c580c6e0-9973-4ef9-bc31-d4617e2173b3",
  "isSidechain": false,
  "type": "system",
  "subtype": "scheduled_task_fire",
  "content": "Claude resuming /loop wakeup (Aug 13 2:36pm)",
  "isMeta": false,
  "timestamp": "2026-08-13T11:36:30.612Z",
  "uuid": "2d45905e-d86b-47aa-adfd-0517b0d19b6f",
  "userType": "external"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Two universal fields, `content` and `isMeta`; no `level`.**

**Absent entirely from stores that use no scheduling**, so like `bridge_status` its absence is uninformative about capability.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
