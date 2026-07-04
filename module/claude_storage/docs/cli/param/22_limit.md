# Parameter :: 22. `limit::`

### Scope

- **Purpose**: Specify the `limit::` CLI parameter.
- **Responsibility**: Type, defaults, valid values, and command usage for `limit::`.
- **In Scope**: Value constraints, default behavior, command interactions.
- **Out of Scope**: Type definitions (→ `type/`), command behavior (→ `command/`).

Maximum number of main sessions to display per project. Zero means unlimited.

**Type:** Integer

**Fundamental Type:** Integer

**Constraints:**
- Must be a non-negative integer
- `0` means no cap (all sessions shown)
- Error on negative: `"limit must be non-negative"`

**Default:** `0` (unlimited)

**Commands:** `.projects`

**Purpose:** Caps how many sessions are shown per project in the default view of `.projects`. Useful when a project has many sessions and you only want a preview. Does not apply in `show_tree::1` mode.

**Examples:**
```bash
# Show at most 5 sessions per project
.projects limit::5

# No cap (default)
.projects limit::0

# Combined with scope
.projects scope::global limit::3
```

### Referenced Type
| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| Integer | Base type | Integer | Non-negative (≥ 0); `0` means no cap |

### Referenced Parameter Groups

None.

### Referenced Commands
| # | Command | Default | Notes |
|---|---------|---------|-------|
| 7 | [`.projects`](../command/07_projects.md) | `0` | Caps sessions per project in default display |

### Referenced User Stories
| # | User Story | Persona |
|---|------------|---------|
| 2 | [Find Past Conversation](../user_story/002_find_past_conversation.md) | developer |
