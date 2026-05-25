# Type :: 5. `ProjectId`

**Purpose:** Multi-format project identifier. Claude Code uses different naming schemes for projects; this type accepts all of them and resolves to the internal path-encoded key.

**Fundamental Type:** Wrapper around string (multi-format)

**Constants:**
- DEFAULT = current working directory (resolved at runtime)

**Accepted Formats:**
- Absolute path: `/home/alice/projects/my-app`
- Path-encoded ID: `-home-alice-projects-my-app`
- UUID: `8d795a1c-c81d-4010-8d29-b4e678272419`
- `Path(...)` form from `.list`: `Path("/home/alice/projects/my-app")`

**Constraints:**
- Non-empty string
- Error if project not found after resolution: `"project not found: {value}"`

**Parsing:**
```
Detect format and resolve to internal key:
  Input starts with "/"  → treat as absolute path, encode to path ID
  Input starts with "-"  → treat as path-encoded ID directly
  Input is UUID pattern  → treat as UUID project name
  Input starts with "Path(" → extract path from Path(...) form, encode
  Input empty            → Error("project must be non-empty")
  Resolve → Error("project not found: " + input) if not in storage
```

**Methods:**
- `get() -> string` — Raw input value
- `path_encoded() -> string` — Resolved path-encoded form
- `is_uuid() -> boolean` — True when project uses UUID naming

**Commands:** `.list`, `.show`, `.count`, `.search`, `.export`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 2 | [`.list`](../command/02_list.md) | `project::` |
| 3 | [`.show`](../command/03_show.md) | `project::` |
| 4 | [`.count`](../command/04_count.md) | `project::` |
| 5 | [`.search`](../command/05_search.md) | `project::` |
| 6 | [`.export`](../command/06_export.md) | `project::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 10 | [`project::`](../param/10_project.md) | 5 |
