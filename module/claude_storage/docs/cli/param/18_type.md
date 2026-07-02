# Parameter :: 18. `type::`

### Scope

- **Purpose**: Specify the `type::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `type::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Project naming scheme filter for `.list`.

**Type:** [`ProjectType`](../type/06_project_type.md)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `uuid`, `path`, `all`, `conversation`
- Case-insensitive on input
- Error on invalid: `"Invalid type: {value}. Valid values: uuid, path, all"`
- `conversation` requires `project::` parameter

**Default:** `all`

**Commands:** `.list`

**Purpose:** Filters projects by how their directory is named in `~/.claude/projects/`. Path-encoded projects (e.g., `-home-alice-projects`) are opened by filesystem path. UUID projects (`8d795a1c-...`) are created by other means. `conversation` lists conversation IDs within a specific project (one per line).

**Examples:**
```bash
# Valid values
type::all           # No filter (default)
type::path          # Path-encoded projects only (e.g., -home-alice-projects)
type::uuid          # UUID-named projects only (e.g., 8d795a1c-...)
type::conversation  # List conversation IDs for a project (requires project::)

# Invalid values
type::both   # "Invalid type: both. Valid values: uuid, path, all"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`ProjectType`](../type/06_project_type.md) | String enum wrapper | String | `uuid`, `path`, `all`, `conversation` |

### Referenced Parameter Groups

None.

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) | `all` | Filters projects by naming scheme |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
