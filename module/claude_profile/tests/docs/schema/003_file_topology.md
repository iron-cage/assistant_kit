# Schema 003: File Topology — `ClaudePaths`

SC test cases for `docs/schema/003_file_topology.md`. Verifies the `ClaudePaths` path
method contracts: correct resolved paths, the sibling invariant for `claude_json_file()`,
and the pure-computation guarantee (no filesystem access in any path method).

**Source:** [docs/schema/003_file_topology.md](../../../../docs/schema/003_file_topology.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | `credentials_file()` resolves to `~/.claude/.credentials.json` | Path Resolution | ✅ |
| SC-2 | `claude_json_file()` resolves to `~/.claude.json` (sibling, NOT inside `.claude/`) | Sibling Invariant | ✅ |
| SC-3 | `settings_file()` resolves to `~/.claude/settings.json` | Path Resolution | ✅ |
| SC-4 | All path methods are pure computations — no filesystem access | Purity | ✅ |

---

### SC-1: `credentials_file()` resolves to `~/.claude/.credentials.json`

- **Given:** `ClaudePaths::with_home(home)` constructed with an explicit home directory
- **When:** `credentials_file()` is called
- **Then:** Returns `{home}/.claude/.credentials.json` — the live session credential file path; inside `.claude/` subdirectory
- **Source fn:** `credentials_file_returns_dot_credentials_json` (paths_tests.rs)
- **Source:** [docs/schema/003_file_topology.md §Path Methods](../../../../docs/schema/003_file_topology.md)

---

### SC-2: `claude_json_file()` returns `~/.claude.json` — sibling to `.claude/`, not inside it

- **Given:** `ClaudePaths::with_home(home)` constructed with an explicit home directory
- **When:** `claude_json_file()` is called
- **Then:** Returns `{home}/.claude.json` — a sibling file to the `{home}/.claude/` directory, NOT `{home}/.claude/claude.json`
- **Note:** Key invariant — callers must not assume this path is inside `.claude/`. Constructing it as `.claude/claude.json` would target a different file.
- **Source fn:** `ft04_claude_json_file_returns_home_dot_claude_json` and `ft05_claude_json_file_is_sibling_not_inside_dot_claude` (paths_tests.rs)
- **Source:** [docs/schema/003_file_topology.md §Key Invariant](../../../../docs/schema/003_file_topology.md)

---

### SC-3: `settings_file()` resolves to `~/.claude/settings.json`

- **Given:** `ClaudePaths::with_home(home)` constructed with an explicit home directory
- **When:** `settings_file()` is called
- **Then:** Returns `{home}/.claude/settings.json` — the Claude session settings file; inside `.claude/` subdirectory
- **Source fn:** `settings_file_returns_settings_json` (paths_tests.rs)
- **Source:** [docs/schema/003_file_topology.md §Path Methods](../../../../docs/schema/003_file_topology.md)

---

### SC-4: All path methods are pure computations — no filesystem access

- **Given:** `ClaudePaths::with_home(home)` constructed with a path that does not exist on disk
- **When:** Any of `credentials_file()`, `claude_json_file()`, `settings_file()`, `projects_dir()`, `stats_file()`, `session_env_dir()`, `sessions_dir()` are called
- **Then:** All return `PathBuf` values without error, without filesystem access, and without panicking — path derivation is purely string manipulation on the `HOME` root
- **Source fn:** `ft05_claude_json_file_is_sibling_not_inside_dot_claude` (paths_tests.rs; uses non-existent temp path)
- **Source:** [docs/schema/003_file_topology.md §Construction](../../../../docs/schema/003_file_topology.md)
