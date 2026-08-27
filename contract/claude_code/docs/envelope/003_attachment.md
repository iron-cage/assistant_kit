# ENVELOPE: Attachment

### Scope

- **Purpose**: Specify the `attachment` envelope, whose nested `attachment.type` is the second dispatch level of the session log.
- **Responsibility**: Authoritative instance for the `attachment` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `attachment` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "attachment"` · **Class A** (Full Envelope) · **407,370 lines** (8.07% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `attachment` | object | always |
| `entrypoint` *(ambient)* | string | always |
| `slug` *(ambient)* | string | 399,777 of 407,370 (98.1%) |
| `session_id` *(ambient)* | string | 152,964 of 407,370 (37.5%) |
| `agentId` *(ambient)* | string | 33,084 of 407,370 (8.1%) |

Fields marked *(ambient)* are Class A envelope decorations that appear across several kinds rather than payload specific to this one. The nine common fields are omitted from the table; Class A membership fixes which of them are present — see [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md).

Captured example:

```json
{
  "parentUuid": "e2ede909-f85c-4589-872c-b48399c55c61",
  "isSidechain": false,
  "attachment": {
    "type": "deferred_tools_delta",
    "addedNames": [
      "…"
    ],
    "addedLines": [
      "…"
    ],
    "removedNames": [
      "…"
    ],
    "readdedNames": [
      "…"
    ],
    "pendingMcpServers": [
      "…"
    ],
    "needsAuthMcpServers": [
      "…"
    ]
  },
  "type": "attachment",
  "uuid": "26ed881e-1654-40da-bcce-b99f25d6fdbc",
  "timestamp": "2026-08-04T20:01:56.748Z",
  "userType": "external",
  "entrypoint": "sdk-cli",
  "cwd": "/data/repos/yrd_review/-commit"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**The envelope is nearly contentless on its own.** Its entire payload is the nested `attachment` object, which carries its own `type` discriminator. Dispatching on the top-level `type` alone under-resolves this kind 23 ways — see [`../attachment/`](../attachment/readme.md).

**This is the newest envelope in the taxonomy.** It appears only in the two most recent versions present in the store, which is a genuine introduction signal rather than a sampling floor.

**It is the context-reconstruction channel.** Together its payload kinds record what the harness injected into each turn — token budget, tool roster, skill catalog, file contents, task state — without any API call or subprocess.

### Since

Observed 2.1.197 – 2.1.220 (2 distinct versions). The range starts strictly inside the store's 2.0.56 – 2.1.220 span, which is a genuine introduction signal. 

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract this kind satisfies |
| attachment | [`../attachment/readme.md`](../attachment/readme.md) | All 23 payload kinds carried by this envelope |
