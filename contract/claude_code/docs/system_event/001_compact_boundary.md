# SYSTEM EVENT: Compact Boundary

### Scope

- **Purpose**: Specify the `compact_boundary` subtype, which marks a compaction and repairs the threading chain across it.
- **Responsibility**: Authoritative instance for the `compact_boundary` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `compact_boundary`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "compact_boundary"` · **18,282 lines** (40.45% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `compactMetadata` | object | always |
| `content` | string | always |
| `level` | string | always |
| `logicalParentUuid` | string | always |
| `isMeta` | boolean | 16,228 of 18,282 (88.8%) |

Captured example — full line:

```json
{
  "parentUuid": null,
  "logicalParentUuid": "4cfbe808-7cc5-430d-a084-7afdf82346d3",
  "isSidechain": false,
  "type": "system",
  "subtype": "compact_boundary",
  "content": "Conversation compacted",
  "isMeta": false,
  "timestamp": "2026-07-30T03:07:55.938Z",
  "uuid": "b07daf52-fbda-42c9-8209-84483060c016"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`logicalParentUuid` is the thread-repair mechanism.** The raw `parentUuid` chain breaks at a compaction gap; this field points past it. A consumer walking the thread without consulting it will see every compacted session as several disconnected fragments.

**`isMeta` is not universal on this subtype.** It is absent on 2,054 of these lines — the only subtype other than `api_error` where it is missing, and the reason `isMeta` cannot be treated as a system-wide field.

**The most common system event**, and the one whose frequency scales directly with long-session usage.

**`compactMetadata` carries the compaction's own accounting** and is universal on this subtype.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
