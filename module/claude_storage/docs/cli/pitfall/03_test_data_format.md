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

### Full-Attribute Shapes

The minimal example above covers the required-field floor, but omits several
fields that exist only on richer entries — needed when a test fixture must
exercise `fields::all` or any single one of `.show`'s 18 canonical attributes
(see [`../type/15_field_selector.md`](../type/15_field_selector.md)). Two gaps
recur:

**`thinkingMetadata` is a top-level sibling of `message`, not nested inside
it** — mirroring `requestId` below, not `message.content`:

```json
{"type": "user", "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
 "parentUuid": null, "timestamp": "2025-11-24T10:00:00.000Z",
 "cwd": "/tmp", "sessionId": "f0e1d2c3-b4a5-6789-0123-456789abcdef",
 "version": "2.0.31", "gitBranch": "main", "userType": "external",
 "isSidechain": false,
 "message": {"role": "user", "content": "hello"},
 "thinkingMetadata": {"level": "high", "disabled": false}}
```

**An assistant entry's `message.content` array holds heterogeneous content
blocks** — `requestId` is top-level (sibling of `message`, like
`thinkingMetadata` above); `message.model`/`message.id` are required:

```json
{"type": "assistant", "uuid": "56a226b5-0ec6-4214-af16-b13cc326f8dc",
 "parentUuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
 "timestamp": "2025-11-24T10:00:05.000Z", "cwd": "/tmp",
 "sessionId": "f0e1d2c3-b4a5-6789-0123-456789abcdef", "version": "2.0.31",
 "gitBranch": "main", "userType": "external", "isSidechain": false,
 "requestId": "req_01ABC",
 "message":
 {
   "model": "claude-sonnet-4-5", "id": "msg_01ABC", "role": "assistant",
   "content":
   [
     {"type": "thinking", "thinking": "Let me check...", "signature": "sig123"},
     {"type": "text", "text": "Running the tool now."},
     {"type": "tool_use", "id": "toolu_01", "name": "read_file", "input": {"path": "/tmp/x"}},
     {"type": "tool_result", "tool_use_id": "toolu_01", "content": "file contents", "is_error": false}
   ],
   "stop_reason": "end_turn", "stop_sequence": null
 }}
```

Each content-block `"type"` value maps to a distinct required-field set —
`text` needs `text`; `thinking` needs `thinking` (`signature` optional,
defaults to `""`); `tool_use` needs `id`/`name`/`input`; `tool_result` needs
`tool_use_id`/`content` (`is_error` optional, defaults to `false`). Omitting
a block's required field fails that block's parse the same way a missing
top-level field fails the whole entry — see `parse_content_block` in
`claude_storage_core/src/entry.rs`.

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
- [`../../invariant/003_entry_type_format.md`](../../invariant/003_entry_type_format.md) — formal entry type contract
