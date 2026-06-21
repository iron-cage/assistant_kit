# Storage: Support Directories

### Scope

- **Purpose**: Document the operational support directories in `~/.claude/` that store debug logs, task tracking, shell environment, session metadata, and command definitions.
- **Responsibility**: Authoritative instance for the 9 support directories — purpose, format, growth characteristics, and maintenance guidance for each.
- **In Scope**: `debug/`, `todos/`, `shell-snapshots/`, `session-env/`, `commands/`, `sessions/`, `agents/`, `skills/`, `hooks/`.
- **Out of Scope**: `projects/` (conversation storage, → [001_projects_directory.md](001_projects_directory.md)); global root files (→ [003_root_files.md](003_root_files.md)); file format internals (→ [`../formats/`](../formats/readme.md)).

### Structure

```
~/.claude/
├── debug/                    # 429MB - Debug log files
├── todos/                    # 63MB - Per-session task JSON files
├── shell-snapshots/          # 45MB - Session shell environment captures
├── session-env/              # 2.2MB - Session metadata (empty directories)
├── commands/                 # <1MB - Custom slash command definitions
├── sessions/                 # Session tracking metadata
├── agents/                   # Agent configuration and state
├── skills/                   # User-defined skill definitions
└── hooks/                    # Hook script storage
```

### Contents

#### debug/ — Debug Logs (429MB)

**Purpose**: Debug output from Claude Code operations.
**Format**: Plain text; one `[DEBUG] message` line per log entry.
**Growth**: Continuous append during operation. Can grow to 100MB+ per file over time.
**Maintenance**: Safe to delete entirely. No impact on conversations or settings.

Content types: setting file watching, plugin loading, LSP server init, shell snapshot creation, process lifecycle events.

See [`../formats/003_debug_log.md`](../formats/003_debug_log.md) for format spec.

#### todos/ — Task Tracking (63MB)

**Purpose**: Store todo lists for conversation sessions.
**File organization**: One JSON file per session UUID: `todos/{session-uuid}.json`.
**Format**: JSON array of task objects with `content`, `status`, `activeForm` fields.
**Growth**: One file per active session; updated on task status changes.
**Maintenance**: Can be deleted if corresponding sessions are no longer needed.

See [`../formats/005_todo.md`](../formats/005_todo.md) for format spec.

#### shell-snapshots/ — Shell Environment Captures (45MB)

**Purpose**: Preserve shell environment for session restoration.
**File naming**: UUID matches session ID: `shell-snapshots/{session-uuid}.sh`.
**Format**: Executable bash script; functions base64-encoded to preserve complex syntax.
**Growth**: One file per CLI session with shell context. Size: 5KB–500KB per snapshot.
**Maintenance**: Old snapshots can be deleted safely; only affects ability to restore old sessions.

See [`../formats/004_shell_snapshot.md`](../formats/004_shell_snapshot.md) for format spec.

#### session-env/ — Session Metadata (2.2MB)

**Purpose**: Store session-specific metadata.
**Current status**: Empty directories named by session UUID. No files observed — directories exist as placeholders.
**Growth**: One empty directory per session (minimal disk impact).

#### commands/ — Command Definitions (<1MB)

**Purpose**: Store custom slash command definitions available as `/{command-name}` in Claude Code sessions.
**File format**: Markdown files (`.md`): 48 command files observed.
**Examples**: `commit.md`, `refactor_extracting.md`, `test_clean.md`
**Growth**: Static — only grows when user adds new custom commands.
**Maintenance**: Do not delete unless removing custom commands intentionally.

See [`../formats/006_command_definition.md`](../formats/006_command_definition.md) for format spec.

#### sessions/ — Session Tracking Metadata

**Purpose**: Store session-level tracking data and metadata independent of conversation content.
**File organization**: Session tracking files keyed by session UUID.
**Growth**: One entry per session.
**Maintenance**: Safe to delete; does not affect conversation history.

#### agents/ — Agent Configuration and State

**Purpose**: Store agent configuration — registered custom agents and their definitions.
**File format**: JSON files defining agent properties (description, prompt, model).
**Growth**: Static — grows only when user registers new agents via `claude agents` subcommand.
**Maintenance**: Safe to delete individual agent configs; agent will need to be re-registered.

#### skills/ — User-Defined Skill Definitions

**Purpose**: Store user-created skills (slash commands) beyond the built-in skill set. Distinct from `commands/` which stores the markdown-format command definitions — `skills/` stores skill metadata and registration.
**Growth**: Static — grows only when user creates new skills.
**Maintenance**: Safe to delete; skills will need to be re-created.

#### hooks/ — Hook Script Storage

**Purpose**: Store hook scripts referenced by the `hooks` settings configuration. Scripts executed at `PreToolUse`, `PostToolUse`, and `UserPromptSubmit` lifecycle events.
**Growth**: Static — grows only when user creates new hook scripts.
**Maintenance**: Do not delete while hooks are active in settings; will cause hook execution failures.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Storage master index: full directory structure |
| formats | [`../formats/003_debug_log.md`](../formats/003_debug_log.md) | Debug log `[DEBUG]` line format |
| formats | [`../formats/004_shell_snapshot.md`](../formats/004_shell_snapshot.md) | Shell snapshot bash script format |
| formats | [`../formats/005_todo.md`](../formats/005_todo.md) | Todo JSON array format |
| formats | [`../formats/006_command_definition.md`](../formats/006_command_definition.md) | Command definition markdown format |
