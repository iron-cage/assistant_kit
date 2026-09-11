# JSONL: Common Fields

### Scope

- **Purpose**: Specify the fields present in all JSONL entry types, regardless of whether the entry is a user or assistant message.
- **Responsibility**: Authoritative instance for JSONL common fields — every field that appears in both user and assistant entries.
- **In Scope**: `uuid`, `parentUuid`, `timestamp`, `type`, `cwd`, `sessionId`, `version`, `gitBranch`, `userType`, `isSidechain`, and optional `agentId`/`slug`/`entrypoint` fields present on both `user` and `assistant` entries.
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
| `agentId` | string | ❌ | Agent identifier — present in agent entries only (`isSidechain: true`); pure hex 7–17 chars or typed prefix |
| `slug` | string | ❌ | Human-readable conversation label (e.g., `"jaunty-painting-hinton"`) — present on ~98.6% of ALL entries, not agent-only (corrected; see Notes) |
| `entrypoint` | string | ❌ | Invocation source identifier — present on ~95–96% of entries since v2.1.74. Observed values: `"cli"`, `"sdk-cli"` (a random sample found only these two; not confirmed exhaustive) |

### Notes

**`parentUuid`**: `null` on the first entry in a session; references the `uuid` of the previous entry in subsequent messages. The chain is self-contained within one session file (see behavior B17) with a known exception at context-compaction boundaries.

**`sessionId`**: For main session entries, equals the session's own UUID (matching the `.jsonl` filename). For agent entries, equals the parent root session UUID (not the agent's own ID) — this is behavior B12.

**`isSidechain`**: `false` for main session entries; `true` for all agent session entries.

**`agentId`**: Optional field present only in agent session entries (`isSidechain: true`). A full local-store scan (2026-09-06 snapshot, 5,446,921 lines) found exact correlation: every entry with `agentId` has `isSidechain: true`, and vice versa, with no exceptions.

**`slug`** (corrected — previously documented as agent-only alongside `agentId`): present on ~98.6% of `user` entries and ~98.7% of `assistant` entries regardless of `isSidechain`, not agent-only. Shared across all sibling agents of one parent when the entry is itself an agent entry.

**`entrypoint`**: present on ~95–96% of entries (both types), first observed at v2.1.74 in the local store — a real introduction point, not a sampling floor artifact (contrast with `agentId`/`slug`, which span the store's entire v2.0.56–v2.1.220 range and therefore predate it).

**`session_id`** (anomaly, not yet a stable field): a second, snake_case session identifier distinct from `sessionId` above. Observed on 15.5% of `user` and 17.5% of `assistant` entries, exclusively at `version: "2.1.220"` — the single newest version in the local store. Consistent with an in-progress migration or newly-introduced parallel field rather than an established part of the schema; prefer `sessionId` until this stabilizes or disappears.

**Non-conversation `type` values**: some JSONL lines carry a top-level `"type"` other than `"user"`/`"assistant"` (e.g. `"queue-operation"`, `"summary"`) and do not necessarily share this file's common-fields schema — these must be skipped by consumers iterating conversation entries. The full taxonomy of all 19 top-level kinds is [`../envelope/readme.md`](../envelope/readme.md); see also [`003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) for the skip-handling contract.

### Since

pre-v1.0 (unverified) for the ten required fields above. `agentId` and `slug` span the full v2.0.56–v2.1.220 range observed in a 2026-09-06 full-store scan (17,210 session files, 5,446,921 lines) and predate it; `entrypoint` first appears at v2.1.74 within that same scan — a genuine lifecycle signal, not a floor artifact.

**Verify yourself**: `grep -c '"entrypoint"' <session>.jsonl` vs `wc -l <session>.jsonl` — the ratio should land near 95% for any session recorded on v2.1.74 or later, and 0% for older sessions (check the `version` field on any line in the file).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| invariant | [`../../../../module/claude_storage/docs/invariant/003_entry_type_format.md`](../../../../module/claude_storage/docs/invariant/003_entry_type_format.md) | Non-conversation `type` value contract and skip-handling rules |
| jsonl | [002_user_entry.md](002_user_entry.md) | User-specific fields: `message.role`, `message.content` (string), `thinkingMetadata` |
| jsonl | [003_assistant_entry.md](003_assistant_entry.md) | Assistant-specific fields: `message.model`, `message.content` (array), `requestId` |
| jsonl | [009_threading_model.md](009_threading_model.md) | Threading model: `parentUuid` chain structure and self-containment |
| behavior | [`../behavior/010_b10_entry_threading.md`](../behavior/010_b10_entry_threading.md) | `parentUuid` threading model |
| behavior | [`../behavior/012_b12_agent_session_id.md`](../behavior/012_b12_agent_session_id.md) | Agent `sessionId` equals parent UUID |
| behavior | [`../behavior/015_b15_agent_slug.md`](../behavior/015_b15_agent_slug.md) | Agent `slug` field semantics |
| jsonl | [010_sidechain_sessions.md](010_sidechain_sessions.md) | Sidechain entry format: `isSidechain`, `agentId`, `slug` in agent entries |
