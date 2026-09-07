# ATTACHMENT: Budget USD

### Scope

- **Purpose**: Specify the `budget_usd` payload, which reports a dollar-denominated spend cap and remaining balance for the session.
- **Responsibility**: Authoritative instance for the `budget_usd` attachment payload: its fields and observed occurrence, for a kind absent from this collection's original 23-instance scan.
- **In Scope**: The `attachment.type` value `budget_usd`, every payload field observed on it with type, and the independent local evidence supporting its existence.
- **Out of Scope**: The `attachment` envelope carrying this payload (→ [`../envelope/003_attachment.md`](../envelope/003_attachment.md)); other payload kinds (→ [readme.md](readme.md)); the token-denominated sibling reminder (→ [001_total_tokens_reminder.md](001_total_tokens_reminder.md)).

❌ **Gap in the original 23-instance scan.** This collection's readme.md documents exactly 23 `attachment.type` values from an 18,332-file / 5,049,738-line scan snapshotted 2026-08-27, with line counts summing to exactly 407,370. `budget_usd` is a real, distinct 24th type absent from that count entirely — not a rare kind the scan under-sampled, but one that (per the evidence below) only started appearing the day *after* the snapshot was taken. Numbered `024` rather than re-sorted into the frequency-descending 001-023 range because this instance rests on a smaller, independent, single-day local sample, not the original full-corpus recount — its true global rank relative to the other 23 is unknown.

### Schema

**Discriminator**: `attachment.type == "budget_usd"`

Fields listed are those of the nested `attachment` object. The enclosing line is a Class A envelope carrying all nine common fields.

| Field | Type | Presence |
|-------|------|-----------|
| `used` | number (USD) | always (observed 0 and non-zero, e.g. `0.1099337`) |
| `total` | number (USD) | always (observed `0.05` and `0.2` — a configured cap, not a constant) |
| `remaining` | number (USD) | always; equals `total - used` on every occurrence observed |

Captured example — the `attachment` object only:

```json
{
  "type": "budget_usd",
  "used": 0.1099337,
  "total": 0.2,
  "remaining": 0.09006630000000002
}
```

Long string values are elided with `…` and deep structures with `"…"`; field names and types are verbatim.

### Notes

**Independent local evidence, not the collection's own scan.** Found via a direct grep/jq sweep of this machine's own `~/.claude/projects/**/*.jsonl` store (17,390 files visible at time of check — close to, but not identical to, the original scan's 18,332, consistent with normal rotation over the ~10 days since the 2026-08-27 snapshot). 7 occurrences located, across 5 session files, all on a single day: **2026-08-28**, all `version: "2.1.220"`, all `entrypoint: "sdk-cli"`. No occurrence was found on any other date in the visible store.

**Distinct from `total_tokens_reminder`.** That payload reports remaining *context window* budget (tokens, per-turn). This payload reports remaining *spend* budget (USD, presumably per-session or per-invocation) — a different resource axis entirely. The `entrypoint: "sdk-cli"` clustering on every observed occurrence suggests this is specific to SDK/headless invocations with a configured cost cap, rather than a general interactive-session feature — this is an inference from the sample, not confirmed against source.

**Sample is too small for a presence-rate or exact introduction-version claim.** Unlike the other 23 instances (each backed by the full 5M-line scan), this entry rests on 7 lines. Treat the field table above as structurally reliable (three fields, consistent arithmetic relationship, confirmed via `jq`-parsed real lines) but the frequency/ranking claims as provisional.

### Since

Not established. Only observed on 2026-08-28, `version: "2.1.220"`, in this machine's local store — one day after this collection's own 2026-08-27 snapshot date. This is consistent with (but does not prove) introduction at or shortly before that date; it is equally consistent with a feature that existed earlier but is conditional on a cost-cap being configured (observed only under `sdk-cli` entrypoint), which the original scan's session mix may simply not have included. Needs a full-corpus recount to date properly — see the caveat under Scope above.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| attachment | [readme.md](readme.md) | Attachment master index and evidence base |
| attachment | [001_total_tokens_reminder.md](001_total_tokens_reminder.md) | Sibling budget-reporting payload — token-denominated, not USD |
| envelope | [`../envelope/003_attachment.md`](../envelope/003_attachment.md) | The envelope carrying this payload |
| envelope_class | [`../envelope_class/001_full_envelope.md`](../envelope_class/001_full_envelope.md) | Class A field contract the enclosing line satisfies |
