# Type :: 10. `StoragePath`

### Scope

- **Purpose**: Specify the `StoragePath` semantic type.
- **Responsibility**: Validation rules, fundamental type, and parameter mapping for `StoragePath`.
- **In Scope**: Parsing rules, valid/invalid values, error messages.
- **Out of Scope**: Parameter usage (→ `param/`), command context (→ `command/`).

**Purpose:** Filesystem path string for storage operations. Accepts both absolute and `~`-prefixed paths.

**Fundamental Type:** Wrapper around string (filesystem path)

**Constants:**
- DEFAULT_ROOT = `~/.claude/` (for `.status`)
- DEFAULT_CWD = current working directory (for `.project.exists`, `.projects`)

**Constraints:**
- Non-empty string
- `~` prefix is shell-expanded to the home directory
- Parent directory must exist for write operations (`output::`)
- Error on empty: `"path must be non-empty"`

**Parsing:**
```
Validate and normalize path:
  Input: "/absolute/path"   → StoragePath("/absolute/path")
  Input: "~/relative/path"  → StoragePath(expand("~/relative/path"))
  Input: "relative/path"    → StoragePath(relative/path)
  Input: ""                 → Error("path must be non-empty")
```

**Methods:**
- `get() -> string` — Raw path string
- `expanded() -> string` — Returns path with `~` expanded
- `exists() -> boolean` — True when path exists on filesystem

**Commands:** `.status`, `.show`, `.count`, `.search`, `.projects`, `.usage`, `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure` (via `path::`); `.export` (via both `output::` and `path::`)

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`.status`](../command/01_status.md) | `path::` |
| 3 | [`.show`](../command/03_show.md) | `path::` |
| 4 | [`.count`](../command/04_count.md) | `path::` |
| 5 | [`.search`](../command/05_search.md) | `path::` |
| 6 | [`.export`](../command/06_export.md) | `output::`, `path::` |
| 7 | [`.projects`](../command/07_projects.md) | `path::` |
| 8 | [`.project.path`](../command/08_project_path.md) | `path::` |
| 9 | [`.project.exists`](../command/09_project_exists.md) | `path::` |
| 10 | [`.session.dir`](../command/10_session_dir.md) | `path::` |
| 11 | [`.session.ensure`](../command/11_session_ensure.md) | `path::` |
| 13 | [`.usage`](../command/13_usage.md) | `path::` |

Not listed: `.list` (deprecated; its `path::` was [`PathSubstring`](../type/04_path_substring.md)-typed, not `StoragePath` — that substring role is now `.projects`' [`filter::`](../param/29_filter.md)).

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 8 | [`output::`](../param/08_output.md) | 1 |
| 9 | [`path::`](../param/09_path.md) | 11 |
