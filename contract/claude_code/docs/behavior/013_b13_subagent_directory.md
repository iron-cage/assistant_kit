# Behavior B13: Subagent Directory Hierarchy

### Scope

- **Purpose**: Document the hierarchical agent storage layout where agents live in `{parent-uuid}/subagents/` subdirectories.
- **Responsibility**: Authoritative instance for behavior B13 — defines the hierarchical format, certainty level, and supporting evidence. Describes the newer format; see B7 for the older flat format.
- **In Scope**: `{parent-uuid}/subagents/agent-{agentId}.jsonl` layout; `tool-results/` sibling; filesystem-encoded parent link; coexistence with flat format.
- **Out of Scope**: Flat agent layout (→ [B7](007_b7_agent_sessions_sibling.md)); agent `meta.json` sidecars (→ [B14](014_b14_agent_meta_json.md)); agent `sessionId` semantics (→ [B12](012_b12_agent_session_id.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: VALIDATED | **Evidence**: E23, E27

New-format agent sessions are stored in a subdirectory tree rooted at the parent session UUID:

```
project-dir/
  {parent-uuid}.jsonl                   # root session file
  {parent-uuid}/
    subagents/
      agent-{agentId}.jsonl             # child agent session
      agent-{agentId}.meta.json         # agent metadata sidecar
    tool-results/                       # tool output artifacts
```

The filesystem path itself encodes the parent-child relationship. This supersedes the older flat layout (B7) where agents were siblings of main sessions in the project root.

Both formats may coexist in real storage — older projects use flat, newer use hierarchical.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E23 | B13 | Observation | Live storage | `~/.claude/projects/*/` | `{uuid}/subagents/agent-*.jsonl` directories observed; parent UUID in directory name matches root `{uuid}.jsonl` |
| E27 | B13 | Test | `../../tests/behavior/b13_subagent_directory_structure.rs` | `b13_subagent_dir_exists_for_root_session` | At least one root session has a matching `{uuid}/subagents/` directory |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [007_b7_agent_sessions_sibling.md](007_b7_agent_sessions_sibling.md) | Flat agent layout (older format) |
| behavior | [012_b12_agent_session_id.md](012_b12_agent_session_id.md) | Agent `sessionId` equals parent directory UUID |
| behavior | [014_b14_agent_meta_json.md](014_b14_agent_meta_json.md) | Agent `.meta.json` sidecars |
| test | `../../tests/behavior/b13_subagent_directory_structure.rs` | Invalidation test |
