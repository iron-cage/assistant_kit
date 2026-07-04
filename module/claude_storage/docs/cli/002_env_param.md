# Environment Parameters

### Scope

- **Purpose**: Catalog environment variables influencing CLI behavior.
- **Responsibility**: Variable names, defaults, and precedence rules.
- **In Scope**: `HOME`, `CLAUDE_STORAGE_ROOT`, and any future env vars.
- **Out of Scope**: CLI parameters (→ `param/`), config files (not applicable).

Environment variables that influence `claude_storage` CLI behavior. These are read at startup and may be overridden by explicit CLI parameters.

### Environment Variable Catalog

| Variable | Purpose | Default | Precedence |
|----------|---------|---------|------------|
| `HOME` | User home directory — base for default storage root | (system) | System-provided; not overridable by CLI |
| `CLAUDE_STORAGE_ROOT` | Override the storage root directory | `$HOME/.claude/` | CLI `path::` > env var > default |

---

### `HOME`

**Purpose:** Standard Unix user home directory. The CLI uses `HOME` to derive the default storage root (`$HOME/.claude/`) and to compress absolute paths to `~/...` in display output.

**Default:** System-provided (typically set by the login shell).

**Affected commands:** All commands. If `HOME` is unset, the CLI exits with an error unless `CLAUDE_STORAGE_ROOT` or an explicit `path::` provides the storage root.

**Source:** `src/cli/storage.rs::create_storage`, `src/cli/projects.rs::compress_home`

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

### Provenance

| File | Notes |
|------|-------|
| [env_param.md](env_param.md) | Original un-migrated source; retained as reference |
