# Feature: File Topology

### Scope

- **Purpose**: Provide a typed API for all `~/.claude/` canonical paths and `~/.claude.json` so tooling doesn't hard-code path strings.
- **Responsibility**: Documents the `ClaudePaths` type and `.paths` CLI command (FR-12, FR-19).
- **In Scope**: All `~/.claude/` paths and `~/.claude.json` exposed via `ClaudePaths`, HOME-unset handling, CLI output.
- **Out of Scope**: Writing to these paths (owned by their respective modules), credential store path (→ 001_account_store_init.md), `$PRO`-based persistent storage (→ 010_persistent_storage.md).

### Design

`claude_profile` must expose all `~/.claude/` canonical paths via `ClaudePaths::new()`.

**Construction:** `ClaudePaths::new()` returns `None` if `HOME` environment variable is not set. `ClaudePaths::with_home(home: &Path)` constructs from an explicit path — used in unit tests to avoid mutating the `HOME` env var in parallel test processes.

**Path methods:**

| Method | Resolves to |
|--------|-------------|
| `credentials_file()` | `~/.claude/.credentials.json` |
| `claude_json_file()` | `~/.claude.json` (Claude Code state: `oauthAccount`, model preference) |
| `projects_dir()` | `~/.claude/projects/` |
| `stats_file()` | `~/.claude/stats-cache.json` |
| `settings_file()` | `~/.claude/settings.json` |
| `session_env_dir()` | `~/.claude/session-env/` |
| `sessions_dir()` | `~/.claude/sessions/` |

All methods return `PathBuf` computed from `HOME`. No filesystem access is performed — these are pure path computations.

**Ownership boundaries:**
- `~/.claude/projects/` — owned by `claude_storage_core`
- `~/.claude/session-env/`, `~/.claude/sessions/` — owned by `claude_runner_core`
- Remaining paths — owned by `claude_profile`

`ClaudePaths` is the single authoritative registry; callers must not construct these paths independently.

### Acceptance Criteria

- **AC-01**: `ClaudePaths::new()` returns `None` when `HOME` is unset.
- **AC-02**: Each path method returns the correct path relative to `HOME`.
- **AC-03**: `clp .paths` exits 2 when `HOME` is unset with actionable error.
- **AC-04**: `claude_json_file()` returns `$HOME/.claude.json` (one level above `base()`).
- **AC-05**: `claude_json_file()` is NOT inside the `.claude/` directory — it is a sibling to it at `$HOME`.
- **AC-06**: `ClaudePaths::with_home(home)` returns a `ClaudePaths` whose `credentials_file()` resolves to `{home}/.claude/.credentials.json`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/paths.rs` | `ClaudePaths` struct and all path methods |
| source | `src/commands.rs` | `paths_routine()` — CLI handler |
| test | `tests/paths_tests.rs` | All path methods return correct values |
| doc | [cli/commands.md](../cli/commands.md#command--8-paths) | CLI command specification |
