# ENVELOPE: File History Snapshot

### Scope

- **Purpose**: Specify the `file-history-snapshot` envelope, which records file state so edits can be reverted to a prior message.
- **Responsibility**: Authoritative instance for the `file-history-snapshot` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `file-history-snapshot` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "file-history-snapshot"` · **Class C** (Detached) · **8,016 lines** (0.159% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `isSnapshotUpdate` | boolean | always |
| `messageId` | string | always |
| `snapshot` | object | always |

The nine common fields are omitted from the table; Class C membership fixes which of them are present — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md).

Captured example:

```json
{
  "type": "file-history-snapshot",
  "messageId": "7aae3354-bbc5-4266-945f-bcf15fd5478e",
  "snapshot": {
    "messageId": "7aae3354-bbc5-4266-945f-bcf15fd5478e",
    "trackedFileBackups": {
      "…": "…"
    },
    "timestamp": "2025-12-18T09:39:59.118Z"
  },
  "isSnapshotUpdate": false
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All three payload fields are universal.** `messageId`, `snapshot`, and `isSnapshotUpdate` are present on every line — an unusually rigid contract for a Class C kind.

**`messageId` is the only link to the conversation.** The line carries no `sessionId`; attribution comes either from the file it was found in or from resolving `messageId` against a Class A `uuid`.

**`isSnapshotUpdate` distinguishes a delta from a full capture**, so a consumer reconstructing file state at a message must fold updates rather than reading the newest line.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md) | Class C field contract this kind satisfies |
| tool | [`../tool/readme.md`](../tool/readme.md) | Editing tools whose effects these snapshots capture |
