# ENVELOPE: PR Link

### Scope

- **Purpose**: Specify the `pr-link` envelope, which associates a session with a GitHub pull request.
- **Responsibility**: Authoritative instance for the `pr-link` top-level line kind: its payload fields, structural class, and observed frequency.
- **In Scope**: The `pr-link` discriminator value, every payload field observed on it with type and presence rate, its envelope class, and its version lifecycle.
- **Out of Scope**: The common-envelope field contract shared with other kinds of the same class (→ [`../envelope_class/`](../envelope_class/readme.md)); other top-level kinds (→ [readme.md](readme.md)).

### Schema

**Discriminator**: `"type": "pr-link"` · **Class B** (Session-Scoped) · **677 lines** (0.013% of the store)

| Field | Type | Presence |
|-------|------|-----------|
| `prNumber` | number | always |
| `prRepository` | string | always |
| `prUrl` | string | always |

The nine common fields are omitted from the table; Class B membership fixes which of them are present — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md).

Captured example:

```json
{
  "type": "pr-link",
  "sessionId": "f568b248-703b-45d9-babd-c5d1289575b9",
  "prNumber": 122,
  "prUrl": "https://github.com/obox-systems/2026_troy_venue_pipeline_dev/pull/122",
  "prRepository": "obox-systems/2026_troy_venue_pipeline_dev",
  "timestamp": "2026-08-01T13:20:09.243Z"
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**All three fields are universal** — `prNumber`, `prUrl`, and `prRepository` always appear together, so the association is never partial.

**One of three Class B kinds carrying `timestamp`**, which orders the association against conversation entries by time.

**Produced by the `--from-pr` entry path** documented in [`../behavior/024_b24_from_pr.md`](../behavior/024_b24_from_pr.md).

### Since

Not attributable. This kind carries no `version` field — see [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md). Class A is the only class whose lines can be attributed to a release from the line alone, so this kind's lifecycle cannot be read from the store.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| envelope | [readme.md](readme.md) | Envelope master index and evidence base |
| envelope_class | [`../envelope_class/002_session_scoped.md`](../envelope_class/002_session_scoped.md) | Class B field contract this kind satisfies |
| behavior | [`../behavior/024_b24_from_pr.md`](../behavior/024_b24_from_pr.md) | `--from-pr` behavior producing this record |
