# Behavior B15: Agent Slug Field

### Scope

- **Purpose**: Document that agent entries carry a `slug` field (human-readable label) shared by all agents of one parent session.
- **Responsibility**: Authoritative instance for behavior B15 — defines the slug field semantics, certainty level, and supporting evidence.
- **In Scope**: `slug` field; shared value across sibling agents; absence in root session entries; human-readable family identifier.
- **Out of Scope**: `agentId` field (→ [`../jsonl/001_common_fields.md`](../jsonl/001_common_fields.md)); agent `sessionId` parent link (→ [B12](012_b12_agent_session_id.md)).

### Behavior

**Status**: 🎯 Observed | **Certainty**: 85% | **Tier**: VALIDATED | **Since**: pre-v1.0 | **Evidence**: E25, E29

Agent entries carry a `slug` field — a human-readable conversation label like `"jaunty-painting-hinton"`. All agents spawned from the same parent share an identical slug.

Root session entries typically lack the `slug` field; their first entry is usually of type `queue-operation` (metadata, not conversation content).

The slug serves as a human-friendly family identifier that could be displayed instead of UUIDs. It is consistent across all sibling agents in one Session Family.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E25 | B15 | Observation | Live storage | `~/.claude/projects/*/subagents/agent-*.jsonl` | All sibling agent entries share identical `slug` value (e.g., `"jaunty-painting-hinton"`); root session first entry has no `slug` |
| E29 | B15 | Test | `../../tests/behavior/b15_agent_slug_field.rs` | `b15_sibling_agents_share_slug` | All sibling agents under one parent share the same `slug` value |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [012_b12_agent_session_id.md](012_b12_agent_session_id.md) | Agent `sessionId` as programmatic parent link |
| jsonl | [`../jsonl/001_common_fields.md`](../jsonl/001_common_fields.md) | `agentId` and `slug` as optional common fields |
| jsonl | [`../jsonl/010_sidechain_sessions.md`](../jsonl/010_sidechain_sessions.md) | Sidechain JSONL entry format (`slug` field in agent entries) |
| test | `../../tests/behavior/b15_agent_slug_field.rs` | Invalidation test |
