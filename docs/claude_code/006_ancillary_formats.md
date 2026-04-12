# Claude Code: Ancillary Formats

### Scope

- **Purpose**: Specify the file formats for ancillary Claude Code storage files that are not covered by the primary session or settings docs.
- **Responsibility**: Authoritative schema for history.jsonl, credentials, debug logs, shell snapshots, todos, and command definition formats.
- **In Scope**: history.jsonl entry structure, .credentials.json structure, debug log format, shell snapshot format, todos JSON format, command definition markdown format, format comparison table.
- **Out of Scope**: Session JSONL format (→ [004_jsonl_format.md](004_jsonl_format.md)); settings.json structure (→ [005_settings_format.md](005_settings_format.md)); directory layout and sizes (→ [002_storage_organization.md](002_storage_organization.md)).

---

### History Format (history.jsonl)

**File location**: `~/.claude/history.jsonl`

**Format**: Line-delimited JSON — one entry per line.

**Purpose**: Track project access history and context across all sessions.

**Entry structure**:
```json
{
  "display": "https://www.youtube-transcript.io/api\nread page...",
  "pastedContents": {},
  "timestamp": 1758992388766,
  "project": "/home/user1/pro/lib/willbe/module/reasoner"
}
```

**Field specifications**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `display` | string | yes | User query or context preview |
| `pastedContents` | object | yes | Pasted file contents (usually empty) |
| `timestamp` | number | yes | Unix timestamp (milliseconds) |
| `project` | string | yes | Filesystem path to project |

**Growth**: Appends one entry per conversation start (~4324 entries observed, ~254 bytes/entry average, ~1.1MB total).

### Credentials Format (.credentials.json)

**File location**: `~/.claude/.credentials.json`

**Format**: Single JSON object.

**Purpose**: Store API authentication tokens.

**Structure**:
```json
{
  "claudeAiOauth": {
    "... (authentication data)"
  }
}
```

**Security**: Sensitive file. Recommended permissions: `chmod 600 ~/.claude/.credentials.json`

### Debug Log Format (debug/*.txt)

**File location**: `~/.claude/debug/*.txt`

**Format**: Plain text, line-oriented.

**Line format**: `[DEBUG] message`

**Example entries**:
```
[DEBUG] Watching for changes in setting files...
[DEBUG] Found 0 plugins (0 enabled, 0 disabled)
[DEBUG] Total LSP servers loaded: 0
[DEBUG] Creating shell snapshot at /home/user1/.claude/shell-snapshots/...
```

**Content types**: Setting file watching, plugin loading, LSP server init, shell snapshot creation, process lifecycle events.

**Growth**: Continuous append during Claude Code operation. Can grow to 100MB+ over time. Safe to delete or rotate.

### Shell Snapshot Format (shell-snapshots/*.sh)

**File location**: `~/.claude/shell-snapshots/{uuid}.sh`

**Format**: Executable bash script.

**Purpose**: Preserve and restore shell environment when resuming sessions.

**Structure**:
```bash
# Snapshot file
# Unset all aliases to avoid conflicts with functions
unalias -a 2>/dev/null || true

# Functions
eval "$(echo 'Z2F3a2xpYnBhdGhfYXBwZW5k...' | base64 -d)"

# Shell options
shopt -s checkwinsize
shopt -s cmdhist
shopt -s expand_aliases
...
```

**Components**:
1. **Alias cleanup**: `unalias -a 2>/dev/null || true`
2. **Function restoration**: `eval "$(echo 'BASE64_ENCODED_FUNCTIONS' | base64 -d)"`
3. **Shell options**: `shopt -s option_name` / `shopt -u option_name`

**Encoding**: Bash functions base64-encoded to preserve complex syntax.

**File naming**: UUID matches session ID. Size: 5KB – 500KB per snapshot.

**Growth**: One file per CLI session with shell context.

### Todo Format (todos/*.json)

**File location**: `~/.claude/todos/{session-uuid}.json`

**Format**: JSON array of task objects.

**Purpose**: Track tasks for conversation sessions.

**Structure**:
```json
[
  {
    "content": "Plan comprehensive manual testing strategy",
    "status": "completed",
    "activeForm": "Planning comprehensive manual testing strategy"
  },
  {
    "content": "Implement manual test runner",
    "status": "in_progress",
    "activeForm": "Implementing manual test runner"
  },
  {
    "content": "Write test documentation",
    "status": "pending",
    "activeForm": "Writing test documentation"
  }
]
```

**Task object specification**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | string | yes | Task description (imperative form) |
| `status` | string | yes | Task state: `pending`, `in_progress`, `completed` |
| `activeForm` | string | yes | Task description (present continuous form) |

**File organization**: One file per session UUID. Updates on task status changes.

### Command Definition Format (commands/*.md)

**File location**: `~/.claude/commands/{command-name}.md`

**Format**: Markdown document.

**Purpose**: Define custom slash commands available as `/{command-name}` in Claude Code sessions.

**Example** (`commit.md`):
```markdown
Act as an expert git assistant. Analyze the repository state, manage the staging area, and execute commits with high-quality, conventional commit messages.

## Instructions

1. Run git status to see changes
2. Run git diff to understand modifications
3. Draft commit message following conventions
4. Execute git commit with message
...
```

**Structure**: Free-form markdown with instructions.

**File count**: 48 command definitions observed.

**Examples**: `commit.md`, `refactor_extracting.md`, `test_clean.md`

### Format Comparison

| Format | Size/Entry | Parsing | Mutability | Purpose |
|--------|-----------|---------|------------|---------|
| `.jsonl` (session) | 500B–5KB | Line-by-line | Append-only | Conversations — see [004_jsonl_format.md](004_jsonl_format.md) |
| `history.jsonl` | ~250B | Line-by-line | Append-only | Project tracking |
| `.json` (settings) | 1KB–5MB | Full parse | Overwrite | Configuration — see [005_settings_format.md](005_settings_format.md) |
| `.json` (todos) | 1KB–5MB | Full parse | Overwrite | Task tracking |
| `.txt` (debug) | ~100B | Line-by-line | Append-only | Logging |
| `.sh` | 5KB–500KB | Source | Create-once | Environment |
| `.md` | 1KB–50KB | Text | Static | Commands |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`004_jsonl_format.md`](004_jsonl_format.md) | Primary session JSONL entry schema |
| doc | [`005_settings_format.md`](005_settings_format.md) | Settings.json and credentials structure |
| doc | [`002_storage_organization.md`](002_storage_organization.md) | Directory layout, sizes, and growth characteristics |
