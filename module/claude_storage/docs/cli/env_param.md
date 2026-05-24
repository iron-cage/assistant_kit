# Environment Parameters

Environment variables that influence `claude_storage` CLI behavior. These are read at startup and may be overridden by explicit CLI parameters.

## Environment Variable Catalog

| Variable | Purpose | Default | Precedence |
|----------|---------|---------|------------|
| `CLAUDE_STORAGE_ROOT` | Override the storage root directory | `~/.claude/` | CLI `path::` > env var > default |

---

### `CLAUDE_STORAGE_ROOT`

**Purpose:** Sets the storage root directory that the CLI reads from when no explicit `path::` parameter is given. Primarily used for test isolation and non-standard installations.

**Default:** `~/.claude/`

**Precedence:** An explicit `path::` CLI parameter always wins. If `CLAUDE_STORAGE_ROOT` is set and non-empty, it overrides the default `~/.claude/`. An empty string is treated as unset (falls back to default).

**Affected commands:** All commands that access storage (`.status`, `.list`, `.show`, `.count`, `.search`, `.export`, `.projects`, `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure`) — except those where `path::` is supplied directly by the caller.

**Source:** `src/cli/storage.rs::create_storage`

**Example:**
```bash
export CLAUDE_STORAGE_ROOT=/backup/.claude
cls .status
# Reads from /backup/.claude instead of ~/.claude/
```
