# SYSTEM EVENT: API Error

### Scope

- **Purpose**: Specify the `api_error` subtype, the only in-log record of API failure and retry behavior.
- **Responsibility**: Authoritative instance for the `api_error` system subtype: its fields, presence rates, and severity semantics.
- **In Scope**: The `subtype` value `api_error`, every subtype-specific field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `system` envelope carrying this subtype (→ [`../envelope/009_system.md`](../envelope/009_system.md)); other subtypes (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `type == "system"` and `subtype == "api_error"` · **1,271 lines** (2.81% of all system events)

The line is a Class A envelope carrying all nine common fields, plus `subtype` and the fields below.

| Field | Type | Presence |
|-------|------|-----------|
| `error` | object | always |
| `level` | string | always |
| `maxRetries` | number | always |
| `retryAttempt` | number | always |
| `retryInMs` | number | always |
| `source` | string | 1,168 of 1,271 (91.9%) |
| `cause` | object | 2 of 1,271 (0.16%) |

Captured example — full line:

```json
{
  "parentUuid": "b6e7e3a6-1f09-45d0-ae34-df86d8d3b46c",
  "isSidechain": false,
  "type": "system",
  "subtype": "api_error",
  "level": "error",
  "error": {
    "message": "429 {\"type\":\"error\",\"error\":{\"type\":\"rate_limit_error\",\"message\":\"Rate limited\"},\"reque…",
    "status": 429,
    "requestId": "req_011Cdvz2imkzrcjcHhLAdv6e",
    "formatted": "429 Rate limited",
    "connection": null,
    "isNetworkDown": false,
    "rateLimits": null
  },
  "retryInMs": 592,
  "retryAttempt": 1,
  "maxRetries": 10
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Retry accounting is self-contained.** `retryAttempt`, `maxRetries`, and `retryInMs` are all universal on this subtype, so backoff behavior is fully reconstructable from the log with no external telemetry.

**`isMeta` is absent on every one of these lines.** This subtype and `compact_boundary` are the only two where it is missing, which is why the field cannot be assumed present on `system` lines generally.

**`source` is not universal** — 1,168 of 1,271 lines carry it — and `cause` appears on exactly 2.

**API failure is routine, not exceptional.** At 1,271 occurrences it is the fifth most common system event; any consumer computing session success rates must account for it.

### Since

Observed across the full 2.0.56 – 2.1.220 store range for the `system` envelope as a whole, so the floor is a sampling artifact rather than an introduction claim. Per-subtype introduction was not separately attributed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| system_event | [readme.md](readme.md) | System event master index and evidence base |
| envelope | [`../envelope/009_system.md`](../envelope/009_system.md) | The envelope carrying this subtype |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this line satisfies |
