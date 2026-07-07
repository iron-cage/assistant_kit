# Feature: Path Discovery

### Scope

- **Purpose**: Document the `.paths` command that reports all filesystem paths clv reads from or writes to.
- **Responsibility**: Specify the command's output format, path resolution via `ClaudeVersionPaths`, single-key mode, and verbosity-tiered unresolvable-path handling.
- **In Scope**: `.paths` command, `ClaudeVersionPaths` struct, labeled/unlabeled output modes, `key::` single-path lookup, unresolvable path handling.
- **Out of Scope**: Unlabeled pipeline-only enumeration (→ `feature/008_runtime_file_discovery.md`), individual runtime file lifecycle and ownership (→ `runtime_file/`), settings resolution algorithm (→ `algorithm/002_config_resolution.md`).

### Design

`.paths` reports every filesystem path clv is aware of — both paths clv itself creates/manages (versions directory, binary symlink, version history cache) and paths clv only reads (user and project `settings.json`, owned by the external Claude Code system). It is built on `ClaudeVersionPaths`, a `claude_version_core` struct that composes over `claude_core::ClaudePaths` (which supplies `settings_file()`) and adds four version-specific resolvers: `versions_dir()`, `binary_symlink()`, `version_history_cache_file()`, and `project_settings_file(cwd)`.

**Output format:**
- show-all mode (no `key::`): one labeled path per line, `<key>:  <path>` (v::1, default), or plain unlabeled paths only (v::0), or labeled with a one-line description (v::2)
- single mode (`key::K`): the resolved path for that key alone, in the same label/plain/described style
- At v::0, a path that failed to resolve (e.g. no project config found for `project_settings`) is omitted entirely — the output stays pipeline-composable
- At v::1/v::2, an unresolved path is shown with a `(none found)` placeholder instead of being silently dropped

**Path list** (current, 5 keys):

| Key | Path | Owner |
|-----|------|-------|
| `settings` | `$HOME/.claude/settings.json` | External Claude Code system |
| `project_settings` | `<cwd>/.claude/settings.json` (nearest ancestor) | External Claude Code system |
| `versions_dir` | `$HOME/.local/share/claude/versions` | `perform_install()`, `purge_stale_versions()` |
| `binary_symlink` | `$HOME/.local/bin/claude` | `hot_swap_binary()` |
| `version_history_cache` | `$HOME/.claude/.transient/version_history_cache.json` | `fetch_releases_json()` |

**Exit codes:** 0 (success) | 1 (invalid `key::` value) | 2 (HOME unset)

**Behavior:**
- `$HOME` must be set; missing HOME exits 2
- The command succeeds and outputs paths even when the underlying file does not yet exist on disk (matching `.runtime_files`' existing convention)
- `project_settings` is the only key that can be legitimately absent (no project config found); this is not an error and does not affect the exit code
- No subprocess spawning; paths are computed from environment variables and directory traversal only

### Runtime Files

| File | Relationship |
|------|-------------|
| [runtime_file/002_versions_directory.md](../runtime_file/002_versions_directory.md) | Enumerated by this command via `versions_dir` |
| [runtime_file/003_binary_symlink.md](../runtime_file/003_binary_symlink.md) | Enumerated by this command via `binary_symlink` |
| [runtime_file/001_version_history_cache.md](../runtime_file/001_version_history_cache.md) | Enumerated by this command via `version_history_cache` |

### Sources

| File | Relationship |
|------|-------------|
| `../../src/commands/paths.rs` | .paths command handler |
| `../../../claude_version_core/src/paths.rs` | `ClaudeVersionPaths` struct implementation |

### Provenance

| Source | Notes |
|--------|-------|
| [feature/008_runtime_file_discovery.md](008_runtime_file_discovery.md) | Precedent command this feature complements with labels and externally-owned paths |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/feature/009_path_discovery.md](../../tests/docs/feature/009_path_discovery.md) | Feature test spec |
