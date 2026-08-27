# ATTACHMENT: Deferred Tools Delta

### Scope

- **Purpose**: Specify the `deferred_tools_delta` payload, which records tool-roster changes rather than the roster itself.
- **Responsibility**: Authoritative instance for the `deferred_tools_delta` attachment payload: its fields, presence rates, and what it contributes to reconstructing session context.
- **In Scope**: The `attachment.type` value `deferred_tools_delta`, every payload field observed on it with type and presence rate, and its observed frequency.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `attachment.type == "deferred_tools_delta"` · **38,133 lines** (9.36% of all attachments)

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `addedLines` | array | always |
| `addedNames` | array | always |
| `readdedNames` | array | always |
| `removedNames` | array | always |
| `pendingMcpServers` | array | 6,093 of 38,133 (16.0%) |
| `needsAuthMcpServers` | array | 5,770 of 38,133 (15.1%) |

Captured example — the `attachment` object only:

```json
{
  "type": "deferred_tools_delta",
  "addedNames": [
    "CronCreate",
    "CronDelete",
    "CronList",
    "…"
  ],
  "addedLines": [
    "CronCreate",
    "CronDelete",
    "CronList",
    "…"
  ],
  "removedNames": [],
  "readdedNames": [],
  "pendingMcpServers": [],
  "needsAuthMcpServers": []
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Four universal fields, two optional.** `addedNames`, `removedNames`, `readdedNames`, and `addedLines` always appear; `pendingMcpServers` and `needsAuthMcpServers` appear on roughly one line in six.

**`readdedNames` is distinct from `addedNames`.** A tool returning to the roster after removal is recorded separately from a first-time addition — a consumer folding only `addedNames` will lose re-additions.

**This kind has no `isInitial` field.** Unlike `skill_listing` and `agent_listing_delta`, there is no marker for a first full roster, so reconstruction must fold from the start of the session.

### Since

2.1.197 – 2.1.220 (2 distinct versions) — the window in which the `attachment` envelope exists at all. Per-payload introduction within that window was not separately attributed. See [`../envelope/003_attachment.md`](../envelope/003_attachment.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
