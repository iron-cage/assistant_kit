# Parameter :: 16. `target::`

### Scope

- **Purpose**: Specify the `target::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `target::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Specifies what `.count` should count.

**Type:** [`TargetType`](../type/11_target_type.md)

**Fundamental Type:** String enum wrapper

**Constraints:**
- Valid values: `projects`, `sessions`, `entries`, `conversations`
- Case-insensitive on input
- Error on invalid: `"target must be projects|sessions|entries|conversations, got {value}"`

**Default:** `projects`

**Commands:** `.count`

**Purpose:** Selects the granularity of counting. `projects` counts all projects in storage. `sessions` counts sessions within a project. `entries` counts individual conversation entries. `conversations` counts conversations within a project (requires `project::`; currently 1:1 with sessions).

**Examples:**
```bash
# Valid values
target::projects       # Count all projects (default)
target::sessions       # Count sessions in a project
target::entries        # Count entries in a session
target::conversations  # Count conversations in a project

# Invalid values
target::files      # "Invalid target: files"
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`TargetType`](../type/11_target_type.md) | String enum wrapper | String | `projects`, `sessions`, `entries`, `conversations` |

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 4 | [`.count`](../command/04_count.md) | `projects` | Selects counting granularity |

### Referenced Parameter Groups

None.

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 1 | [Audit Session History](../user_story/001_audit_session_history.md) | developer |
| 4 | [Query Storage Programmatically](../user_story/004_query_storage_programmatically.md) | developer |
