# SYSTEM EVENT: Turn Duration

### Scope

- **Purpose**: Specify the `turn_duration` subtype, the only in-log source of turn timing.
- **Responsibility**: Authoritative instance for the `turn_duration` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `turn_duration`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "turn_duration"` · **8,058 lines** (17.83% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `durationMs` | number | always |
| `isMeta` | boolean | always |
| `messageCount` | number | 6,977 of 8,058 (86.6%) |
| `pendingBackgroundAgentCount` | number | 1,350 of 8,058 (16.8%) |
| `pendingWorkflowCount` | number | 4 of 8,058 (0.05%) |

Captured example — full line:

```json
{
  "parentUuid": "c2324278-0a0a-4259-8b51-9f5cb8f854f3",
  "isSidechain": false,
  "type": "system",
  "subtype": "turn_duration",
  "durationMs": 572391,
  "messageCount": 102,
  "timestamp": "2026-07-18T04:49:20.565Z",
  "uuid": "c7d9acb8-8087-4f93-89d2-aaa17f51b688",
  "isMeta": false
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`messageCount` is not universal.** It appears on 6,977 of 8,058 lines — 86.6%. A consumer averaging messages per turn must handle its absence rather than defaulting to zero.

**`pendingBackgroundAgentCount` and `pendingWorkflowCount` are rare.** The first appears on about one line in six, the second on four lines in the entire store.

**`durationMs` is universal**, so turn timing is always available even when the message count is not.

**No `level` field.** This subtype is telemetry, not a severity-bearing event.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
