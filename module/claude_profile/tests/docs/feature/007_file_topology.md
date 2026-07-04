# Test: Feature 007 — File Topology

### Scope

- **Purpose**: Test cases for `ClaudePaths` construction and path resolution methods relative to `HOME`.
- **Source**: `docs/feature/007_file_topology.md`
- **Covers**: AC-01 through AC-06

Feature behavioral requirement test cases for `docs/feature/007_file_topology.md` (FR-12, FR-19). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | `ClaudePaths::new()` returns `None` when `HOME` unset | AC-01 |
| FT-02 | Each path method returns correct path relative to `HOME` | AC-02 |
| FT-03 | `clp .paths` exits 2 when `HOME` unset | AC-03 |
| FT-04 | `claude_json_file()` returns `$HOME/.claude.json` | AC-04 |
| FT-05 | `claude_json_file()` is NOT inside `.claude/` directory | AC-05 |
| FT-06 | `ClaudePaths::with_home(home)` resolves `credentials_file()` correctly | AC-06 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | `ClaudePaths::new()` returns None when HOME unset | AC-01 | Construction |
| FT-02 | All path methods return correct paths relative to HOME | AC-02 | Path Methods |
| FT-03 | `clp .paths` exits 2 with actionable error when HOME unset | AC-03 | CLI Error |
| FT-04 | `claude_json_file()` returns `$HOME/.claude.json` | AC-04 | Path Shape |
| FT-05 | `claude_json_file()` is sibling to `.claude/`, not inside it | AC-05 | Path Shape |
| FT-06 | `with_home()` resolves credentials path from explicit home | AC-06 | Construction |

**Total:** 6 FT cases

---

### FT-01: `ClaudePaths::new()` returns None when HOME unset

- **Given:** `HOME` environment variable is not set.
- **When:** `ClaudePaths::new()` is called.
- **Then:** Returns `None`. No panic.
- **Exit:** Ok(None)
- **Source fn:** `new_returns_none_when_home_not_set`
- **Source:** [007_file_topology.md AC-01](../../../docs/feature/007_file_topology.md)

---

### FT-02: All path methods return correct paths relative to HOME

- **Given:** `HOME` is set to a known temp directory. `ClaudePaths::with_home(home)` is constructed.
- **When:** Each path method is called: `credentials_file()`, `projects_dir()`, `stats_file()`, `settings_file()`, `session_env_dir()`, `sessions_dir()`.
- **Then:**
  - `credentials_file()` → `{home}/.claude/.credentials.json`
  - `projects_dir()` → `{home}/.claude/projects/`
  - `stats_file()` → `{home}/.claude/stats-cache.json`
  - `settings_file()` → `{home}/.claude/settings.json`
  - `session_env_dir()` → `{home}/.claude/session-env/`
  - `sessions_dir()` → `{home}/.claude/sessions/`
- **Exit:** Ok
- **Source fn:** `credentials_file_returns_dot_credentials_json`, `projects_dir_returns_projects_under_base`, `stats_file_returns_stats_cache_json`, `settings_file_returns_settings_json`, `session_env_dir_returns_session_env_under_base`, `sessions_dir_returns_sessions_under_base`
- **Source:** [007_file_topology.md AC-02](../../../docs/feature/007_file_topology.md)

---

### FT-03: `clp .paths` exits 2 with actionable error when HOME unset

- **Given:** `HOME` environment variable is not set.
- **When:** `clp .paths`
- **Then:** Exits 2. Stderr contains an actionable error message (not a generic panic).
- **Exit:** 2
- **Source fn:** `p05_paths_home_unset_exits_2`
- **Source:** [007_file_topology.md AC-03](../../../docs/feature/007_file_topology.md)

---

### FT-04: `claude_json_file()` returns `$HOME/.claude.json`

- **Given:** `ClaudePaths::with_home(home)` constructed with a known `home` path.
- **When:** `paths.claude_json_file()` is called.
- **Then:** Returns `{home}/.claude.json` — one level above the `.claude/` base directory.
- **Exit:** Ok
- **Source fn:** `ft04_claude_json_file_returns_home_dot_claude_json`
- **Source:** [007_file_topology.md AC-04](../../../docs/feature/007_file_topology.md)

---

### FT-05: `claude_json_file()` is sibling to `.claude/`, not inside it

- **Given:** `ClaudePaths::with_home(home)` constructed.
- **When:** `paths.claude_json_file()` is called.
- **Then:** The returned path does NOT contain `.claude/claude.json`. It is `{home}/.claude.json` — outside the `.claude/` directory, not inside it.
- **Exit:** Ok
- **Source fn:** `ft05_claude_json_file_is_sibling_not_inside_dot_claude`
- **Source:** [007_file_topology.md AC-05](../../../docs/feature/007_file_topology.md)

---

### FT-06: `with_home()` resolves credentials path from explicit home

- **Given:** An explicit `home: &Path` is passed to `ClaudePaths::with_home(home)`.
- **When:** `paths.credentials_file()` is called.
- **Then:** Returns `{home}/.claude/.credentials.json`.
- **Exit:** Ok
- **Source fn:** `credentials_file_returns_dot_credentials_json`
- **Source:** [007_file_topology.md AC-06](../../../docs/feature/007_file_topology.md)
