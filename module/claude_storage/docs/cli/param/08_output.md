# Parameter :: 8. `output::`

### Scope

- **Purpose**: Specify the `output::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `output::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Output file path for export operations.

**Type:** [`StoragePath`](../type/10_storage_path.md)

**Fundamental Type:** String (filesystem path)

**Constraints:**
- Must be a non-empty string
- Parent directory must exist (error if parent does not exist)
- File is overwritten without warning if it exists

**Default:** none — **required**

**Commands:** `.export`

**Purpose:** Specifies where the exported session content is written. Accepts absolute and `~`-prefixed paths.

**Examples:**
```bash
# Valid values
output::conversation.md              # Relative path
output::/home/user/exports/chat.md   # Absolute path
output::~/exports/session.json       # Home-relative path

# Error cases
output::                             # Empty path error
output::/nonexistent/dir/file.md     # Parent directory does not exist
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`StoragePath`](../type/10_storage_path.md) | String (filesystem path) | String | Non-empty; parent directory must exist |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 6 | [`.export`](../command/06_export.md) | none — required | Destination path for exported file |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 3 | [Export Session for Review](../user_story/003_export_session_for_review.md) | developer |
