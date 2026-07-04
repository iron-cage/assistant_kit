# Format Doc Entity

### Scope

- **Purpose**: Specify data formats produced or consumed by Claude Code — file formats for ancillary storage files and output schemas for CLI response encoding.
- **Responsibility**: Master file for the `formats` collection — lists all 7 format instances covering history, credentials, debug logs, shell snapshots, todos, command definitions, and JSON response output.
- **In Scope**: history.jsonl entry structure, .credentials.json structure, debug log format, shell snapshot format, todos JSON format, command definition markdown format, `--output-format json` response schema.
- **Out of Scope**: Session JSONL format (→ [`../jsonl/`](../jsonl/readme.md)); settings.json structure (→ [`../settings/`](../settings/readme.md)); directory layout and storage areas (→ [`../storage/`](../storage/readme.md)).

### Overview Table

| ID | Name | Responsibility |
|----|------|----------------|
| [001](001_history_jsonl.md) | History JSONL | `~/.claude/history.jsonl` — global project access index; one entry per conversation start |
| [002](002_credentials.md) | Credentials | `~/.claude/.credentials.json` — API authentication tokens; `claudeAiOauth` object |
| [003](003_debug_log.md) | Debug Log | `~/.claude/debug/*.txt` — plain text debug output; `[DEBUG]` prefix per line; append-only |
| [004](004_shell_snapshot.md) | Shell Snapshot | `~/.claude/shell-snapshots/{uuid}.sh` — bash script preserving shell environment for session restoration |
| [005](005_todo.md) | Todo | `~/.claude/todos/{session-uuid}.json` — JSON array of task objects for conversation sessions |
| [006](006_command_definition.md) | Command Definition | `~/.claude/commands/{name}.md` — markdown documents defining custom slash commands |
| [007](007_json_response.md) | JSON Response | `--output-format json` stdout — single JSON object with message, content blocks, and usage |

### Format Comparison

| Format | Size/Entry | Parsing | Mutability | Purpose |
|--------|-----------|---------|------------|---------|
| `history.jsonl` | ~250B | Line-by-line | Append-only | Project tracking |
| `.credentials.json` | ~1KB | Full parse | Overwrite | API authentication |
| `debug/*.txt` | ~100B | Line-by-line | Append-only | Logging |
| `shell-snapshots/*.sh` | 5KB–500KB | Source | Create-once | Environment restore |
| `todos/*.json` | 1KB–5MB | Full parse | Overwrite | Task tracking |
| `commands/*.md` | 1KB–50KB | Text | Static | Custom commands |
| JSON response | 200B–500KB | Full parse | Transient (stdout) | Structured response |

### Type-Specific Requirements

All `formats` doc instances must include:

1. **Title**: `# Format: {File Type Name}` — using `Format` as the type prefix
2. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
3. **Location** (H3): File path pattern and storage area
4. **Schema** (H3): Field table or structured example illustrating the format
5. **Growth** (H3): How the file grows over time and maintenance guidance
6. **Cross-References** (H3): Flat table with `Type | File | Responsibility` columns

### Cross-Collection Dependencies

**This entity depends on**:
- `../storage/` — directory locations for all ancillary format files

**This entity consumed by**:
- `../../../../module/claude_storage/docs/` — storage implementation docs
- `../behavior/` — B8 references zero-byte JSONL (related storage pattern)
