# ENVELOPE: System

### Scope

- **Purpose**: Specify the `system` envelope, whose `subtype` field is the third dispatch level of the session log.
- **Responsibility**: Authoritative instance for the `system` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `system` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "system"` · **Class A** (Full Envelope) · **45,201 lines** (0.895% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `subtype` | string | always |
| `slug` *(ambient)* | string | 42,814 of 45,201 (94.7%) |
| `isMeta` | boolean | 41,876 of 45,201 (92.6%) |
| `content` | string | 35,871 of 45,201 (79.4%) |
| `level` | string | 35,223 of 45,201 (77.9%) |
| `entrypoint` *(ambient)* | string | 28,608 of 45,201 (63.3%) |
| `compactMetadata` | object | 18,282 of 45,201 (40.4%) |
| `logicalParentUuid` | string | 18,282 of 45,201 (40.4%) |
| `durationMs` | number | 8,058 of 45,201 (17.8%) |
| `messageCount` | number | 6,977 of 45,201 (15.4%) |
| `agentId` *(ambient)* | string | 5,501 of 45,201 (12.2%) |
| `pendingBackgroundAgentCount` | number | 1,350 of 45,201 (3.0%) |
| `error` | object | 1,271 of 45,201 (2.8%) |
| `maxRetries` | number | 1,271 of 45,201 (2.8%) |
| `retryAttempt` | number | 1,271 of 45,201 (2.8%) |
| `retryInMs` | number | 1,271 of 45,201 (2.8%) |
| `source` | string | 1,168 of 45,201 (2.6%) |
| `url` | string | 86 of 45,201 (0.19%) |
| `choice` | string | 40 of 45,201 (0.09%) |
| `fallbackModel` | string | 40 of 45,201 (0.09%) |
| `originalModel` | string | 40 of 45,201 (0.09%) |
| `persistedAsDefault` | boolean | 40 of 45,201 (0.09%) |
| `session_id` *(ambient)* | string | 8 of 45,201 (0.02%) |
| `pendingWorkflowCount` | number | 4 of 45,201 (0.009%) |
| `cause` | object | 2 of 45,201 (0.004%) |

Fields marked *(ambient)* are Class A envelope decorations that appear across several kinds rather than payload specific to this one. The nine common fields are omitted from the table; Class A membership fixes which of them are present — see [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md).

Captured example:

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

**`subtype` is universal on this kind and only this kind.** Dispatching on the top-level `type` alone under-resolves it 10 ways — see [`../system_event/`](../system_event/readme.md).

**`level` is a severity field but is not universal**, appearing on five of the ten subtypes. Its absence is not a default severity.

**`logicalParentUuid` appears only under `compact_boundary`** and is the mechanism that repairs the `parentUuid` chain across a compaction gap.

**This envelope carries the only error accounting in the log.** The `api_error` subtype records retry attempts, ceilings, and backoff on the line itself.

### Since

Observed 2.0.56 – 2.1.220 (20 distinct versions) — the full range present in the sampled store, so the floor is a sampling artifact and not a claim about introduction.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this kind satisfies |
| system_event | [`../system_event/readme.md`](../system_event/readme.md) | All 10 subtypes carried by this envelope |
| behavior | [`../behavior/017_b17_parentuuid_self_contained.md`](../behavior/017_b17_parentuuid_self_contained.md) | Self-containment rule and its compaction exception |
