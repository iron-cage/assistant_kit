# SYSTEM EVENT: Agents Killed

### Scope

- **Purpose**: Specify the `agents_killed` subtype, the rarest system event in the store.
- **Responsibility**: Authoritative instance for the `agents_killed` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `agents_killed`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "agents_killed"` · **1 lines** (0.0022% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `isMeta` | boolean | always |

Captured example — full line:

```json
{
  "parentUuid": "4f5ec606-6b8f-4850-8e47-581bde5b7786",
  "isSidechain": false,
  "type": "system",
  "subtype": "agents_killed",
  "timestamp": "2026-07-05T14:21:57.089Z",
  "uuid": "06e7d727-4c07-4cc6-9f1a-dbeceaed5026",
  "isMeta": false,
  "userType": "external",
  "entrypoint": "cli"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**A single occurrence across 5,049,738 lines.** Its field set is documented from that one line and is provisional — a second observation could reveal subtype-specific fields absent from this one.

**Carries only `isMeta` beyond the common fields** — no `content`, no `level`. The event is the entire signal.

**Names no agents.** Despite the plural, the line carries no `agentId` or count, so which subagents were terminated is not recoverable from it.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
