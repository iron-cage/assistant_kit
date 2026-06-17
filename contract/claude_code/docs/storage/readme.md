# Storage

### Scope

- **Purpose**: Describe how Claude Code organizes conversation data, settings, and metadata on disk within the `~/.claude/` root.
- **Responsibility**: Master file for the `storage` collection — lists all 3 storage area instances, defines the conceptual model, and declares scope boundaries.
- **In Scope**: Storage root layout, project/session/entry containment hierarchy, agent storage layouts (flat and hierarchical), directory purposes, access patterns, growth characteristics, security considerations, design principles.
- **Out of Scope**: Entry-level JSONL field schema (→ [`../jsonl/`](../jsonl/readme.md)); settings and credentials file format internals (→ [`../settings/`](../settings/readme.md), [`../formats/`](../formats/readme.md)); runtime filesystem paths managed by claude_version (→ [`../filesystem/`](../filesystem/readme.md)).

### Conceptual Model

Claude Code stores all conversation data, settings, and metadata in `~/.claude/` using filesystem-native architecture.

**Storage model**: Append-only JSONL files organized into project/session hierarchy.

**Key characteristics**:
- Single source of truth (no caching)
- Filesystem-native (no database engine)
- Human-readable formats (JSONL, JSON)
- Append-only write pattern
- No schema migrations required

**Four-level containment hierarchy** from storage root to individual message payload:

```
Storage Root  (~/.claude/)
└── Project      (one directory per filesystem path or UUID)
    └── Session  (one .jsonl file — the physical storage unit)
        └── Entry  (one line per turn)
            ├── [envelope]  uuid, parentUuid, timestamp, sessionId, isSidechain, cwd, gitBranch
            └── message     (Claude API Message payload)
                ├── role     "user" | "assistant"
                ├── content  text / tool_use / tool_result / thinking blocks
                ├── model    (assistant only)
                └── usage    (assistant only)
```

### Overview Table

| ID | Name | Responsibility |
|----|------|----------------|
| [001](001_projects_directory.md) | Projects Directory | `projects/` — conversation storage; UUID and path projects; flat and hierarchical agent layouts |
| [002](002_support_directories.md) | Support Directories | `debug/`, `todos/`, `shell-snapshots/`, `session-env/`, `commands/` — operational support storage |
| [003](003_root_files.md) | Root Files | `history.jsonl`, `.credentials.json`, `settings.json` — global files at `~/.claude/` root |

### Directory Structure

```
~/.claude/                        # Root storage
├── projects/                     # All conversation projects
│   ├── {uuid}/                   # UUID projects (web/IDE sessions)
│   │   ├── {session-id}.jsonl   # Main conversation
│   │   ├── agent-{id}.jsonl     # Sub-agent sessions (flat format)
│   │   └── {session-id}/        # Session family directory (hierarchical format)
│   │       ├── subagents/
│   │       │   ├── agent-{id}.jsonl      # Agent session
│   │       │   └── agent-{id}.meta.json  # Agent metadata
│   │       └── tool-results/    # Tool output artifacts
│   └── -{path-encoded}/          # Path projects (CLI sessions)
│       └── {session-id}.jsonl   # CLI conversation
├── debug/                        # Debug logs
├── todos/                        # Task tracking
├── shell-snapshots/              # Shell environment captures
├── session-env/                  # Session metadata
├── commands/                     # Command definitions
├── history.jsonl                 # Global project index
├── .credentials.json             # API credentials
└── settings.json                 # User settings
```

### Type-Specific Requirements

All `storage` doc instances must include:

1. **Title**: `# Storage: {Area Name}` — using `Storage` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Structure** (H3): Directory tree or file layout for the storage area
4. **Contents** (H3): Purpose, format, and growth characteristics of each item
5. **Cross-References** (H3): Flat table with `Type | File | Responsibility` columns

### Cross-Collection Dependencies

**This entity depends on**:
- `../jsonl/` — entry schema for session JSONL files
- `../settings/` — settings.json and credentials format
- `../formats/` — ancillary format specs (history.jsonl, debug, shell-snapshots, todos, commands)

**This entity consumed by**:
- `../../../../module/claude_storage/docs/` — storage implementation docs
- `../../../../module/claude_storage_core/docs/data_structure/001_storage_hierarchy.md` — Session Family data structure
- `../behavior/` — behaviors B2, B6–B9, B12–B15, B22–B23 reference storage layout
