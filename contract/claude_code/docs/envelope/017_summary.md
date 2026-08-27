# ENVELOPE: Summary

### Scope

- **Purpose**: Specify the `summary` envelope, which records a generated summary anchored to a thread leaf.
- **Responsibility**: Authoritative instance for the `summary` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `summary` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "summary"` · **Class C** (Detached) · **178 lines** (0.0035% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `leafUuid` | string | always |
| `summary` | string | always |

The nine common fields are omitted from the table; Class C membership fixes which of them are present — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md).

Captured example:

```json
{
  "type": "summary",
  "summary": "User Login and Model Configuration Setup",
  "leafUuid": "ea7ee9f7-6237-4d78-ad3b-2e991e0c3fc4"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Both fields are universal** — `summary` and `leafUuid` always appear together.

**`leafUuid` anchors the summary to a thread position**, which is the only correlation available; the line carries no `sessionId` and no `uuid` of its own.

**Named in the storage invariant as a skipped type.** Like `queue-operation`, it is one of the four types [`003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) enumerates, and the same `uuid` correction applies — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md).

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/003_detached.md`](../envelope_class/003_detached.md) | Class C field contract this kind satisfies |
| invariant | [`../../../../module/claude_storage/docs/invariant/003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) | Skip-handling contract naming this type |
