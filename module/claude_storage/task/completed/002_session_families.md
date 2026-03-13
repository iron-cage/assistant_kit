### Task

Implement session family tree detection — group agent sub-sessions under their parent
root sessions to replace the flat noise list with a hierarchical display.

**Status:** ✅ Complete — Phase 2 implemented and verified.

**Docs updated (plan/007):** spec.md, storage_organization.md, advanced_topics.md, behavior.md, task/002, plan/readme.md.
**Docs updated (beyond plan):** cli/dictionary.md, docs/readme.md, jsonl_format.md (found by post-plan sweep for stale agent references).

### Context

Real `~/.claude/projects/` storage uses a hierarchical structure:

```
project-dir/
  {uuid}.jsonl                          # root session
  {uuid}/
    subagents/
      agent-{agentId}.jsonl             # child agent session
      agent-{agentId}.meta.json         # agent type + description sidecar
    tool-results/                       # tool output artifacts
```

Key relationships discovered (see `docs/claude_code/behavior.md` B12-B15):

1. **`sessionId` in agent entries = parent UUID** — the JSONL `sessionId` field in agent
   entries references the parent session UUID, not the agent's own ID
2. **Filesystem hierarchy encodes parent link** — `{parent-uuid}/subagents/agent-*.jsonl`
3. **`meta.json` sidecars** — contain `agentType` (Explore/general-purpose/Plan) and
   optional `description`
4. **`slug` field** — human-readable conversation label shared by all agents of one parent;
   root session entries typically lack slug (first entry is `queue-operation` type)

### Algorithm: `build_session_families()`

**Phase 1 — Discover root sessions:** Collect all `{uuid}.jsonl` files in project root
(non-agent, non-zero-byte).

**Phase 2 — Discover subagent directories:** For each UUID found in Phase 1, check if
`{uuid}/subagents/` exists.

**Phase 3 — Collect agents per family:** List `agent-*.jsonl` in each subagent directory.
Parse `meta.json` sidecars for type/description.

**Phase 4 — Handle orphans:** Scan for `{uuid}/subagents/` directories whose UUID has no
matching root `.jsonl` — these are orphan families (root deleted or never written).

**Phase 5 — Build display:** Sort families by root mtime. Show root session with agent
count, then optionally expand to show individual agents grouped by type.

### Proposed Output (v1)

```
~/project: (2 conversations, 47 agents)
  * 79f86582 3h ago  347 entries  [12 agents: 8×Explore, 3×general-purpose, 1×Plan]
  - 38809aee 2d ago   42 entries  [5 agents: 4×Explore, 1×general-purpose]
```

### Dependencies

- B12-B15 behavior hypotheses (documented in `docs/claude_code/behavior.md`)
- Invalidation tests for B12-B15 (in `tests/behavior/`)

### Outcomes

**Delivered:**
- 6 new functions: `build_families`, `parse_agent_meta`, `format_type_breakdown`, `extract_parent_hierarchical`, `extract_parent_flat`, `is_hierarchical_format`
- 3 structs: `SessionFamily`, `AgentInfo`, `AgentMeta`
- 2 display renderers: `render_families_v1`, `render_families_v2`
- 3 test fixture helpers: `write_hierarchical_session`, `write_flat_agent_session`, `write_agent_meta_json` + path variant `write_hierarchical_path_session`
- 8 new integration tests (IT-36 through IT-43)
- Updated 2 existing tests (IT-19, IT-20)
- Updated 6 documentation files (spec.md, commands.md, sessions.md, readme.md, advanced_topics.md, YAML)

**Files changed:**
- `src/cli/mod.rs` — family detection algorithm + Algorithm C rewrite
- `tests/common/mod.rs` — fixture helpers
- `tests/sessions_command_test.rs` — 8 new tests
- `tests/sessions_output_format_test.rs` — updated IT-19, IT-20

**Key design decisions:**
- Both hierarchical (`{uuid}/subagents/`) and flat (`agent-*.jsonl` siblings) formats handled via format detection per-project
- Family detection is CLI-only (not in `claude_storage_core`) — it's a display concern
- `agent::` filter disables family grouping (flat display when explicitly requesting agents)
- Uses `claude_storage_core::parse_json` for meta.json parsing (no serde_json dependency)
- `parse_agent_meta` falls back to "unknown" for any error (empty, malformed, missing file)

**Test results:** 283 tests pass, clippy clean, doc tests pass
