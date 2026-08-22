# Parameter :: 31. `ids::`

### Scope

- **Purpose**: Specify the `ids::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `ids::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Raw conversation-ID scripting output mode for `.projects`, paired with `project::`. Absorbed from `.list`'s former `type::conversation` early-dispatch path (see [`../command/02_list.md`](../command/02_list.md)).

**Type:** Boolean

**Fundamental Type:** Boolean flag

**Constraints:**
- Requires `project::`
- When `1`: outputs one conversation ID per line for the given project (or, with `count::1`, a single bare integer); no path headers, no session detail
- When `0` (default): normal `.projects` listing behavior — `project::` plays no special early-dispatch role

**Default:** `0`

**Commands:** [`.projects`](../command/07_projects.md)

**Purpose:** Enables scripting use cases where a specific project's conversation IDs (not the full session listing) are needed. For example, `clg .projects project::abc123 ids::1` lists one conversation ID per line; `clg .projects project::abc123 ids::1 count::1` outputs only the count. Replaces `.list`'s `type::conversation` early-dispatch path — same underlying algorithm (load project, build session families, group into conversations, emit IDs), reachable through `.projects` instead of a separate command.

**Examples:**
```bash
# List conversation IDs for a project
.projects project::abc123 ids::1

# Count conversations in a project as a bare integer
.projects project::abc123 ids::1 count::1

# Without project:: — error (required)
.projects ids::1   # error: ids:: requires project::
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Boolean | Base type | Boolean flag | `0` (default) or `1` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | `0` | Requires `project::`; pairs with `count::` |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
