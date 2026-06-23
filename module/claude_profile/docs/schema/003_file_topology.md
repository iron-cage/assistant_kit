# Schema: File Topology — `ClaudePaths`

### Scope

- **Purpose**: Canonical reference for all `~/.claude/` paths and `~/.claude.json` exposed by the `ClaudePaths` type.
- **In Scope**: Every path method on `ClaudePaths`, their resolved values, ownership boundaries.
- **Out of Scope**: Credential store paths (→ [schema/004](004_storage_root.md)); file formats at these paths (→ [schema/006](006_settings_json.md), [schema/007](007_claude_json.md)).

### Construction

```rust
ClaudePaths::new()              // returns None if $HOME unset
ClaudePaths::with_home(home)    // construct from explicit path (used in tests)
```

### Path Methods

| Method | Resolves to | Owned by |
|--------|-------------|----------|
| `credentials_file()` | `~/.claude/.credentials.json` | `claude_profile` |
| `claude_json_file()` | `~/.claude.json` (sibling to `.claude/`, NOT inside it) | `claude` binary (read-only for clp) |
| `settings_file()` | `~/.claude/settings.json` | `claude_profile` (reads + writes `model`, `effortLevel`) |
| `projects_dir()` | `~/.claude/projects/` | `claude_storage_core` |
| `stats_file()` | `~/.claude/stats-cache.json` | `claude` binary (read-only for clp) |
| `session_env_dir()` | `~/.claude/session-env/` | `claude_runner_core` |
| `sessions_dir()` | `~/.claude/sessions/` | `claude_runner_core` |

All methods return `PathBuf` computed from `HOME`. No filesystem access is performed — pure path computations.

### Key Invariant

`claude_json_file()` returns `$HOME/.claude.json` — it is a **sibling** to `$HOME/.claude/`, not inside it. Callers must not construct this path as `$HOME/.claude/claude.json`.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/007_file_topology.md](../feature/007_file_topology.md) | Feature spec with acceptance criteria and test refs |
| [schema/006](006_settings_json.md) | Fields clp reads/writes in `settings_file()` |
| [schema/007](007_claude_json.md) | Fields clp reads from `claude_json_file()` |
| [schema/004](004_storage_root.md) | Credential store path (separate from `ClaudePaths`) |
