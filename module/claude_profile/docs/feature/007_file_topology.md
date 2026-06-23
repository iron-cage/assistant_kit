# Feature: File Topology

### Scope

- **Purpose**: Provide a typed API for all `~/.claude/` canonical paths and `~/.claude.json` so tooling doesn't hard-code path strings.
- **Responsibility**: Documents the `ClaudePaths` type and `.paths` CLI command (FR-12, FR-19).
- **In Scope**: All `~/.claude/` paths and `~/.claude.json` exposed via `ClaudePaths`, HOME-unset handling, CLI output.
- **Out of Scope**: Writing to these paths (owned by their respective modules), credential store path (→ 001_account_store_init.md), `$PRO`-based persistent storage (→ 010_persistent_storage.md).

### Design

`claude_profile` must expose all `~/.claude/` canonical paths via `ClaudePaths::new()`. Path methods, construction semantics, and ownership boundaries are documented in [schema/003_file_topology.md](../schema/003_file_topology.md).

### Acceptance Criteria

- **AC-01**: `ClaudePaths::new()` returns `None` when `HOME` is unset.
- **AC-02**: Each path method returns the correct path relative to `HOME`.
- **AC-03**: `clp .paths` exits 2 when `HOME` is unset with actionable error.
- **AC-04**: `claude_json_file()` returns `$HOME/.claude.json` (one level above `base()`).
- **AC-05**: `claude_json_file()` is NOT inside the `.claude/` directory — it is a sibling to it at `$HOME`.
- **AC-06**: `ClaudePaths::with_home(home)` returns a `ClaudePaths` whose `credentials_file()` resolves to `{home}/.claude/.credentials.json`.

### Commands

| File | Relationship |
|------|--------------|
| [command/004_paths.md](../cli/command/004_paths.md#command--8-paths) | CLI command specification |

### Invariants

| File | Relationship |
|------|--------------|
| [007_json_storage_format.md](../invariant/007_json_storage_format.md) | Files written at these paths (`credentials_file()`, `settings_file()`, `claude_json_file()`) must use 2-space pretty-printed JSON with trailing newline |

### Features

| File | Relationship |
|------|--------------|
| [014_rich_account_metadata.md](014_rich_account_metadata.md) | Uses `claude_json_file()` path for `~/.claude.json` access |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.paths`](../cli/command/004_paths.md#command--8-paths) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/paths.rs` | `ClaudePaths` struct and all path methods |
| `src/commands/token_paths.rs` | `paths_routine()` — CLI handler |

### Tests

| File | Relationship |
|------|--------------|
| `tests/paths_tests.rs` | All path methods return correct values |

### Schema

| File | Relationship |
|------|-------------|
| [schema/003_file_topology.md](../schema/003_file_topology.md) | Canonical path method reference — extracted from this feature |
| [schema/006_settings_json.md](../schema/006_settings_json.md) | Fields clp reads/writes in `settings_file()` |
| [schema/007_claude_json.md](../schema/007_claude_json.md) | Fields clp reads from `claude_json_file()` |
