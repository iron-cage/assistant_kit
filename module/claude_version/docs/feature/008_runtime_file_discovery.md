# Feature: Runtime File Discovery

### Scope

- **Purpose**: Document the `.runtime_files` command that enumerates all paths managed by clv at runtime.
- **Responsibility**: Specify the discovery command output format, path resolution, and pipeline-composable behavior.
- **In Scope**: `.runtime_files` command, absolute path output, pipeline-composable format, missing HOME behavior.
- **Out of Scope**: Version history cache content and lifecycle (→ `runtime_file/001_version_history_cache.md`), version history API behavior (→ `feature/001_version_management.md`).

### Design

`.runtime_files` enumerates all paths managed by clv that could exist for the current configuration. Output represents paths that WILL exist after relevant commands run — not only paths currently on disk.

**Output format:**
- One absolute path per line, no headers, no decorations
- Suitable for pipeline composition: `clv .runtime_files | xargs ls -la`
- Trailing newline after the last path

**Path list** (current):

| Path | Owner | Created By |
|------|-------|-----------|
| `$HOME/.claude/.transient/version_history_cache.json` | `.version.history` | `fetch_releases_json()` |

**Exit codes:** 0 (success) | 2 (HOME unset or I/O error)

**Behavior:**
- `$HOME` must be set; missing HOME exits 2
- The command succeeds and outputs paths even when listed files do not yet exist on disk
- No subprocess spawning; paths are computed from environment variables only

### Runtime Files

| File | Relationship |
|------|-------------|
| [runtime_file/001_version_history_cache.md](../runtime_file/001_version_history_cache.md) | Enumerated by this command |

### Sources

| File | Relationship |
|------|-------------|
| `../../src/commands/runtime_files.rs` | .runtime_files command handler |

### Provenance

| Source | Notes |
|--------|-------|
| `l0_gov.rulebook.md § CLI : Runtime File Discovery Mandate` | Mandated when docs/runtime_file/ collection is created |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/feature/008_runtime_file_discovery.md](../../tests/docs/feature/008_runtime_file_discovery.md) | Feature test spec |
