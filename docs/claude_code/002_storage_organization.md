# Claude Code: Storage Organization

### Scope

- **Purpose**: Describe how Claude Code organizes conversation data, settings, and metadata on disk.
- **Responsibility**: Authoritative reference for the `~/.claude/` storage architecture, directory purposes, containment hierarchy, and access/growth patterns.
- **In Scope**: Storage root layout, project/session/entry hierarchy, agent storage layouts (flat and hierarchical), directory purposes, path encoding, access patterns, security considerations, design principles.
- **Out of Scope**: Entry-level JSONL field schema (→ [004_jsonl_format.md](004_jsonl_format.md)); file format internals for settings/credentials (→ [005_settings_format.md](005_settings_format.md)); runtime filesystem paths managed by claude_version (→ [003_filesystem_layout.md](003_filesystem_layout.md)).

---

### Conceptual Model

Claude Code stores all conversation data, settings, and metadata in `~/.claude/` directory using filesystem-native architecture.

**Storage model**: Append-only JSONL files organized into project/session hierarchy.

**Key characteristics**:
- Single source of truth (no caching)
- Filesystem-native (no database engine)
- Human-readable formats (JSONL, JSON)
- Append-only write pattern
- No schema migrations required

Four-level containment hierarchy from storage root to individual message payload:

```
Storage Root  (~/.claude/)
└── Project      (one directory per filesystem path or UUID)
    └── Session  (one .jsonl file — the physical storage unit)
        └── Entry  (one line per turn)
            ├── [envelope]  uuid, parentUuid, timestamp,
            │               sessionId, isSidechain, cwd, gitBranch
            └── message     (Claude API Message payload)
                ├── role     "user" | "assistant"
                ├── content  text / tool_use / tool_result / thinking blocks
                ├── model    (assistant only)
                └── usage    (assistant only)
```

**User-facing layer**: The `claude_storage` tool introduces **Conversation** as the user-facing concept sitting between Project and Session. One Conversation corresponds to one Session (root file) in the current implementation; in future it may span a chain of sessions. See [007_concept_taxonomy.md](007_concept_taxonomy.md) for the full four-level user/storage taxonomy.

Two agent storage layouts coexist (format is per-project, neither is deprecated).

**Flat agent layout (older projects, B7):**
```
projects/{project-id}/
├── {session-id}.jsonl        ← Main Session   (isSidechain: false)
└── agent-{id}.jsonl          ← Agent Session  (isSidechain: true)
```

**Hierarchical agent layout (newer projects, B13):**
```
projects/{project-id}/
├── {session-id}.jsonl              ← Root Session
└── {session-id}/
    ├── subagents/
    │   ├── agent-{id}.jsonl        ← Agent Session
    │   └── agent-{id}.meta.json    ← Agent Metadata (B14)
    └── tool-results/               ← Tool Output Artifacts
```

A root session and its agents form a **Session Family** (see [data_structure/001_storage_hierarchy.md](../../module/claude_storage_core/docs/data_structure/001_storage_hierarchy.md)). See [001_session_behaviors.md](001_session_behaviors.md) B7, B12–B15 for evidence.

Entries within a Session are threaded by `parentUuid` into a conversation chain:

```
Entry 1 (user)       uuid=A  parentUuid=null
  └── Entry 2 (assistant)    uuid=B  parentUuid=A
        └── Entry 3 (user)   uuid=C  parentUuid=B
              └── Entry 4 (assistant)  uuid=D  parentUuid=C
```

For Entry field schema and content block types see [004_jsonl_format.md](004_jsonl_format.md).

### Directory Structure

```
~/.claude/                        # Root storage (1.9GB total)
├── projects/                     # 1.4GB - All conversation projects
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
├── debug/                        # 429MB - Debug logs
├── todos/                        # 63MB - Task tracking
├── shell-snapshots/              # 45MB - Shell environment captures
├── session-env/                  # 2.2MB - Session metadata
├── commands/                     # <1MB - Command definitions
├── history.jsonl                 # 1.1MB - Global project index
├── .credentials.json             # API credentials
└── settings.json                 # User settings
```

### Directory Purposes

#### projects/ - conversation storage (1.4GB)

**Purpose**: Primary storage for all Claude Code conversations.

**Organization**: Two project types based on launch context.

**UUID projects** (web/IDE sessions):
```
projects/26dd749d-5b4b-bfee-f4f3-9e03803b8cad/
├── 8d795a1c-c81d-4010-8d29-b4e678272419.jsonl    # Main session
├── agent-f3e2d1c4-a5b6-8910-1234-567890abcdef.jsonl  # Sub-agent
└── agent-a1b2c3d4-e5f6-7890-abcd-ef1234567890.jsonl  # Sub-agent
```

**Path projects** (CLI sessions):
```
projects/-home-user1-pro-lib-consumer-module-wplan_agent/
├── 3a4b5c6d-e7f8-9012-3456-789abcdef012.jsonl    # Session 1
└── 7e8f9a0b-c1d2-3456-7890-abcdef123456.jsonl    # Session 2
```

**Path encoding rules**:
1. Prefix with `-` (hyphen)
2. Replace all `/` with `-`
3. Preserve spaces and other characters

**Examples**:
- `/home/user/project` → `-home-user-project`
- `/home/user/my project/code` → `-home-user-my project-code`

**Growth characteristics**:
- 225 total projects discovered
- Mix of UUID (web/IDE) and path (CLI) projects
- Average project size: ~6.2MB

#### debug/ - debug logs (429MB)

**Purpose**: Debug output from Claude Code operations.

**File format**: Plain text with `[DEBUG]` prefix per line.

**Content types**: Setting file watching, plugin loading, LSP server init, shell snapshot creation, process lifecycle events.

**Maintenance**: Can be safely deleted to reclaim space (no impact on conversations).

#### todos/ - task tracking (63MB)

**Purpose**: Store todo lists for conversation sessions.

**File organization**: One JSON file per session UUID.

**File format**: JSON array of task objects (`content`, `status`, `activeForm`).

#### shell-snapshots/ - environment captures (45MB)

**Purpose**: Preserve shell environment for session restoration.

**File format**: Bash script with base64-encoded functions.

**File naming**: UUID-based (matches session IDs).

#### session-env/ - session metadata (2.2MB)

**Purpose**: Store session-specific metadata.

**Organization**: Empty directories named by session UUID (current status: directories exist but are empty).

#### commands/ - command definitions

**Purpose**: Store custom command definitions for Claude Code.

**File format**: Markdown files (`.md`). 48 command files observed.

#### history.jsonl - global project index (1.1MB)

**Purpose**: Track all project accesses and context across sessions.

**Entry structure**:
```json
{
  "display": "https://www.youtube-transcript.io/api\nread page...",
  "pastedContents": {},
  "timestamp": 1758992388766,
  "project": "/home/user1/pro/lib/consumer/module/reasoner"
}
```

**Growth**: Appends one entry per conversation start (~4324 entries, ~250 bytes/entry).

### File Formats Summary

| Format | Usage | Size Range | Growth Pattern |
|--------|-------|------------|----------------|
| `.jsonl` | Conversations | 10KB – 50MB | Append-only per session |
| `.json` | Settings, todos, credentials | 1KB – 5MB | Overwrite updates |
| `.txt` | Debug logs | 1KB – 100MB | Continuous append |
| `.sh` | Shell snapshots | 5KB – 500KB | One per session |
| `.md` | Commands | 1KB – 50KB | Static definitions |

### Storage Characteristics

**Total size distribution**:
```
Total: ~1.9GB
├── projects/          1.4GB (74%)
├── debug/             429MB (23%)
├── todos/             63MB  (3%)
├── shell-snapshots/   45MB  (<1%)
├── session-env/       2.2MB (<1%)
└── other files        <1MB  (<1%)
```

**Maintenance**:
- **Safe to delete**: `debug/`, old `shell-snapshots/`
- **Never delete**: `projects/`, `history.jsonl`, `.credentials.json`, `settings.json`

### Access Patterns

**High frequency**: Current session `.jsonl`, `settings.json`, `.credentials.json`

**Medium frequency**: `history.jsonl`, `todos/{session}.json`, `shell-snapshots/{session}.sh`

**Low frequency**: Old sessions, `debug/`, `commands/`

**Append-only**: `projects/{project}/{session}.jsonl`, `history.jsonl`, `debug/*.txt`

**Overwrite**: `settings.json`, `todos/{session}.json`

### Security Considerations

**High sensitivity**: `.credentials.json` (API tokens), `projects/` (may contain proprietary code/data)

**Medium sensitivity**: `history.jsonl` (project paths), `shell-snapshots/` (env vars)

**Recommended permissions**:
```
chmod 700 ~/.claude/
chmod 600 ~/.claude/.credentials.json
chmod 700 ~/.claude/projects/
chmod 644 ~/.claude/settings.json
```

### Design Principles

**Filesystem-native**: No database engine — all data stored in plain files (JSONL, JSON, text). Standard tools work (jq, grep, less). No schema migrations required.

**Append-only writes**: Conversation files only append, never modify existing entries. No data loss from corruption; atomic appends preserve audit trail.

**Single source of truth**: Always read from disk. No cache invalidation bugs; multiple processes can read safely.

**Lazy loading**: Only load session entries when explicitly requested. Fast session listing; low memory footprint.

### Comparison with Traditional Databases

| Aspect | Claude Code Storage | Traditional DB |
|--------|---------------------|----------------|
| Format | JSONL files | Binary/proprietary |
| Schema | Implicit (JSON) | Explicit (DDL) |
| Migrations | Not needed | Required |
| Tools | jq, grep, less | SQL client |
| Concurrency | File locks | MVCC/locks |
| Scale | Conversation-scale | Production-scale |
| Backup | cp/rsync | Dump/restore |
| Query | Sequential scan | Indexed |

**Appropriate for**: Conversation history (read-mostly, modest scale, human-readable priority).

**Not appropriate for**: High-frequency writes, complex queries, multi-GB datasets.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`001_session_behaviors.md`](001_session_behaviors.md) | Session behavior evidence (B1–B16h) including agent layouts |
| doc | [`004_jsonl_format.md`](004_jsonl_format.md) | Entry-level JSONL field schema and content blocks |
| doc | [`005_settings_format.md`](005_settings_format.md) | Settings.json structure and atomic write protocol |
| doc | [`003_filesystem_layout.md`](003_filesystem_layout.md) | claude_version runtime path reference table |
| doc | [`../../module/claude_storage_core/docs/data_structure/001_storage_hierarchy.md`](../../module/claude_storage_core/docs/data_structure/001_storage_hierarchy.md) | Session Family data structure |
| source | [`../../module/claude_storage/src/`](../../module/claude_storage/src/) | Storage layer implementation |
