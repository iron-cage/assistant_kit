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

- **Entry type field**: top-level `"type"` — values: `"user"` | `"assistant"`
- **Message role**: nested `"message"."role"` — not for entry classification
- **Metadata entries**: top-level `"type"` may be `"queue-operation"`, `"summary"`, or other
  non-conversation values — skip these when counting conversation entries
- **UUID field**: mandatory; entries missing `"uuid"` must be silently skipped
- **Format version**: Claude Code v2.0+ format; validated against real v2.0.31 sessions

### Violation Conditions

- Using `"role"` instead of `"type"` to count or classify entries (causes entry count 0)
- Treating `"message"` as a valid value for the top-level `"type"` field
- Counting `"queue-operation"` or `"summary"` entries as conversation entries
- Failing to skip entries with missing `"uuid"` (causes parse errors or silent data loss)

### Referenced Commands

| # | Command | Context |
|---|---------|---------|
| 1 | [`.status`](../cli/command/01_status.md) | Entry counting requires correct type field |
| 3 | [`.show`](../cli/command/03_show.md) | Entry display requires correct classification |
| 4 | [`.count`](../cli/command/04_count.md) | `target::entries` depends on correct type field |
| 5 | [`.search`](../cli/command/05_search.md) | Content search iterates over classified entries |

### Sources

- `tests/cli_commands.rs:85` — Known Pitfalls section with production format validation evidence
- `claude_storage_core/src/` — entry parser implementation (`entry.rs`)
