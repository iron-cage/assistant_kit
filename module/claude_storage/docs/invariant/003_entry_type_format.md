# Invariant: Entry Type Format

### Scope

- **Purpose**: Define the JSONL entry type field contract.
- **Responsibility**: Which field identifies entry type, valid values, field location.
- **In Scope**: Claude Code v2.0+ JSONL format, `type` field semantics and location.
- **Out of Scope**: Content fields, token counting fields, history.jsonl format.

### Statement

Every conversation entry in a Claude Code session JSONL file has a top-level `"type"` field
with value `"user"` or `"assistant"`. The `"role"` field exists but is nested inside the
`"message"` object and must never be used for entry type identification or counting.

### Field Location

```json
{
  "type": "user",           ← top-level: use this for entry classification
  "message": {
    "role": "user",         ← nested: do NOT use for classification
    "content": "..."
  },
  "uuid": "...",            ← mandatory: entries missing uuid must be skipped
  "timestamp": "...",
  ...
}
```

### Contract

- **Entry type field**: top-level `"type"` — conversation values: `"user"` | `"assistant"`
- **Message role**: nested `"message"."role"` — not for entry classification
- **Metadata entries**: top-level `"type"` may be `"queue-operation"`, `"summary"`, or `"system"`
  — all three appear together in a synthetic test fixture (see Sources); `"queue-operation"`/
  `"summary"` additionally have dedicated contract tests. `"file-history-snapshot"` is named in
  export-skip logic but has no test fixture. None of these four has a reproducible, committed
  artifact confirming it was observed in an actual real-world session — claims to that effect
  found in prose elsewhere in this repo are not independently verifiable from anything committed
  here (see Sources). The parser rejects any `"type"` value other than `"user"`/`"assistant"`,
  so other non-conversation values may exist beyond these four — skip all non-conversation types
  when counting conversation entries.
- **Skip mechanism**: an entry is skipped whenever it fails to parse as a conversation entry,
  for any reason — not only a missing `"uuid"`. A `"type"` other than `"user"`/`"assistant"`
  is itself one such failure. The parser checks `"uuid"` and `"timestamp"` before `"type"`
  (see `entry.rs`), so a non-conversation line missing either of those fails at that earlier
  check instead — the end result (line skipped) is the same regardless of which check fails.
  The parser returns a per-line error in all these cases; the caller discards the line
  silently rather than propagating the error.
- **UUID field presence is not a type discriminator**: the parser requires `"uuid"` on every
  line, conversation or not, and never uses its presence to decide entry type — only the
  `"type"` field value determines classification (`"queue-operation"`/`"summary"` are confirmed
  to carry a `uuid` in the IN-3/IN-4 contract-test fixtures).
- **Format version**: Claude Code v2.0+ format. `tests/cli_commands.rs`'s bug-history comment
  narrates validation against a real v2.0.31 session with specific entry-count figures; those
  figures have no reproducible fixture or log backing them in this repo and should be treated
  as unverified narrative, not settled fact.

### Violation Conditions

- Using `"role"` instead of `"type"` to count or classify entries (causes entry count 0)
- Treating `"message"` as a valid value for the top-level `"type"` field
- Counting `"queue-operation"`, `"summary"`, or any other non-conversation `"type"` as a conversation entry
- Assuming `"uuid"` presence implies a conversation entry (`"queue-operation"`/`"summary"` are confirmed to carry one — see IN-3/IN-4 tests)
- Failing to skip entries whose `"type"` is unrecognized or whose required fields are missing (causes parse errors or silent data loss)

### Referenced Commands

| # | Command | Context |
|---|---------|---------|
| 1 | [`.status`](../cli/command/01_status.md) | Entry counting requires correct type field |
| 3 | [`.show`](../cli/command/03_show.md) | Entry display requires correct classification |
| 4 | [`.count`](../cli/command/04_count.md) | `target::entries` depends on correct type field |
| 5 | [`.search`](../cli/command/05_search.md) | Content search iterates over classified entries |

### Sources

| File | Relationship |
|------|--------------|
| `tests/cli_commands.rs:85` | Known Pitfalls section documenting the type-vs-role bug fix; the cited production-session figures (4792 entries etc.) are narrative only, with no reproducible fixture in this repo |
| `claude_storage_core/src/entry.rs` | Entry parser; errors on unrecognized `"type"`, caught silently by `Session::load_entries()`'s `if let Ok(...)` in `session.rs` |
| `claude_storage_core/tests/count_entries_bug.rs` | Synthetic fixture exercising `"queue-operation"`/`"system"`/`"summary"` mixed with conversation entries |
| `claude_storage_core/tests/export.rs:335-360` | Opportunistic scan for `"queue-operation"`/`"summary"`/`"file-history-snapshot"`; since commit `d11c1641` it is rooted at a synthetic empty tmp directory, not real user storage, and gracefully skips when nothing is found — not a confirmation of real-world presence |
| `tests/invariant_contracts_test.rs` (IN-3, IN-4) | Confirms parser skips `"queue-operation"`/`"summary"` even when a `uuid` is present |
