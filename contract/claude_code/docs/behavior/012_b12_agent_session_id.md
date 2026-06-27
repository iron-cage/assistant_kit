# Behavior B12: Agent sessionId Is Parent UUID

### Scope

- **Purpose**: Document that agent JSONL entries carry `sessionId` equal to the parent session UUID, not the agent's own ID.
- **Responsibility**: Authoritative instance for behavior B12 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: Agent entry `sessionId` field; parent session UUID linkage; programmatic parent-child link.
- **Out of Scope**: Agent filename conventions (→ [B7](007_b7_agent_sessions_sibling.md), [B13](013_b13_subagent_directory.md)); agent `slug` field (→ [B15](015_b15_agent_slug.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: VALIDATED | **Since**: pre-v1.0 | **Evidence**: E22, E26

In agent JSONL entries, the `sessionId` field does **not** refer to the agent's own session. Instead it contains the UUID of the parent (root) session. This is the primary programmatic link between a sub-agent and the conversation that spawned it.

For example, an agent stored at `43860c56-…/subagents/agent-a6061d6e….jsonl` has `"sessionId": "43860c56-f828-44bd-953a-432920676b63"` — the parent directory UUID.

This `sessionId` field is what enables the `claude_storage` library to group agent sessions with their parent root session into a **Session Family**.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E22 | B12 | Observation | Live storage | `~/.claude/projects/*/subagents/agent-*.jsonl` | Agent entry `sessionId` field equals the parent directory UUID, not the agent filename ID |
| E26 | B12 | Test | `../../tests/behavior/b12_agent_session_id_is_parent.rs` | `b12_agent_session_id_matches_parent_dir` | Agent entry `sessionId` equals the UUID from the parent directory path |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [007_b7_agent_sessions_sibling.md](007_b7_agent_sessions_sibling.md) | Flat agent layout (sibling files) |
| behavior | [013_b13_subagent_directory.md](013_b13_subagent_directory.md) | Hierarchical agent layout (directory structure) |
| behavior | [015_b15_agent_slug.md](015_b15_agent_slug.md) | Agent `slug` field (human-readable family ID) |
| jsonl | [`../jsonl/001_common_fields.md`](../jsonl/001_common_fields.md) | Common fields: `sessionId` field definition |
| jsonl | [`../jsonl/010_sidechain_sessions.md`](../jsonl/010_sidechain_sessions.md) | Sidechain JSONL entry format |
| test | `../../tests/behavior/b12_agent_session_id_is_parent.rs` | Invalidation test |
