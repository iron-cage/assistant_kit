# JSONL: Common Fields

### Scope

- **Purpose**: Specify the fields present in all JSONL entry types, regardless of whether the entry is a user or assistant message.
- **Responsibility**: Authoritative instance for JSONL common fields — every field that appears in both user and assistant entries.
- **In Scope**: `uuid`, `parentUuid`, `timestamp`, `type`, `cwd`, `sessionId`, `version`, `gitBranch`, `userType`, `isSidechain`, and optional `agentId`/`slug` fields.
- **Out of Scope**: User-specific fields (→ [002_user_entry.md](002_user_entry.md)); assistant-specific fields (→ [003_assistant_entry.md](003_assistant_entry.md)).

### Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `uuid` | string | ✅ | Unique identifier for this entry (UUID v4) |
| `parentUuid` | string \| null | ✅ | UUID of parent entry (null for first message in session) |
| `timestamp` | string | ✅ | ISO 8601 timestamp (e.g., `"2025-11-08T23:30:10.039Z"`) |
| `type` | string | ✅ | Entry type: `"user"` or `"assistant"` for conversation entries (see Notes for non-conversation values) |
| `cwd` | string | ✅ | Working directory when message was sent |
| `sessionId` | string | ✅ | Session UUID this entry belongs to (for agent entries: parent session UUID) |
| `version` | string | ✅ | Claude Code version (e.g., `"2.0.31"`) |
| `gitBranch` | string \| null | ✅ | Git branch name (null if not in git repo) |
| `userType` | string | ✅ | User type: always `"external"` (human) |
| `isSidechain` | boolean | ✅ | Whether this is a sidechain/agent conversation entry |
| `agentId` | string | ❌ | Agent identifier (present in agent entries only; pure hex 7–17 chars or typed prefix) |
| `slug` | string | ❌ | Human-readable conversation label (agent entries only, e.g., `"jaunty-painting-hinton"`) |

### Notes

**`parentUuid`**: `null` on the first entry in a session; references the `uuid` of the previous entry in subsequent messages. The chain is self-contained within one session file (see behavior B17) with a known exception at context-compaction boundaries.

**`sessionId`**: For main session entries, equals the session's own UUID (matching the `.jsonl` filename). For agent entries, equals the parent root session UUID (not the agent's own ID) — this is behavior B12.

**`isSidechain`**: `false` for main session entries; `true` for all agent session entries.

**`agentId` and `slug`**: Optional fields present only in agent session entries (where `isSidechain: true`). `slug` is shared across all sibling agents of one parent.

**Non-conversation `type` values**: some JSONL lines carry a top-level `"type"` other than `"user"`/`"assistant"` (e.g. `"queue-operation"`, `"summary"`) and do not necessarily share this file's common-fields schema — these must be skipped by consumers iterating conversation entries. See [`03_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/03_entry_type_format.md) for the full non-conversation type contract and skip-handling rules.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| invariant | [`../../../../module/claude_storage/docs/invariant/03_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/03_entry_type_format.md) | Non-conversation `type` value contract and skip-handling rules |
| jsonl | [002_user_entry.md](002_user_entry.md) | User-specific fields: `message.role`, `message.content` (string), `thinkingMetadata` |
| jsonl | [003_assistant_entry.md](003_assistant_entry.md) | Assistant-specific fields: `message.model`, `message.content` (array), `requestId` |
| jsonl | [009_threading_model.md](009_threading_model.md) | Threading model: `parentUuid` chain structure and self-containment |
| behavior | [`../behavior/010_b10_entry_threading.md`](../behavior/010_b10_entry_threading.md) | `parentUuid` threading model |
| behavior | [`../behavior/012_b12_agent_session_id.md`](../behavior/012_b12_agent_session_id.md) | Agent `sessionId` equals parent UUID |
| behavior | [`../behavior/015_b15_agent_slug.md`](../behavior/015_b15_agent_slug.md) | Agent `slug` field semantics |
| jsonl | [010_sidechain_sessions.md](010_sidechain_sessions.md) | Sidechain entry format: `isSidechain`, `agentId`, `slug` in agent entries |
