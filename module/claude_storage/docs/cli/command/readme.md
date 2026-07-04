# Commands Reference

### Scope

- **Purpose**: Document all CLI commands for the `claude_storage` binary.
- **Responsibility**: Per-command detail pages with full parameter tables and cross-refs.
- **In Scope**: All 12 commands with syntax, parameters, examples, and exit codes.
- **Out of Scope**: Parameter definitions (→ `param/`), type constraints (→ `type/`), parameter groups (→ `param_group/`).

All commands for the `claude_storage` CLI. Parameters use `param::value` syntax. All commands are read-only except `.session.ensure`, which creates the session working directory on disk.

See [param/readme.md](../param/readme.md) for full parameter specs and [type/readme.md](../type/readme.md) for type definitions.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_status.md` | .status — storage overview and statistics |
| `02_list.md` | .list — list projects or sessions |
| `03_show.md` | .show — display session or project details |
| `04_count.md` | .count — fast counting of items |
| `05_search.md` | .search — search session content by query |
| `06_export.md` | .export — export session to file |
| `07_projects.md` | .projects — scoped project list with conversation grouping |
| `08_project_path.md` | .project.path — compute Claude storage path for a directory |
| `09_project_exists.md` | .project.exists — check conversation history exists |
| `10_session_dir.md` | .session.dir — compute session working directory path |
| `11_session_ensure.md` | .session.ensure — ensure session directory exists |
| `12_tail.md` | .tail — print last N conversation entries for current directory |

### Commands Table

| # | Command | Purpose | Params | Example |
|---|---------|---------|--------|---------|
| 1 | [`.status`](01_status.md) | Show storage overview and statistics | 2 | `claude_storage .status` |
| 2 | [`.list`](02_list.md) | List projects or sessions | 9 | `claude_storage .list` |
| 3 | [`.show`](03_show.md) | Display session or project details | 8 | `claude_storage .show session_id::ID` |
| 4 | [`.count`](04_count.md) | Fast counting of items | 5 | `claude_storage .count` |
| 5 | [`.search`](05_search.md) | Search session content by query | 7 | `claude_storage .search query::QUERY` |
| 6 | [`.export`](06_export.md) | Export session to file | 6 | `claude_storage .export session_id::ID output::PATH` |
| 7 | [`.projects`](07_projects.md) | Scoped project list with per-project conversation listing | 7 | `claude_storage .projects` |
| 8 | [`.project.path`](08_project_path.md) | Compute Claude storage path for a directory | 2 | `claude_storage .project.path` |
| 9 | [`.project.exists`](09_project_exists.md) | Check conversation history exists (exits 1 when absent) | 2 | `claude_storage .project.exists` |
| 10 | [`.session.dir`](10_session_dir.md) | Compute session working directory path | 2 | `claude_storage .session.dir` |
| 11 | [`.session.ensure`](11_session_ensure.md) | Ensure session directory exists, report resume strategy | 3 | `claude_storage .session.ensure` |
| 12 | [`.tail`](12_tail.md) | Print last N conversation entries for current directory | 3 | `claude_storage .tail` |

**Total:** 12 commands
