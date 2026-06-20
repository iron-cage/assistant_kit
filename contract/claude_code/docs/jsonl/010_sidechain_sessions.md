# JSONL: Sidechain Sessions

### Scope

- **Purpose**: Specify the entry format and storage layout for agent/sidechain sessions.
- **Responsibility**: Authoritative instance for sidechain JSONL entries — `isSidechain: true` entries, `agentId`, `slug`, and both storage layout formats.
- **In Scope**: `isSidechain: true` marker; `agentId` and `slug` fields; flat and hierarchical storage layouts; agent metadata sidecar.
- **Out of Scope**: Agent file discovery and storage architecture (→ [`../storage/001_projects_directory.md`](../storage/001_projects_directory.md)); agent layout behaviors (→ B7, B12–B15); common fields (→ [001_common_fields.md](001_common_fields.md)).

### Agent Entry Fields

Sidechain entries have the same common fields plus:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `isSidechain` | boolean | ✅ | Always `true` for agent entries |
| `agentId` | string | ✅ | Agent identifier — pure hex 7–17 chars or typed prefix (e.g., `"a6061d6e2a0c37a78"`) |
| `slug` | string | ✅ | Human-readable label shared by all agents of one parent (e.g., `"jaunty-painting-hinton"`) |
| `sessionId` | string | ✅ | **Parent** session UUID (not the agent's own ID) — behavior B12 |

### Storage Layouts

Two formats coexist (per-project, neither deprecated):

**Flat layout (older projects, B7):**
```
projects/{project-id}/
├── {session-id}.jsonl        ← Main Session   (isSidechain: false)
└── agent-{id}.jsonl          ← Agent Session  (isSidechain: true)
```

**Hierarchical layout (newer projects, B13):**
```
projects/{project-id}/
├── {session-id}.jsonl              ← Root Session
└── {session-id}/
    ├── subagents/
    │   ├── agent-{id}.jsonl        ← Agent Session
    │   └── agent-{id}.meta.json    ← Agent Metadata (B14)
    └── tool-results/               ← Tool Output Artifacts
```

### Agent Metadata Sidecar

`agent-{id}.meta.json` (hierarchical layout only):
```json
{"agentType":"Explore"}
{"agentType":"general-purpose","description":"Read organizational principles"}
{"agentType":"Plan"}
```

Known `agentType` values: `Explore` (~63%), `general-purpose` (~36%), `Plan` (<1%).

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | JSONL master index |
| jsonl | [001_common_fields.md](001_common_fields.md) | Common fields including `isSidechain`, `agentId`, `slug` definitions |
| behavior | [`../behavior/007_b7_agent_sessions_sibling.md`](../behavior/007_b7_agent_sessions_sibling.md) | B7: flat agent layout |
| behavior | [`../behavior/012_b12_agent_session_id.md`](../behavior/012_b12_agent_session_id.md) | B12: agent sessionId equals parent UUID |
| behavior | [`../behavior/013_b13_subagent_directory.md`](../behavior/013_b13_subagent_directory.md) | B13: hierarchical agent layout |
| behavior | [`../behavior/014_b14_agent_meta_json.md`](../behavior/014_b14_agent_meta_json.md) | B14: agent meta.json sidecar |
| behavior | [`../behavior/015_b15_agent_slug.md`](../behavior/015_b15_agent_slug.md) | B15: agent slug field |
| storage | [`../storage/001_projects_directory.md`](../storage/001_projects_directory.md) | Projects directory: agent file locations |
