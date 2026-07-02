# Invariant: Session Family

### Scope

- **Purpose**: Define the session family grouping contract.
- **Responsibility**: What constitutes a family, flat vs hierarchical storage layouts.
- **In Scope**: Root session identification, agent session association, both storage layouts.
- **Out of Scope**: Entry parsing, token counting, display formatting.

### Statement

A **Session Family** is a root session plus all agent sessions it spawned. Agent sessions
are always subordinate to exactly one root session. The family grouping is invariant
regardless of storage layout (flat or hierarchical).

### Storage Layouts

| Layout | Detection | Agent Location |
|--------|-----------|----------------|
| Flat (older, B7) | No `{uuid}/` subdirectory present | `agent-{id}.jsonl` at project root |
| Hierarchical (newer, B13) | `{uuid}/subagents/` directory exists | `{uuid}/subagents/agent-{id}.jsonl` |

Both layouts coexist within a single project directory. Neither is deprecated.

### Contract

- Every `agent-*.jsonl` file belongs to exactly one root session
- Root sessions are identified by UUID-format filenames (`{uuid}.jsonl`)
- In flat layout: agent-to-root mapping is established via the `sessionId` field in the first agent entry
- In hierarchical layout: agent-to-root mapping is established by directory structure (`{uuid}/subagents/`)
- Both layouts must be discovered and supported simultaneously

### Violation Conditions

- Treating agent sessions as top-level conversations (breaks family grouping)
- Assuming only one storage layout exists per project (both coexist)
- Using `sessionId` in hierarchical agents to establish family membership (use directory structure instead)
- Displaying root and agent sessions as siblings without family grouping

### Referenced Commands

| # | Command | Context |
|---|---------|---------|
| 2 | [`.list`](../cli/command/02_list.md) | Lists sessions with optional agent inclusion |
| 3 | [`.show`](../cli/command/03_show.md) | Displays a single session (root or agent) |
| 7 | [`.projects`](../cli/command/07_projects.md) | Groups sessions into families for tree display |

### Sources

- [`guide/001_advanced_storage_topics.md § Agent Sessions`](../guide/001_advanced_storage_topics.md) — layout examples and detection algorithms
- `claude_storage_core` — `SessionFamily` Domain Type definition
