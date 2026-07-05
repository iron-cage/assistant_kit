# Pitfall: Test Data Must Match Production Format

### Scope

- **Purpose**: Document the test data format pitfall.
- **Responsibility**: Why synthetic test JSONL must exactly replicate the production format.
- **In Scope**: UUID session IDs, `type` field values, `uuid` field requirement in test data.
- **Out of Scope**: Token counting fields, UI display fields, history.jsonl format.

### Pitfall

Tests written with simplified identifiers (`"test-session-123"`, using `"role"` instead of
`"type"`) pass structural checks but miss coverage for production data patterns. Entry counts
return 0 because the parser's mandatory `uuid` field check fails silently — the test
appears to pass while producing no results.

This was discovered after issue-011: tests passed with `count: 0` when `count: 42` was
expected. Root cause: test JSONL used `"type": "message"` instead of `"type": "user"`,
causing all entries to be skipped by the type classifier.

### Required Pattern

Test JSONL entries must include **all** of:

| Field | Required value |
|-------|----------------|
| `"type"` | `"user"` or `"assistant"` (never `"message"`, `"role"`, or `"entry"`) |
| `"uuid"` | unique UUID-format string (never `"entry-1"` or simple integers) |
| `"sessionId"` | valid UUID-format session ID matching the session file |
| `"message"` | object with `"role"` and `"content"` nested inside |

**Wrong**:
```json
{"type": "message", "role": "user", "content": "hello", "uuid": "test-1"}
```

**Correct**:
```json
{"type": "user", "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
 "sessionId": "f0e1d2c3-b4a5-6789-0123-456789abcdef",
 "message": {"role": "user", "content": "hello"},
 "timestamp": "2025-11-24T10:00:00.000Z"}
```

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`.status`](../command/01_status.md) | Entry counting requires correct `type` field (show_tokens::1 path) |
| 3 | [`.show`](../command/03_show.md) | Entry display requires correct type classification |
| 4 | [`.count`](../command/04_count.md) | `target::entries` depends on correct `type` field |
| 5 | [`.search`](../command/05_search.md) | Content search iterates over classified entries |

### Sources

- `tests/cli_commands.rs:85` — Known Pitfalls section with production format validation
- `changelog.md:106` — pitfall documented after issue-011
- [`invariant/003_entry_type_format.md`](../../invariant/003_entry_type_format.md) — formal entry type contract
