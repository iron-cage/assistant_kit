# Behavior B7: Agent Sessions Are Siblings (Flat Layout)

### Scope

- **Purpose**: Document the flat agent session layout where agent files are siblings of main sessions.
- **Responsibility**: Authoritative instance for behavior B7 — defines the flat agent layout, certainty level, and supporting evidence. Describes the older format; see B13 for the newer hierarchical format.
- **In Scope**: Flat agent layout (`agent-*.jsonl` as siblings); `isSidechain: true` marker; `agentId` field; user-invisible behavior.
- **Out of Scope**: Hierarchical agent layout (→ [B13](013_b13_subagent_directory.md)); agent `sessionId` semantics (→ [B12](012_b12_agent_session_id.md)); agent metadata sidecars (→ [B14](014_b14_agent_meta_json.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: VALIDATED | **Since**: pre-v1.0 | **Evidence**: E6, E17

Agent sessions are stored as `agent-*.jsonl` files with `isSidechain: true` in every entry; they are siblings of main sessions in the same project directory, not children.

Distinguishing characteristics:
- Filename prefix `agent-`
- `isSidechain: true` in every entry
- `agentId` field present in entries

From the user's perspective agent sessions are invisible — `--continue` skips them entirely when selecting the most recent session.

**Note:** B7 describes the flat (older) agent layout. Newer projects use the hierarchical layout (B13): `{uuid}/subagents/agent-{agentId}.jsonl`. Both formats coexist in real storage — older projects use flat, newer use hierarchical.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E6 | B7 | Observation | Live storage | `~/.claude/projects/*/agent-*.jsonl` | Agent session files observed as siblings of main sessions; entries contain `"isSidechain":true` |
| E17 | B7 | Test | `../../tests/behavior/b07_agent_sessions.rs` | `b7_real_agent_session_has_issidechain_true` | Real `agent-*.jsonl` file contains `"isSidechain":true` in first entry |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [012_b12_agent_session_id.md](012_b12_agent_session_id.md) | Agent `sessionId` equals parent session UUID |
| behavior | [013_b13_subagent_directory.md](013_b13_subagent_directory.md) | Hierarchical agent layout (newer format) |
| behavior | [014_b14_agent_meta_json.md](014_b14_agent_meta_json.md) | Agent `.meta.json` sidecars |
| test | `../../tests/behavior/b07_agent_sessions.rs` | Invalidation test |
