# Storage Doc Entity

### Scope

- **Purpose**: Describe how Claude Code organizes conversation data, settings, and metadata on disk within the `~/.claude/` root.
- **Responsibility**: Master file for the `storage` collection — lists all 3 storage area instances, defines the conceptual model, and declares scope boundaries.
- **In Scope**: Storage root layout, project/session/entry containment hierarchy, agent storage layouts (flat and hierarchical), directory purposes, access patterns, growth characteristics, security considerations, design principles, and the provenance method distinguishing binary-created paths from co-located files owned by other tools.
- **Out of Scope**: Entry-level JSONL field schema (→ [`../jsonl/`](../jsonl/readme.md)); settings and credentials file format internals (→ [`../settings/`](../settings/readme.md), [`../format/`](../format/readme.md)); runtime filesystem paths managed by claude_version (→ [`../filesystem/`](../filesystem/readme.md)).

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
| [002](002_support_directories.md) | Support Directories | 26 binary-confirmed subdirectories — `tasks/` (was `todos/`), `shell-snapshots/`, `session-env/`, `commands/`, `sessions/`, `agents/`, `skills/`, `hooks/`, `debug/`, `cache/`, `paste-cache/`, `plans/`, `plugins/`, `backups/`, `ide/`, `chrome/`, `jobs/`, `daemon/`, `workflows/`, `routines/`, `rules/`, `output-styles/`, `file-history/`, `logs/`, `statsig/` — plus the provenance method |
| [003](003_root_files.md) | Root Files | `history.jsonl`, `.credentials.json`, `settings.json`, `stats-cache.json`, `CLAUDE.md`, `.last-cleanup`, `.last-update-result.json`, `scheduled_tasks.json`, `launch.json`, `daemon.json`, `policy-limits.json` — global files at `~/.claude/` root, and which co-located files are **not** the binary's |

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
├── tasks/                        # Task tracking (renamed from todos/)
├── shell-snapshots/              # Shell environment captures
├── session-env/                  # Session metadata
├── commands/                     # Command definitions
├── sessions/                     # Session tracking metadata
├── agents/                       # Agent configuration and state
├── skills/                       # User-defined skill definitions
├── hooks/                        # Hook script storage
├── debug/                        # Debug logs
├── cache/  paste-cache/          # Caches
├── plans/  plugins/  backups/    # Saved plans, installed plugins, backups
├── file-history/                 # Per-file edit history
├── ide/  chrome/                 # Integration state
├── jobs/  daemon/                # Background execution state
├── workflows/  routines/         # Workflow and routine definitions
├── rules/  output-styles/        # Rule and output-style definitions
├── logs/  statsig/  todos/       # Cleanup-swept (todos/ is legacy)
├── history.jsonl                 # Global project index
├── .credentials.json             # API credentials
├── settings.json                 # User settings
├── stats-cache.json              # Usage statistics cache
├── CLAUDE.md                     # Global instructions
├── .last-cleanup                 # Cleanup sweep timestamp
├── .last-update-result.json      # Self-update outcome
└── scheduled_tasks.json  launch.json  daemon.json  policy-limits.json
```

Directories are created lazily on first use of the owning feature, so any given machine
shows a subset. Every name above is confirmed against the v2.1.220 binary rather than
inferred from one machine's listing — method, controls, and the substring-matching trap
it avoids are in [002_support_directories.md](002_support_directories.md) § Provenance.

`cld-timeout-config.json` appeared in this tree in earlier revisions and has been removed:
it scores 0 against the binary and is not a Claude Code file. See
[003_root_files.md](003_root_files.md) § Not Claude Code's.

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
- `../format/` — ancillary format specs (history.jsonl, debug, shell-snapshots, tasks, commands)

**This entity consumed by**:
- `../../../../module/claude_storage/docs/` — storage implementation docs
- `../../../../module/claude_storage_core/docs/data_structure/001_storage_hierarchy.md` — Session Family data structure
- `../behavior/` — behaviors B2, B6–B9, B12–B15, B22–B23 reference storage layout
