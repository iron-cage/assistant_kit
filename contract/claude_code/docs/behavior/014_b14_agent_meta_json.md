# Behavior B14: Agent meta.json Sidecars

### Scope

- **Purpose**: Document that agent JSONL files have sibling `.meta.json` files containing `agentType` and optional `description`.
- **Responsibility**: Authoritative instance for behavior B14 — defines the sidecar format, certainty level, and supporting evidence.
- **In Scope**: `.meta.json` sidecar file; `agentType` field; optional `description` field; known `agentType` value distribution.
- **Out of Scope**: Agent JSONL entry format (→ [`../jsonl/010_sidechain_sessions.md`](../jsonl/010_sidechain_sessions.md)); agent directory layout (→ [B13](013_b13_subagent_directory.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 90% | **Tier**: VALIDATED | **Since**: pre-v1.0 | **Evidence**: E24, E28

Each agent JSONL file may have a sibling `.meta.json` file containing agent metadata:

```json
{"agentType":"Explore"}
{"agentType":"general-purpose"}
{"agentType":"Plan"}
{"agentType":"claude-code-guide"}
{"agentType":"Explore","description":"Read organizational principles rulebook"}
```

**Known `agentType` values** (observed distribution):
- `Explore` (~63%)
- `general-purpose` (~36%)
- `Plan` (<1%)
- `claude-code-guide` (rare)

The `description` field is optional and present only on some `Explore` agents. Other agent types may also have it but were not observed.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E24 | B14 | Observation | Live storage | `~/.claude/projects/*/subagents/*.meta.json` | `meta.json` files contain `{"agentType":"Explore"}` or `{"agentType":"general-purpose"}` or `{"agentType":"Plan"}`; some include `description` |
| E28 | B14 | Test | `../../tests/behavior/b14_agent_meta_json.rs` | `b14_meta_json_contains_agent_type` | Real `.meta.json` file contains `agentType` field with known value |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [013_b13_subagent_directory.md](013_b13_subagent_directory.md) | Hierarchical agent layout (directory containing the sidecar) |
| jsonl | [`../jsonl/010_sidechain_sessions.md`](../jsonl/010_sidechain_sessions.md) | Agent JSONL entry format |
| test | `../../tests/behavior/b14_agent_meta_json.rs` | Invalidation test |
