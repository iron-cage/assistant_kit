# ENVELOPE: Queue Operation

### Scope

- **Purpose**: Specify the `queue-operation` envelope, which records the prompt queue's activity during an in-flight turn.
- **Responsibility**: Authoritative instance for the `queue-operation` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `queue-operation` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "queue-operation"` · **Class B** (Session-Scoped) · **76,222 lines** (1.51% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `operation` | string | always |
| `content` | string | 41,662 of 76,222 (54.7%) |

The nine common fields are omitted from the table; Class B membership fixes which of them are present — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md).

Captured example:

```json
{
  "type": "queue-operation",
  "operation": "enqueue",
  "timestamp": "2026-08-04T20:01:56.708Z",
  "sessionId": "2bb4b6c2-0b05-405f-9355-9f29517b09b8",
  "content": "Generate a conventional commit message for the staged changes.\n\nSTAGED CHANGES SUMMARY:…"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**`content` is optional.** Just over half of these lines carry it — the operations that do not are queue state changes with no associated text.

**One of three Class B kinds carrying `timestamp`**, which makes queue activity orderable against Class A entries by time even though it carries no `uuid` or thread link.

**Named in the storage invariant as a skipped type.** It is one of the four non-conversation types [`003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) enumerates — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) for a correction to that document's `uuid` claim.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) | Class B field contract this kind satisfies |
| attachment | [`../attachment/010_queued_command.md`](../attachment/010_queued_command.md) | The queued command's own attachment payload |
| invariant | [`../../../../module/claude_storage/docs/invariant/003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) | Skip-handling contract naming this type |
