# ENVELOPE: Last Prompt

### Scope

- **Purpose**: Specify the `last-prompt` envelope, the record `--continue` and `--resume` read to restore a session's position.
- **Responsibility**: Authoritative instance for the `last-prompt` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `last-prompt` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "last-prompt"` · **Class B** (Session-Scoped) · **262,195 lines** (5.19% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `leafUuid` | string | 260,651 of 262,195 (99.4%) |
| `lastPrompt` | string | 254,320 of 262,195 (97.0%) |

The nine common fields are omitted from the table; Class B membership fixes which of them are present — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md).

Captured example:

```json
{
  "type": "last-prompt",
  "lastPrompt": "Generate a conventional commit message for the staged changes.  STAGED CHANGES SUMMARY:…",
  "leafUuid": "e2713364-5250-4262-b27a-a4d4a4e50b5a",
  "sessionId": "2bb4b6c2-0b05-405f-9355-9f29517b09b8"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Two independent optional fields.** `lastPrompt` carries the prompt text and `leafUuid` the entry it attached to; neither is universal, and a line may carry one without the other.

**`leafUuid` is a Class A `uuid` reference.** It points into the conversation thread, which is how resume re-anchors — the line itself has no `uuid` of its own.

**High frequency, low information density.** At over a quarter-million lines this is the fourth most common kind in the store, yet it is pure session bookkeeping. A consumer reconstructing conversation content should skip it entirely.

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) | Class B field contract this kind satisfies |
| behavior | [`../behavior/004_b4_continue_flag.md`](../behavior/004_b4_continue_flag.md) | `--continue` behavior that consumes this record |
| behavior | [`../behavior/019_b19_resume_flag.md`](../behavior/019_b19_resume_flag.md) | `--resume` behavior that consumes this record |
