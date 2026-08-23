# Parameter :: 18. `type::`

### Scope

- **Purpose**: Specify the `type::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `type::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Project naming scheme filter for `.projects`. Absorbed from `.list` (see [`../command/02_list.md`](../command/02_list.md)) with its `conversation` value dropped — that capability is now `ids::` (see [`31_ids.md`](31_ids.md)), which is orthogonal to naming-scheme filtering rather than a value of it.

**Type:** [`ProjectType`](../type/06_project_type.md)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `uuid`, `path`, `all`
- Case-insensitive on input
- Error on invalid: `"type must be uuid|path|all, got {value}"`

**Default:** `all`

**Commands:** [`.projects`](../command/07_projects.md)

**Purpose:** Filters projects by how their directory is named in `~/.claude/projects/`. Path-encoded projects (e.g., `-home-alice-projects`) are opened by filesystem path. UUID projects (`feed0001-...`) are created by other means. Composes with `scope::` (discovery boundary) and `filter::` (substring narrowing) — all three narrow the same resolved project set independently.

**Examples:**
```bash
# Valid values
type::all    # No filter (default)
type::path   # Path-encoded projects only (e.g., -home-alice-projects)
type::uuid   # UUID-named projects only (e.g., feed0001-...)

# Invalid values
type::both   # "type must be uuid|path|all, got both"

# Combined with detail:: for a terse naming-scheme-filtered list
.projects scope::global type::uuid detail::projects
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`ProjectType`](../type/06_project_type.md) | String enum wrapper | String | `uuid`, `path`, `all` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | `all` | Filters projects by naming scheme; absorbed from `.list` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
