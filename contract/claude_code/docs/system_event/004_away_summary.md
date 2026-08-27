# SYSTEM EVENT: Away Summary

### Scope

- **Purpose**: Specify the `away_summary` subtype, which records a summary produced during user absence.
- **Responsibility**: Authoritative instance for the `away_summary` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `away_summary`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "away_summary"` · **1,801 lines** (3.98% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `content` | string | always |
| `isMeta` | boolean | always |

Captured example — full line:

```json
{
  "parentUuid": "17e60ce2-60a6-4f0f-8934-d9b7ae234c91",
  "isSidechain": false,
  "type": "system",
  "subtype": "away_summary",
  "content": "We root-caused your sync failure: it was a lost git push race against another auto-sync…",
  "timestamp": "2026-07-18T05:45:34.091Z",
  "uuid": "61ea3906-e315-4f5b-9e15-f88ac9b19382",
  "isMeta": false,
  "userType": "external"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Two universal fields, `content` and `isMeta`; no `level`.**

**Distinct from the `summary` envelope**, which is a Class C thread summary anchored by `leafUuid`. This one is a Class A system event and carries full common fields.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
