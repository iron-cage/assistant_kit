# Parameter :: 10. `project::`

### Scope

- **Purpose**: Specify the `project::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `project::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Project identifier for scoping operations to a specific project.

**Type:** [`ProjectId`](../type/05_project_id.md)

**Fundamental Type:** String (multi-format identifier)

**Constraints:**
- Accepts multiple formats (see type definition)
- Error if project not found: `"project not found: {value}"`

**Default:** resolves to the project for the current working directory

**Commands:** `.show`, `.count`, `.search`, `.export`

**Purpose:** Restricts an operation to a specific project. Without `project::`, most commands default to the current directory's project. Use `project::` when working with a project other than the current directory.

**Accepted formats:**
- Absolute path: `project::/home/alice/projects/my-app`
- Path-encoded ID: `project::-home-alice-projects-my-app`
- UUID: `project::8d795a1c-c81d-4010-8d29-b4e678272419`
- `Path(...)` form from `.list`: `project::Path("/home/alice/projects/my-app")`

**Examples:**
```bash
# All equivalent — identify the same project
project::/home/alice/projects/my-app
project::-home-alice-projects-my-app
project::8d795a1c-c81d-4010-8d29-b4e678272419

# Invalid
project::               # Empty path error
project::nonexistent    # Project not found error
```

**Group:** [Project Scope](../param_group/02_project_scope.md)

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`ProjectId`](../type/05_project_id.md) | String (multi-format) | String | Non-empty; project must exist in storage |

### Referenced Parameter Groups
| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Project Scope](../param_group/02_project_scope.md) | Full | *(sole member)* |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`.list`](../command/02_list.md) | — | Required for `type::conversation` mode |
| 3 | [`.show`](../command/03_show.md) | current dir project | Restricts session search to this project |
| 4 | [`.count`](../command/04_count.md) | — | Scopes count to this project |
| 5 | [`.search`](../command/05_search.md) | — | Restricts search to this project |
| 6 | [`.export`](../command/06_export.md) | current dir | Source project for session lookup |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
