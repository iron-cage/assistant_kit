# SYSTEM EVENT: Model Consent Fallback

### Scope

- **Purpose**: Specify the `model_consent_fallback` subtype, which records a model substitution and the user's response to it.
- **Responsibility**: Authoritative instance for the `model_consent_fallback` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `model_consent_fallback`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "model_consent_fallback"` · **40 lines** (0.088% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `choice` | string | always |
| `content` | string | always |
| `fallbackModel` | string | always |
| `isMeta` | boolean | always |
| `level` | string | always |
| `originalModel` | string | always |
| `persistedAsDefault` | boolean | always |

Captured example — full line:

```json
{
  "parentUuid": "12886219-b8e2-4a7f-8b15-fcdd7ac23d21",
  "isSidechain": false,
  "type": "system",
  "subtype": "model_consent_fallback",
  "content": "Switched to Opus 4.8 (1M context) for this session · Fable 5 requires usage credits · /…",
  "level": "warning",
  "choice": "cancelled",
  "originalModel": "claude-fable-5",
  "fallbackModel": "claude-opus-4-8[1m]"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Seven universal fields** — the richest subtype in the collection, recording `originalModel`, `fallbackModel`, `choice`, and `persistedAsDefault` alongside `content`, `level`, and `isMeta`.

**`persistedAsDefault` distinguishes a one-off substitution from a saved preference**, which is what makes this record actionable rather than merely informational.

**The in-log audit trail for which model actually served a turn** when the requested one was unavailable.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
