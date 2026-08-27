# ATTACHMENT: Queued Command

### Scope

- **Purpose**: Specify the `queued_command` payload, which records a command entered while a turn was in flight.
- **Responsibility**: Authoritative instance for the `queued_command` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `queued_command`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "queued_command"` · **10,141 lines** (2.49% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `commandMode` | string | always |
| `prompt` | string | always |
| `timestamp` | string | always |
| `origin` | object | 491 of 10,141 (4.8%) |
| `isMeta` | boolean | 1 of 10,141 (0.010%) |

Captured example — the `attachment` object only:

```json
{
  "type": "queued_command",
  "prompt": "<task-notification>\n<task-id>b19w3sdt7</task-id>\n<tool-use-id>toolu_01TWij4q67sCNiMUh6T…",
  "commandMode": "task-notification",
  "timestamp": "2026-07-30T03:52:21.127Z"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Three universal fields, two rare ones.** `prompt`, `commandMode`, and `timestamp` always appear; `origin` appears on about 5% of lines and `isMeta` on exactly one.

**Carries its own `timestamp`** distinct from the envelope's — the queueing moment, not the injection moment.

**Pairs with the `queue-operation` envelope**, which records the queue state change; this records the command's content.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
