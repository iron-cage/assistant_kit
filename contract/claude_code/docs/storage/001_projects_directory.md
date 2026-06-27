# Storage: Projects Directory

### Scope

- **Purpose**: Document the `projects/` directory — the primary storage for all Claude Code conversations.
- **Responsibility**: Authoritative instance for the `projects/` storage area — project types, path encoding, agent layouts, growth characteristics, and access patterns.
- **In Scope**: UUID projects (web/IDE), path projects (CLI), path encoding rules, flat and hierarchical agent layouts, Session Family concept, growth characteristics.
- **Out of Scope**: Support directories (→ [002_support_directories.md](002_support_directories.md)); global root files (→ [003_root_files.md](003_root_files.md)); entry-level JSONL schema (→ [`../jsonl/`](../jsonl/readme.md)).

### Structure

```
~/.claude/projects/                   # Conversation storage root (1.4GB)
├── {uuid}/                           # UUID project (web/IDE session)
│   ├── {session-id}.jsonl           # Main session
│   ├── agent-{id}.jsonl             # Sub-agent (flat format, B7)
│   └── {session-id}/                # Session family directory (hierarchical format, B13)
│       ├── subagents/
│       │   ├── agent-{id}.jsonl     # Agent session
│       │   └── agent-{id}.meta.json # Agent metadata (B14)
│       └── tool-results/            # Tool output artifacts
└── -{path-encoded}/                  # Path project (CLI session)
    └── {session-id}.jsonl           # CLI conversation
```

### Contents

**UUID projects** (web/IDE sessions): Named by UUID. Created when Claude Code is launched from a web browser or IDE integration rather than the CLI.

```
projects/26dd749d-5b4b-bfee-f4f3-9e03803b8cad/
├── 8d795a1c-c81d-4010-8d29-b4e678272419.jsonl
└── agent-f3e2d1c4.jsonl
```

**Path projects** (CLI sessions): Named by encoded filesystem path. Created when Claude Code is launched from the CLI in a working directory.

```
projects/-home-alice-projects-consumer-app-module-wplan_agent/
├── 3a4b5c6d-e7f8-9012-3456-789abcdef012.jsonl
└── 7e8f9a0b-c1d2-3456-7890-abcdef123456.jsonl
```

**Path encoding rules** (B9):
1. Prefix with `-` (hyphen)
2. Replace all `/` with `-`
3. Preserve spaces and other characters

Examples:
- `/home/user/project` → `-home-user-project`
- `/home/user/my project` → `-home-user-my project`

**Session files** (`.jsonl`): One per independent invocation (B2). Accumulate without rotation (B6). See [`../jsonl/`](../jsonl/readme.md) for entry schema.

**Agent storage layouts** — two formats coexist (neither deprecated):
- Flat (B7): `agent-*.jsonl` as siblings in project root
- Hierarchical (B13): `{parent-uuid}/subagents/agent-{id}.jsonl`

A root session and its agents form a **Session Family**.

### Growth

- 225 total projects observed (mix of UUID and path types)
- Average project size: ~6.2MB
- Total projects/ size: ~1.4GB (74% of `~/.claude/` total)
- One `.jsonl` file per independent `claude` invocation; no compaction or rotation

**Maintenance**: Cannot be safely deleted without losing conversation history. Old agent sessions (`agent-*.jsonl`, `*/subagents/`) can be deleted if their parent conversations are no longer needed.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Storage master index: conceptual model, full directory structure |
| behavior | [`../behavior/009_b9_storage_path_encoding.md`](../behavior/009_b9_storage_path_encoding.md) | Path encoding rule (`/`→`-`) |
| behavior | [`../behavior/006_b6_session_accumulation.md`](../behavior/006_b6_session_accumulation.md) | Session accumulation without rotation |
| behavior | [`../behavior/007_b7_agent_sessions_sibling.md`](../behavior/007_b7_agent_sessions_sibling.md) | Flat agent layout |
| behavior | [`../behavior/013_b13_subagent_directory.md`](../behavior/013_b13_subagent_directory.md) | Hierarchical agent layout |
| jsonl | [`../jsonl/010_sidechain_sessions.md`](../jsonl/010_sidechain_sessions.md) | Sidechain JSONL entry format and storage layout diagrams |
| source | `../../../../module/claude_storage/src/` | Storage implementation |
